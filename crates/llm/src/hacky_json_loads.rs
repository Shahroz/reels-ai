 //! Provides a 'hacky' way to load JSON strings that might be malformed.
 //!
 //! This module contains `hacky_json_loads`, a function that attempts to parse
//! a string as JSON using several heuristics. It tries direct parsing, removing/escaping
//! newlines, and isolating the outermost JSON structure.
//! It's intended for cases where JSON might be embedded in noisy text or slightly broken.

// Helper function to escape newlines, carriage returns, and tabs *only* within JSON string literals.
// Improved version with more robust escape sequence handling.
fn escape_control_chars_in_json_strings(s: &str) -> String {
    let mut sb = String::with_capacity(s.len() + s.len() / 10); // Extra capacity for escapes
    let mut in_string = false;
    let mut chars = s.chars().peekable();

    while let Some(char_code) = chars.next() {
        match char_code {
            '\\' if in_string => {
                // Handle escape sequences more carefully
                sb.push('\\');
                if let Some(&next_char) = chars.peek() {
                    match next_char {
                        '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u' => {
                            // Valid JSON escape sequence, consume and add the next character
                            sb.push(chars.next().unwrap());
                        }
                        _ => {
                            // Not a standard JSON escape, but don't change string state
                        }
                    }
                }
            }
            '\\' if !in_string => {
                // Backslash outside string, just pass through
                sb.push('\\');
            }
            '"' => {
                // More robust quote detection: count preceding backslashes
                let mut backslash_count = 0;
                let mut temp = sb.clone();
                
                // Count consecutive backslashes at the end
                while temp.ends_with('\\') {
                    backslash_count += 1;
                    temp.pop();
                }
                
                // If even number of backslashes (including 0), the quote is not escaped
                if backslash_count % 2 == 0 {
                    in_string = !in_string;
                }
                sb.push('"');
            }
            '\n' if in_string => {
                // Newline inside a string: escape it.
                sb.push_str("\\n");
            }
            '\r' if in_string => {
                // Carriage return inside a string: escape it.
                sb.push_str("\\r");
            }
            '\t' if in_string => {
                // Tab inside a string: escape it.
                sb.push_str("\\t");
            }
            _ => {
                // Any other character, append as is.
                sb.push(char_code);
            }
        }
    }
    sb
}

// Helper function to detect if content likely contains problematic long strings with formatting
fn has_long_formatted_strings(content: &str) -> bool {
    // Look for patterns that suggest markdown/formatted content in JSON strings
    content.contains("### ") || content.contains("**") || content.contains("\n\n-") || 
    content.contains("\n\n*") || content.lines().count() > 50
}

fn find_and_parse_outermost_structure(content: &str) -> String {
    let first_brace = content.find('{');
    let first_bracket = content.find('[');
    let last_brace = content.rfind('}');
    let last_bracket = content.rfind(']');

    let (start, end) = match (first_brace, first_bracket, last_brace, last_bracket) {
        (Some(fb), Some(fbk), Some(lb), Some(lbk)) => {
            // Choose the earliest opening and the latest closing delimiters.
            let start = fb.min(fbk);
            let end = lb.max(lbk) + 1; // Include the closing character.
            (start, end)
        }
        (Some(fb), None, Some(lb), None) => (fb, lb + 1),
        (None, Some(fbk), None, Some(lbk)) => (fbk, lbk + 1),
        _ => return content.to_string(),
    };

    content[start..end].to_string()
}

