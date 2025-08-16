mod cli;
mod config;
mod content_info;
mod crawl;
mod file_detector;
mod file_metadata;
mod handlers;
mod metrics;
mod utils;
use crate::cli::Cli;
use crate::crawl::search_dir;
use crate::file_metadata::FileContext;
use clap::Parser;
use file_metadata::FileMetadataError;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let config = config::Config::new(&cli.configuration).expect("Cannot parse config");
    let results: Vec<Result<FileContext, FileMetadataError>> = config
        .rules
        .iter()
        .flat_map(|rule| {
            rule.locations
                .iter()
                .flat_map(|d| match search_dir(d, &config, rule, cli.verbose) {
                    Ok(file_contexts) => file_contexts.into_iter().map(Ok).collect::<Vec<_>>(),
                    Err(e) => vec![Err(e)],
                })
        })
        .collect();
    //println!("Results: {:?}", results);
    tracing::info!(files = results.len(), "Files scanned");

    //for f in results.iter() {
    //    let mime = f
    //        .content_info
    //        .as_ref()
    //        .map(|c| c.mime_type.as_str())
    //        .unwrap_or("application/octet-stream");
    //    let ftype = &f.metadata.file_type;

    //    println!(
    //        "name {}, type {}, ftype: {}",
    //        f.path.as_path().to_string_lossy(),
    //        mime,
    //        ftype.as_str()
    //    );
    //}
    // next phase is rule phase
}
