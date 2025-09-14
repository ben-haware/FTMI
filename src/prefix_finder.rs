use std::fs;
use std::path::Path;
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum PrefixMode {
    /// Only search for prefixes within specified delimiters
    DelimiterOnly {
        delimiters: Vec<(String, String)>,
    },
    /// Only search for specific prefixes
    SpecificPrefixes {
        prefixes: Vec<String>,
    },
    /// Detect all possible prefixes (current behavior)
    DetectAll {
        delimiters: Vec<(String, String)>,
    },
}

#[derive(Debug, Clone)]
pub struct PrefixOptions {
    pub mode: PrefixMode,
    pub min_occurrences: usize,
    /// Regex pattern to filter prefixes (e.g., r"\[.*\]" for bracket-delimited prefixes)
    pub filter_regex: Option<String>,
}

impl Default for PrefixOptions {
    fn default() -> Self {
        Self {
            mode: PrefixMode::DetectAll {
                delimiters: vec![
                    ("(".to_string(), ")".to_string()),
                    ("[".to_string(), "]".to_string()),
                    ("{".to_string(), "}".to_string()),
                    ("\"".to_string(), "\"".to_string()),
                    ("'".to_string(), "'".to_string()),
                ],
            },
            min_occurrences: 2,
            filter_regex: Some(r"\[.*\]".to_string()), // Default to bracket-delimited prefixes
        }
    }
}

impl PrefixOptions {
    /// Create options with a custom regex filter
    pub fn with_regex(regex_pattern: &str) -> Self {
        Self {
            filter_regex: Some(regex_pattern.to_string()),
            ..Default::default()
        }
    }
    
    /// Create options with no regex filter (accept all prefixes)
    pub fn no_filter() -> Self {
        Self {
            filter_regex: None,
            ..Default::default()
        }
    }
    
    /// Create options for bracket-delimited prefixes specifically
    pub fn bracket_only() -> Self {
        Self {
            filter_regex: Some(r"\[.*\]".to_string()),
            ..Default::default()
        }
    }
    
