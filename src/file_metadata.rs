use std::collections::HashMap;
use std::time::SystemTime;

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

pub enum SizeCategory {
    Tiny,   // < 1KB
    Small,  // 1KB - 1MB
    Medium, // 1MB - 100MB
    Large,  // 100MB - 1GB
    Huge,   // > 1GB
}

pub enum AgeCategory {
    Recent, // < 1 day
    Week,   // 1-7 days
    Month,  // 1-30 days
    Year,   // 30 days - 1 year
    Old,    // > 1 year
}

pub enum FileType {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Executable,
    SystemFile,
    Unknown,
}
