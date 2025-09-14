use ftmi::{find_longest_prefix, PrefixOptions, PrefixedPath, RenameDatabase, generate_operation_id, tracked_rename};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::fs;
use std::env;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut options = PrefixOptions::default();
    let mut directories: Vec<String> = Vec::new();
    let mut continuous_mode = false;
    let mut undo_mode = false;
    let mut list_operations = false;
    let mut undo_operation_id: Option<String> = None;
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
            "--continuous" | "-c" => {
                continuous_mode = true;
                i += 1;
            }
            "--undo" | "-u" => {
                undo_mode = true;
                if i + 1 < args.len() && !args[i + 1].starts_with("-") {
                    // Operation ID provided
                    undo_operation_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    // No operation ID provided, will undo most recent operation
                    undo_operation_id = None;
                    i += 1;
                }
            }
            "--list" | "-l" => {
                list_operations = true;
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
    
    // Initialize database
    let db_path = RenameDatabase::default_path()?;
    let db = RenameDatabase::new(db_path);
    db.initialize()?;
    
    // Handle different modes
    if list_operations {
        return list_recent_operations(&db);
    }
    
    if undo_mode {
        if let Some(op_id) = undo_operation_id {
            return undo_operation(&db, &op_id);
        } else {
            return undo_most_recent_operation(&db);
        }
    }
    
    if continuous_mode {
        return run_continuous_mode(&db, &options);
    }
    
    // Normal mode
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
                            
                            let operation_id = generate_operation_id();
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
                                    
                                    match tracked_rename(&db, old_path, &new_path, &prefixed_path.prefix, &operation_id) {
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

/// Run continuous mode that listens for pasted paths
fn run_continuous_mode(db: &RenameDatabase, options: &PrefixOptions) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Continuous mode started. Paste directory paths and press Enter.");
    println!("💡 Each path will be processed immediately after a brief delay.");
    println!("   Press Ctrl+C to exit.\n");
    
    let stdin = io::stdin();
    
    loop {
        // Read a line from stdin
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = line.trim().to_string();
                if input.is_empty() {
                    continue;
                }
                
                // Split the input by spaces to handle multiple paths pasted at once
                let paths: Vec<String> = input
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                
                // Wait 200ms before processing to prevent screen tearing
                thread::sleep(Duration::from_millis(200));
                
                // Process the paths (could be one or multiple)
                process_paths_batch(db, options, &paths)?;
            }
            Err(e) => {
                eprintln!("❌ Error reading input: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

/// Process a batch of directory paths one at a time
fn process_paths_batch(
    db: &RenameDatabase, 
    options: &PrefixOptions, 
    paths: &[String]
) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Ok(());
    }
    
    if paths.len() == 1 {
        println!("📂 Processing directory: {}", paths[0]);
    } else {
        println!("📂 Processing {} directories (pasted together):", paths.len());
        for (i, path) in paths.iter().enumerate() {
            println!("   {}: {}", i + 1, path);
        }
    }
    println!();
    
    for (i, dir_path) in paths.iter().enumerate() {
        if paths.len() > 1 {
            println!("🔍 Processing directory {} of {}: {}", i + 1, paths.len(), dir_path);
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
        
        match find_longest_prefix(&path, options) {
            Ok(prefixed_paths) => {
                if prefixed_paths.is_empty() {
                    println!("📁 Directory: {}", dir_path);
                    println!("ℹ️  No bracket-delimited prefixes found");
                } else {
                    process_directory_prefixes(db, &path, dir_path, &prefixed_paths)?;
                }
            }
            Err(e) => {
                eprintln!("❌ Error processing directory {}: {}", dir_path, e);
            }
        }
        
        if paths.len() > 1 && i < paths.len() - 1 {
            println!("{}", "─".repeat(40));
        }
    }
    
    println!("{}", "═".repeat(60));
    println!("✅ Batch processing completed! Waiting for more paths...\n");
    
    Ok(())
}

/// Process the prefixes found in a directory (extracted from main function)
fn process_directory_prefixes(
    db: &RenameDatabase,
    path: &Path,
    dir_path: &str,
    prefixed_paths: &[PrefixedPath],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Directory: {}", dir_path);
    println!("Found {} prefix group(s) with highest occurrence count:", prefixed_paths.len());
    
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
                
                let operation_id = generate_operation_id();
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
                        
                        match tracked_rename(db, old_path, &new_path, &prefixed_path.prefix, &operation_id) {
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
                if success_count > 0 {
                    println!("💾 Operation ID: {} (use this to undo if needed)", operation_id);
                }
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
    
    Ok(())
}

/// List recent rename operations
fn list_recent_operations(db: &RenameDatabase) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Recent rename operations:");
    
    let operations = db.get_recent_operations(20)?;
    
    if operations.is_empty() {
        println!("   No operations found.");
        return Ok(());
    }
    
    for (i, op_id) in operations.iter().enumerate() {
        let records = db.get_operation_renames(op_id)?;
        if let Some(first_record) = records.first() {
            println!("{}. Operation ID: {}", i + 1, op_id);
            println!("   Timestamp: {}", first_record.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("   Directory: {}", first_record.directory.display());
            println!("   Prefix removed: [{}]", first_record.prefix_removed);
            println!("   Files renamed: {}", records.len());
            
            // Show first few files as examples
            let show_count = std::cmp::min(3, records.len());
            for record in records.iter().take(show_count) {
                if let (Some(old_name), Some(new_name)) = (
                    record.old_path.file_name().and_then(|s| s.to_str()),
                    record.new_path.file_name().and_then(|s| s.to_str())
                ) {
                    println!("     {} → {}", old_name, new_name);
                }
            }
            
            if records.len() > show_count {
                println!("     ... and {} more files", records.len() - show_count);
            }
            
            println!();
        }
    }
    
    println!("💡 Use 'rename --undo <operation_id>' to undo any operation.");
    
    Ok(())
}

/// Undo the most recent operation
fn undo_most_recent_operation(db: &RenameDatabase) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Finding most recent operation to undo...");
    
    let operations = db.get_recent_operations(1)?;
    
    if operations.is_empty() {
        println!("❌ No operations found to undo.");
        return Ok(());
    }
    
    let most_recent_op_id = &operations[0];
    println!("🎯 Most recent operation: {}", most_recent_op_id);
    
    undo_operation(db, most_recent_op_id)
}

/// Undo a specific operation
fn undo_operation(db: &RenameDatabase, operation_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Undoing operation: {}", operation_id);
    
    // First, get the operation details to show what will be undone
    let records = db.get_operation_renames(operation_id)?;
    
    if records.is_empty() {
        eprintln!("❌ Operation ID '{}' not found.", operation_id);
        return Ok(());
    }
    
    let first_record = &records[0];
    println!("📂 Directory: {}", first_record.directory.display());
    println!("🏷️  Prefix: [{}]", first_record.prefix_removed);
    println!("📅 Original timestamp: {}", first_record.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("📊 Files to restore: {}", records.len());
    println!();
    
    // Show preview of what will be restored
    println!("🔄 Preview of restore operation:");
    for record in &records {
        if let (Some(current_name), Some(original_name)) = (
            record.new_path.file_name().and_then(|s| s.to_str()),
            record.old_path.file_name().and_then(|s| s.to_str())
        ) {
            println!("   {} → {}", current_name, original_name);
        }
    }
    
    // Ask for confirmation
    print!("\n💡 Are you sure you want to undo this operation? (y/N): ");
    io::stdout().flush()?;
    
    let mut response = String::new();
    
    // Read user input from terminal directly
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
        "y" | "yes" => {
            println!("✅ Proceeding with undo...");
            
            let (success_count, error_count) = db.undo_operation(operation_id)?;
            
            println!("📊 Undo results: {} successful, {} failed", success_count, error_count);
            
            if success_count > 0 {
                println!("✅ Operation successfully undone!");
            }
            if error_count > 0 {
                println!("⚠️  Some files could not be restored (they may have been moved or modified).");
            }
        }
        _ => {
            println!("❌ Undo cancelled.");
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("🔧 FTMI Interactive Prefix Removal Tool");
    println!();
    println!("USAGE:");
    println!("    rename [OPTIONS] [DIRECTORIES...]");
    println!("    echo 'directory' | rename [OPTIONS] [DIRECTORIES...]");
    println!();
    println!("OPTIONS:");
    println!("    -r, --regex PATTERN    Use custom regex to filter prefixes (default: \\[.*\\])");
    println!("    --no-filter           Accept all prefixes (no regex filtering)");
    println!("    -c, --continuous      Continuous mode: listen for pasted paths");
    println!("    -u, --undo [ID]       Undo an operation (most recent if no ID given)");
    println!("    -l, --list            List recent rename operations");
    println!("    -h, --help            Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Default: Find bracket-delimited prefixes");
    println!("    rename ./music");
    println!();
    println!("    # Custom regex: Find parentheses-delimited prefixes");
    println!("    rename --regex '\\(.*\\)' ./music");
    println!();
    println!("    # No filter: Find all prefixes");
    println!("    rename --no-filter ./music");
    println!();
    println!("    # Continuous mode for pasting multiple paths");
    println!("    rename --continuous");
    println!();
    println!("    # List recent operations");
    println!("    rename --list");
    println!();
    println!("    # Undo the most recent operation");
    println!("    rename --undo");
    println!();
    println!("    # Undo a specific operation");
    println!("    rename --undo op_1234567890");
    println!();
    println!("    # Pipe in directories with custom regex");
    println!("    echo './music' | rename --regex 'IMG_.*'");
}