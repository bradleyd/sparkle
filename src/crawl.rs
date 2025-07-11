use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::file_detector::{get_age_category, get_file_size_category, get_file_type};
use crate::{FileContext, FileMetadata};

/// Recursively searches a directory and its subdirectories
pub fn search_dir(
    dir: &Path,
    results: &mut Vec<FileContext>,
    config: &crate::config::Config,
    quiet: bool,
) {
    // bail early
    if !dir.is_dir() {
        return;
    }

    // Read the directory entries
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            if !quiet {
                eprintln!("Warning: Could not read directory {}: {}", dir.display(), e);
            }
            return;
        }
    };

    // Iterate over each entry in the directory
    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                // Could not access entry so print warning and keep seearching
                if !quiet {
                    eprintln!("Warning: error accessing entry in {}: {}", dir.display(), e);
                }
                continue;
            }
        };

        let path = entry.path();

        // If the entry is a directory, recursively search it
        if path.is_dir() {
            search_dir(&path, results, config, quiet);
        } else {
            // We have a file, check if file matches criteria
            let filters = &config.rules[0].filters;
            let actions = &config.rules[0].actions;
            match fs::metadata(&path) {
                Ok(metadata) => match metadata.modified() {
                    Ok(modified_time) => {
                        let fmeta = FileMetadata {
                            size: metadata.len(),
                            modified: modified_time,
                            created: get_created_time(&metadata),
                            accessed: get_access_time(&metadata),
                            permissions: metadata.permissions(),
                            is_file: metadata.is_file(),
                            is_dir: metadata.is_dir(),
                            is_symlink: metadata.is_symlink(),
                            extended_attributes: HashMap::new(),
                            size_category: get_file_size_category(&metadata),
                            age_category: get_age_category(&metadata),
                            file_type: get_file_type(&path),
                        };

                        for f in filters.iter() {
                            match f {
                                crate::config::Filter::Extension { extension } => {
                                    if extension.as_str() == path.extension().unwrap() {
                                        println!("matched extension rule");
                                        // get action
                                        for a in actions {
                                            if let Err(e) = crate::handlers::action::run(
                                                a,
                                                &config.rules[0],
                                                &path,
                                            ) {
                                                println!("{}", e)
                                            }
                                        }
                                    }
                                }
                                crate::config::Filter::Size { size_gt, size_lt } => {
                                    let size = if size_gt.is_some() {
                                        size_gt.unwrap()
                                    } else {
                                        size_lt.unwrap()
                                    };
                                    println!("size {}", size)
                                }
                                crate::config::Filter::Age { days_older_than } => {
                                    println!("age {}", days_older_than.unwrap_or_default())
                                }
                            }
                        }

                        results.push(FileContext {
                            path: path.clone(),
                            metadata: fmeta,
                            content_info: None,
                            parent_dir: get_parent_dir(&path),
                            base_dir: std::env::current_dir().unwrap(),
                        })
                    }
                    Err(e) => {
                        if !quiet {
                            eprintln!(
                                "Warning: Could not get modification time for {}: {}",
                                path.display(),
                                e
                            );
                        }
                    }
                },
                Err(e) => {
                    if !quiet {
                        eprintln!(
                            "Warning: Could not get metadata for {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
    }
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

fn get_parent_dir(p: &Path) -> PathBuf {
    match p.parent() {
        Some(parent) => parent.to_path_buf(),
        None => PathBuf::new(),
    }
}
