use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// configuration file location
    #[arg(long, short)]
    pub configuration: String,

    /// verbosity
    #[arg(long, short, default_value_t = false)]
    pub verbose: bool,

    /// dry-run
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}
