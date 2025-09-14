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
Find the longest matching prefixes in your directories with configurable regex filtering:

- **Main Mode**: Finds longest prefixes matching regex pattern (default: `\[.*\]` for bracket-delimited)
- **Configurable filtering**: Use `--regex` to specify custom pattern or `--no-filter` for all prefixes
- **Returns multiple results** when there are ties (e.g., multiple artists with same number of songs)
- **Structured output** with `PrefixedPath` containing file paths and prefix information

Additional specialized modes:
1. **Delimiter-Only Mode**: Find prefixes within delimiters like `[Artist]`, `(Draft)`, `{ID}`
2. **Specific Prefix Mode**: Search for user-specified prefixes  
3. **Detect All Mode**: Automatically discover all common prefixes

### 🎯 Interactive Prefix Removal
Interactive tool to remove prefixes from files with undo capability:

- **Preview before rename**: See exactly what files will be renamed to
- **Configurable regex filtering**: Target specific prefix patterns
- **Interactive confirmation**: Y/n/s (yes/no/skip) with Y as default
- **Multiple input sources**: Command line args + stdin piping
- **Relative path support**: Works with `./music`, `../photos`, etc.
- **Undo capability**: SQLite database tracks renames for rollback (coming soon)

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

### Main Binary - Find Longest Prefixes
```bash
# Find longest bracket-delimited prefixes in a directory
echo "/path/to/music" | ftmi

# Analyze multiple directories
echo -e "/path/to/music\n/path/to/documents" | ftmi
```

### Interactive Prefix Removal - Main Tool
```bash
# Default: Remove bracket-delimited prefixes interactively
ftmi rename ./music

# Custom regex: Remove parentheses-delimited prefixes
ftmi rename --regex '\(.*\)' ./music

# Multiple sources: Command line + piped directories
echo "./photos" | ftmi rename ./music ./docs

# No filtering: Show all prefixes for selection
ftmi rename --no-filter ./mixed_files
```

Example output:
```
🔧 FTMI Interactive Prefix Removal Tool
🔍 Using regex filter: \[.*\]
📊 Processing 1 directories total

📁 Directory: ./music
🏷️  Prefix 1: [Dua Lipa]
   Files (3):
   [Dua Lipa] Levitating.mp3 → Levitating.mp3
   [Dua Lipa] Don't Start Now.mp3 → Don't Start Now.mp3
   [Dua Lipa] Physical.mp3 → Physical.mp3

💡 Remove prefix [Dua Lipa] from these 3 files? (Y/n/s=skip, default=Y): 
✅ Proceeding with prefix removal...
   🔄 Renaming: [Dua Lipa] Levitating.mp3 → Levitating.mp3
   ✓ Success!
📊 Results: 3 successful, 0 failed
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

**Helper functions for common patterns:**
```rust
use ftmi::PrefixOptions;

// Custom regex pattern
let options = PrefixOptions::with_regex(r"\(.*\)"); // Parentheses-delimited

// No filtering (all prefixes)
let options = PrefixOptions::no_filter();

// Specific delimiter types
let options = PrefixOptions::bracket_only(); // [Artist]
let options = PrefixOptions::paren_only();   // (Draft)
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
    filter_regex: Some(r"\[.*\]".to_string()),
};

let prefixes = find_common_prefix(Path::new("/music"), &options)?;
```

## Project Structure

- `ftmi` - **Main binary (longest prefix detection)**
- `ftmi rename` - **Interactive prefix removal tool (main functionality)**
- `ftmi find-delimited` - Find only delimited prefixes
- `ftmi find-specific` - Find specific prefixes
- `ftmi detect-all` - Explicit detect all mode  
- `ftmi extract-paths` - Extract paths from text
- `ftmi remove-prefix` - Preview prefix removal

**New in latest version:**
- `PrefixedPath` struct with `Vec<PathBuf>` and `prefix` fields
- **Configurable regex filtering** for prefix matching (`--regex`, `--no-filter`)
- Multiple results returned for tied occurrence counts
- **Interactive rename tool** with preview and confirmation
- **Multiple input sources** (command line + stdin piping)
- **Relative path support** for convenience

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