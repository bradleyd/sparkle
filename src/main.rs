use std::fs;

use clap::Parser;
use comfy_table::Table;
use directories::ProjectDirs;
use sparkle::cli::Cli;
use sparkle::crawl::search_dir;
use sparkle::database;
use sparkle::file_metadata::{FileContext, FileMetadataError};
use sparkle::{config, utils};
use thiserror::Error;
use tracing_appender::rolling;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, prelude::*};

#[derive(Debug, Error)]
pub enum CliError {
    #[error("failed to parse log level: {0}")]
    EnvFilter(#[from] tracing_subscriber::filter::FromEnvError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse filter: {0}")]
    From(#[from] tracing_subscriber::filter::ParseError),
}

fn setup_logging(
    level: &str,
) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>, CliError> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "sparkle") {
        let config_dir = proj_dirs.config_dir();
        let log_dir = &config_dir.join("logs");
        fs::create_dir_all(log_dir)?;
        // Daily rolling JSON file
        let file_appender = rolling::daily(log_dir, "sparkle.jsonl");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // JSON layer to file
        let json_layer = fmt::layer()
            .event_format(fmt::format().json())
            .with_writer(non_blocking)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_ansi(false);

        let filter = EnvFilter::try_new(level)?;

        tracing_subscriber::registry()
            .with(filter)
            .with(json_layer)
            .init();

        Ok(Some(guard))
    } else {
        panic!("Cannot setup configuratgion directory");
    }
}

fn main() {
    //tracing_subscriber::fmt()
    //    .with_env_filter(EnvFilter::from_default_env())
    //    .init();

    // TODO create default directories to move files into using cli
    // TODO add dry-run
    // TODO rename when using a glob filter like name_contains, will overwrite the destionation
    // file with the last file from the filter applied.
    let cli = Cli::parse();
    let _guard = setup_logging("debug");
    let config = config::Config::new(&cli.configuration).expect("Cannot parse config");
    tracing::debug!("sparkle config: {:?}", config);
    let conn = match database::DatabaseConnection::new_file("sparkle.db") {
        Ok(c) => c,
        Err(e) => panic!("Cannot create database connection {}", e),
    };

    if let Err(e) = database::run_migrations(&conn) {
        panic!("Migrations failed with {}", e);
    }

    let banner = r#"
     __                  _    _      
/ _\_ __   __ _ _ __| | _| | ___ 
\ \| '_ \ / _` | '__| |/ / |/ _ \
_\ \ |_) | (_| | |  |   <| |  __/
\__/ .__/ \__,_|_|  |_|\_\_|\___|
   |_|                           
"#;
    println!("{banner}");
    tracing::debug!("Ran DB migrations successfully!");

    let results: Vec<Result<FileContext, FileMetadataError>> = config
        .rules
        .iter()
        .flat_map(|rule| {
            rule.locations.iter().flat_map(|d| {
                match search_dir(d, &config, rule, cli.verbose, cli.dry_run, &conn) {
                    Ok(file_contexts) => file_contexts.into_iter().map(Ok).collect::<Vec<_>>(),
                    Err(e) => vec![Err(e)],
                }
            })
        })
        .collect();

    utils::log_to_terminal("✨Sparkled!");
    utils::log_to_terminal(format!("🧽Files cleaned {}", results.len()).as_str());

    if cli.verbose && !results.is_empty() {
        let mut table = Table::new();
        table.set_header(vec!["Name", "Mime", "Type"]);
        for f in results {
            match f {
                Ok(fmeta) => {
                    table.add_row(vec![
                        fmeta.path.to_string_lossy(),
                        std::borrow::Cow::Borrowed(fmeta.metadata.file_type.mime_hint.unwrap()),
                        std::borrow::Cow::Borrowed(fmeta.metadata.file_type.category.as_str()),
                    ]);
                }
                Err(e) => println!("Got error from results {:?}", e),
            }
        }
        println!("{table}");
    }

    utils::log_to_terminal("Done ✅");
}
