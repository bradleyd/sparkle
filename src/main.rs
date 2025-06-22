use std::path::Path;
use std::path::PathBuf;
mod content_info;
mod crawl;
mod file_detector;
mod file_metadata;
use crate::content_info::ContentInfo;
use crate::crawl::search_dir;
use crate::file_metadata::FileMetadata;
use std::env;

#[derive(Debug)]
pub struct RuleError {
    pub message: String,
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuleError {}

pub trait OrganizationRule: Send + Sync {
    fn name(&self) -> &str;
    fn applies_to(&self, context: &FileContext) -> RuleMatch;
    fn destination(&self, context: &FileContext) -> Result<PathBuf, RuleError>;
    fn priority(&self) -> u32;
    fn description(&self) -> &str;
}

pub enum RuleMatch {
    No,
    Yes,
    Conditional(String), // e.g., "if directory exists"
}

pub struct FileContext {
    pub path: PathBuf,
    pub metadata: FileMetadata,
    pub content_info: Option<ContentInfo>, // MIME type, etc.
    pub parent_dir: PathBuf,
    pub base_dir: PathBuf, // The root we're organizing from
}

pub struct ContentTypeRule {
    mime_patterns: Vec<String>,
    destination_fn: Box<dyn Fn(&FileContext) -> PathBuf + Send + Sync>,
    priority: u32,
}

fn main() {
    // Collect all arguments into a vector of Strings
    let args: Vec<String> = env::args().collect();

    // The first argument is always the program name
    println!("Program name: {}", args[0]);
    println!("Program name: {}", args[1]);

    let mut results: Vec<FileMetadata> = Vec::new();
    search_dir(Path::new(&args[1].to_string()), &mut results, false);
}
