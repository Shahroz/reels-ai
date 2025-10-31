use serde_yaml::{self, Value};

// Updated helper to extract YAML content enclosed in triple backticks or single backticks
fn extract_outside_backticks(content: &str) -> String {
    let content = content.trim();

    // ```yaml
    if let Some(start) = content.find("```yaml") {
        if let Some(end) = content.rfind("```") {
            if start < end {
                return content[start + 7..end].to_string();
            }
        }
    }

    // Prefer triple backticks
    if let Some(start) = content.find("```") {
        if let Some(end) = content.rfind("```") {
            if start < end {
                return content[start + 3..end].to_string();
            }
        }
    }
    // Fallback to single backticks
    if let Some(first) = content.find('`') {
        if let Some(last) = content.rfind('`') {
            if first < last {
                return content[first + 1..last].to_string();
            }
        }
    }
    content.to_string()
}

/// Attempts to parse YAML content with several strategies, including handling content wrapped in backticks.
pub fn hacky_yaml_loads(raw_content: &str) -> Option<Value> {
    // First attempt: Direct parsing
    if let Ok(data) = serde_yaml::from_str::<Value>(raw_content) {
        return Some(data);
    }

    // Second attempt: Extract YAML content enclosed in backticks (triple or single) and try parsing it
    let extracted = extract_outside_backticks(raw_content);
    if extracted != raw_content {
        if let Ok(data) = serde_yaml::from_str::<Value>(&extracted) {
            return Some(data);
        }
    }

    // Third attempt: Trim whitespace and try parsing
    let trimmed = raw_content.trim();
    if let Ok(data) = serde_yaml::from_str::<Value>(trimmed) {
        return Some(data);
    }

    // Fourth attempt: Replace newlines with spaces and try parsing
    let without_newlines = trimmed.replace("\n", " ");
    if let Ok(data) = serde_yaml::from_str::<Value>(&without_newlines) {
        return Some(data);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;

    #[test]
    fn test_direct_parsing() {
        let yaml = "key: value";
        let result = hacky_yaml_loads(yaml);
        assert!(result.is_some());
        if let Some(Value::Mapping(map)) = result {
            assert_eq!(map.get(&Value::String("key".to_string())).unwrap(), &Value::String("value".to_string()));
        }
    }

    #[test]
    fn test_single_backticks() {
        let yaml = "`key: value`";
        let result = hacky_yaml_loads(yaml);
        assert!(result.is_some());
        if let Some(Value::Mapping(map)) = result {
            assert_eq!(map.get(&Value::String("key".to_string())).unwrap(), &Value::String("value".to_string()));
        }
    }

    #[test]
    fn test_triple_backticks() {
        let yaml = "```key: value```";
        let result = hacky_yaml_loads(yaml);
        assert!(result.is_some());
        if let Some(Value::Mapping(map)) = result {
            assert_eq!(map.get(&Value::String("key".to_string())).unwrap(), &Value::String("value".to_string()));
        }
    }

    #[test]
    fn test_other_backticks() {
        let yaml = r#"'```yaml
reasoning: Create a simple Python script containing a "Hello World" function that accepts a name parameter.
actions:
- !create_file
  human_name: Create hello_world.py script
  filename: hello_world.py
  content: |-
    def hello(name):
        return f"Hello, {name}!"

    if __name__ == "__main__":
        name = input("Enter your name: ")
        print(hello(name))
```"#;
        let result = hacky_yaml_loads(yaml);
        assert!(result.is_some());
    }
}
