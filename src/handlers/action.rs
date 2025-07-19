use std::io::Error;
use std::path::Path;
use std::{fs, io};

pub fn run(actions: &[crate::config::Action], file_path: &Path) -> Result<(), Error> {
    for action in actions {
        match action {
            crate::config::Action::Echo(message) => {
                tracing::info!("running action echo: {}", message);
            }
            crate::config::Action::Move(path_buf) => {
                tracing::info!("moving file to {}", path_buf.to_string_lossy());
                if let Err(e) = crate::utils::move_file(&file_path, &path_buf) {
                    tracing::error!("There was an issue trying to run the move action {}", e);
                    return Err(Error::new(
                        io::ErrorKind::Other,
                        format!("Cannot move a directory named '{}'", e.to_string()),
                    ));
                };
                tracing::info!(
                    "Moved file {} to {}",
                    file_path.to_string_lossy(),
                    path_buf.as_path().to_string_lossy()
                );
            }
            crate::config::Action::Copy(path_buf) => {
                tracing::info!("coping file to {}", path_buf.to_string_lossy());
                if let Err(e) = crate::utils::copy_file(&file_path, &path_buf) {
                    tracing::error!("There was an issue trying to run the copy action {}", e);
                    return Err(Error::new(
                        io::ErrorKind::Other,
                        format!("Cannot copy a directory named '{}'", e.to_string()),
                    ));
                }
                tracing::info!(
                    "Copied file {} to {}",
                    file_path.to_string_lossy(),
                    path_buf.as_path().to_string_lossy()
                );
            }
            crate::config::Action::Delete => {
                fs::remove_file(file_path)?; // propagates error if delete fails
                tracing::info!("Deleted {}", file_path.display());
            }
            crate::config::Action::Rename {
                pattern,
                replacement,
            } => {
                tracing::info!("running rename action");
            }
            crate::config::Action::Compress { format } => {
                tracing::info!("Running the compress action {:?}", format);
            }
            crate::config::Action::SetPermissions(_) => {
                tracing::info!("Running the set permissions action");
            }
        }
    }
    Ok(())
}
