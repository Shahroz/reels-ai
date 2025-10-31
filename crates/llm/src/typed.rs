use serde::Serialize;
use serde_json;
use serde_yaml;
use toml;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("TOML serialization error: {0}")]
    Toml(#[from] toml::ser::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    YAML,
    TOML,
}

/// Serializes the given data into the specified format (JSON, YAML, TOML, or JSON with CDATA support).
/// For JsonCData, it uses serde_json_cdata::to_string. Note: When using JsonCData, the data must be compatible with JsonCDataValue.
pub fn serialize_data<T: Serialize>(data: &T, format: SerializationFormat) -> Result<String, SerializationError> {
    match format {
        SerializationFormat::Json => {
            serde_json::to_string_pretty(data).map_err(SerializationError::Json)
        },
        SerializationFormat::YAML => {
            let yaml = serde_yaml::to_string(data).map_err(SerializationError::Yaml)?;
            Ok(yaml.replace("'<![CDATA[", "<![CDATA[").replace("]]>'", "]]>") )
        },
        SerializationFormat::TOML => {
            let toml_str = toml::to_string_pretty(data).map_err(SerializationError::Toml)?;
            Ok(convert_toml_multiline_to_literals(&toml_str))
        },
    }
}


// Helper function to convert TOML multiline strings to literal strings for better LLM compatibility
pub fn convert_toml_multiline_to_literals(toml_str: &str) -> String {
    // Regular expression would be ideal here, but let's use a simpler approach
    let mut result = String::new();
    let mut in_multiline = false;
    let mut multiline_content = String::new();
    
    for line in toml_str.lines() {
        if line.contains("\"\"\"") && !in_multiline {
            // Start of multiline string
            in_multiline = true;
            
            // Get the position of the triple quotes
            let _pos = line.find("\"\"\"");
            
            // Add everything before the triple quotes
            if let Some(part) = line.split("\"\"\"").next() {
                result.push_str(part);
                result.push_str("'''");
            }
            
            // Capture content after opening quotes
            if let Some(content) = line.split("\"\"\"").nth(1) {
                multiline_content.push_str(content);
                // If there's a closing triple quote on the same line
                if content.contains("\"\"\"") {
                    in_multiline = false;
                    let processed = content.replace("\"\"\"", "'''");
                    result.push_str(&processed);
                    result.push('\n');
                } else {
                    multiline_content.push('\n');
                }
            } else {
                // No content after opening quotes
                multiline_content.push('\n');
            }
        } else if line.contains("\"\"\"") && in_multiline {
            // End of multiline string
            in_multiline = false;
            
            // Add content before the closing triple quotes
            if let Some(part) = line.split("\"\"\"").next() {
                multiline_content.push_str(part);
            }
            
            // Convert the multiline content to a literal string
            result.push_str(&escape_for_literal_string(&multiline_content));
            
            // Add content after the closing triple quotes
            if let Some(content) = line.split("\"\"\"").nth(1) {
                result.push_str("'''");
                result.push_str(content);
            }
            result.push('\n');
            
            // Reset multiline content
            multiline_content.clear();
        } else if in_multiline {
            // Inside multiline string
            multiline_content.push_str(line);
            multiline_content.push('\n');
        } else {
            // Regular line
            result.push_str(line);
            result.push('\n');
        }
    }
    
    result
}

// Helper function to escape content for TOML literal strings
fn escape_for_literal_string(content: &str) -> String {
    // Literal strings (''') don't need escaping except for single quotes that need to be doubled
    content.replace("'", "''")
}

// Additional functions for few-shot serialization/deserialization can utilize the above functions accordingly.

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[test]
    fn test_toml_multiline_conversion() {
        // Test with a simple struct containing multiline strings
        #[derive(Serialize, Deserialize)]
        struct TestData {
            title: String,
            description: String,
            code: String,
        }

        let test_data = TestData {
            title: "Test Title".to_string(),
            description: "This is a\nmultiline\ndescription".to_string(),
            code: "function test() {\n  console.log(\"Hello\");\n  return true;\n}".to_string(),
        };

        let toml_string = serialize_data(&test_data, SerializationFormat::TOML).unwrap();

        // The result should use literal strings (''') instead of basic multiline strings (""")
        assert!(toml_string.contains("'''"), "TOML output should contain literal string markers");
        assert!(!toml_string.contains("\"\"\""), "TOML output should not contain basic multiline string markers");
        
        // Make sure the content is preserved
        assert!(toml_string.contains("function test()"), "Content should be preserved in conversion");
        
        // The conversion should be reversible but we need to use toml directly due to trait bounds in deserialize_data
        let deserialized: TestData = toml::from_str(&toml_string).unwrap();
        assert_eq!(deserialized.title, test_data.title);
        assert_eq!(deserialized.description, test_data.description);
        assert_eq!(deserialized.code, test_data.code);
    }
}
