# FTMI - File Tools for Mass Interaction

FTMI is a Rust-based file renaming utility that intelligently detects and removes file prefixes. Built for efficiency with SQLite-powered undo capabilities and interactive workflows.

## 🚀 Quick Start

```bash
# Install via cargo-binstall (fastest)
cargo binstall ftmi

# Interactive prefix removal (main functionality)
rename ./music

# Continuous mode for multiple directories
rename --continuous

# Undo the last operation
rename --undo
```

## ✨ Key Features

### 🎯 **Smart Interactive Renaming**
- **Preview before rename**: See exactly what files will be renamed
- **Multiple prefixes**: Handles ties (e.g., multiple artists with same song count)
- **Regex filtering**: Target specific patterns like `[Artist]`, `(Draft)`, `IMG_*`
- **Continuous mode**: Paste multiple directory paths for batch processing
- **200ms debounce**: Prevents screen tearing during rapid input

### 🔄 **Complete Undo System**
- **SQLite database**: Tracks every rename operation with timestamps
- **One-click undo**: `rename --undo` restores the most recent operation
- **Operation history**: `rename --list` shows all recent operations
- **Selective undo**: `rename --undo <operation_id>` for specific operations
- **Preview before undo**: See exactly what will be restored

### 🔍 **Advanced Prefix Detection**
- **Bracket-delimited**: `[Artist] Song.mp3` → `Song.mp3`
- **Parentheses**: `(Draft) Document.pdf` → `Document.pdf`
- **Custom patterns**: Use `--regex` for any pattern
- **Multiple results**: Returns all prefixes with highest occurrence count

## 📖 Usage Examples

### Basic Interactive Renaming
```bash
rename ./music
```
```
🔧 FTMI Interactive Prefix Removal Tool
📁 Directory: ./music
Found 1 prefix group(s) with highest occurrence count:

🏷️  Prefix 1: [Dua Lipa]
   Files (3):
   [Dua Lipa] Levitating.mp3 → Levitating.mp3
   [Dua Lipa] Don't Start Now.mp3 → Don't Start Now.mp3
   [Dua Lipa] Physical.mp3 → Physical.mp3

💡 Remove prefix [Dua Lipa] from these 3 files? (Y/n/s=skip, default=Y): y
✅ Proceeding with prefix removal...
   🔄 Renaming: [Dua Lipa] Levitating.mp3 → Levitating.mp3
   ✓ Success!
📊 Results: 3 successful, 0 failed
💾 Operation ID: op_1757889353 (use this to undo if needed)
```

### Continuous Mode (Perfect for Multiple Directories)
```bash
rename --continuous
```
```
🔄 Continuous mode started. Paste directory paths and press Enter.
💡 Each path will be processed immediately after a brief delay.
   Press Ctrl+C to exit.

# Paste: /Users/me/Music/Rock /Users/me/Music/Pop /Users/me/Music/Jazz
📂 Processing 3 directories (pasted together):
   1: /Users/me/Music/Rock
   2: /Users/me/Music/Pop
   3: /Users/me/Music/Jazz

🔍 Processing directory 1 of 3: /Users/me/Music/Rock
[Interactive processing for each directory...]
```

### Undo Operations
```bash
# Undo the most recent operation
rename --undo
```
```
🔄 Finding most recent operation to undo...
🎯 Most recent operation: op_1757889353
🔄 Undoing operation: op_1757889353
📂 Directory: /Users/me/music
🏷️  Prefix: [Dua Lipa]
📅 Original timestamp: 2025-09-14 22:35:53 UTC
📊 Files to restore: 3

🔄 Preview of restore operation:
   Levitating.mp3 → [Dua Lipa] Levitating.mp3
   Don't Start Now.mp3 → [Dua Lipa] Don't Start Now.mp3
   Physical.mp3 → [Dua Lipa] Physical.mp3

💡 Are you sure you want to undo this operation? (y/N): y
✅ Proceeding with undo...
📊 Undo results: 3 successful, 0 failed
✅ Operation successfully undone!
```

