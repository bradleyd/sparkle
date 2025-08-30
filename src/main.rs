mod cli;
mod config;
mod content_info;
mod crawl;
mod file_detector;
mod file_metadata;
mod handlers;
mod utils;
use crate::cli::Cli;
use crate::crawl::search_dir;
use crate::file_metadata::FileContext;
use clap::Parser;
use comfy_table::Table;
use file_metadata::FileMetadataError;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // TODO create default directories to move files into using cli
    // TODO add dry-run
    // TODO rename when using a glob filter like name_contains, will overwrite the destionation
    // file with the last file from the filter applied.
    let cli = Cli::parse();
    let config = config::Config::new(&cli.configuration).expect("Cannot parse config");
    tracing::debug!("config: {:?}", config);
    let results: Vec<Result<FileContext, FileMetadataError>> = config
        .rules
        .iter()
        .flat_map(|rule| {
            rule.locations.iter().flat_map(|d| {
                match search_dir(d, &config, rule, cli.verbose, cli.dry_run) {
                    Ok(file_contexts) => file_contexts.into_iter().map(Ok).collect::<Vec<_>>(),
                    Err(e) => vec![Err(e)],
                }
            })
        })
        .collect();

    tracing::info!("✨Sparkled!");
    tracing::info!(files = results.len(), "Files scanned");

    for f in results {
        match f {
            Ok(fmeta) => {
                let mut table = Table::new();
                table.set_header(vec!["Name", "Mime", "Type"]).add_row(vec![
                    fmeta.path.to_string_lossy(),
                    std::borrow::Cow::Borrowed(fmeta.metadata.file_type.mime_hint.unwrap()),
                    std::borrow::Cow::Borrowed(fmeta.metadata.file_type.category.as_str()),
                ]);

                println!("{table}");
            }
            Err(e) => println!("Got error from results {:?}", e),
        }
    }
}
