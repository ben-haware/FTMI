use ftmi::{find_longest_prefix, PrefixOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::fs;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments for regex pattern
    let mut options = PrefixOptions::default();
    let mut directories: Vec<String> = Vec::new();
    let mut i = 1;
    
    while i < args.len() {
        match args[i].as_str() {
            "--regex" | "-r" => {
                if i + 1 < args.len() {
                    options.filter_regex = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("❌ Error: --regex requires a pattern argument");
                    return Ok(());
                }
            }
            "--no-filter" => {
                options.filter_regex = None;
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            arg if arg.starts_with("--") => {
                eprintln!("❌ Unknown option: {}", arg);
                print_help();
                return Ok(());
            }
            _ => {
                directories.push(args[i].clone());
                i += 1;
            }
        }
    }
    
    println!("🔧 FTMI Interactive Prefix Removal Tool");
    
    if let Some(regex_pattern) = &options.filter_regex {
        println!("🔍 Using regex filter: {}", regex_pattern);
    } else {
        println!("🔍 No regex filter (accepting all prefixes)");
    }
    
    // Phase 1: Aggregate all directory paths from all sources
    // (directories from args already parsed above)
    
    // Add command line arguments if any
    if !directories.is_empty() {
        println!("📝 Adding {} directories from command line arguments", directories.len());
    }
    
    // Add stdin input if available (check if stdin is piped)
    if !atty::is(atty::Stream::Stdin) {
        println!("📝 Reading additional directories from stdin...");
        let stdin = io::stdin();
        let stdin_dirs: Vec<String> = stdin.lock().lines().collect::<Result<Vec<_>, _>>()?;
        println!("📝 Adding {} directories from stdin", stdin_dirs.len());
        directories.extend(stdin_dirs);
    }
    
    if directories.is_empty() {
        eprintln!("❌ No directories provided. Use command line arguments or pipe directories via stdin.");
        eprintln!("Examples:");
        eprintln!("  cargo run --bin interactive-rename ./music ./photos");
        eprintln!("  echo './music' | cargo run --bin interactive-rename");
        eprintln!("  echo './music' | cargo run --bin interactive-rename ./photos");
        return Ok(());
    }
    
    println!("📊 Processing {} directories total\n", directories.len());
    
    // Phase 2: Process each directory
    for dir_path in directories {
        let dir_path = dir_path.trim();
        
        if dir_path.is_empty() {
            continue;
        }
        
        // Convert relative paths to absolute paths
        let path = if Path::new(dir_path).is_relative() {
            env::current_dir()?.join(dir_path)
        } else {
            Path::new(dir_path).to_path_buf()
        };
        if !path.exists() {
            eprintln!("❌ Warning: Directory does not exist: {}", dir_path);
            continue;
        }
        
        if !path.is_dir() {
            eprintln!("❌ Warning: Not a directory: {}", dir_path);
            continue;
        }
        
        match find_longest_prefix(&path, &options) {
            Ok(prefixed_paths) => {
                if prefixed_paths.is_empty() {
                    println!("📁 Directory: {}", dir_path);
                    println!("ℹ️  No bracket-delimited prefixes found\n");
                    continue;
                }
                
                println!("📁 Directory: {}", dir_path);
                println!("Found {} prefix group(s) with highest occurrence count:\n", prefixed_paths.len());
                
                for (i, prefixed_path) in prefixed_paths.iter().enumerate() {
                    println!("🏷️  Prefix {}: [{}]", i + 1, prefixed_path.prefix);
                    println!("   Files ({}):", prefixed_path.paths.len());
                    
                    // Show preview of what files would look like after prefix removal
                    for path in &prefixed_path.paths {
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            let new_filename = remove_bracket_prefix(filename, &prefixed_path.prefix);
                            println!("   {} → {}", filename, new_filename);
                        }
                    }
                    
                    // Ask for confirmation
                    print!("\n💡 Remove prefix [{}] from these {} files? (Y/n/s=skip, default=Y): ", 
                           prefixed_path.prefix, prefixed_path.paths.len());
                    io::stdout().flush()?;
                    
                    let mut response = String::new();
                    
                    // Read user input from terminal even when stdin is piped
                    #[cfg(unix)]
                    {
                        use std::fs::OpenOptions;
                        use std::io::BufReader;
                        let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
                        let mut tty_reader = BufReader::new(tty);
                        tty_reader.read_line(&mut response)?;
                    }
                    
                    #[cfg(not(unix))]
                    {
                        // On Windows, try to read from CONIN$
                        use std::fs::OpenOptions;
                        use std::io::BufReader;
                        match OpenOptions::new().read(true).open("CONIN$") {
                            Ok(con) => {
                                let mut con_reader = BufReader::new(con);
                                con_reader.read_line(&mut response)?;
                            }
                            Err(_) => {
                                // Fallback to regular stdin
                                io::stdin().read_line(&mut response)?;
                            }
                        }
                    }
                    let response = response.trim().to_lowercase();
                    
                    match response.as_str() {
                        "y" | "yes" | "" => {  // Empty string (just Enter) defaults to yes
                            println!("✅ Proceeding with prefix removal...");
                            
                            let mut success_count = 0;
                            let mut error_count = 0;
                            
                            for old_path in &prefixed_path.paths {
                                if let Some(filename) = old_path.file_name().and_then(|s| s.to_str()) {
                                    let new_filename = remove_bracket_prefix(filename, &prefixed_path.prefix);
                                    
                                    // Skip if new filename would be the same
                                    if new_filename == filename {
                                        println!("   ⏭️  {} (no change needed)", filename);
                                        continue;
                                    }
                                    
                                    let new_path = old_path.with_file_name(&new_filename);
                                    
                                    // Check if target file already exists
                                    if new_path.exists() {
                                        error_count += 1;
                                        eprintln!("   ❌ Target file already exists: {}", new_filename);
                                        continue;
                                    }
                                    
                                    println!("   🔄 Renaming: {} → {}", filename, new_filename);
                                    
                                    match fs::rename(old_path, &new_path) {
                                        Ok(_) => {
                                            success_count += 1;
                                            println!("   ✓ Success!");
                                        }
                                        Err(e) => {
                                            error_count += 1;
                                            eprintln!("   ❌ Failed: {}", e);
                                        }
                                    }
                                }
                            }
                            
                            println!("📊 Results: {} successful, {} failed", success_count, error_count);
                        }
                        "n" | "no" => {
                            println!("❌ Skipped prefix removal for [{}]", prefixed_path.prefix);
                        }
                        "s" | "skip" => {
                            println!("⏭️  Skipped prefix [{}]", prefixed_path.prefix);
                        }
                        _ => {
                            println!("❓ Unknown response '{}', skipping...", response);
                        }
                    }
                    
                    println!();
                }
            }
            Err(e) => {
                eprintln!("❌ Error processing directory {}: {}", dir_path, e);
            }
        }
        
        println!("{}", "─".repeat(60));
    }
    
    println!("🏁 Interactive prefix removal completed!");
    Ok(())
}

