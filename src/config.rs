use serde::Deserialize;
use std::fmt::write;
use std::path::PathBuf;
use std::{fmt, fs};

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
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Echo(_) => write!(f, "Echo"),
            Action::Move(path_buf) => write!(f, "Move"),
            Action::Copy(path_buf) => write!(f, "Copy"),
            Action::Delete => write!(f, "Delete"),
            Action::Rename {
                pattern,
                replacement,
            } => write!(f, "Rename"),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Filter {
    Extension {
        extension: String,
    },
    NameContains {
        name_contains: String,
    },
    Age {
        days_older_than: Option<u32>,
    },
    Size {
        size_gt: Option<u64>,
        size_lt: Option<u64>,
    },
}

impl Config {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_new_valid_toml() {
        let toml_content = r#"
[[rules]]
name = "test_rule"
locations = ["/tmp"]
subfolders = true
filters = [
    { extension = "txt" }
]
actions = [
    { echo = "Found file" }
]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();
        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "test_rule");
        assert_eq!(config.rules[0].locations.len(), 1);
        assert!(config.rules[0].subfolders);
        assert_eq!(config.rules[0].filters.len(), 1);
        assert_eq!(config.rules[0].actions.len(), 1);
    }

    #[test]
    fn test_config_new_invalid_toml() {
        let invalid_toml = "invalid toml content [[[";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", invalid_toml).unwrap();

        let result = Config::new(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_config_new_nonexistent_file() {
        let result = Config::new("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_new_empty_rules() {
        let toml_content = r#"
rules = []
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();
        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.rules.len(), 0);
    }

    #[test]
    fn test_filter_variants() {
        let toml_content = r#"
[[rules]]
name = "multi_filter_rule"
locations = ["/tmp"]
subfolders = false
filters = [
    { extension = "log" },
    { size_gt = 1024, size_lt = 1048576 },
    { days_older_than = 30 },
    { name_contains = "backup" }
]
actions = [
    { echo = "Processing" }
]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();
        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.rules[0].filters.len(), 4);

        match &config.rules[0].filters[0] {
            Filter::Extension { extension } => assert_eq!(extension, "log"),
            _ => panic!("Expected Extension filter"),
        }

        match &config.rules[0].filters[3] {
            Filter::NameContains { name_contains } => assert_eq!(name_contains, "backup"),
            _ => panic!("Expected NameContains filter"),
        }
    }

    #[test]
    fn test_action_variants() {
        let toml_content = r#"
[[rules]]
name = "action_test"
locations = ["/tmp"]
subfolders = false
filters = [{ extension = "tmp" }]
actions = [
    { echo = "Test message" },
    { move = "/archive" },
    { copy = "/backup" },
    "delete",
    { rename = { pattern = "old", replacement = "new" } },
    { set_permissions = 644 }
]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();
        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.rules[0].actions.len(), 6);

        match &config.rules[0].actions[0] {
            Action::Echo(msg) => assert_eq!(msg, "Test message"),
            _ => panic!("Expected Echo action"),
        }
    }
    #[test]
    fn test_name_contains() {
        let toml_content = r#"
[[rules]]
name = "action_test"
locations = ["/tmp"]
subfolders = false
filters = [{ name_contains = "tmp" }]
actions = [
    { echo = "Test message" },
    { move = "/archive" },
    { copy = "/backup" },
    "delete",
    { rename = { pattern = "old", replacement = "new" } },
    { set_permissions = 644 }
]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();
        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.rules[0].actions.len(), 6);

        match &config.rules[0].actions[0] {
            Action::Echo(msg) => assert_eq!(msg, "Test message"),
            _ => panic!("Expected Echo action"),
        }
    }
}
