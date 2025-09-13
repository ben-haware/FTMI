use regex::Regex;
use std::collections::BTreeSet;
use std::io::{self, BufRead};

fn main() {
    let mut paths = BTreeSet::new();
    let stdin = io::stdin();
    
    let path_regex = Regex::new(r#"(?x)
        # Unix absolute paths
        (?:^|[\s"'])(/(?:[^/\s"']+/)*[^/\s"']+)(?:$|[\s"'])
        |
        # Unix relative paths with ./ or ../
        (?:^|[\s"'])(\.\.?/(?:[^/\s"']+/)*[^/\s"']+)(?:$|[\s"'])
        |
        # Windows paths with drive letter
        (?:^|[\s"'])([A-Za-z]:\\(?:[^\\/:*?"<>|\s]+\\)*[^\\/:*?"<>|\s]+)(?:$|[\s"'])
        |
        # UNC paths
        (?:^|[\s"'])(\\\\[^\\/:*?"<>|\s]+\\[^\\/:*?"<>|\s]+(?:\\[^\\/:*?"<>|\s]+)*)(?:$|[\s"'])
    "#).unwrap();
    
    for line in stdin.lock().lines() {
        if let Ok(text) = line {
            for cap in path_regex.captures_iter(&text) {
                for i in 1..=cap.len()-1 {
                    if let Some(matched) = cap.get(i) {
                        let path = matched.as_str();
                        if !path.is_empty() {
                            paths.insert(path.to_string());
                        }
                    }
                }
            }
        }
    }
    
    let final_paths = deduplicate_paths(paths);
    
    for path in final_paths {
        println!("{}", path);
    }
}

fn deduplicate_paths(paths: BTreeSet<String>) -> Vec<String> {
    let mut sorted_paths: Vec<String> = paths.into_iter().collect();
    sorted_paths.sort_by(|a, b| b.len().cmp(&a.len()));
    
    let mut result: Vec<String> = Vec::new();
    
    for path in &sorted_paths {
        let mut is_subpath = false;
        
        for existing in &result {
            if is_subpath_of(path, existing) {
                is_subpath = true;
                break;
            }
        }
        
        if !is_subpath {
            result.push(path.clone());
        }
    }
    
    result.sort();
    result
}

fn is_subpath_of(potential_sub: &str, parent: &str) -> bool {
    let normalized_sub = normalize_path(potential_sub);
    let normalized_parent = normalize_path(parent);
    
    if normalized_sub == normalized_parent {
        return false;
    }
    
    if normalized_parent.starts_with(&normalized_sub) {
        let remainder = &normalized_parent[normalized_sub.len()..];
        return remainder.starts_with('/') || remainder.starts_with('\\');
    }
    
    false
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/home/user/"), "/home/user");
        assert_eq!(normalize_path("/home/user"), "/home/user");
        assert_eq!(normalize_path("C:\\Users\\test"), "C:/Users/test");
        assert_eq!(normalize_path("C:\\Users\\test\\"), "C:/Users/test");
        assert_eq!(normalize_path("\\\\server\\share\\"), "//server/share");
    }

    #[test]
    fn test_is_subpath_of() {
        // Basic Unix paths
        assert!(is_subpath_of("/home", "/home/user"));
        assert!(is_subpath_of("/home/user", "/home/user/documents"));
        assert!(!is_subpath_of("/home/user", "/home/user"));
        assert!(!is_subpath_of("/home/user2", "/home/user"));
        assert!(!is_subpath_of("/usr", "/home/user"));
        
        // Windows paths
        assert!(is_subpath_of("C:\\Users", "C:\\Users\\test"));
        assert!(is_subpath_of("C:/Users", "C:\\Users\\test\\file.txt"));
        assert!(!is_subpath_of("C:\\Users\\test2", "C:\\Users\\test"));
        
        // Edge cases
        assert!(!is_subpath_of("/home/user", "/home/username"));
        assert!(!is_subpath_of("/home/use", "/home/user"));
    }

    #[test]
    fn test_deduplicate_paths() {
        let mut paths = BTreeSet::new();
        paths.insert("/home/user".to_string());
        paths.insert("/home/user/documents".to_string());
        paths.insert("/home/user/documents/report.pdf".to_string());
        paths.insert("/usr/bin".to_string());
        paths.insert("/usr/bin/ls".to_string());
        
        let result = deduplicate_paths(paths);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"/home/user/documents/report.pdf".to_string()));
        assert!(result.contains(&"/usr/bin/ls".to_string()));
    }

    #[test]
    fn test_deduplicate_paths_windows() {
        let mut paths = BTreeSet::new();
        paths.insert("C:\\Users".to_string());
        paths.insert("C:\\Users\\test".to_string());
        paths.insert("C:\\Users\\test\\Documents".to_string());
        paths.insert("C:\\Program Files".to_string());
        
        let result = deduplicate_paths(paths);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"C:\\Users\\test\\Documents".to_string()));
        assert!(result.contains(&"C:\\Program Files".to_string()));
    }

    #[test]
    fn test_deduplicate_paths_mixed() {
        let mut paths = BTreeSet::new();
        paths.insert("/home/user".to_string());
        paths.insert("/home/user/project".to_string());
        paths.insert("./relative/path".to_string());
        paths.insert("./relative/path/file.txt".to_string());
        paths.insert("C:\\Windows".to_string());
        paths.insert("C:\\Windows\\System32".to_string());
        
        let result = deduplicate_paths(paths);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"/home/user/project".to_string()));
        assert!(result.contains(&"./relative/path/file.txt".to_string()));
        assert!(result.contains(&"C:\\Windows\\System32".to_string()));
    }

    #[test]
    fn test_path_regex_unix_absolute() {
        let regex = create_path_regex();
        
        let text = "Check the file /home/user/test.txt please";
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "/home/user/test.txt");
        
        let text = "Files: /usr/bin/ls and /etc/config";
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec!["/usr/bin/ls", "/etc/config"]);
    }

    #[test]
    fn test_path_regex_unix_relative() {
        let regex = create_path_regex();
        
        let text = "Run ./scripts/build.sh now";
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "./scripts/build.sh");
        
        let text = "Check ../parent/file.txt and ./current/file.txt";
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec!["../parent/file.txt", "./current/file.txt"]);
    }

    #[test]
    fn test_path_regex_windows() {
        let regex = create_path_regex();
        
        let text = r#"File at "C:\Users\test\file.txt" found"#;
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], r"C:\Users\test\file.txt");
        
        let text = r#"Paths: "C:\Windows\System32" and "D:\Data\report.pdf""#;
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec![r"C:\Windows\System32", r"D:\Data\report.pdf"]);
    }

    #[test]
    fn test_path_regex_unc() {
        let regex = create_path_regex();
        
        let text = r#"Network path "\\server\share\file.txt" available"#;
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], r"\\server\share\file.txt");
    }

    #[test]
    fn test_path_regex_quoted_paths() {
        let regex = create_path_regex();
        
        // Note: Our regex doesn't capture paths with spaces, even in quotes
        // This is a known limitation - paths with spaces need to be escaped
        let text = r#"Use '/home/user/myfile.txt' and "/usr/local/bin/app""#;
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec!["/home/user/myfile.txt", "/usr/local/bin/app"]);
        
        // Test that we do capture the non-space part
        let text = r#"Use '/home/user/my file.txt' path"#;
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec!["/home/user/my"]);
    }

    #[test]
    fn test_path_regex_edge_cases() {
        let regex = create_path_regex();
        
        // Path at start of line
        let text = "/home/user/file.txt is the path";
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec!["/home/user/file.txt"]);
        
        // Path at end of line
        let text = "The path is /home/user/file.txt";
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec!["/home/user/file.txt"]);
        
        // Multiple paths without quotes
        let text = "Compare /etc/hosts with /etc/hostname files";
        let paths = extract_all_paths(&regex, text);
        assert_eq!(paths, vec!["/etc/hosts", "/etc/hostname"]);
    }

    // Helper functions for tests
    fn create_path_regex() -> Regex {
        Regex::new(r#"(?x)
            # Unix absolute paths
            (?:^|[\s"'])(/(?:[^/\s"']+/)*[^/\s"']+)(?:$|[\s"'])
            |
            # Unix relative paths with ./ or ../
            (?:^|[\s"'])(\.\.?/(?:[^/\s"']+/)*[^/\s"']+)(?:$|[\s"'])
            |
            # Windows paths with drive letter
            (?:^|[\s"'])([A-Za-z]:\\(?:[^\\/:*?"<>|\s]+\\)*[^\\/:*?"<>|\s]+)(?:$|[\s"'])
            |
            # UNC paths
            (?:^|[\s"'])(\\\\[^\\/:*?"<>|\s]+\\[^\\/:*?"<>|\s]+(?:\\[^\\/:*?"<>|\s]+)*)(?:$|[\s"'])
        "#).unwrap()
    }

    fn extract_all_paths(regex: &Regex, text: &str) -> Vec<String> {
        let mut paths = Vec::new();
        for cap in regex.captures_iter(text) {
            for i in 1..=cap.len()-1 {
                if let Some(matched) = cap.get(i) {
                    let path = matched.as_str();
                    if !path.is_empty() {
                        paths.push(path.to_string());
                    }
                }
            }
        }
        paths
    }
}