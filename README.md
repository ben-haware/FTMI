# FTMI - File Tools for Mass Interaction

FTMI is a Rust-based file manipulation utility focused on detecting and managing file prefixes. It provides powerful tools for analyzing file naming patterns and preparing bulk rename operations.

## Features

### 🔍 Path Extraction
Extract file paths from any text input:
```bash
echo "Check the file /home/user/doc.txt and C:\Windows\System32" | ftmi extract-paths
```

### 🏷️ Prefix Detection
Three modes for finding common prefixes in filenames:

1. **Delimiter-Only Mode**: Find prefixes within delimiters like `[Artist]`, `(Draft)`, `{ID}`
2. **Specific Prefix Mode**: Search for user-specified prefixes
3. **Detect All Mode**: Automatically discover all common prefixes

### 🎯 Prefix Removal Preview
Preview how files would look after prefix removal without actually renaming them.

## Installation

### Using cargo-binstall (fastest)
```bash
cargo binstall ftmi
```

### From source
```bash
cargo install ftmi
```

### From repository
```bash
git clone https://github.com/ben-haware/FTMI
cd FTMI
cargo build --release
```

## Usage

### Main Binary - Detect All Prefixes
```bash
# Analyze directories for common prefixes
echo "/path/to/directory" | ftmi

# Analyze multiple directories
echo -e "/path/to/music\n/path/to/documents" | ftmi
```

### Find Delimited Prefixes Only
Perfect for organized media files:
```bash
echo "/path/to/music" | ftmi find-delimited
```

Example output:
```
Directory: /path/to/music
--------------------------------------------------
Prefix: The Beatles (within []) - 5 files
  - [The Beatles] Hey Jude.mp3
  - [The Beatles] Let It Be.mp3
  - [The Beatles] Yesterday.mp3
  ...
```

### Find Specific Prefixes
Search for known prefix patterns:
```bash
echo "/path/to/photos" | ftmi find-specific IMG_ DSC_ 
```

### Extract Paths from Text
```bash
cat logfile.txt | ftmi extract-paths
```

### Preview Prefix Removal
See what files would be renamed:
```bash
# Auto-detect prefixes and show removal preview
ftmi remove-prefix auto /path/to/files

# Interactive mode
ftmi remove-prefix prefix /path/to/files
```

## Examples

### Music Library Organization
```bash
$ echo "test_data/music/summer_hits" | ftmi find-delimited

Directory: test_data/music/summer_hits
--------------------------------------------------
Prefix: Dua Lipa (within []) - 3 files
  - [Dua Lipa] Levitating.mp3
  - [Dua Lipa] Physical.mp3
  - [Dua Lipa] Don't Start Now.mp3

Prefix: The Beach Boys (within []) - 3 files
  - [The Beach Boys] Good Vibrations.mp3
  - [The Beach Boys] California Girls.mp3
  - [The Beach Boys] Surfin USA.mp3
```

### Photo Collection Analysis
```bash
$ echo "/photos/vacation" | ftmi find-specific IMG_ DSC_

Directory: /photos/vacation
--------------------------------------------------
Prefix: IMG_ - 45 files
  - IMG_2024_001.jpg
  - IMG_2024_002.jpg
  ...

Prefix: DSC_ - 12 files
  - DSC_0001.jpg
  - DSC_0002.jpg
  ...
```

### Path Extraction from Logs
```bash
$ tail -n 100 app.log | ftmi extract-paths
/var/log/app/error.log
/home/user/config/settings.json
C:\Program Files\App\config.ini
```

## Library Usage

FTMI can also be used as a library in your Rust projects:

```rust
use ftmi::{find_common_prefix, PrefixOptions, PrefixMode};

let options = PrefixOptions {
    mode: PrefixMode::DelimiterOnly {
        delimiters: vec![
            ("[".to_string(), "]".to_string()),
            ("(".to_string(), ")".to_string()),
        ],
    },
    min_occurrences: 2,
};

let prefixes = find_common_prefix(Path::new("/music"), &options)?;
```

## Project Structure

- `ftmi` - Main binary (detect all mode)
- `ftmi find-delimited` - Find only delimited prefixes
- `ftmi find-specific` - Find specific prefixes
- `ftmi detect-all` - Explicit detect all mode
- `ftmi extract-paths` - Extract paths from text
- `ftmi remove-prefix` - Preview prefix removal

## Development

### Running Tests
```bash
cargo test
```

### Building All Binaries
```bash
cargo build --release --bins
```

## License

This project is free for individual use. Corporations require a commercial license.

See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [ ] Actual file renaming functionality (currently preview only)
- [ ] Recursive directory scanning
- [ ] Custom delimiter configuration
- [ ] Undo/redo functionality
- [ ] Dry-run mode with detailed reports
- [ ] Pattern-based renaming rules