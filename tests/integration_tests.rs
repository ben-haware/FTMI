use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn test_simple_path_extraction() {
    let output = run_ftmi("Found file at /home/user/test.txt");
    assert_eq!(output.trim(), "/home/user/test.txt");
}

#[test]
fn test_subpath_deduplication() {
    let input = "Found /home/user/documents/report.pdf and /home/user/documents";
    let output = run_ftmi(input);
    assert_eq!(output.trim(), "/home/user/documents/report.pdf");
}

#[test]
fn test_multiple_paths() {
    let input = r#"Check /usr/bin/ls and /etc/config files"#;
    let output = run_ftmi(input);
    let lines: Vec<&str> = output.trim().split('\n').collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.contains(&"/etc/config"));
    assert!(lines.contains(&"/usr/bin/ls"));
}

#[test]
fn test_windows_paths() {
    let input = r#"File at "C:\Users\test\file.txt" exists"#;
    let output = run_ftmi(input);
    assert_eq!(output.trim(), r"C:\Users\test\file.txt");
}

#[test]
fn test_relative_paths() {
    let input = "Run ./scripts/build.sh and check ../config/settings.json";
    let output = run_ftmi(input);
    let lines: Vec<&str> = output.trim().split('\n').filter(|s| !s.is_empty()).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.contains(&"../config/settings.json"));
    assert!(lines.contains(&"./scripts/build.sh"));
}

#[test]
fn test_unc_paths() {
    let input = r#"Access "\\server\share\file.txt" on network"#;
    let output = run_ftmi(input);
    assert_eq!(output.trim(), r"\\server\share\file.txt");
}

#[test]
fn test_mixed_path_types() {
    let input = r#"Files: /home/user/doc.txt, "C:\Windows\System32", ./local/file.sh"#;
    let output = run_ftmi(input);
    let lines: Vec<&str> = output.trim().split('\n').collect();
    assert_eq!(lines.len(), 3);
    assert!(lines.contains(&"./local/file.sh"));
    assert!(lines.contains(&"/home/user/doc.txt,"));
    assert!(lines.contains(&r"C:\Windows\System32"));
}

#[test]
fn test_nested_paths_deduplication() {
    let input = concat!(
        "Paths found:\n",
        "/home/user\n",
        "/home/user/project\n",
        "/home/user/project/src\n",
        "/home/user/project/src/main.rs\n",
        "/home/other/file.txt"
    );
    let output = run_ftmi(input);
    let lines: Vec<&str> = output.trim().split('\n').collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.contains(&"/home/other/file.txt"));
    assert!(lines.contains(&"/home/user/project/src/main.rs"));
}

#[test]
fn test_empty_input() {
    let output = run_ftmi("");
    assert_eq!(output.trim(), "");
}

#[test]
fn test_no_paths() {
    let output = run_ftmi("This text contains no valid paths");
    assert_eq!(output.trim(), "");
}

// Helper function to run ftmi extract-paths with input
fn run_ftmi(input: &str) -> String {
    // Use the main ftmi binary with extract-paths subcommand
    let binary_path = env!("CARGO_BIN_EXE_ftmi");
    
    let mut child = Command::new(binary_path)
        .arg("extract-paths")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ftmi extract-paths");

    // Write input and explicitly close stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
        // Dropping stdin here explicitly closes it
    }

    let output = child.wait_with_output().expect("Failed to read output");
    
    // Check for errors
    if !output.status.success() {
        eprintln!("ftmi extract-paths failed with stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    String::from_utf8_lossy(&output.stdout).to_string()
}