use std::time::Duration;
use std::time::SystemTime;

#[derive(Debug, Default)]
pub struct ContentInfo {
    // MIME type detection
    pub mime_type: String,    // "image/jpeg", "text/plain", etc.
    pub mime_confidence: f32, // How confident the detection is

    // Content-specific metadata
    pub media_info: Option<MediaInfo>,
    pub document_info: Option<DocumentInfo>,
    pub archive_info: Option<ArchiveInfo>,

    // Content analysis
    pub text_encoding: Option<String>, // UTF-8, ASCII, etc.
    pub language: Option<String>,      // For text files
    pub hash: Option<String>,          // SHA-256 for deduplication
    pub has_metadata: bool,            // EXIF, ID3 ta
}

#[derive(Debug)]
pub struct MediaInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<Duration>,     // For video/audio
    pub date_taken: Option<SystemTime>, // From EXIF
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps_coordinates: Option<(f64, f64)>,
}

#[derive(Debug)]
pub struct DocumentInfo {
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
    pub author: Option<String>,
    pub title: Option<String>,
    pub created_date: Option<SystemTime>,
}

#[derive(Debug)]
pub struct ArchiveInfo {
    pub format: String, // "zip", "tar.gz", etc.
    pub file_count: u32,
    pub uncompressed_size: u64,
    pub compression_ratio: f32,
}
