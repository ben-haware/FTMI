use crate::prefix_finder::{find_common_prefix, PrefixOptions};
use std::io::{self, BufRead};
use std::path::Path;

pub fn detect_all_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Check for help flag
    if args.len() > 2 && (args[2] == "--help" || args[2] == "-h") {
        print_help();
        return Ok(());
    }
    
    // Parse options
    let mut min_occurrences = 2;
    let mut directories = Vec::new();
    let mut i = 2;
    
    while i < args.len() {
        if args[i] == "--min" && i + 1 < args.len() {
            if let Ok(min) = args[i + 1].parse::<usize>() {
                min_occurrences = min;
            }
            i += 2;
        } else if !args[i].starts_with('-') {
            directories.push(args[i].clone());
            i += 1;
        } else {
            i += 1;
        }
    }
    
    let options = PrefixOptions {
        min_occurrences,
        filter_regex: None, // No filtering - show all prefixes
        ..PrefixOptions::default()
    };
    
    // Process directories from command line or stdin
    if !directories.is_empty() {
        for dir in directories {
            process_directory(&dir, &options)?;
        }
    } else if !atty::is(atty::Stream::Stdin) {
        // Read from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let dir_path = line?.trim().to_string();
            if !dir_path.is_empty() {
                process_directory(&dir_path, &options)?;
            }
        }
    } else {
        eprintln!("detect-all: Detect all common prefixes automatically");
        eprintln!("Usage: ftmi detect-all ./directory");
        eprintln!("       echo './directory' | ftmi detect-all");
    }
    
    Ok(())
}

fn process_directory(dir_path: &str, options: &PrefixOptions) -> Result<(), Box<dyn std::error::Error>> {
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
    println!("Minimum occurrences: {}", options.min_occurrences);
    println!("{}", "-".repeat(50));
    
    match find_common_prefix(path, options) {
        Ok(prefixes) => {
            if prefixes.is_empty() {
                println!("No common prefixes found (minimum {} occurrences required)", options.min_occurrences);
            } else {
                println!("Found {} prefix group(s):", prefixes.len());
                println!();
                
                for (i, prefix) in prefixes.iter().enumerate() {
                    if let Some((open, close)) = &prefix.delimiter {
                        println!("{}. Delimited prefix: {}{}{} - {} files", 
                               i + 1, open, prefix.prefix, close, prefix.occurrences);
                    } else {
                        println!("{}. Prefix: {} - {} files", 
                               i + 1, prefix.prefix, prefix.occurrences);
                    }
                    
                    for file in &prefix.files {
                        println!("   - {}", file);
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
    println!("detect-all - Detect all common prefixes automatically");
    println!();
    println!("USAGE:");
    println!("    ftmi detect-all [OPTIONS] [DIRECTORIES...]");
    println!("    echo './directory' | ftmi detect-all");
    println!();
    println!("OPTIONS:");
    println!("    --min NUM          Minimum occurrences required (default: 2)");
    println!("    -h, --help        Show this help message");
    println!();
    println!("DESCRIPTION:");
    println!("    Automatically detects all types of common prefixes in files:");
    println!("    - Delimited prefixes: [Artist], (Draft), {{Project}}");
    println!("    - Separator-based: IMG_, test-, photo.001");
    println!("    - Character-based: common letter/number patterns");
    println!();
    println!("    Shows all prefixes that appear at least --min times (default 2).");
    println!();
    println!("EXAMPLES:");
    println!("    # Detect all prefixes in music directory");
    println!("    ftmi detect-all ./music");
    println!();
    println!("    # Lower threshold for detection");
    println!("    ftmi detect-all --min 1 ./photos");
    println!();
    println!("    # Process multiple directories");
    println!("    ftmi detect-all ./music ./photos ./documents");
}