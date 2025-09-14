use ftmi::{find_common_prefix, PrefixOptions, PrefixMode};
use std::io::{self, BufRead};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let options = PrefixOptions {
        mode: PrefixMode::DelimiterOnly {
            delimiters: vec![
                ("(".to_string(), ")".to_string()),
                ("[".to_string(), "]".to_string()),
                ("{".to_string(), "}".to_string()),
                ("\"".to_string(), "\"".to_string()),
                ("'".to_string(), "'".to_string()),
            ],
        },
        min_occurrences: 2,
        filter_regex: None, // No additional regex filtering
    };
    
    for line in stdin.lock().lines() {
        let dir_path = line?;
        let dir_path = dir_path.trim();
        
        if dir_path.is_empty() {
            continue;
        }
        
        let path = Path::new(dir_path);
        if !path.exists() {
            eprintln!("Warning: Directory does not exist: {}", dir_path);
            continue;
        }
        
        if !path.is_dir() {
            eprintln!("Warning: Not a directory: {}", dir_path);
            continue;
        }
        
        println!("\nDirectory: {}", dir_path);
        println!("{}", "-".repeat(50));
        
        match find_common_prefix(path, &options) {
            Ok(prefixes) => {
                if prefixes.is_empty() {
                    println!("No delimited prefixes found (minimum {} occurrences required)", options.min_occurrences);
                } else {
                    for prefix in prefixes {
                        if let Some((open, close)) = &prefix.delimiter {
                            println!("Prefix: {} (within {}{}) - {} files", prefix.prefix, open, close, prefix.occurrences);
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
    }
    
    Ok(())
}