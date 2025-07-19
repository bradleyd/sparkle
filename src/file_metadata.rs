use crate::content_info::ContentInfo;
use crate::file_detector::{get_age_category, get_file_size_category, get_file_type};
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, Metadata};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub enum FileMetadataError {
    NoMetaData,
    InvalidInput(String),
    NoMtime(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for FileMetadataError {
    fn from(err: std::io::Error) -> FileMetadataError {
        FileMetadataError::Io(err)
    }
}
impl fmt::Display for FileMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileMetadataError::NoMetaData => write!(f, "Can't access file metadata"),
            FileMetadataError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            FileMetadataError::Io(e) => write!(f, "IO error: {}", e),
            FileMetadataError::NoMtime(msg) => {
                write!(f, "Could not get mtime for the file: {}", msg)
            }
        }
    }
}

impl std::error::Error for FileMetadataError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FileMetadataError::Io(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct FileContext {
    pub path: PathBuf,
    pub metadata: FileMetadata,
    pub content_info: Option<ContentInfo>, // MIME type, etc.
    pub parent_dir: PathBuf,
    pub base_dir: PathBuf, // The root we're organizing from
}

#[derive(Debug)]
pub struct FileMetadata {
    // Core filesystem info
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: SystemTime,
    pub accessed: Option<SystemTime>,
    pub permissions: std::fs::Permissions,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,

    // Platform-specific extended attributes (macOS/Linux)
    pub extended_attributes: HashMap<String, Vec<u8>>,

    // Derived/computed info
    pub size_category: SizeCategory, // Tiny, Small, Medium, Large, Huge
    pub age_category: AgeCategory,   // Recent, Week, Month, Year, Old
    pub file_type: FileType,         // Based on extension + content
}

impl Clone for FileMetadata {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            created: self.created,
            modified: self.modified,
            accessed: self.accessed,
            permissions: std::fs::Permissions::from_mode(self.permissions.mode()),
            is_file: self.is_file,
            is_dir: self.is_dir,
            is_symlink: self.is_symlink,
            extended_attributes: self.extended_attributes.clone(),
            size_category: self.size_category.clone(),
            age_category: self.age_category.clone(),
            file_type: self.file_type.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SizeCategory {
    Tiny,   // < 1KB
    Small,  // 1KB - 1MB
    Medium, // 1MB - 100MB
    Large,  // 100MB - 1GB
    Huge,   // > 1GB
}

#[derive(Debug, Clone)]
pub enum AgeCategory {
    Recent, // < 1 day
    Week,   // 1-7 days
    Month,  // 1-30 days
    Year,   // 30 days - 1 year
    Old,    // > 1 year
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Configuration,
    Text,
    Unknown,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Document => "document",
            FileType::Image => "image",
            FileType::Video => "video",
            FileType::Audio => "audio",
            FileType::Archive => "archive",
            FileType::Code => "code",
            FileType::Text => "text",
            FileType::Configuration => "configuration",
            FileType::Unknown => "unknown",
        }
    }
}

impl FileMetadata {
    pub fn build(path: &Path, quiet: bool) -> Result<FileMetadata, FileMetadataError> {
        let metadata = fs::metadata(path)?;
        match metadata.modified() {
            Ok(modified_time) => {
                let fm = FileMetadata {
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
                    file_type: get_file_type(path),
                };
                Ok(fm)
            }
            Err(e) => {
                if !quiet {
                    tracing::error!(
                        "Warning: Could not get modification time for {}: {}",
                        path.display(),
                        e
                    );
                }
                Err(FileMetadataError::NoMtime(e.to_string()))
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
