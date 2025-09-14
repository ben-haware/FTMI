use ftmi::{find_common_prefix, remove_prefix_with_delimiter, PrefixOptions, PrefixMode};
use std::env;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} auto <directory>     - Auto-detect and show prefix removal preview", args[0]);
        eprintln!("  {} prefix <directory>   - Remove specific prefix from files (interactive)", args[0]);
        std::process::exit(1);
    }
    
    let mode = &args[1];
    
    match mode.as_str() {
        "auto" => {
            if args.len() < 3 {
                eprintln!("Please specify a directory");
                std::process::exit(1);
            }
            auto_detect_and_preview(&args[2])
        },
        "prefix" => {
            if args.len() < 3 {
                eprintln!("Please specify a directory");
                std::process::exit(1);
            }
            interactive_prefix_removal(&args[2])
        },
        _ => {
            eprintln!("Unknown mode: {}. Use 'auto' or 'prefix'", mode);
            std::process::exit(1);
        }
    }
}

fn auto_detect_and_preview(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        eprintln!("Directory does not exist: {}", dir_path);
        std::process::exit(1);
    }
    
    // Use delimiter-only mode for cleaner results
    let options = PrefixOptions {
        mode: PrefixMode::DelimiterOnly {
            delimiters: vec![
                ("(".to_string(), ")".to_string()),
                ("[".to_string(), "]".to_string()),
                ("{".to_string(), "}".to_string()),
            ],
        },
        min_occurrences: 1,
        filter_regex: None, // No additional regex filtering
    };
    
    let prefixes = find_common_prefix(path, &options)?;
    
    if prefixes.is_empty() {
        println!("No delimited prefixes found in directory");
        return Ok(());
    }
    
    println!("Directory: {}", dir_path);
    println!("{}", "=".repeat(50));
    
    for prefix_info in prefixes {
        if let Some((open, close)) = &prefix_info.delimiter {
            println!("\nPrefix: {} (within {}{}) - {} files", prefix_info.prefix, open, close, prefix_info.occurrences);
            println!("Preview of prefix removal:");
            println!("{}", "-".repeat(30));
            
            for file in &prefix_info.files {
                let new_name = remove_prefix_with_delimiter(file, &prefix_info.prefix, open, close);
                println!("  {} -> {}", file, new_name);
            }
        }
    }
    
    Ok(())
}

fn interactive_prefix_removal(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        eprintln!("Directory does not exist: {}", dir_path);
        std::process::exit(1);
    }
    
    // Get all files in directory
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.path().is_file() {
            if let Some(filename) = entry.file_name().to_str() {
                files.push(filename.to_string());
            }
        }
    }
    
    if files.is_empty() {
        println!("No files found in directory");
        return Ok(());
    }
    
    files.sort();
    println!("Files in directory:");
    for (i, file) in files.iter().enumerate() {
        println!("  {}: {}", i + 1, file);
    }
    
    println!("\nThis is a preview tool. It shows what files would be renamed but doesn't actually rename them.");
    println!("Enter a prefix to remove (or press Enter to quit):");
    
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    let prefix = input.trim();
    
    if prefix.is_empty() {
        return Ok(());
    }
    
    println!("\nPreview of removing prefix '{}':", prefix);
    println!("{}", "-".repeat(40));
    
    for file in &files {
        if file.starts_with(prefix) {
            let new_name = file[prefix.len()..].trim_start();
            println!("  {} -> {}", file, new_name);
        }
    }
    
    Ok(())
}