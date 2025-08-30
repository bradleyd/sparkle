use sparkle::config::Config;
use sparkle::crawl::search_dir;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::{NamedTempFile, tempdir};

#[test]
fn test_basic_workflow_with_echo_action() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let test_file = temp_path.join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let config_content = format!(
        r#"
[[rules]]
name = "echo_test"
locations = ["{}"]
subfolders = false
filters = [
    {{ extension = "txt" }}
]
actions = [
    {{ echo = "Found text file" }}
]
"#,
        temp_path.display()
    );

    let mut config_file = NamedTempFile::new().unwrap();
    write!(config_file, "{}", config_content).unwrap();

    let config = Config::new(config_file.path().to_str().unwrap()).unwrap();
    let rule = &config.rules[0];

    let results = search_dir(temp_path, &config, rule, true, false).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, test_file);
    assert!(results[0].path.exists());
}

#[test]
fn test_workflow_with_multiple_file_types() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    fs::write(temp_path.join("code.rs"), "fn main() {}").unwrap();
    fs::write(temp_path.join("config.yml"), "key: value").unwrap();
    fs::write(temp_path.join("readme.md"), "# Title").unwrap();
    fs::write(temp_path.join("ignored.xyz"), "unknown").unwrap();

    let config_content = format!(
        r#"
[[rules]]
name = "code_files"
locations = ["{}"]
subfolders = false
filters = [
    {{ extension = "rs" }}
]
actions = [
    {{ echo = "Found Rust file" }}
]

[[rules]]
name = "config_files"
locations = ["{}"]
subfolders = false
filters = [
    {{ extension = "yml" }}
]
actions = [
    {{ echo = "Found config file" }}
]
"#,
        temp_path.display(),
        temp_path.display()
    );

    let mut config_file = NamedTempFile::new().unwrap();
    write!(config_file, "{}", config_content).unwrap();

    let config = Config::new(config_file.path().to_str().unwrap()).unwrap();

    let rust_results = search_dir(temp_path, &config, &config.rules[0], true, false).unwrap();
    let yml_results = search_dir(temp_path, &config, &config.rules[1], true, false).unwrap();

    assert_eq!(rust_results.len(), 1);
    assert!(rust_results[0].path.file_name().unwrap() == "code.rs");

    assert_eq!(yml_results.len(), 1);
    assert!(yml_results[0].path.file_name().unwrap() == "config.yml");
}

#[test]
fn test_workflow_empty_directory() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let config_content = format!(
        r#"
[[rules]]
name = "empty_test"
locations = ["{}"]
subfolders = false
filters = [
    {{ extension = "txt" }}
]
actions = [
    {{ echo = "Found file" }}
]
"#,
        temp_path.display()
    );

    let mut config_file = NamedTempFile::new().unwrap();
    write!(config_file, "{}", config_content).unwrap();

    let config = Config::new(config_file.path().to_str().unwrap()).unwrap();
    let rule = &config.rules[0];

    let results = search_dir(temp_path, &config, rule, true, false).unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_workflow_nonexistent_directory() {
    let nonexistent_path = Path::new("/nonexistent/directory");

    let config_content = format!(
        r#"
[[rules]]
name = "nonexistent_test"
locations = ["{}"]
subfolders = false
filters = [
    {{ extension = "txt" }}
]
actions = [
    {{ echo = "Found file" }}
]
"#,
        nonexistent_path.display()
    );

    let mut config_file = NamedTempFile::new().unwrap();
    write!(config_file, "{}", config_content).unwrap();

    let config = Config::new(config_file.path().to_str().unwrap()).unwrap();
    let rule = &config.rules[0];

    let results = search_dir(nonexistent_path, &config, rule, true, false).unwrap();

    assert_eq!(results.len(), 0);
}

