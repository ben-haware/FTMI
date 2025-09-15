use ftmi::process_directories_longest_prefix;
use std::env;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "rename" => {
                return ftmi::subcommands::rename_command(args);
            }
            "analyze" => {
                // Execute the analysis functionality
                return process_directories_longest_prefix();
            }
            "extract-paths" => {
                return ftmi::subcommands::extract_paths_command(args);
            }
            "find-delimited" => {
                return ftmi::subcommands::find_delimited_command(args);
            }
            "find-specific" => {
                return ftmi::subcommands::find_specific_command(args);
            }
            "detect-all" => {
                return ftmi::subcommands::detect_all_command(args);
            }
            "remove-prefix" => {
                return ftmi::subcommands::remove_prefix_command(args);
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
    
    // Default behavior: show help
    print_help();
    Ok(())
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
    println!("    analyze             Analyze directories for longest prefix detection");
    println!("    extract-paths       Extract file paths from text input");
    println!("    find-delimited      Find delimited prefixes like [Artist], (Draft)");
    println!("    find-specific       Search for specific prefix patterns");
    println!("    detect-all          Detect all common prefixes automatically");
    println!("    remove-prefix       Preview prefix removal operations");
    println!();
    println!("DEFAULT (no subcommand):");
    println!("    Shows this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Main interactive renaming tool");
    println!("    ftmi rename ./music");
    println!("    ftmi rename --continuous");
    println!("    ftmi rename --undo");
    println!();
    println!("    # Analysis");
    println!("    ftmi analyze");
    println!("    echo './music' | ftmi analyze");
    println!();
    println!("    # Other tools");
    println!("    ftmi extract-paths < logfile.txt");
    println!("    ftmi find-delimited ./photos");
    println!();
    println!("For detailed help on each subcommand, use:");
    println!("    ftmi <subcommand> --help");
}