/// Remove bracket-delimited prefix from filename
fn remove_bracket_prefix(filename: &str, prefix: &str) -> String {
    let prefix_pattern = format!("[{}]", prefix);
    if let Some(pos) = filename.find(&prefix_pattern) {
        if pos == 0 {
            // Prefix is at the beginning
            let remaining = &filename[prefix_pattern.len()..];
            // Remove leading whitespace and underscores, but preserve dashes and dots
            remaining.trim_start_matches(&[' ', '_'][..]).to_string()
        } else {
            // Prefix is not at the beginning, return as-is
            filename.to_string()
        }
    } else {
        // No matching prefix found, return as-is
        filename.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_bracket_prefix() {
        assert_eq!(
            remove_bracket_prefix("[Artist] Song.mp3", "Artist"),
            "Song.mp3"
        );
        assert_eq!(
            remove_bracket_prefix("[The Beatles] Hey Jude.mp3", "The Beatles"),
            "Hey Jude.mp3"
        );
        assert_eq!(
            remove_bracket_prefix("[Artist]_Song.mp3", "Artist"),
            "Song.mp3"
        );
        assert_eq!(
            remove_bracket_prefix("[Artist] - Song.mp3", "Artist"),
            "- Song.mp3"
        );
        assert_eq!(
            remove_bracket_prefix("No Prefix Song.mp3", "Artist"),
            "No Prefix Song.mp3"
        );
    }
}

fn print_help() {
    println!("🔧 FTMI Interactive Prefix Removal Tool");
    println!();
    println!("USAGE:");
    println!("    interactive-rename [OPTIONS] [DIRECTORIES...]");
    println!("    echo 'directory' | interactive-rename [OPTIONS] [DIRECTORIES...]");
    println!();
    println!("OPTIONS:");
    println!("    -r, --regex PATTERN    Use custom regex to filter prefixes (default: \\[.*\\])");
    println!("    --no-filter           Accept all prefixes (no regex filtering)");
    println!("    -h, --help            Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Default: Find bracket-delimited prefixes");
    println!("    interactive-rename ./music");
    println!();
    println!("    # Custom regex: Find parentheses-delimited prefixes");
    println!("    interactive-rename --regex '\\(.*\\)' ./music");
    println!();
    println!("    # No filter: Find all prefixes");
    println!("    interactive-rename --no-filter ./music");
    println!();
    println!("    # Pipe in directories with custom regex");
    println!("    echo './music' | interactive-rename --regex 'IMG_.*'");
}