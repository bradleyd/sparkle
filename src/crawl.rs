use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::config::Filter;
use crate::file_metadata::{FileContext, FileMetadata, FileMetadataError};

// return FileMetadata/FileContext
pub fn search_dir(
    dir: &Path,
    config: &crate::config::Config,
    rule: &crate::config::Rule,
    quiet: bool,
) -> Result<Vec<FileContext>, FileMetadataError> {
    // bail early
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    // Read the directory entries
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            if !quiet {
                tracing::error!("Warning: Could not read directory {}: {}", dir.display(), e);
            }
            return Ok(Vec::new());
        }
    };

    let mut results = Vec::new();
    // Iterate over each entry in the directory
    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                // Could not access entry so print warning and keep seearching
                if !quiet {
                    tracing::error!("Warning: error accessing entry in {}: {}", dir.display(), e);
                }
                continue;
            }
        };

        let path = entry.path();

        // If the entry is a directory, recursively search it
        if path.is_dir() {
            search_dir(&path, config, rule, quiet)?;
        } else {
            // We have a file, check if file matches criteria
            let fmeta = FileMetadata::build(&path, quiet)?;
            if matches_filters(&path, &rule.filters) {
                if let Err(e) = crate::handlers::action::run(&rule.actions, &path) {
                    tracing::error!("Error applying actions to {}: {}", path.display(), e);
                    continue;
                }
                let metadata = fmeta.clone();
                results.push(FileContext {
                    path: path.clone(),
                    metadata,
                    content_info: None,
                    parent_dir: get_parent_dir(&path),
                    base_dir: std::env::current_dir().unwrap(),
                })
            }
        }
    }
    Ok(results)
}

fn get_created_time(metadata: &Metadata) -> Option<SystemTime> {
    if let Ok(created_at) = metadata.created() {
        // Success case: The pattern matched, we have the creation time.
        Some(created_at)
    } else {
        // Failure case: The pattern did not match, it must be an Err.
        // We can print a message or log the error before returning None.
        None
    }
}

fn get_access_time(metadata: &Metadata) -> Option<SystemTime> {
    if let Ok(atime) = metadata.accessed() {
        // Success case: The pattern matched, we have the creation time.
        Some(atime)
    } else {
        // Failure case: The pattern did not match, it must be an Err.
        // We can print a message or log the error before returning None.
        None
    }
}

fn matches_filters(path: &Path, filters: &[Filter]) -> bool {
    filters.iter().any(|filter| match filter {
        Filter::Extension { extension } => path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext_str| ext_str.eq_ignore_ascii_case(extension))
            .unwrap_or(false),
        Filter::NameContains { name } => path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name_str| name_str.contains(name))
            .unwrap_or(false),
        Filter::Size { size_gt, size_lt } => {
            if let Ok(metadata) = std::fs::metadata(path) {
                let file_size = metadata.len();
                let gt_pass = size_gt.map(|min| file_size > min).unwrap_or(true);
                let lt_pass = size_lt.map(|max| file_size < max).unwrap_or(true);
                gt_pass && lt_pass
            } else {
                false
            }
        }
        Filter::Age { days_older_than } => {
            if let Some(days) = days_older_than {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration_since_mod) = modified.elapsed() {
                            return duration_since_mod.as_secs() > (*days as u64 * 86400);
                        }
                    }
                }
            }
            false
        }
    })
}

fn get_parent_dir(p: &Path) -> PathBuf {
    match p.parent() {
        Some(parent) => parent.to_path_buf(),
        None => PathBuf::new(),
    }
}
