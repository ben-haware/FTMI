use crate::prefix_finder::{find_common_prefix, PrefixOptions, PrefixMode};
use std::io::{self, BufRead};
use std::path::Path;

pub fn find_delimited_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Check for help flag
    if args.len() > 2 && (args[2] == "--help" || args[2] == "-h") {
        print_help();
        return Ok(());
    }
    
    // Parse delimiter options
    let mut delimiters = vec![
        ("[".to_string(), "]".to_string()),
        ("(".to_string(), ")".to_string()),
        ("{".to_string(), "}".to_string()),
    ];
    
    // Check for custom delimiter argument
    let mut directories = Vec::new();
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--delimiter" && i + 1 < args.len() {
            // Parse custom delimiter like "--delimiter []" or "--delimiter ()"
            let delim_str = &args[i + 1];
            if delim_str.len() >= 2 {
                let open = delim_str.chars().nth(0).unwrap().to_string();
                let close = delim_str.chars().nth(1).unwrap().to_string();
                delimiters = vec![(open, close)];
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
        mode: PrefixMode::DelimiterOnly { delimiters },
        min_occurrences: 2,
        filter_regex: None,
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
        eprintln!("find-delimited: Find delimited prefixes like [Artist], (Draft)");
        eprintln!("Usage: ftmi find-delimited ./directory");
        eprintln!("       echo './directory' | ftmi find-delimited");
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
    println!("{}", "-".repeat(50));
    
    match find_common_prefix(path, options) {
        Ok(prefixes) => {
            if prefixes.is_empty() {
                println!("No delimited prefixes found (minimum {} occurrences required)", options.min_occurrences);
            } else {
                for prefix in prefixes {
                    if let Some((open, close)) = &prefix.delimiter {
                        println!("Delimited prefix: {}{}{} - {} files", open, prefix.prefix, close, prefix.occurrences);
                        for file in &prefix.files {
                            println!("  - {}", file);
                        }
                        println!();
                    }
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
    println!("find-delimited - Find delimited prefixes like [Artist], (Draft)");
    println!();
    println!("USAGE:");
    println!("    ftmi find-delimited [OPTIONS] [DIRECTORIES...]");
    println!("    echo './directory' | ftmi find-delimited");
    println!();
    println!("OPTIONS:");
    println!("    --delimiter DELIM    Custom delimiter pair (e.g., [], (), {{}})");
    println!("    -h, --help          Show this help message");
    println!();
    println!("DESCRIPTION:");
    println!("    Finds prefixes that are enclosed in delimiters like brackets, parentheses,");
    println!("    or braces. Only shows prefixes that appear at least 2 times.");
    println!();
    println!("EXAMPLES:");
    println!("    # Find all delimited prefixes in music directory");
    println!("    ftmi find-delimited ./music");
    println!();
    println!("    # Find only bracket-delimited prefixes");
    println!("    ftmi find-delimited --delimiter [] ./photos");
    println!();
    println!("    # Process multiple directories");
    println!("    ftmi find-delimited ./music ./photos ./documents");
}