    /// Create options for parentheses-delimited prefixes
    pub fn paren_only() -> Self {
        Self {
            filter_regex: Some(r"\(.*\)".to_string()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommonPrefix {
    pub prefix: String,
    pub delimiter: Option<(String, String)>,
    pub occurrences: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrefixedPath {
    pub paths: Vec<std::path::PathBuf>,
    pub prefix: String,
}

pub fn find_common_prefix(directory: &Path, options: &PrefixOptions) -> Result<Vec<CommonPrefix>, std::io::Error> {
    let mut prefix_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut delimiter_prefix_map: HashMap<(String, Option<(String, String)>), Vec<String>> = HashMap::new();
    
    // Read all files in the directory
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                match &options.mode {
                    PrefixMode::DelimiterOnly { delimiters } => {
                        // Only check for prefixes within delimiters
                        for (open, close) in delimiters {
                            if let Some(prefix) = extract_prefix_with_delimiter(filename, open, close) {
                                let key = (prefix.clone(), Some((open.clone(), close.clone())));
                                delimiter_prefix_map.entry(key).or_insert_with(Vec::new).push(filename.to_string());
                            }
                        }
                    },
                    PrefixMode::SpecificPrefixes { prefixes } => {
                        // Only check for specific prefixes
                        for prefix in prefixes {
                            if filename.starts_with(prefix) {
                                prefix_map.entry(prefix.clone()).or_insert_with(Vec::new).push(filename.to_string());
                            }
                        }
                    },
                    PrefixMode::DetectAll { delimiters } => {
                        // Check for prefixes within delimiters
                        for (open, close) in delimiters {
                            if let Some(prefix) = extract_prefix_with_delimiter(filename, open, close) {
                                let key = (prefix.clone(), Some((open.clone(), close.clone())));
                                delimiter_prefix_map.entry(key).or_insert_with(Vec::new).push(filename.to_string());
                            }
                        }
                        
                        // Also check for common prefixes without delimiters
                        let prefix_candidates = generate_prefix_candidates(filename);
                        for prefix in prefix_candidates {
                            prefix_map.entry(prefix).or_insert_with(Vec::new).push(filename.to_string());
                        }
                    }
                }
            }
        }
    }
    
    let mut results = Vec::new();
    
    // Process delimiter-based prefixes
    for ((prefix, delimiter), files) in delimiter_prefix_map {
        if files.len() >= options.min_occurrences {
            results.push(CommonPrefix {
                prefix,
                delimiter,
                occurrences: files.len(),
                files,
            });
        }
    }
    
    // Process non-delimiter prefixes
    let mut non_delimiter_results: Vec<CommonPrefix> = Vec::new();
    for (prefix, mut files) in prefix_map {
        if files.len() >= options.min_occurrences {
            // Deduplicate files
            files.sort();
            files.dedup();
            
            // After deduplication, check if we still meet minimum occurrences
            if files.len() < options.min_occurrences {
                continue;
            }
            
            // Check if this prefix is already covered by a delimiter-based prefix
            let covered = results.iter().any(|cp| {
                cp.delimiter.is_some() && files.iter().all(|f| cp.files.contains(f))
            });
            
            if !covered {
                // Skip prefixes that end with an open delimiter
                if prefix.ends_with('[') || prefix.ends_with('(') || prefix.ends_with('{') ||
                   prefix.ends_with('"') || prefix.ends_with('\'') {
                    continue;
                }
                
                non_delimiter_results.push(CommonPrefix {
                    prefix,
                    delimiter: None,
                    occurrences: files.len(),
                    files,
                });
            }
        }
    }
    
    // Remove redundant prefixes (e.g., if we have "IMG_2024" don't also show "IMG", "IMG_", etc.)
    non_delimiter_results.sort_by(|a, b| {
        // Sort by prefix length (longest first) then by occurrences
        b.prefix.len().cmp(&a.prefix.len()).then(b.occurrences.cmp(&a.occurrences))
    });
    
    let mut filtered_results = Vec::new();
    for candidate in non_delimiter_results {
        // Check if this prefix's files are a subset of any already selected prefix
        let is_subset = filtered_results.iter().any(|selected: &CommonPrefix| {
            candidate.files.iter().all(|f| selected.files.contains(f)) &&
            selected.prefix.starts_with(&candidate.prefix)
        });
        
        if !is_subset {
            filtered_results.push(candidate);
        }
    }
    
    results.extend(filtered_results);
    
    // Sort by number of occurrences (descending)
    results.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
    
    Ok(results)
}

fn extract_prefix_with_delimiter(filename: &str, open: &str, close: &str) -> Option<String> {
    if let Some(open_pos) = filename.find(open) {
        if let Some(close_pos) = filename[open_pos + open.len()..].find(close) {
            let prefix = &filename[open_pos + open.len()..open_pos + open.len() + close_pos];
            if !prefix.is_empty() {
                return Some(prefix.to_string());
            }
        }
    }
    None
}

fn generate_prefix_candidates(filename: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    
    // Remove file extension
    let name = if let Some(pos) = filename.rfind('.') {
        &filename[..pos]
    } else {
        filename
    };
    
    // Generate prefixes based on common separators
    let separators = vec!['_', '-', '.', ' '];
    
    for separator in separators {
        let parts: Vec<&str> = name.split(separator).collect();
        if parts.len() > 1 {
            // Try prefixes of increasing length
            for i in 1..parts.len() {
                let prefix = parts[..i].join(&separator.to_string());
                if !prefix.is_empty() {
                    // Skip if it looks like a partial delimiter match
                    if prefix.ends_with('[') || prefix.ends_with('(') || prefix.ends_with('{') {
                        continue;
                    }
                    candidates.push(prefix);
                }
            }
        }
    }
    
    // Also try character-based prefixes (first n characters)
    // Skip single character prefixes to avoid noise
    for i in 2..name.len().min(20) {
        let candidate = &name[..i];
        // Skip if it looks like a partial delimiter match
        if candidate.ends_with('[') || candidate.ends_with('(') || candidate.ends_with('{') {
            continue;
        }
        candidates.push(candidate.to_string());
    }
    
    candidates
}

/// Extract prefix from a filename and return (prefix, remaining_filename)
pub fn extract_prefix_from_filename(filename: &str, options: &PrefixOptions) -> Option<(String, String)> {
    match &options.mode {
        PrefixMode::DelimiterOnly { delimiters } => {
            for (open, close) in delimiters {
                if let Some(prefix) = extract_prefix_with_delimiter(filename, open, close) {
                    let prefix_with_delim = format!("{}{}{}", open, prefix, close);
                    if filename.starts_with(&prefix_with_delim) {
                        let remaining = filename[prefix_with_delim.len()..].trim_start();
                        return Some((prefix, remaining.to_string()));
                    }
                }
            }
            None
        },
        PrefixMode::SpecificPrefixes { prefixes } => {
            for prefix in prefixes {
                if filename.starts_with(prefix) {
                    let remaining = filename[prefix.len()..].trim_start();
                    return Some((prefix.clone(), remaining.to_string()));
                }
            }
            None
        },
        PrefixMode::DetectAll { delimiters } => {
            // First try delimiter-based extraction
            for (open, close) in delimiters {
                if let Some(prefix) = extract_prefix_with_delimiter(filename, open, close) {
                    let prefix_with_delim = format!("{}{}{}", open, prefix, close);
                    if filename.starts_with(&prefix_with_delim) {
                        let remaining = filename[prefix_with_delim.len()..].trim_start();
                        return Some((prefix, remaining.to_string()));
                    }
                }
            }
            
            // If no delimiter match, try to detect common prefix patterns
            // This is a simplified version - in practice you'd want to use the results
            // from find_common_prefix to determine the actual prefix
            None
        }
    }
}

/// Remove prefix from a filename
pub fn remove_prefix(filename: &str, prefix: &str) -> String {
    if filename.starts_with(prefix) {
        filename[prefix.len()..].trim_start().to_string()
    } else {
        filename.to_string()
    }
}

/// Remove prefix with delimiter from a filename
pub fn remove_prefix_with_delimiter(filename: &str, prefix: &str, open: &str, close: &str) -> String {
    let prefix_with_delim = format!("{}{}{}", open, prefix, close);
    if filename.starts_with(&prefix_with_delim) {
        filename[prefix_with_delim.len()..].trim_start().to_string()
    } else {
        filename.to_string()
    }
}

/// Find the longest matching prefixes for a directory and return structured results
/// Uses configurable regex pattern to filter prefixes 
/// Returns multiple results if there are ties in occurrence count
pub fn find_longest_prefix(directory: &Path, options: &PrefixOptions) -> Result<Vec<PrefixedPath>, std::io::Error> {
    let all_prefixes = find_common_prefix(directory, options)?;
    
    if all_prefixes.is_empty() {
        return Ok(Vec::new());
    }
    
    // Filter prefixes using regex pattern if provided
    let filtered_prefixes: Vec<&CommonPrefix> = if let Some(regex_pattern) = &options.filter_regex {
        match Regex::new(regex_pattern) {
            Ok(regex) => {
                all_prefixes.iter()
                    .filter(|prefix| {
                        // Create the full prefix pattern based on delimiter
                        let full_prefix = if let Some((open, close)) = &prefix.delimiter {
                            format!("{}{}{}", open, prefix.prefix, close)
                        } else {
                            prefix.prefix.clone()
                        };
                        regex.is_match(&full_prefix)
                    })
                    .collect()
            }
            Err(e) => {
                eprintln!("Warning: Invalid regex pattern '{}': {}", regex_pattern, e);
                all_prefixes.iter().collect()
            }
        }
    } else {
        all_prefixes.iter().collect()
    };
    
    let candidates = if !filtered_prefixes.is_empty() {
        filtered_prefixes
    } else {
        // Fall back to any prefix if no filtered prefixes found
        all_prefixes.iter().collect()
    };
    
    // Find the maximum occurrence count
    let max_occurrences = candidates.iter()
        .map(|prefix| prefix.occurrences)
        .max()
        .unwrap_or(0);
    
    // Collect all prefixes with the maximum occurrence count
    let best_prefixes: Vec<&CommonPrefix> = candidates.iter()
        .filter(|prefix| prefix.occurrences == max_occurrences)
        .cloned()
        .collect();
    
    // Convert to PrefixedPath results
    let results: Vec<PrefixedPath> = best_prefixes.iter()
        .map(|prefix| {
            let paths: Vec<std::path::PathBuf> = prefix.files.iter()
                .map(|filename| directory.join(filename))
                .collect();
            
            PrefixedPath {
                paths,
                prefix: prefix.prefix.clone(),
            }
        })
        .collect();
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_extract_prefix_with_delimiter() {
        assert_eq!(
            extract_prefix_with_delimiter("file[prefix]_001.txt", "[", "]"),
            Some("prefix".to_string())
        );
        assert_eq!(
            extract_prefix_with_delimiter("(TEST)_file.pdf", "(", ")"),
            Some("TEST".to_string())
        );
        assert_eq!(
            extract_prefix_with_delimiter("no_delimiter.txt", "[", "]"),
            None
        );
    }

    #[test]
    fn test_generate_prefix_candidates() {
        let candidates = generate_prefix_candidates("test_file_001.txt");
        assert!(candidates.contains(&"test".to_string()));
        assert!(candidates.contains(&"test_file".to_string()));
        
        let candidates = generate_prefix_candidates("prefix-document.pdf");
        assert!(candidates.contains(&"prefix".to_string()));
    }

    #[test]
    fn test_find_common_prefix_delimiter_only() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();
        
        // Create test files
        File::create(dir_path.join("[PROJECT]_doc1.txt"))?;
        File::create(dir_path.join("[PROJECT]_doc2.txt"))?;
        File::create(dir_path.join("[PROJECT]_doc3.txt"))?;
        File::create(dir_path.join("test_file_001.txt"))?;
        File::create(dir_path.join("test_file_002.txt"))?;
        File::create(dir_path.join("other.txt"))?;
        
        let options = PrefixOptions {
            mode: PrefixMode::DelimiterOnly {
                delimiters: vec![("[".to_string(), "]".to_string())],
            },
            min_occurrences: 2,
        };
        let results = find_common_prefix(dir_path, &options)?;
        
        // Should only find [PROJECT] with delimiter, no other prefixes
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].prefix, "PROJECT");
        assert!(results[0].delimiter.is_some());
        assert_eq!(results[0].occurrences, 3);
        
