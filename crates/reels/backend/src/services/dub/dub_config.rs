//! Dub configuration for lead attribution tracking
//!
//! This module handles configuration loading for Dub API integration,
//! following the project's pattern of using dotenvy for environment variables.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for Dub API integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DubConfig {
    /// Dub API key (secret)
    pub api_key: String,
    /// Dub workspace ID (can be public)
    pub workspace_id: String,
    /// Base URL for Dub API
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Whether Dub tracking is enabled
    pub enabled: bool,
}

impl DubConfig {
    /// Load Dub configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let env_fn = |key: &str| env::var(key);
        Self::from_env_map(&env_fn)
    }

    /// Load Dub configuration from provided environment variable function
    /// This allows for dependency injection and easier testing
    pub fn from_env_map(env_var_fn: &dyn Fn(&str) -> Result<String, env::VarError>) -> Result<Self> {
        let enabled = env_var_fn("DUB_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .context("DUB_ENABLED must be 'true' or 'false'")?;

        if !enabled {
            return Ok(Self::disabled());
        }

        let api_key = env_var_fn("DUB_API_KEY")
            .context("DUB_API_KEY environment variable is required when DUB_ENABLED=true")?;

        let workspace_id = env_var_fn("DUB_WORKSPACE_ID")
            .context("DUB_WORKSPACE_ID environment variable is required when DUB_ENABLED=true")?;

        let base_url = env_var_fn("DUB_BASE_URL")
            .unwrap_or_else(|_| "https://api.dub.co".to_string());

        let timeout_seconds = env_var_fn("DUB_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .context("DUB_TIMEOUT_SECONDS must be a valid number")?;

        Ok(DubConfig {
            api_key,
            workspace_id,
            base_url,
            timeout_seconds,
            enabled,
        })
    }

    /// Create a disabled configuration for testing or when Dub is not available
    pub fn disabled() -> Self {
        DubConfig {
            api_key: String::new(),
            workspace_id: String::new(),
            base_url: "https://api.dub.co".to_string(),
            timeout_seconds: 30,
            enabled: false,
        }
    }

    /// Create a test configuration with mock values
    #[cfg(test)]
    pub fn test() -> Self {
        DubConfig {
            api_key: "test_api_key".to_string(),
            workspace_id: "test_workspace".to_string(),
            base_url: "https://api.dub.co".to_string(),
            timeout_seconds: 30,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_disabled_config() {
        let config = DubConfig::disabled();
        assert!(!config.enabled);
        assert!(config.api_key.is_empty());
        assert!(config.workspace_id.is_empty());
    }

    #[test]
    fn test_config_from_env_disabled() {
        use std::collections::HashMap;
        
        let mut env_vars = HashMap::new();
        env_vars.insert("DUB_ENABLED", "false");
        
        let env_fn = |key: &str| -> Result<String, env::VarError> {
            env_vars.get(key)
                .map(|v| v.to_string())
                .ok_or(env::VarError::NotPresent)
        };
        
        let config = DubConfig::from_env_map(&env_fn).unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn test_config_from_env_enabled_with_required() {
        use std::collections::HashMap;
        
        let mut env_vars = HashMap::new();
        env_vars.insert("DUB_ENABLED", "true");
        env_vars.insert("DUB_API_KEY", "test-api-key");
        env_vars.insert("DUB_WORKSPACE_ID", "test-workspace-id");
        env_vars.insert("DUB_BASE_URL", "https://test.dub.co");
        env_vars.insert("DUB_TIMEOUT_SECONDS", "60");
        
        let env_fn = |key: &str| -> Result<String, env::VarError> {
            env_vars.get(key)
                .map(|v| v.to_string())
                .ok_or(env::VarError::NotPresent)
        };
        
        let config = DubConfig::from_env_map(&env_fn).unwrap();
        assert!(config.enabled);
        assert_eq!(config.api_key, "test-api-key");
        assert_eq!(config.workspace_id, "test-workspace-id");
        assert_eq!(config.base_url, "https://test.dub.co");
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_config_from_env_missing_required() {
        use std::collections::HashMap;
        
        let mut env_vars = HashMap::new();
        env_vars.insert("DUB_ENABLED", "true");
        // Intentionally not setting DUB_API_KEY and DUB_WORKSPACE_ID
        
        let env_fn = |key: &str| -> Result<String, env::VarError> {
            env_vars.get(key)
                .map(|v| v.to_string())
                .ok_or(env::VarError::NotPresent)
        };
        
        let result = DubConfig::from_env_map(&env_fn);
        assert!(result.is_err());
    }

    #[test]
    fn test_test_config() {
        let config = DubConfig::test();
        assert!(config.enabled);
        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.workspace_id, "test_workspace");
    }
}
