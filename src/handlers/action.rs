use std::fs;
use std::io::Error;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run(actions: &[crate::config::Action], file_path: &Path) -> Result<(), Error> {
    for action in actions {
        match action {
            crate::config::Action::Echo(message) => {
                tracing::debug!("Running action: echo, result: {}", message);
                tracing::info!("{}", message);
            }
            crate::config::Action::Move(path_buf) => {
                // check if destination directory exists
                if !path_buf.exists() {
                    let msg = format!(
                        "Destination directory '{}' does not exist",
                        path_buf.display()
                    );
                    return Err(std::io::Error::other(msg));
                };

                tracing::debug!("Moving file to {}", path_buf.to_string_lossy());

                if let Err(e) = crate::utils::move_file(file_path, path_buf) {
                    tracing::error!("There was an issue trying to run the move action {}", e);
                    return Err(std::io::Error::other(format!(
                        "Cannot move a directory named '{}'",
                        e
                    )));
                };

                tracing::info!(
                    "Moved file {} to {}",
                    file_path.to_string_lossy(),
                    path_buf.as_path().to_string_lossy()
                );
            }
            crate::config::Action::Copy(path_buf) => {
                tracing::debug!("Copying file to {}", path_buf.to_string_lossy());
                if !path_buf.exists() {
                    let msg = format!(
                        "Destination directory '{}' does not exist",
                        path_buf.display()
                    );
                    return Err(std::io::Error::other(msg));
                };

                if let Err(e) = crate::utils::copy_file(file_path, path_buf) {
                    tracing::error!("There was an issue trying to run the copy action {}", e);
                    return Err(std::io::Error::other(format!(
                        "Cannot copy a directory named '{}'",
                        e
                    )));
                }
                tracing::info!(
                    "Copied file {} to {}",
                    file_path.to_string_lossy(),
                    path_buf.as_path().to_string_lossy()
                );
            }
            crate::config::Action::Delete => {
                fs::remove_file(file_path)?;
                tracing::info!("Deleted {}", file_path.display());
            }
            crate::config::Action::Rename {
                pattern,
                replacement,
            } => {
                let parent_dir = file_path.parent();
                let replacement_file_path = parent_dir.unwrap().join(replacement);
                tracing::info!(
                    "Renaming file {} to {}",
                    file_path.to_string_lossy(),
                    replacement_file_path.to_string_lossy()
                );
                if replacement_file_path.exists() {
                    // update replacement with timestam
                    let epoch_seconds = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis();
                    let base = Path::new(replacement).file_stem().unwrap();
                    let new_file_name = format!(
                        "{}{}.{}",
                        base.to_string_lossy(),
                        epoch_seconds,
                        file_path.extension().unwrap().to_string_lossy()
                    );
                    let seq = parent_dir.unwrap().join(new_file_name);
                    tracing::debug!(
                        "The replacement file {} exits, adding seq {}",
                        replacement_file_path.to_string_lossy(),
                        seq.to_string_lossy()
                    );
                    fs::rename(file_path, seq)?;
                } else {
                    fs::rename(file_path, replacement_file_path)?;
                }
            }
        }
    }
    Ok(())
}