        Ok(())
    }

    #[test]
    fn test_find_common_prefix_specific() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();
        
        // Create test files
        File::create(dir_path.join("IMG_001.jpg"))?;
        File::create(dir_path.join("IMG_002.jpg"))?;
        File::create(dir_path.join("DOC_001.pdf"))?;
        File::create(dir_path.join("DOC_002.pdf"))?;
        File::create(dir_path.join("other.txt"))?;
        
        let options = PrefixOptions {
            mode: PrefixMode::SpecificPrefixes {
                prefixes: vec!["IMG_".to_string(), "DOC_".to_string()],
            },
            min_occurrences: 1,
        };
        let results = find_common_prefix(dir_path, &options)?;
        
        // Should find both IMG_ and DOC_ prefixes
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|cp| cp.prefix == "IMG_" && cp.occurrences == 2));
        assert!(results.iter().any(|cp| cp.prefix == "DOC_" && cp.occurrences == 2));
        
        Ok(())
    }

    #[test]
    fn test_find_common_prefix_detect_all() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();
        
        // Create test files
        File::create(dir_path.join("[PROJECT]_doc1.txt"))?;
        File::create(dir_path.join("[PROJECT]_doc2.txt"))?;
        File::create(dir_path.join("[PROJECT]_doc3.txt"))?;
        File::create(dir_path.join("test_file_001.txt"))?;
        File::create(dir_path.join("test_file_002.txt"))?;
        File::create(dir_path.join("other.txt"))?;
        
        let options = PrefixOptions::default(); // Uses DetectAll mode
        let results = find_common_prefix(dir_path, &options)?;
        
        // Should find [PROJECT] with delimiter
        assert!(results.iter().any(|cp| cp.prefix == "PROJECT" && cp.delimiter.is_some()));
        
        // Should find test_file prefix (or a more specific one like test_file_00)
        assert!(results.iter().any(|cp| cp.prefix.starts_with("test_file") && cp.delimiter.is_none()));
        
        Ok(())
    }

    #[test]
    fn test_extract_prefix_from_filename() {
        let options = PrefixOptions {
            mode: PrefixMode::DelimiterOnly {
                delimiters: vec![("[".to_string(), "]".to_string())],
            },
            min_occurrences: 1,
        };
        
        let result = extract_prefix_from_filename("[Artist] Song.mp3", &options);
        assert_eq!(result, Some(("Artist".to_string(), "Song.mp3".to_string())));
        
        let result = extract_prefix_from_filename("No Delimiter Song.mp3", &options);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_prefix_specific() {
        let options = PrefixOptions {
            mode: PrefixMode::SpecificPrefixes {
                prefixes: vec!["IMG_".to_string(), "DOC_".to_string()],
            },
            min_occurrences: 1,
        };
        
        let result = extract_prefix_from_filename("IMG_001.jpg", &options);
        assert_eq!(result, Some(("IMG_".to_string(), "001.jpg".to_string())));
        
        let result = extract_prefix_from_filename("DOC_report.pdf", &options);
        assert_eq!(result, Some(("DOC_".to_string(), "report.pdf".to_string())));
        
        let result = extract_prefix_from_filename("OTHER_file.txt", &options);
        assert_eq!(result, None);
    }

    #[test]
    fn test_remove_prefix() {
        assert_eq!(remove_prefix("IMG_001.jpg", "IMG_"), "001.jpg");
        assert_eq!(remove_prefix("test_file.txt", "test_"), "file.txt");
        assert_eq!(remove_prefix("no_match.txt", "IMG_"), "no_match.txt");
    }

    #[test]
    fn test_remove_prefix_with_delimiter() {
        assert_eq!(
            remove_prefix_with_delimiter("[Artist] Song.mp3", "Artist", "[", "]"),
            "Song.mp3"
        );
        assert_eq!(
            remove_prefix_with_delimiter("(Draft) Document.pdf", "Draft", "(", ")"),
            "Document.pdf"
        );
        assert_eq!(
            remove_prefix_with_delimiter("No Match.txt", "Artist", "[", "]"),
            "No Match.txt"
        );
    }
}