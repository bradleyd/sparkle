use std::path::Path;
use std::{
    fs,
    io::{self},
};

use serde::de;

pub fn move_file(source_path: &Path, destination_path: &Path) -> std::io::Result<()> {
    let file_name = source_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "source has no filename"))?;

    let destination = destination_path.join(file_name);
    println!(
        "source {} to destination {}",
        source_path.to_string_lossy(),
        destination.to_string_lossy()
    );

    let source_meta = fs::metadata(source_path)?;
    if source_meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::IsADirectory,
            format!(
                "Cannot move a directory named '{}'",
                source_path.to_string_lossy()
            ),
        ));
    }

    //let dest_meta = fs::metadata(destination_path)?;
    //if dest_meta.is_dir() {
    //    // This is a file move, not a directory move, so we rename the file and remove the containing directory.
    //    fs::remove_dir_all(destination_path)?;
    //    return fs::rename(source_path, destination_path);
    //}

    // Both are files, so let's do the move.
    fs::rename(source_path, destination)
}

pub fn copy_file(source_path: &Path, destination_path: &Path) -> std::io::Result<u64> {
    let file_name = source_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "source has no filename"))?;

    let destination = destination_path.join(file_name);

    println!(
        "source {} to destination {}",
        source_path.to_string_lossy(),
        destination_path.to_string_lossy()
    );
    let source_meta = fs::metadata(source_path)?;
    if source_meta.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::IsADirectory,
            format!(
                "Cannot move a directory named '{}'",
                source_path.to_string_lossy()
            ),
        ));
    }

    //let dest_meta = fs::metadata(destination_path)?;
    //if dest_meta.is_dir() {
    //    // This is a file move, not a directory move, so we rename the file and remove the containing directory.
    //    fs::remove_dir_all(destination_path)?;
    //    return fs::copy(source_path, destination_path);
    //}

    fs::copy(source_path, destination)
}
