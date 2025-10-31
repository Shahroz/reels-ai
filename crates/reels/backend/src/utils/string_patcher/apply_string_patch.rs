//! Applies a patch string to an original content string and returns the patched string.
//!
//! This function uses the `apply-patch` crate to parse a patch and apply its changes
//! to a given string in memory. It is intended for scenarios where modifications
//! are needed without direct file system interaction.
//! The patch format should conform to what the `apply-patch` crate expects.
//! Internally, it uses a temporary file to interface with `apply-patch`'s file-based functions.
//! Currently, it only supports patches with a single `UpdateFile` hunk.

// Note: Per rust_guidelines, using fully qualified paths.

/// Applies a text-based patch to an original string content.
///
/// Takes a patch string (in the `apply-patch` format) and the original string content.
/// Returns the new string content after applying the patch, or an error string
/// if parsing or application fails.
///
/// # Arguments
/// * `patch_str` - A string slice representing the patch to apply.
/// * `original_content` - A string slice representing the original content.
///
/// # Returns
/// * `Result<String, String>` - The patched string on success, or an error message on failure.
///
/// # Errors
/// Returns an error if:
/// - The patch string is invalid.
/// - The patch does not contain exactly one `UpdateFile` hunk.
/// - A temporary file cannot be created or written to.
/// - The patch cannot be applied (e.g., context lines not found by `apply-patch` logic).
pub fn apply_patch_to_string(patch_str: &str, original_content: &str) -> std::result::Result<std::string::String, std::string::String> {
    let hunks = match crate::apply_patch::parser::parse_patch(patch_str) {
        Ok(h) => h,
        Err(e) => return std::result::Result::Err(std::format!("Failed to parse patch: {}", e)),
    };

    if hunks.len() != 1 {
        return std::result::Result::Err(std::format!(
            "Expected exactly one hunk in the patch, found {}",
            hunks.len()
        ));
    }

    match &hunks[0] {
        crate::apply_patch::parser::Hunk::UpdateFile { chunks, .. } => {
            // Create a temporary file
            let mut temp_file = match tempfile::NamedTempFile::new() {
                Ok(f) => f,
                Err(e) => return std::result::Result::Err(std::format!("Failed to create temp file: {}", e)),
            };
            
            if let Err(e) = std::io::Write::write_all(&mut temp_file, original_content.as_bytes()) {
                 return std::result::Result::Err(std::format!("Failed to write to temp file: {}", e));
            }
            
            // Ensure data is flushed to disk before apply-patch reads it
            if let Err(e) = temp_file.flush() {
                return std::result::Result::Err(std::format!("Failed to flush temp file: {}", e));
            }
            
            match crate::apply_patch::unified_diff_from_chunks(temp_file.path(), chunks) {
                Ok(update_result) => std::result::Result::Ok(update_result.content),
                Err(e) => std::result::Result::Err(std::format!("Failed to derive new contents from chunks: {}", e)),
            }
            // Temp file is automatically deleted when `temp_file` goes out of scope.
        }
        _ => std::result::Result::Err(std::string::String::from(
            "Patch hunk was not an UpdateFile hunk, which is required for string patching.",
        )),
    }
}

#[cfg(test)]
mod tests {
    // Per rust_guidelines, use fully qualified paths for items not in prelude.
    // `super::*` is allowed for the item under test.

    #[test]
    fn test_simple_replace() {
        let original = "Hello, world!\nThis is a test.\n";
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\nHello, world!\n-This is a test.\n+This is a successful test.\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("Hello, world!\nThis is a successful test.\n".to_string()));
    }

    #[test]
    fn test_add_line() {
        let original = "First line.\n";
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\nFirst line.\n+Second line.\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("First line.\nSecond line.\n".to_string()));
    }

    #[test]
    fn test_delete_line() {
        let original = "Line to keep.\nLine to delete.\nAnother line to keep.\n";
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\nLine to keep.\n-Line to delete.\nAnother line to keep.\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("Line to keep.\nAnother line to keep.\n".to_string()));
    }

    #[test]
    fn test_empty_original_add_line() {
        let original = "";
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\n+Hello, new world!\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("Hello, new world!\n".to_string()));
    }
    
    #[test]
    fn test_patch_empty_file_no_trailing_newline_in_patch() {
        let original_content = "";
        let patch_content = "*** Begin Patch\n*** Update File: test.txt\n@@\n+Hello\n*** End Patch";
        let result = super::apply_patch_to_string(patch_content, original_content);
        assert_eq!(result, Ok("Hello\n".to_string()));
    }

    #[test]
    fn test_no_trailing_newline_in_original() {
        let original = "Hello, world!"; // No trailing newline
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\n-Hello, world!\n+Hello, patched world!\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("Hello, patched world!\n".to_string()));
    }
    
    #[test]
    fn test_complex_patch_multiple_changes() {
        let original = "Line one\nLine two\nLine three\nLine four\nLine five\n";
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\nLine one\n-Line two\n+Line NEW two\nLine three\n@@\nLine three\n-Line four\n+Line NEW four\nLine five\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("Line one\nLine NEW two\nLine three\nLine NEW four\nLine five\n".to_string()));
    }

    #[test]
    fn test_invalid_patch_format() {
        let original = "Some content.";
        let patch = "This is not a valid patch.";
        let result = super::apply_patch_to_string(patch, original);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse patch"));
    }

    #[test]
    fn test_patch_context_not_found() {
        let original = "Actual first line.\nActual second line.\n";
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\nExpected first line (not present)\n-Actual second line.\n+Patched second line.\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to derive new contents from chunks"));
    }
    
    #[test]
    fn test_add_to_eof() {
        let original = "line1\nline2\n";
        let patch_str = "*** Begin Patch\n*** Update File: test.txt\n@@\nline2\n+line3\n*** End of File\n*** End Patch";
        let result = super::apply_patch_to_string(patch_str, original);
        assert_eq!(result, std::result::Result::Ok("line1\nline2\nline3\n".to_string()));
    }

    #[test]
    fn test_patch_with_add_file_hunk_fails() {
        let original = "Some content.";
        let patch = "*** Begin Patch\n*** Add File: new_file.txt\n+Hello\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Patch hunk was not an UpdateFile hunk, which is required for string patching.");
    }
    
    #[test]
    fn test_unicode_normalization_in_context() {
        let original = "import asyncio  # local import \u{2013} avoids top\u{2011}level dep\n"; // EN DASH, NON-BREAKING HYPHEN
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\n-import asyncio  # local import - avoids top-level dep\n+import asyncio  # HELLO\n*** End Patch"; // ASCII hyphen
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("import asyncio  # HELLO\n".to_string()));
    }

    #[test]
    fn test_insert_at_eof_string() {
        let original = "foo\nbar\nbaz\n";
        let patch = "*** Begin Patch\n*** Update File: <string_content>\n@@\n+quux\n*** End of File\n*** End Patch";
        let result = super::apply_patch_to_string(patch, original);
        assert_eq!(result, std::result::Result::Ok("foo\nbar\nbaz\nquux\n".to_string()));
    }
}
