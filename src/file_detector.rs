use file_format::{FileFormat, Kind};
use std::path::Path;

use crate::file_metadata::FileType;
pub fn get_file_type(f: &Path) -> crate::file_metadata::FileType {
    let fmt = FileFormat::from_file(f);
    match fmt {
        Ok(ff) => match ff.kind() {
            Kind::Document => FileType::Document,
            Kind::Image => FileType::Image,
            Kind::Other => FileType::Unknown,
            _ => FileType::Unknown,
        },
        Err(_) => FileType::Unknown,
    }
}
