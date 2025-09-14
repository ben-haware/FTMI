use crate::path_extraction::extract_paths_from_text;
use std::io::{self, Read};

pub fn extract_paths_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Check for help flag
    if args.len() > 2 && (args[2] == "--help" || args[2] == "-h") {
        print_help();
        return Ok(());
    }
    
    let mut input = String::new();
    
    if atty::is(atty::Stream::Stdin) {
        // Interactive mode - no piped input
        eprintln!("extract-paths: Extract file paths from text input");
        eprintln!("Usage: echo 'text with /path/to/file' | ftmi extract-paths");
        eprintln!("       ftmi extract-paths < logfile.txt");
        return Ok(());
    } else {
        // Read from stdin
        io::stdin().read_to_string(&mut input)?;
    }
    
    let paths = extract_paths_from_text(&input);
    
    if paths.is_empty() {
        eprintln!("No paths found in input");
    } else {
        for path in paths {
            println!("{}", path);
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("extract-paths - Extract file paths from text input");
    println!();
    println!("USAGE:");
    println!("    echo 'text with /path/to/file' | ftmi extract-paths");
    println!("    ftmi extract-paths < logfile.txt");
    println!();
    println!("DESCRIPTION:");
    println!("    Extracts valid file system paths from any text input using pattern matching.");
    println!("    Useful for processing log files, error messages, or any text containing paths.");
    println!();
    println!("EXAMPLES:");
    println!("    # Extract paths from log file");
    println!("    ftmi extract-paths < application.log");
    println!();
    println!("    # Extract paths from command output");
    println!("    find /Users -name '*.txt' 2>&1 | ftmi extract-paths");
}