use ftmi::{find_common_prefix, PrefixOptions, PrefixMode};
use std::env;
use std::io::{self, BufRead};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <prefix1> [prefix2] [prefix3] ...", args[0]);
        eprintln!("Example: {} IMG_ test_ doc_", args[0]);
        std::process::exit(1);
    }
    
    let prefixes: Vec<String> = args[1..].iter().map(|s| s.clone()).collect();
    
    let stdin = io::stdin();
    let options = PrefixOptions {
        mode: PrefixMode::SpecificPrefixes { prefixes },
        min_occurrences: 1, // For specific prefixes, we might want to see single matches
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
                    println!("No files found with specified prefixes");
                } else {
                    for prefix in prefixes {
                        println!("Prefix: {} - {} files", prefix.prefix, prefix.occurrences);
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
    }
    
    Ok(())
}