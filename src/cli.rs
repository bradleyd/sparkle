use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// configuration file location
    #[arg(long, short)]
    pub configuration: String,

    /// verbosity
    #[arg(long, short, default_value_t = false)]
    pub verbose: bool,
}
