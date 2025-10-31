//! Provides a function to write `LlmTypedLog` entries to a YAML file.
//!
//! This utility function serializes an `LlmTypedLog` struct into YAML format.
//! It then writes the serialized data to a specified file within a given directory.
//! The function handles file creation and I/O operations, returning an error on failure.
//! It is designed for internal use within the `llm_typed_unified` module for persisting LLM interaction logs.

//! Revision History
//! - 2025-05-16T14:45:49Z @AI: Initial creation of the `write_log` function and file.

/// Writes a given `LlmTypedLog` entry to a YAML file in the specified directory.
///
/// The function first ensures the target directory exists, creating it recursively if not.
/// It then serializes the `log_entry` to YAML format and writes it to a file
/// named `file_name` within `log_dir`.
///
/// # Arguments
///
/// * `log_entry`: A reference to the `LlmTypedLog` instance to be written.
/// * `log_dir`: A reference to the `std::path::Path` representing the directory where the log file should be saved.
/// * `file_name`: A string slice representing the name of the log file.
///
/// # Returns
///
/// * `std::result::Result<(), std::string::String>`: An empty `Ok` on success, or an `Err`
///   containing a descriptive error message string on failure (e.g., I/O error, serialization error).
pub fn write_log(
    log_entry: &crate::llm_typed_unified::llm_typed_log::LlmTypedLog,
    log_dir: &std::path::Path,
    file_name: &str,
) -> std::result::Result<(), std::string::String> {
    // Ensure the log directory exists.
    std::fs::create_dir_all(log_dir).map_err(|e| {
        std::format!(
            "Failed to create log directory '{}': {}",
            log_dir.display(),
            e
        )
    })?;

    let log_file_path = log_dir.join(file_name);

    // Serialize the log entry to YAML.
    let yaml_content =
        serde_yaml::to_string(log_entry).map_err(|e| std::format!("Failed to serialize log entry to YAML: {}", e))?;

    // Write the YAML content to the file.
    std::fs::write(&log_file_path, yaml_content)
        .map_err(|e| std::format!("Failed to write log to file '{}': {}", log_file_path.display(), e))?;

    std::result::Result::Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Per rust_guidelines, no `use` statements. Access items via `super::` or full paths.

    #[test]
    fn test_write_log_success() {
        // Create a unique temporary directory for this test.
        let temp_dir_path_buf = std::env::temp_dir().join(std::format!(
            "test_write_log_success_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::create_dir_all(&temp_dir_path_buf)
            .expect("Failed to create temp dir for test_write_log_success");

        let log_entry = crate::llm_typed_unified::llm_typed_log::LlmTypedLog {
            request_id: std::option::Option::Some(std::string::String::from("test-req-001")),
            timestamp: std::string::String::from("2025-05-16T15:00:00Z"),
            model_name: std::string::String::from("test-gpt-model"),
            prompt_tokens: std::option::Option::Some(15),
            completion_tokens: std::option::Option::Some(25),
            total_tokens: std::option::Option::Some(40),
            request_payload: serde_json::json!({ "prompt": "Test prompt content" }),
            response_payload: serde_json::json!({ "completion": "Test completion content" }),
            error_message: std::option::Option::None,
            duration_ms: std::option::Option::Some(150),
        };
        let file_name = "test_log_001.yaml";

        let result = super::write_log(&log_entry, &temp_dir_path_buf, file_name);
        std::assert!(
            result.is_ok(),
            "write_log returned an error: {:?}",
            result.err()
        );

        let log_file_path = temp_dir_path_buf.join(file_name);
        std::assert!(
            log_file_path.exists(),
            "Log file was not created at '{}'",
            log_file_path.display()
        );

        let content = std::fs::read_to_string(&log_file_path)
            .expect("Failed to read created log file content");

        std::assert!(content.contains("request_id: test-req-001"));
        std::assert!(content.contains("model_name: test-gpt-model"));
        std::assert!(content.contains("prompt: Test prompt content")); // Check part of JSON payload

        // Cleanup: Remove the created file and directory.
        std::fs::remove_file(&log_file_path).expect("Failed to remove test log file");
        std::fs::remove_dir_all(&temp_dir_path_buf).expect("Failed to remove test temp directory");
    }

    #[test]
    fn test_write_log_directory_creation_failure() {
        // Create a file where we'll try to create a directory for logging.
        let dummy_file_path_base = std::env::temp_dir().join(std::format!(
            "dummy_file_for_dir_fail_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::write(&dummy_file_path_base, "This is a file, not a directory.")
            .expect("Failed to create dummy file for test");

        // This path is now a file, so create_dir_all should fail if we try to make a dir *at* this path.
        let log_dir_path_attempt = &dummy_file_path_base;

        let log_entry = crate::llm_typed_unified::llm_typed_log::LlmTypedLog {
            request_id: std::option::Option::None,
            timestamp: std::string::String::from("2025-01-01T00:00:00Z"),
            model_name: std::string::String::from("error-test-model"),
            prompt_tokens: std::option::Option::None,
            completion_tokens: std::option::Option::None,
            total_tokens: std::option::Option::None,
            request_payload: serde_json::json!(null),
            response_payload: serde_json::json!(null),
            error_message: std::option::Option::None,
            duration_ms: std::option::Option::None,
        };
        let file_name = "error_log.yaml";

        let result = super::write_log(&log_entry, log_dir_path_attempt, file_name);
        std::assert!(result.is_err(), "write_log should have failed due to directory creation issue");

        if let std::result::Result::Err(e) = result {
            std::assert!(
                e.contains("Failed to create log directory"),
                "Error message mismatch: received '{}'",
                e
            );
        }

        // Cleanup: Remove the dummy file.
        std::fs::remove_file(&dummy_file_path_base).expect("Failed to remove dummy file");
    }
}