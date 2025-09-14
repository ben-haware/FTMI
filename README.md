# FTMI - File Tools for Mass Interaction

FTMI is a Rust-based file manipulation utility focused on detecting and managing file prefixes. It provides powerful tools for analyzing file naming patterns and preparing bulk rename operations.

## Quick Install

```bash
git clone https://github.com/ben-haware/FTMI && cd FTMI && make install
```

## Features

### 🔍 Path Extraction
Extract file paths from any text input:
```bash
echo "Check the file /home/user/doc.txt and C:\Windows\System32" | ftmi extract-paths
```

### 🏷️ Prefix Detection
Find the longest matching prefixes in your directories:

- **Main Mode**: Finds longest bracket-delimited prefixes `[Artist]` with highest occurrence count
- **Returns multiple results** when there are ties (e.g., multiple artists with same number of songs)
- **Structured output** with `PrefixedPath` containing file paths and prefix information

Additional specialized modes:
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

### Main Binary - Find Longest Bracket Prefixes
```bash
# Find longest bracket-delimited prefixes in a directory
echo "/path/to/music" | ftmi

# Analyze multiple directories
echo -e "/path/to/music\n/path/to/documents" | ftmi
```

Example output:
```
Directory: test_data/music/summer_hits
Prefix: Dua Lipa
Files (3):
  [Dua Lipa] Levitating.mp3
  [Dua Lipa] Don't Start Now.mp3
  [Dua Lipa] Physical.mp3

Prefix: The Beach Boys
Files (3):
  [The Beach Boys] California Girls.mp3
  [The Beach Boys] Surfin USA.mp3
  [The Beach Boys] Good Vibrations.mp3
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
$ echo "test_data/music/indie_collection" | ftmi

Directory: test_data/music/indie_collection
Prefix: Arctic Monkeys
Files (3):
  [Arctic Monkeys] 505.mp3
  [Arctic Monkeys] R U Mine.mp3
  [Arctic Monkeys] Do I Wanna Know.mp3

Prefix: Tame Impala
Files (3):
  [Tame Impala] The Less I Know The Better.mp3
  [Tame Impala] Elephant.mp3
  [Tame Impala] Feels Like We Only Go Backwards.mp3
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
use ftmi::{find_longest_prefix, PrefixOptions, PrefixedPath};
use std::path::Path;

let options = PrefixOptions::default();
let results: Vec<PrefixedPath> = find_longest_prefix(Path::new("/music"), &options)?;

for result in results {
    println!("Prefix: {}", result.prefix);
    println!("Files: {:?}", result.paths);
}
```

For more control, use the detailed API:
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

- `ftmi` - **Main binary (longest bracket prefix mode)**
- `ftmi find-delimited` - Find only delimited prefixes
- `ftmi find-specific` - Find specific prefixes
- `ftmi detect-all` - Explicit detect all mode  
- `ftmi extract-paths` - Extract paths from text
- `ftmi remove-prefix` - Preview prefix removal

**New in latest version:**
- `PrefixedPath` struct with `Vec<PathBuf>` and `prefix` fields
- Multiple results returned for tied occurrence counts
- Prioritizes bracket-delimited prefixes `[.*]` matching regex pattern

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