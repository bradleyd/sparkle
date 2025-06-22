use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::path::Path;
use std::time::SystemTime;

use crate::FileMetadata;
use crate::file_detector::{get_age_category, get_file_size_category, get_file_type};

/// Recursively searches a directory and its subdirectories
pub fn search_dir(dir: &Path, results: &mut Vec<FileMetadata>, quiet: bool) {
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
            search_dir(&path, results, quiet);
        } else {
            // We have a file, check if file matches criteria
            match fs::metadata(&path) {
                Ok(metadata) => match metadata.modified() {
                    Ok(modified_time) => {
                        results.push(FileMetadata {
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
                        });
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
