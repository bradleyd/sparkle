# Sparkle ✨

A fast, rule-based file organization and management tool written in Rust. Sparkle helps you automatically organize, process, and manage files based on configurable rules and filters.

## Features

- **Rule-based file processing** - Define custom rules with filters and actions
- **Multiple filter types** - Filter by extension, size, age, and name patterns
- **Flexible actions** - Move, copy, delete, rename files, or execute custom actions
- **Fast directory traversal** - Efficient recursive directory scanning
- **Safe operations** - Comprehensive error handling and validation
- **Configurable** - TOML-based configuration with multiple rules support

## Installation

### From Source
```bash
git clone https://github.com/bradleyd/sparkle.git
cd sparkle
cargo build --release
```

The binary will be available at `target/release/sparkle`.

## Quick Start

1. **Create a configuration file** (copy from `config.toml.example`):
```toml
[[rules]]
name = "Organize PDFs"
locations = ["/home/user/Downloads"]
subfolders = true

[[rules.filters]]
extension = "pdf"

[[rules.actions]]
move = "/home/user/Documents/PDFs"

[[rules]]
name = "Clean old logs"
locations = ["/var/log"]
subfolders = false

[[rules.filters]]
extension = "log"
days_older_than = 30

[[rules.actions]]
echo = "Found old log file"
```

2. **Run Sparkle**:
```bash
sparkle --configuration config.toml
```

## Configuration

Sparkle uses TOML configuration files with the following structure:

### Rules
Each rule defines a set of locations to scan, filters to match files, and actions to perform:

```toml
[[rules]]
name = "Rule description"
locations = ["/path/to/scan"]
subfolders = true  # or false
filters = [...]
actions = [...]
```

### Filters

| Filter Type | Description | Example |
|-------------|-------------|---------|
| `extension` | Match file extension | `{ extension = "jpg" }` |
| `size` | Match file size range | `{ size_gt = 1024, size_lt = 1048576 }` |
| `age` | Match files older than N days | `{ days_older_than = 30 }` |
| `name` | Match filename contains string | `{ name = "backup" }` |

### Actions

| Action | Description | Example |
|--------|-------------|---------|
| `echo` | Print message | `{ echo = "Found file" }` |
| `move` | Move file to directory | `{ move = "/archive" }` |
| `copy` | Copy file to directory | `{ copy = "/backup" }` |
| `delete` | Delete file | `"delete"` |
| `rename` | Rename using pattern | `{ rename = { pattern = "old", replacement = "new" } }` |
| `set_permissions` | Set file permissions | `{ set_permissions = 644 }` |

## File Type Detection

Sparkle automatically detects file types based on:
- File extensions (prioritized)
- MIME type detection
- Content analysis fallback

Supported categories:
- **Code**: `.rs`, `.js`, `.java`, `.go`, `.rb`, `.ex`
- **Documents**: `.md`, `.pdf`
- **Configuration**: `.yml`, `.yaml`, `.toml`
- **Text**: `.txt`
- **Images**: `.jpg`, `.png`, `.gif`, `.bmp`, `.svg`
- **Archives**: Various compressed formats

## Development

### Running Tests
```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test
```

### Project Structure
```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library crate root
├── cli.rs            # Command-line interface
├── config.rs         # Configuration parsing
├── file_detector.rs  # File type detection
├── file_metadata.rs  # File metadata extraction
├── crawl.rs          # Directory traversal
├── handlers/         # Action implementations
├── utils.rs          # Utility functions
└── metrics.rs        # Performance metrics

tests/
└── integration_test.rs  # End-to-end tests
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## Examples

### Organize Downloads by Type
```toml
[[rules]]
name = "Images to Pictures"
locations = ["/home/user/Downloads"]
subfolders = false
filters = [{ extension = "jpg" }, { extension = "png" }]
actions = [{ move = "/home/user/Pictures" }]

[[rules]]
name = "Documents to Docs"
locations = ["/home/user/Downloads"]
subfolders = false
filters = [{ extension = "pdf" }, { extension = "docx" }]
actions = [{ move = "/home/user/Documents" }]
```

### Clean Up Old Files
```toml
[[rules]]
name = "Archive old downloads"
locations = ["/home/user/Downloads"]
subfolders = true
filters = [{ days_older_than = 90 }]
actions = [{ move = "/home/user/Archive" }]
```

### Backup Important Files
```toml
[[rules]]
name = "Backup configs"
locations = ["/etc"]
subfolders = true
filters = [{ extension = "conf" }, { extension = "cfg" }]
actions = [{ copy = "/backup/configs" }]
```

## Safety

- Sparkle performs dry-run validation before executing destructive operations
- All file operations include comprehensive error handling
- Symlinks are handled safely to prevent infinite loops
- Permission errors are logged and do not halt processing

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Changelog

### v0.1.0 (Current)
- Initial release with core functionality
- Rule-based file processing
- Multiple filter and action types
- Comprehensive test suite
- TOML configuration support