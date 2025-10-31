//! Provides a function to sanitize HTML output from an Language Model.
//!
//! This function attempts to clean up common issues found in LLM-generated HTML,
//! such as markdown code fences and extraneous text outside the main HTML tags.
//! It aims to extract a valid, self-contained HTML string from the raw input.
//! The sanitization is heuristic and focuses on common LLM output patterns.

/// Sanitizes raw string output, expecting HTML, from an LLM.
///
/// It performs the following steps:
/// 1. Trims leading/trailing whitespace from the input string.
/// 2. Removes common markdown code fences (e.g., "```html\n", "```").
/// 3. Extracts the substring between the first '<' and the last '>'.
/// 4. Returns `Some(String)` containing the cleaned, non-empty HTML string
///    (which will represent the content between the first '<' and last '>'
///    of the processed input) if successful, otherwise returns `None`.
///
/// # Arguments
///
/// * `raw_output`: A string slice (`&str`) representing the raw output from the LLM.
///
/// # Returns
///
/// * `Option<std::string::String>`: The sanitized HTML string or `None`.
pub fn sanitize_llm_html_output(raw_output: &str) -> Option<std::string::String> {
    let mut processed_output = std::string::String::from(raw_output.trim());

    // List of prefixes and suffixes for code fences
    // Order matters: more specific (like "```html\n") should be checked before less specific ("```")
    // to handle cases where one might be a substring of another if not careful.
    // However, `trim_start_matches` and `trim_end_matches` are greedy, so the current structure is okay.
    let fences_to_check = [
        ("```html\n", "\n```"), // With newline before content
        ("```html", "```"),    // No newline before content
        ("```\n", "\n```"),     // Generic fence with newline
        ("```", "```"),        // Generic fence
    ];

    for (prefix, suffix) in fences_to_check.iter() {
        if processed_output.starts_with(prefix) && processed_output.ends_with(suffix) {
            // Ensure prefix and suffix don't make up the whole string or overlap incorrectly
            if processed_output.len() >= prefix.len() + suffix.len() {
                 processed_output = std::string::String::from(
                    processed_output[prefix.len()..(processed_output.len() - suffix.len())]
                        .trim(),
                );
                break; // Found and removed a fence type, no need to check others
            } else if processed_output == *prefix && processed_output == *suffix { // e.g. input is "```"
                 processed_output = std::string::String::from("");
                 break;
            }
        }
    }
    // Final trim after potential fence removal
    processed_output = std::string::String::from(processed_output.trim());


    // Extract content between the first '<' and last '>'
    match processed_output.find('<') {
        Some(start_index) => match processed_output.rfind('>') {
            Some(end_index_inclusive) if end_index_inclusive >= start_index => {
                let extracted_html =
                    processed_output[start_index..=end_index_inclusive].to_string();
                if extracted_html.trim().is_empty() {
                    None
                } else {
                    Some(extracted_html)
                }
            }
            _ => None, // No closing '>' or it's before '<'
        },
        None => None, // No opening '<'
    }
}

#[cfg(test)]
mod tests {
    // Note: Per guidelines, using `super::` to access the function under test.
    // Full paths for other items like `std::string::String` or `Option` (though `Option` is often in prelude).

    #[test]
    fn test_sanitize_with_html_markdown_fence_and_newlines() {
        let raw = "```html\n<html><body><p>Hello</p></body></html>\n```";
        let expected = "<html><body><p>Hello</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }

    #[test]
    fn test_sanitize_with_html_markdown_fence_no_newlines() {
        let raw = "```html<html><body><p>Hello</p></body></html>```";
        let expected = "<html><body><p>Hello</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }

    #[test]
    fn test_sanitize_with_generic_markdown_fence_and_newlines() {
        let raw = "```\n<html><body><p>Hello</p></body></html>\n```";
        let expected = "<html><body><p>Hello</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }

    #[test]
    fn test_sanitize_with_generic_markdown_fence_no_newlines() {
        let raw = "```<html><body><p>Hello</p></body></html>```";
        let expected = "<html><body><p>Hello</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }

    #[test]
    fn test_sanitize_clean_html() {
        let raw = "<html><body><p>Hello</p></body></html>";
        let expected = "<html><body><p>Hello</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }

    #[test]
    fn test_sanitize_html_with_leading_trailing_whitespace() {
        let raw = "  \n<html><body><p>Hello</p></body></html>\t  ";
        let expected = "<html><body><p>Hello</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }

    #[test]
    fn test_sanitize_html_with_extra_text_outside() {
        let raw = "Some leading text <html><body><p>Hello</p></body></html> and some trailing text";
        let expected = "<html><body><p>Hello</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }
    
    #[test]
    fn test_sanitize_html_with_extra_text_and_fence() {
        let raw = "Here is the HTML: ```html\n<p>Test</p>\n``` That's all.";
         let expected = "<p>Test</p>"; // The outer text is stripped by find < and rfind >
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }


    #[test]
    fn test_sanitize_no_html_tags() {
        let raw = "Just some plain text without any tags.";
        assert_eq!(super::sanitize_llm_html_output(raw), None);
    }

    #[test]
    fn test_sanitize_empty_string() {
        let raw = "";
        assert_eq!(super::sanitize_llm_html_output(raw), None);
    }

    #[test]
    fn test_sanitize_whitespace_only_string() {
        let raw = "   \n \t ";
        assert_eq!(super::sanitize_llm_html_output(raw), None);
    }

    #[test]
    fn test_sanitize_fence_around_whitespace() {
        let raw = "```html\n   \n```";
        assert_eq!(super::sanitize_llm_html_output(raw), None);
    }
    
    #[test]
    fn test_sanitize_only_fence() {
        let raw = "```html```";
        assert_eq!(super::sanitize_llm_html_output(raw), None);
    }

   #[test]
   fn test_sanitize_incomplete_html_tags() {
       let raw = "<p>Missing closing tag";
        // The current sanitizer logic extracts the substring from the first '<' to the last '>'.
        // For the input "<p>Missing closing tag", this results in "<p>".
        // The original comment implying rfind('>') would fail was incorrect for this specific input,
        // as a '>' is found (it's the closing bracket of the <p> tag itself).
        // To return None for such unclosed tags (e.g. <p> without subsequent </p>),
        // the sanitizer would need more sophisticated HTML parsing.
        // Given the current documented behavior, Some("<p>") is the expected output.
        assert_eq!(super::sanitize_llm_html_output(raw), Some(std::string::String::from("<p>")));
   }

   #[test]
    fn test_sanitize_malformed_fence() {
        let raw = "```html\n<p>Content</p>"; // missing closing fence
        let expected = "<p>Content</p>"; // Should still extract based on < >
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }
    
    #[test]
    fn test_sanitize_content_with_internal_fences_should_fail_gracefully() {
        // This test case highlights a limitation: it won't deeply parse to distinguish
        // actual HTML content from parts of markdown if LLM mistakenly embeds fences inside.
        // The current logic is to find the *outermost* valid HTML block after stripping *outer* fences.
        let raw = "<html><body><p>```html internal code ```</p></body></html>";
        let expected = "<html><body><p>```html internal code ```</p></body></html>";
        assert_eq!(
            super::sanitize_llm_html_output(raw),
            Some(std::string::String::from(expected))
        );
    }
}