pub fn hacky_json_loads(raw_content: &str) -> Option<serde_json::Value> {
    // First attempt: Direct parsing.
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(raw_content) {
        return Some(data);
    }

    let content = find_and_parse_outermost_structure(&raw_content);

    // Second attempt: Escape newlines, carriage returns, and tabs *only* within string literals.
    // This is often the main issue with "broken" JSON from sources that don't escape control chars in strings.
    let selectively_escaped_content = escape_control_chars_in_json_strings(&content);
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&selectively_escaped_content) {
        return Some(data);
    }

    // Third attempt: If we detect long formatted strings, try aggressive global escaping
    // This handles cases where string boundaries are unclear due to complex content
    if has_long_formatted_strings(&content) {
        let globally_escaped = content.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t");
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&globally_escaped) {
            return Some(data);
        }
    }

    // Fourth attempt: Replace all newlines and carriage returns with spaces globally.
    // This is a more aggressive and potentially lossy approach for string content,
    // but might fix structural issues caused by errant newlines outside of strings.
    let without_any_newlines = content.replace("\n", " ").replace("\r", " ");
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&without_any_newlines) {
        return Some(data);
    }

    // Fifth attempt: Combined approach - apply selective escaping to globally modified content
    // This covers cases where both approaches are needed
    let double_processed = escape_control_chars_in_json_strings(&without_any_newlines);
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&double_processed) {
        return Some(data);
    }

    // Sixth attempt: Last resort - aggressive global escaping without detection
    // For cases where the detection heuristic fails but global escaping would work
    let last_resort_escaped = content.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t");
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&last_resort_escaped) {
        return Some(data);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_valid_json() {
        // Ensure basic functionality still works
        let simple_json = r#"{"key": "value", "number": 42}"#;
        let result = hacky_json_loads(simple_json);
        assert!(result.is_some(), "Should parse valid JSON");
        
        let parsed = result.unwrap();
        assert_eq!(parsed["key"], "value");
        assert_eq!(parsed["number"], 42);
    }

    #[test]
    fn test_json_with_unescaped_newlines() {
        // Test JSON with literal newlines in string values
        let problematic_json = "{\n  \"content\": \"Line 1\nLine 2\nLine 3\"\n}";
        let result = hacky_json_loads(problematic_json);
        assert!(result.is_some(), "Should handle unescaped newlines in strings");
        
        let parsed = result.unwrap();
        let content = parsed["content"].as_str().unwrap();
        assert!(content.contains("Line 1"));
        assert!(content.contains("Line 2"));
        assert!(content.contains("Line 3"));
    }

    #[test]
    fn test_markdown_content() {
        // Test with markdown-like content that has formatting characters
        let markdown_json = "{\n  \"text\": \"### Title\n\n**Bold text** and `code`.\n\n- Item 1\n- Item 2\"\n}";
        let result = hacky_json_loads(markdown_json);
        assert!(result.is_some(), "Should handle markdown-like content");
        
        let parsed = result.unwrap();
        let text = parsed["text"].as_str().unwrap();
        assert!(text.contains("### Title"));
        assert!(text.contains("**Bold text**"));
        assert!(text.contains("- Item 1"));
    }

    #[test]
    fn test_original_failing_case() {
        // Test with a simplified version of the original failing JSON structure
        let complex_json = "{\n  \"agent_reasoning\": \"I have gathered all the necessary information about Bounti.ai and the target company, Spellbook.\",\n  \"user_answer\": \"### **Bounti.ai Go-to-Market Use Cases for Spellbook**\n\nHere is the comprehensive research and sales pitch material based on Bounti.ai's capabilities.\n\n--- \n\n### **Use Case 1: Automated Go-to-Market Strategy & ICP Definition**\n\n**Title:** From Funding to Market Leadership\n\n**Description:** For a rapidly growing company like Spellbook, which has just secured significant funding, this use case focuses on using Bounti's AI to analyze the legal tech market, identify the most profitable customer segments (e.g., large law firms vs. in-house counsel at tech companies), and define the Ideal Customer Profile (ICP) with precision.\n\n**Pain:**\n- Uncertainty about which market segment to prioritize for the fastest growth.\n- Difficulty in creating a data-backed GTM strategy without a large, dedicated market research team.\n- Risk of misallocating new funding on broad, untargeted marketing campaigns.\n\n**Results:**\n- A clear, actionable GTM strategy document delivered in days, not months.\n- Precise ICP and persona definitions, enabling hyper-targeted marketing and sales messaging.\n- Significant reduction in time and resources spent on manual market analysis.\",\n  \"title\": \"Bounti.ai GTM Sales Pitch for Spellbook\",\n  \"is_final\": true,\n  \"actions\": []\n}";

        let result = hacky_json_loads(complex_json);
        assert!(result.is_some(), "Should handle the original failing JSON structure");
        
        if let Some(parsed) = result {
            assert!(parsed.get("agent_reasoning").is_some(), "Should have agent_reasoning field");
            assert!(parsed.get("user_answer").is_some(), "Should have user_answer field");
            assert!(parsed.get("title").is_some(), "Should have title field");
            assert!(parsed.get("is_final").is_some(), "Should have is_final field");
            assert!(parsed.get("actions").is_some(), "Should have actions field");
            
            // Verify the long content is preserved
            let user_answer = parsed["user_answer"].as_str().unwrap();
            assert!(user_answer.contains("### **Bounti.ai Go-to-Market Use Cases for Spellbook**"));
            assert!(user_answer.contains("**Title:** From Funding to Market Leadership"));
            assert!(user_answer.len() > 1000, "Long content should be preserved");
        }
    }

    #[test]
    fn test_long_formatted_string_detection() {
        // Test the helper function
        assert!(has_long_formatted_strings("### Title\n\n**Bold**\n\n- Item"));
        assert!(has_long_formatted_strings("**Bold text**"));
        assert!(has_long_formatted_strings("\n\n- List item"));
        assert!(!has_long_formatted_strings("Simple string"));
        
        // Test with many lines
        let many_lines = (0..60).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
        assert!(has_long_formatted_strings(&many_lines));
    }

    #[test]
    fn test_improved_escape_handling() {
        // Test better handling of escape sequences
        let json_with_escapes = r#"{"text": "He said \"Hello\" and then said \"Goodbye\""}"#;
        let result = hacky_json_loads(json_with_escapes);
        assert!(result.is_some(), "Should handle escaped quotes properly");
        
        let parsed = result.unwrap();
        let text = parsed["text"].as_str().unwrap();
        assert!(text.contains("\"Hello\""));
        assert!(text.contains("\"Goodbye\""));
    }

    #[test]
    fn test_mixed_content() {
        // Test with mixed problematic content
        let mixed_json = "{\n  \"description\": \"This is a **bold** statement.\n\nIt contains multiple paragraphs with:\n- Bullet points\n- And \\\"quoted text\\\"\n- Plus some more content\n\n### A heading\n\nMore text here.\",\n  \"metadata\": {\n    \"length\": 123,\n    \"type\": \"markdown\"\n  }\n}";
        
        let result = hacky_json_loads(mixed_json);
        assert!(result.is_some(), "Should handle mixed problematic content");
        
        let parsed = result.unwrap();
        assert!(parsed.get("description").is_some());
        assert!(parsed.get("metadata").is_some());
        
        let description = parsed["description"].as_str().unwrap();
        assert!(description.contains("**bold**"));
        assert!(description.contains("### A heading"));
    }

    #[test]
    #[ignore]
    fn test_broken_json_2() {
        let broken_json = include_str!("broken_jsons/broken.json");
        let result = super::hacky_json_loads(broken_json);
        // Assert that parsing is successful, as the original test implied by unwrap and print.
        // If this assertion fails, it indicates hacky_json_loads couldn't handle broken.json.
        assert!(result.is_some(), "hacky_json_loads failed to parse broken_jsons/broken.json");
        if let Some(value) = result {
            println!("{}", value); // Keep println for diagnostics if successful
        }
    }
}
