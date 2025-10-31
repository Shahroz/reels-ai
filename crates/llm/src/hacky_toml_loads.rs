use toml::Value;

// Extract TOML content enclosed in triple backticks or single backticks
fn extract_outside_backticks(content: &str) -> String {
    // Do not trim content to preserve original code blocks when unmatched
    // content is used directly

    // Handle content wrapped in triple backticks ```...```
    if content.starts_with("```") && content.ends_with("```") {
        // Strip opening and closing fences
        let mut inner = &content[3..content.len() - 3];
        // Remove leading newline if present
        if inner.starts_with('\n') {
            inner = &inner[1..];
        }
        // Skip a language tag on the first line (letters-only)
        if let Some(nl) = inner.find('\n') {
            let first = &inner[..nl];
            if first.chars().all(|c| c.is_alphanumeric()) {
                inner = &inner[nl + 1..];
            }
        }
        // Remove trailing newline if present
        if inner.ends_with('\n') {
            inner = &inner[..inner.len() - 1];
        }
        return inner.to_string();
    }

    // Single backtick wrapper, only if content starts and ends with backtick
    if content.starts_with('`') && content.ends_with('`') {
        return content[1..content.len() - 1].to_string();
    }

    content.to_string()
}

/// Attempts to parse TOML content with several strategies, including handling content wrapped in backticks.
pub fn hacky_toml_loads(raw_content: &str) -> Option<Value> {
    // Prepare trimmed content of raw input
    let trimmed = raw_content.trim();
    // If content is empty or only whitespace, treat as empty TOML table
    if trimmed.is_empty() {
        return Some(Value::Table(Default::default()));
    }
    // Attempt 1: parse the trimmed raw content directly
    if let Ok(data) = toml::from_str::<Value>(trimmed) {
        return Some(data);
    }

    // Attempt 2: extract content from code fences/backticks, then preprocess and parse
    let extracted = extract_outside_backticks(raw_content);
    // Convert basic multiline strings ("""...""") into literal ones ('''...''') only in extracted content
    let preprocessed_inner = extracted.replace("\"\"\"", "'''");
    let inner_trimmed = preprocessed_inner.trim();
    if let Ok(data) = toml::from_str::<Value>(inner_trimmed) {
        return Some(data);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    #[test]
    fn test_direct_parsing() {
        let toml_str = r#"
            [package]
            name = "example"
            version = "0.1.0"
        "#;
        let result = hacky_toml_loads(toml_str);
        assert!(result.is_some());
        if let Some(Value::Table(table)) = result {
            if let Some(Value::Table(package)) = table.get("package") {
                assert_eq!(package.get("name").unwrap().as_str().unwrap(), "example");
                assert_eq!(package.get("version").unwrap().as_str().unwrap(), "0.1.0");
            } else {
                panic!("Expected package table");
            }
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_single_backticks() {
        let toml_str = "`[package]\nname = \"example\"\nversion = \"0.1.0\"`";
        let result = hacky_toml_loads(toml_str);
        assert!(result.is_some());
        if let Some(Value::Table(table)) = result {
            if let Some(Value::Table(package)) = table.get("package") {
                assert_eq!(package.get("name").unwrap().as_str().unwrap(), "example");
                assert_eq!(package.get("version").unwrap().as_str().unwrap(), "0.1.0");
            } else {
                panic!("Expected package table");
            }
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_triple_backticks() {
        let toml_str = "```[package]\nname = \"example\"\nversion = \"0.1.0\"```";
        let result = hacky_toml_loads(toml_str);
        assert!(result.is_some());
        if let Some(Value::Table(table)) = result {
            if let Some(Value::Table(package)) = table.get("package") {
                assert_eq!(package.get("name").unwrap().as_str().unwrap(), "example");
                assert_eq!(package.get("version").unwrap().as_str().unwrap(), "0.1.0");
            } else {
                panic!("Expected package table");
            }
        } else {
            panic!("Expected table");
        }
    }

    #[test]
    fn test_toml_backticks() {
        let toml_str = r#"```toml
        [package]
        name = "example"
        version = "0.1.0"
        ```"#;
        let result = hacky_toml_loads(toml_str);
        assert!(result.is_some());
        if let Some(Value::Table(table)) = result {
            if let Some(Value::Table(package)) = table.get("package") {
                assert_eq!(package.get("name").unwrap().as_str().unwrap(), "example");
                assert_eq!(package.get("version").unwrap().as_str().unwrap(), "0.1.0");
            } else {
                panic!("Expected package table");
            }
        } else {
            panic!("Expected table");
        }
    }
    // Adversarial cases: unmatched fences and preservation of content
    #[test]
    fn test_extract_unmatched_triple_backticks_preserves_content() {
        let input = r#"```toml
foo = "bar"
"#;
        // Extraction should return original content unchanged
        assert_eq!(extract_outside_backticks(input), input);
        // Parsing should fail for unmatched fences
        assert!(hacky_toml_loads(input).is_none());
    }

    #[test]
    fn test_extract_unmatched_single_backtick_preserves_content() {
        let input = r#"`foo = "bar"
"#;
        assert_eq!(extract_outside_backticks(input), input);
        assert!(hacky_toml_loads(input).is_none());
    }

    #[test]
    fn test_extract_language_tag_not_skipped_when_non_alphanumeric() {
        let input = r#"```toml-lang
key = 1
```"#;
        let extracted = extract_outside_backticks(input);
        // Since the language tag contains a hyphen, it should not be skipped
        assert_eq!(extracted, "toml-lang\nkey = 1");
        assert!(hacky_toml_loads(input).is_none());
    }
}