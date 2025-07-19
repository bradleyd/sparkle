use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub name: String,
    pub locations: Vec<PathBuf>,
    pub subfolders: bool,
    pub filters: Vec<Filter>,
    pub actions: Vec<Action>,
}

#[derive(Deserialize, Debug)]
pub struct CompressionFormat {}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Echo(String),
    Move(PathBuf),
    Copy(PathBuf),
    Delete,
    Rename {
        pattern: String,
        replacement: String,
    },
    Compress {
        format: CompressionFormat,
    },
    SetPermissions(u32),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Filter {
    Extension {
        extension: String,
    },
    Size {
        size_gt: Option<u64>,
        size_lt: Option<u64>,
    },
    Age {
        days_older_than: Option<u32>,
    },
    NameContains {
        name: String,
    },
}

impl Config {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
