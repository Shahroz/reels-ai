//! Loads application configuration from environment variables.
//!
//! Reads settings like server address, providing defaults
//! for some values if they are not explicitly set in the environment.
//! Note: AgentLoop uses in-memory storage - no database required.
//! Conforms to the project's Rust coding standards.

/// Loads configuration values from environment variables.
///
/// Reads the following environment variables:
/// - `PORT`: The port for the HTTP server. Defaults to 8080 if not set or invalid.
    /// - `EVALUATOR_SLEEP_SECONDS`: Interval for the background evaluator. Defaults to 1 second.
/// - `SESSION_TIMEOUT_SECONDS`: Timeout for user sessions. Defaults to 3600.
/// - `MAX_CONVERSATION_LENGTH`: Max messages in conversation history. Defaults to 100.
/// - `COMPACTION_POLICY`: Strategy for conversation compaction (e.g., "truncate"). Defaults to "truncate".
///
/// Returns an `AppConfig` struct populated with these values or defaults.
/// Uses `anyhow::Error` for flexible error handling during parsing.
pub fn load_env_config() -> std::result::Result<crate::config::app_config::AppConfig, anyhow::Error> {
    // Load PORT, default to 8080 if not set or invalid.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| {
            log::info!("PORT environment variable not set or invalid, defaulting to 8080.");
            8080
        });

    // Construct the server address using hardcoded "0.0.0.0" and the loaded/default port.
    let server_address = std::format!("0.0.0.0:{}", port);
    log::info!("Server will listen on: {}", server_address);


    // Load EVALUATOR_SLEEP_SECONDS, default to 1 second if not set or parse fails.
    let evaluator_sleep_seconds = std::env::var("EVALUATOR_SLEEP_SECONDS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| {
            log::info!("EVALUATOR_SLEEP_SECONDS not set or invalid, defaulting to 1 second.");
            1
        });

    // Load SESSION_TIMEOUT_SECONDS, default to 3600 if not set or parse fails.
    let session_timeout_seconds = std::env::var("SESSION_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| {
            log::info!("SESSION_TIMEOUT_SECONDS not set or invalid, defaulting to 3600.");
            3600
        });

    // Load MAX_CONVERSATION_LENGTH, default to 100 if not set or parse fails.
     let max_conversation_length = std::env::var("MAX_CONVERSATION_LENGTH")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| {
            log::info!("MAX_CONVERSATION_LENGTH not set or invalid, defaulting to 100.");
            100
        });

    // Load COMPACTION_POLICY, default to Truncate if not set or invalid.
    // Assuming CompactionPolicy::from_str exists or similar parsing logic.
    let compaction_policy_str = std::env::var("COMPACTION_POLICY").unwrap_or_else(|_| {
         log::info!("COMPACTION_POLICY not set, defaulting to 'truncate'.");
         "truncate".to_string()
    });
    // Example: Using a hypothetical from_str, adjust based on actual implementation.
    let compaction_policy = match compaction_policy_str.to_lowercase().as_str() {
         "truncate" => crate::types::compaction_policy::CompactionPolicy::default(),
         // Add other policies if they exist
         _ => {
             log::warn!("Invalid COMPACTION_POLICY '{}', defaulting to Truncate.", compaction_policy_str);
             crate::types::compaction_policy::CompactionPolicy::default()
         }
     };


    // Construct the AppConfig with loaded values.
    // LlmConfig would need its own loading logic, using default here.
    let config = crate::config::app_config::AppConfig {
        server_address,
        evaluator_sleep_seconds,
        session_timeout_seconds,
        llm_config: crate::config::llm_config::LlmConfig::default(),
        compaction_policy,
        max_conversation_length,
    };

    // Return the populated config struct.
    std::result::Result::Ok(config)
}

#[cfg(test)]
mod tests {
    // Note: These tests rely on manipulating environment variables, which can
    // interfere with other tests if run in parallel without care.
    // Consider using crates like `serial_test` or mutexes if needed.

    fn set_test_env() {
        std::env::set_var("PORT", "9000");
        std::env::set_var("EVALUATOR_SLEEP_SECONDS", "120");
        std::env::set_var("SESSION_TIMEOUT_SECONDS", "1800");
        std::env::set_var("MAX_CONVERSATION_LENGTH", "50");
        std::env::set_var("COMPACTION_POLICY", "truncate");
    }

    fn clear_test_env() {
        std::env::remove_var("PORT");
        std::env::remove_var("EVALUATOR_SLEEP_SECONDS");
        std::env::remove_var("SESSION_TIMEOUT_SECONDS");
        std::env::remove_var("MAX_CONVERSATION_LENGTH");
        std::env::remove_var("COMPACTION_POLICY");
    }

    #[test]
    fn test_load_config_from_env() {
        set_test_env();
        let config_result = super::load_env_config();
        assert!(config_result.is_ok());
        let config = config_result.unwrap();

        assert_eq!(config.server_address, "0.0.0.0:9000");
        assert_eq!(config.evaluator_sleep_seconds, 120);
        assert_eq!(config.session_timeout_seconds, 1800);
        assert_eq!(config.max_conversation_length, 50);
        assert_eq!(config.compaction_policy, crate::types::compaction_policy::CompactionPolicy::default());
        // LlmConfig default check would go here if needed.

        clear_test_env();
    }

    #[test]
    fn test_load_config_defaults() {
        clear_test_env(); // Ensure no env vars are set

        let config_result = super::load_env_config();
        assert!(config_result.is_ok());
        let config = config_result.unwrap();

        assert_eq!(config.server_address, "0.0.0.0:8080");
        assert_eq!(config.evaluator_sleep_seconds, 60);
        assert_eq!(config.session_timeout_seconds, 3600);
        assert_eq!(config.max_conversation_length, 100);
        assert_eq!(config.compaction_policy, crate::types::compaction_policy::CompactionPolicy::default());
         // LlmConfig default check would go here if needed.
    }

     #[test]
     fn test_load_config_invalid_numeric() {
         // Set invalid numeric values for sleep/timeout/length
         std::env::set_var("EVALUATOR_SLEEP_SECONDS", "not-a-number");
         std::env::set_var("SESSION_TIMEOUT_SECONDS", "invalid");
         std::env::set_var("MAX_CONVERSATION_LENGTH", "bad");
         // Set invalid PORT
         std::env::set_var("PORT", "not-a-port");

         let config_result = super::load_env_config();
         assert!(config_result.is_ok());
         let config = config_result.unwrap();

         // Check that defaults are used for invalid numeric env vars
         assert_eq!(config.evaluator_sleep_seconds, 60);
         assert_eq!(config.session_timeout_seconds, 3600);
         assert_eq!(config.max_conversation_length, 100);
         // Check that default PORT is used
         assert_eq!(config.server_address, "0.0.0.0:8080");


         clear_test_env();
         // Clear the invalid PORT setting specifically
         std::env::remove_var("PORT");
     }
}