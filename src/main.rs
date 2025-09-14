use ftmi::process_directories_from_stdin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    process_directories_from_stdin()
}