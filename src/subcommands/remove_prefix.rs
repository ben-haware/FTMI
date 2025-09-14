use crate::prefix_finder::{find_longest_prefix, PrefixOptions, remove_prefix};
use std::io::{self, BufRead};
use std::path::Path;

pub fn remove_prefix_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Check for help flag
    if args.len() > 2 && (args[2] == "--help" || args[2] == "-h") {
        print_help();
        return Ok(());
    }
    
    // Parse options
    let mut preview_only = true; // Default to preview mode
    let mut custom_regex: Option<String> = None;
    let mut directories = Vec::new();
    let mut i = 2;
    
    while i < args.len() {
        if args[i] == "--execute" {
            preview_only = false;
            i += 1;
        } else if args[i] == "--regex" && i + 1 < args.len() {
            custom_regex = Some(args[i + 1].clone());
            i += 2;
        } else if !args[i].starts_with('-') {
            directories.push(args[i].clone());
            i += 1;
        } else {
            i += 1;
        }
    }
    
    let options = if let Some(regex) = custom_regex {
        PrefixOptions::with_regex(&regex)
    } else {
        PrefixOptions::default()
    };
    
    // Process directories from command line or stdin
    if !directories.is_empty() {
        for dir in directories {
            process_directory(&dir, &options, preview_only)?;
        }
    } else if !atty::is(atty::Stream::Stdin) {
        // Read from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let dir_path = line?.trim().to_string();
            if !dir_path.is_empty() {
                process_directory(&dir_path, &options, preview_only)?;
            }
        }
    } else {
        eprintln!("remove-prefix: Preview prefix removal operations");
        eprintln!("Usage: ftmi remove-prefix ./directory");
        eprintln!("       ftmi remove-prefix --execute ./directory  # Actually perform renames");
    }
    
    Ok(())
}

fn process_directory(dir_path: &str, options: &PrefixOptions, preview_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(dir_path);
    if !path.exists() {
        eprintln!("Warning: Directory does not exist: {}", dir_path);
        return Ok(());
    }
    
    if !path.is_dir() {
        eprintln!("Warning: Not a directory: {}", dir_path);
        return Ok(());
    }
    
    println!("Directory: {}", dir_path);
    if preview_only {
        println!("Mode: PREVIEW ONLY (use --execute to actually rename files)");
    } else {
        println!("Mode: EXECUTE (files will be renamed)");
    }
    println!("{}", "-".repeat(50));
    
    match find_longest_prefix(path, options) {
        Ok(prefixed_paths) => {
            if prefixed_paths.is_empty() {
                println!("No common prefixes found for removal");
            } else {
                for prefixed_path in prefixed_paths {
                    println!("Prefix to remove: {}", prefixed_path.prefix);
                    println!("Files ({}):", prefixed_path.paths.len());
                    
                    for file_path in &prefixed_path.paths {
                        if let Some(filename) = file_path.file_name().and_then(|s| s.to_str()) {
                            let new_name = remove_prefix(filename, &prefixed_path.prefix);
                            
                            if preview_only {
                                println!("  {} → {}", filename, new_name);
                            } else {
                                // Actually rename the file
                                let new_path = file_path.with_file_name(&new_name);
                                match std::fs::rename(file_path, &new_path) {
                                    Ok(_) => println!("  ✓ {} → {}", filename, new_name),
                                    Err(e) => eprintln!("  ✗ {} → {}: {}", filename, new_name, e),
                                }
                            }
                        }
                    }
                    println!();
                }
                
                if preview_only {
                    println!("💡 This was a preview. Use --execute to actually rename files.");
                    println!("💡 For interactive renaming with undo support, use: ftmi rename");
                }
            }
        }
        Err(e) => {
            eprintln!("Error processing directory {}: {}", dir_path, e);
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("remove-prefix - Preview prefix removal operations");
    println!();
    println!("USAGE:");
    println!("    ftmi remove-prefix [OPTIONS] [DIRECTORIES...]");
    println!("    echo './directory' | ftmi remove-prefix");
    println!();
    println!("OPTIONS:");
    println!("    --execute          Actually perform the renames (default is preview only)");
    println!("    --regex PATTERN    Custom regex pattern for prefix filtering");
    println!("    -h, --help        Show this help message");
    println!();
    println!("DESCRIPTION:");
    println!("    Shows what files would be renamed if prefixes were removed. By default,");
    println!("    only previews the changes. Use --execute to actually rename files.");
    println!();
    println!("    WARNING: This tool does NOT have undo functionality. For safe interactive");
    println!("    renaming with undo support, use 'ftmi rename' instead.");
    println!();
    println!("EXAMPLES:");
    println!("    # Preview prefix removal");
    println!("    ftmi remove-prefix ./music");
    println!();
    println!("    # Actually remove prefixes (DANGEROUS - no undo!)");
    println!("    ftmi remove-prefix --execute ./music");
    println!();
    println!("    # Custom pattern for parentheses prefixes");
    println!("    ftmi remove-prefix --regex '\\(.*\\)' ./documents");
    println!();
    println!("    # Safe interactive alternative with undo support");
    println!("    ftmi rename ./music");
}