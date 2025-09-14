use crate::prefix_finder::{find_common_prefix, PrefixOptions, PrefixMode};
use std::io::{self, BufRead};
use std::path::Path;

pub fn find_specific_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Check for help flag
    if args.len() > 2 && (args[2] == "--help" || args[2] == "-h") {
        print_help();
        return Ok(());
    }
    
    // Parse prefix options
    let mut prefixes = Vec::new();
    let mut directories = Vec::new();
    let mut i = 2;
    
    while i < args.len() {
        if args[i] == "--prefix" && i + 1 < args.len() {
            prefixes.push(args[i + 1].clone());
            i += 2;
        } else if !args[i].starts_with('-') {
            directories.push(args[i].clone());
            i += 1;
        } else {
            i += 1;
        }
    }
    
    if prefixes.is_empty() {
        // Default prefixes to search for
        prefixes = vec![
            "IMG_".to_string(),
            "DSC_".to_string(),
            "PHOTO_".to_string(),
            "VIDEO_".to_string(),
            "DOC_".to_string(),
            "DRAFT_".to_string(),
        ];
    }
    
    let options = PrefixOptions {
        mode: PrefixMode::SpecificPrefixes { prefixes: prefixes.clone() },
        min_occurrences: 1,
        filter_regex: None,
    };
    
    // Process directories from command line or stdin
    if !directories.is_empty() {
        for dir in directories {
            process_directory(&dir, &options, &prefixes)?;
        }
    } else if !atty::is(atty::Stream::Stdin) {
        // Read from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let dir_path = line?.trim().to_string();
            if !dir_path.is_empty() {
                process_directory(&dir_path, &options, &prefixes)?;
            }
        }
    } else {
        eprintln!("find-specific: Search for specific prefix patterns");
        eprintln!("Usage: ftmi find-specific ./directory");
        eprintln!("       ftmi find-specific --prefix IMG_ ./photos");
    }
    
    Ok(())
}

fn process_directory(dir_path: &str, options: &PrefixOptions, search_prefixes: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
    println!("Searching for prefixes: {}", search_prefixes.join(", "));
    println!("{}", "-".repeat(50));
    
    match find_common_prefix(path, options) {
        Ok(prefixes) => {
            if prefixes.is_empty() {
                println!("No matching prefixes found");
            } else {
                for prefix in prefixes {
                    println!("Found prefix: {} - {} files", prefix.prefix, prefix.occurrences);
                    for file in &prefix.files {
                        println!("  - {}", file);
                    }
                    println!();
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
    println!("find-specific - Search for specific prefix patterns");
    println!();
    println!("USAGE:");
    println!("    ftmi find-specific [OPTIONS] [DIRECTORIES...]");
    println!("    echo './directory' | ftmi find-specific");
    println!();
    println!("OPTIONS:");
    println!("    --prefix PREFIX     Specific prefix to search for (can be used multiple times)");
    println!("    -h, --help         Show this help message");
    println!();
    println!("DESCRIPTION:");
    println!("    Searches for files that start with specific prefixes. If no prefixes are");
    println!("    specified, searches for common patterns like IMG_, DSC_, PHOTO_, etc.");
    println!();
    println!("EXAMPLES:");
    println!("    # Search for default prefixes (IMG_, DSC_, etc.)");
    println!("    ftmi find-specific ./photos");
    println!();
    println!("    # Search for specific prefixes");
    println!("    ftmi find-specific --prefix IMG_ --prefix DSC_ ./photos");
    println!();
    println!("    # Search for document prefixes");
    println!("    ftmi find-specific --prefix DRAFT_ --prefix FINAL_ ./documents");
}