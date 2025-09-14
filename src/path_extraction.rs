use regex::Regex;
use std::collections::BTreeSet;

pub fn extract_paths_from_text(text: &str) -> Vec<String> {
    let mut paths = BTreeSet::new();
    let path_regex = create_path_regex();
    
    for line in text.lines() {
        for cap in path_regex.captures_iter(line) {
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
    
    deduplicate_paths(paths)
}

pub fn deduplicate_paths(paths: BTreeSet<String>) -> Vec<String> {
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
    fn test_extract_paths_from_text() {
        let text = "Check /home/user/test.txt and /home/user for files";
        let paths = extract_paths_from_text(text);
        assert_eq!(paths, vec!["/home/user/test.txt"]);
        
        let text = r#"Files at "C:\Users\test\doc.pdf" and /usr/bin/app"#;
        let paths = extract_paths_from_text(text);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"C:\\Users\\test\\doc.pdf".to_string()));
        assert!(paths.contains(&"/usr/bin/app".to_string()));
    }
}