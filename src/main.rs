use std::path::Path;
use std::path::PathBuf;
mod config;
mod content_info;
mod crawl;
mod file_detector;
mod file_metadata;
mod handlers;
mod rule;
mod utils;
use crate::content_info::ContentInfo;
use crate::crawl::search_dir;
use crate::file_metadata::FileMetadata;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// directory to search
    #[arg(long, short)]
    directory: String,
}

#[derive(Debug)]
pub struct FileContext {
    pub path: PathBuf,
    pub metadata: FileMetadata,
    pub content_info: Option<ContentInfo>, // MIME type, etc.
    pub parent_dir: PathBuf,
    pub base_dir: PathBuf, // The root we're organizing from
}

fn main() {
    let cli = Cli::parse();
    let config = config::Config::new("./config.toml.example").expect("Cannot parse config");
    let mut results: Vec<FileContext> = Vec::new();
    search_dir(Path::new(&cli.directory), &mut results, &config, false);
    for f in results.iter() {
        let mime = f
            .content_info
            .as_ref()
            .map(|c| c.mime_type.as_str())
            .unwrap_or("application/octet-stream");
        let ftype = &f.metadata.file_type;

        println!(
            "name {}, type {}, ftype: {}",
            f.path.as_path().to_string_lossy(),
            mime,
            ftype.as_str()
        );
    }
    // next phase is rule phase
}
