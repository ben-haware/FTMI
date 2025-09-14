use ftmi::process_directories_longest_prefix;
use std::env;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "rename" => {
                // Execute the rename functionality
                let rename_args: Vec<String> = args[0..1].iter().chain(args[2..].iter()).cloned().collect();
                env::set_var("CARGO_PKG_NAME", "ftmi-rename");
                
                // Import and call the rename main function
                return ftmi::rename_main(rename_args);
            }
            "extract-paths" => {
                println!("extract-paths functionality - use separate binary for now");
                return Ok(());
            }
            "find-delimited" => {
                println!("find-delimited functionality - use separate binary for now");
                return Ok(());
            }
            "find-specific" => {
                println!("find-specific functionality - use separate binary for now");
                return Ok(());
            }
            "detect-all" => {
                println!("detect-all functionality - use separate binary for now");
                return Ok(());
            }
            "remove-prefix" => {
                println!("remove-prefix functionality - use separate binary for now");
                return Ok(());
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown subcommand: {}", args[1]);
                print_help();
                process::exit(1);
            }
        }
    }
    
    // Default behavior: longest prefix detection
    process_directories_longest_prefix()
}

fn print_help() {
    println!("🔧 FTMI - File Tools for Mass Interaction");
    println!();
    println!("USAGE:");
    println!("    ftmi [SUBCOMMAND] [OPTIONS] [ARGS...]");
    println!("    echo 'directory' | ftmi [SUBCOMMAND] [OPTIONS]");
    println!();
    println!("SUBCOMMANDS:");
    println!("    rename              Interactive prefix removal tool (main functionality)");
    println!("    extract-paths       Extract file paths from text input");
    println!("    find-delimited      Find delimited prefixes like [Artist], (Draft)");
    println!("    find-specific       Search for specific prefix patterns");
    println!("    detect-all          Detect all common prefixes automatically");
    println!("    remove-prefix       Preview prefix removal operations");
    println!();
    println!("DEFAULT (no subcommand):");
    println!("    Analyzes directories for longest prefix detection");
    println!();
    println!("EXAMPLES:");
    println!("    # Main interactive renaming tool");
    println!("    ftmi rename ./music");
    println!("    ftmi rename --continuous");
    println!("    ftmi rename --undo");
    println!();
    println!("    # Analysis (default behavior)");
    println!("    ftmi");
    println!("    echo './music' | ftmi");
    println!();
    println!("    # Other tools");
    println!("    ftmi extract-paths < logfile.txt");
    println!("    ftmi find-delimited ./photos");
    println!();
    println!("For detailed help on each subcommand, use:");
    println!("    ftmi <subcommand> --help");
}