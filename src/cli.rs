use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// configuration file location
    #[arg(long, short)]
    pub configuration: String,
}
