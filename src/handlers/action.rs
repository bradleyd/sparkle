use std::error::Error;
use std::io;
use std::path::Path;

pub enum ActionStatus {
    Success,
    Failure(String),
}

pub fn run(
    action: &crate::config::Action,
    rule: &crate::config::Rule,
    file_path: &Path,
) -> io::Result<ActionStatus> {
    match action {
        crate::config::Action::Echo(message) => {
            println!("running action echo: {}", message);
            Ok(ActionStatus::Success)
        }
        crate::config::Action::Move(path_buf) => {
            println!("moving file to {}", path_buf.to_string_lossy());
            if let Err(e) = crate::utils::move_file(&file_path, &path_buf) {
                eprintln!("There was an issue trying to run the move action {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot move a directory named '{}'", e.to_string()),
                ));
            };
            Ok(ActionStatus::Success)
        }
        crate::config::Action::Copy(path_buf) => {
            println!("coping file to {}", path_buf.to_string_lossy());
            if let Err(e) = crate::utils::copy_file(&file_path, &path_buf) {
                eprintln!("There was an issue trying to run the copy action {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Cannot copy a directory named '{}'", e.to_string()),
                ));
            }
            Ok(ActionStatus::Success)
        }
        crate::config::Action::Delete => todo!(),
        crate::config::Action::Rename {
            pattern,
            replacement,
        } => {
            println!("running rename action");
            Ok(ActionStatus::Success)
        }
        crate::config::Action::Compress { format } => {
            println!("Running the compress action");
            Ok(ActionStatus::Success)
        }
        crate::config::Action::SetPermissions(_) => {
            println!("Running the set permissions action");
            Ok(ActionStatus::Success)
        }
    }
}
