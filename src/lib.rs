pub mod path_extraction;
pub mod prefix_finder;

use std::io::{self, BufRead};
use std::path::Path;

pub use path_extraction::{extract_paths_from_text, deduplicate_paths};
pub use prefix_finder::{
    find_common_prefix, find_longest_prefix, PrefixOptions, CommonPrefix, PrefixedPath, PrefixMode,
    extract_prefix_from_filename, remove_prefix, remove_prefix_with_delimiter
};

/// Main application logic for processing directories from stdin
pub fn process_directories_from_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let options = PrefixOptions::default();
    
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
                    println!("No common prefixes found (minimum {} occurrences required)", options.min_occurrences);
                } else {
                    for prefix in prefixes {
                        if let Some((open, close)) = &prefix.delimiter {
                            println!("Prefix: {} (within {}{}) - {} files", prefix.prefix, open, close, prefix.occurrences);
                        } else {
                            println!("Prefix: {} - {} files", prefix.prefix, prefix.occurrences);
                        }
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

/// Process directories from stdin and return only the longest matching prefix
pub fn process_directories_longest_prefix() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let options = PrefixOptions::default();
    
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
        
        match find_longest_prefix(path, &options) {
            Ok(prefixed_paths) => {
                if prefixed_paths.is_empty() {
                    println!("Directory: {}", dir_path);
                    println!("No common prefix found");
                    println!();
                } else {
                    println!("Directory: {}", dir_path);
                    for prefixed_path in &prefixed_paths {
                        println!("Prefix: {}", prefixed_path.prefix);
                        println!("Files ({}):", prefixed_path.paths.len());
                        for path in &prefixed_path.paths {
                            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                                println!("  {}", filename);
                            }
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

/// Process paths from stdin using the path extraction functionality
pub fn process_paths_from_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut all_text = String::new();
    
    for line in stdin.lock().lines() {
        all_text.push_str(&line?);
        all_text.push('\n');
    }
    
    let paths = extract_paths_from_text(&all_text);
    for path in paths {
        println!("{}", path);
    }
    
    Ok(())
}