### Operation History
```bash
rename --list
```
```
📋 Recent rename operations:
1. Operation ID: op_1757889353
   Timestamp: 2025-09-14 22:35:53 UTC
   Directory: /Users/me/music
   Prefix removed: [Dua Lipa]
   Files renamed: 3
     [Dua Lipa] Levitating.mp3 → Levitating.mp3
     [Dua Lipa] Don't Start Now.mp3 → Don't Start Now.mp3
     [Dua Lipa] Physical.mp3 → Physical.mp3

💡 Use 'rename --undo <operation_id>' to undo any operation.
```

### Custom Patterns
```bash
# Remove parentheses-delimited prefixes
rename --regex '\(.*\)' ./documents

# Remove any prefixes (no filtering)
rename --no-filter ./mixed_files

# Process multiple directories
rename ./music ./photos ./documents
```

## 🛠 Installation

### Using cargo-binstall (recommended)
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

## 📋 Command Reference

### Main Tool: `rename`
```bash
rename [OPTIONS] [DIRECTORIES...]

OPTIONS:
    -r, --regex PATTERN    Use custom regex to filter prefixes (default: \[.*\])
    --no-filter           Accept all prefixes (no regex filtering)
    -c, --continuous      Continuous mode: listen for pasted paths
    -u, --undo [ID]       Undo an operation (most recent if no ID given)
    -l, --list            List recent rename operations
    -h, --help            Show help message

EXAMPLES:
    rename ./music                    # Interactive rename with preview
    rename --continuous               # Continuous mode for multiple dirs
    rename --undo                     # Undo most recent operation
    rename --list                     # Show operation history
    rename --regex '\(.*\)' ./docs    # Custom pattern matching
```

### Analysis Tools
```bash
ftmi                     # Find longest prefixes (analysis only)
echo "./music" | ftmi    # Pipe directory paths for analysis
```

## 🔧 Advanced Usage

### Multiple Input Sources
```bash
# Command line + piped input
echo "./photos" | rename ./music ./documents
```

### Pattern Matching Examples
```bash
# Bracket prefixes: [Artist] Song.mp3
rename ./music

# Parentheses prefixes: (Draft) Document.pdf  
rename --regex '\(.*\)' ./documents

# Image prefixes: IMG_001.jpg, DSC_002.jpg
rename --regex 'IMG_.*|DSC_.*' ./photos

# No filtering (show all prefixes)
rename --no-filter ./mixed_files
```

### Workflow Integration
```bash
# Process multiple music directories in sequence
find ~/Music -maxdepth 1 -type d | rename --continuous

# Quick undo if something goes wrong
rename --undo
```

## 🏗 Library Usage

Use FTMI as a library in your Rust projects:

```rust
use ftmi::{find_longest_prefix, PrefixOptions, PrefixedPath, RenameDatabase, tracked_rename};

// Find prefixes
let options = PrefixOptions::default();
let results: Vec<PrefixedPath> = find_longest_prefix(Path::new("/music"), &options)?;

// Track renames with database
let db = RenameDatabase::new(db_path);
db.initialize()?;
let operation_id = generate_operation_id();
tracked_rename(&db, &old_path, &new_path, &prefix, &operation_id)?;
```

## 📂 Project Structure

**Main Binaries:**
- `rename` - **Interactive prefix removal tool (primary)**
- `ftmi` - Prefix analysis and detection

**Database:**
- SQLite database in `~/.ftmi/renames.db`
- Automatic cleanup of old operations
- Cross-platform compatibility

## 🤝 Contributing

Contributions welcome! Please submit pull requests or open issues.

## 📄 License

Free for individual use. Commercial license required for corporations.
See [LICENSE](LICENSE) for details.

## 🛣 Recent Updates

- ✅ **Full undo system** with SQLite tracking
- ✅ **Continuous mode** for batch processing  
- ✅ **Preview before rename** and undo
- ✅ **Operation history** and selective undo
- ✅ **200ms debounce** for smooth UX
- ✅ **Multiple prefix handling** for tied results
- ✅ **Configurable regex filtering**