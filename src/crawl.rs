use std::fs::{self};
use std::path::{Path, PathBuf};

use crate::config::Filter;
use crate::database::DatabaseConnection;
use crate::file_metadata::{FileContext, FileMetadata, FileMetadataError};
use crate::models::{self, ActionRepo, FilterRepo};
use crate::utils;

// return FileMetadata/FileContext
pub fn search_dir(
    dir: &Path,
    _config: &crate::config::Config,
    rule: &crate::config::Rule,
    quiet: bool,
    dry_run: bool,
    conn: &DatabaseConnection,
) -> Result<Vec<FileContext>, FileMetadataError> {
    // bail early
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    utils::log_to_terminal("🔍 Searching...");
    utils::log_to_terminal(format!("Using Rule '{}'", rule.name).as_str());

    tracing::info!(
        "Searching directory: {}\nUsing rule: {}",
        dir.to_string_lossy(),
        rule.name
    );
    if dry_run {
        for action in &rule.actions {
            tracing::info!("Action {} would have run", action);
        }
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

    let run_result = models::run::Run::create(conn, dry_run.into(), None);
    tracing::debug!("Run DB result: {:?}", run_result);
    let mut run_id: Option<i64> = None;
    if let Ok(rid) = run_result {
        run_id = rid.id;
    }

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
            if rule.subfolders {
                search_dir(&path, _config, rule, quiet, dry_run, conn)?;
            } else {
                continue;
            }
        } else {
            // We have a file, check if file matches criteria
            let fmeta = FileMetadata::build(&path, quiet)?;
            if matches_filters(&path, &rule.filters) {
                let mut filter_id: Option<i64> = None;
                for filter in rule.filters.iter() {
                    let res =
                        FilterRepo::create(conn, filter.as_str(), filter.to_string().as_str());
                    tracing::debug!("filter results: {:?}", res);
                    if let Ok(model_filter) = res {
                        filter_id = model_filter.id;
                        break;
                    }
                }

                tracing::info!(
                    "Matched {}",
                    &rule
                        .filters
                        .iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                );

                let action_repo = models::action::ActionRepo {
                    id: None,
                    created_at: None,
                    run_id,
                    filter_id,
                    op: rule.actions[0].to_string(),
                    src_path: path.to_string_lossy().to_string(),
                    dst_path: String::from("foobar"),
                    error_msg: None,
                };

                if !dry_run {
                    if let Err(e) = crate::handlers::action::run(&rule.actions, &path) {
                        tracing::error!("Error applying actions to {}: {}", path.display(), e);
                        continue;
                    }
                    tracing::debug!("filter id: {:?}", filter_id);
                    if filter_id.is_some() {
                        tracing::debug!("filter id: {:?}", filter_id);
                        let action = ActionRepo::new(action_repo);
                        let action_res = ActionRepo::create(conn, action);
                        tracing::debug!("action results: {:?}", action_res);
                    }
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
    // update run
    if let Some(id) = run_id {
        let res = models::run::Run::update(conn, id);
        tracing::debug!("Update run finished with {:?}", res);
    }
    Ok(results)
}

//fn get_created_time(metadata: &Metadata) -> Option<SystemTime> {
//    if let Ok(created_at) = metadata.created() {
//        // Success case: The pattern matched, we have the creation time.
//        Some(created_at)
//    } else {
//        // Failure case: The pattern did not match, it must be an Err.
//        // We can print a message or log the error before returning None.
//        None
//    }
//}
//
//fn get_access_time(metadata: &Metadata) -> Option<SystemTime> {
//    if let Ok(atime) = metadata.accessed() {
//        // Success case: The pattern matched, we have the creation time.
//        Some(atime)
//    } else {
//        // Failure case: The pattern did not match, it must be an Err.
//        // We can print a message or log the error before returning None.
//        None
//    }
//}

fn matches_filters(path: &Path, filters: &[Filter]) -> bool {
    filters.iter().any(|filter| match filter {
        Filter::Extension { extension } => path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext_str| ext_str.eq_ignore_ascii_case(extension))
            .unwrap_or(false),
        Filter::NameContains { name_contains } => path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name_str| name_str.contains(name_contains))
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
        Filter::Age { days_older_than } => days_older_than.is_some_and(|days| {
            std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|dur| dur.as_secs() > (days as u64 * 86_400))
        }),
    })
}

fn get_parent_dir(p: &Path) -> PathBuf {
    match p.parent() {
        Some(parent) => parent.to_path_buf(),
        None => PathBuf::new(),
    }
}
