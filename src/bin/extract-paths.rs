use ftmi::process_paths_from_stdin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    process_paths_from_stdin()
}