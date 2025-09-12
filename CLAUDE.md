# FTMI Project Progress

## Overview
FTMI is a Rust-based file renaming utility with cargo-binstall support and a custom license model.

## Current Status

### Completed Tasks
- ✅ Created GitHub repository: https://github.com/ben-haware/FTMI
- ✅ Initialized Rust project with cargo (name: ftmi)
- ✅ Created hello world application that prints "FTMI - File renaming utility"
- ✅ Set up GitHub Actions for cargo-binstall support
- ✅ Configured multi-platform builds (Linux x86_64/musl, Windows, macOS x86_64/ARM64)
- ✅ Added custom license (free for individuals, commercial for corporations)
- ✅ Created README with installation instructions
- ✅ Pushed initial commit to GitHub

### Project Structure
```
FTMI/
├── .github/
│   └── workflows/
│       └── release.yml    # GitHub Actions for cargo-binstall
├── src/
│   └── main.rs           # Hello world entry point
├── .gitignore
├── Cargo.toml           # Configured with binstall metadata
├── LICENSE              # Custom dual license
├── LICENSE-MIT          # MIT license template
└── README.md            # Project documentation
```

### Key Configuration

**Cargo.toml metadata for binstall:**
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }{ binary-ext }"
bin-dir = "{ bin }{ binary-ext }"
pkg-fmt = "bin"
```

### Next Steps
To create a release and enable cargo-binstall:
```bash
git tag v0.1.0
git push origin v0.1.0
```

This will trigger the GitHub Actions workflow to build and publish binaries.

### Installation Methods
Once released, users can install via:
- `cargo binstall ftmi` (fastest, downloads pre-built binaries)
- `cargo install ftmi` (builds from source)

### License Model
- Free for individual use
- Commercial license required for corporate use
- See LICENSE file for full terms