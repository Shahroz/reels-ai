//! Defines the available models for the Anthropic Claude API.
//!
//! This enum represents the different Claude models that can be specified in API requests.
//! Includes methods for converting to string representation and a default model.
//! Supports serialization and deserialization.
//! Models include Opus, Sonnet, Haiku, and the latest versions.

/// Represents the available Claude models.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub enum ClaudeModel {
    #[default]
    #[serde(rename = "claude-3-5-sonnet-latest")]
    Claude35SonnetLatest,
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus,
    #[serde(rename = "claude-3-sonnet-20240229")]
    Claude3Sonnet,
    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku,
    #[serde(rename = "claude-3-7-sonnet-latest")]
    Claude37SonnetLatest,
}

impl ClaudeModel {
    /// Returns the string representation of the model for the API.
    pub fn as_str(&self) -> &'static str {
        match self {
            ClaudeModel::Claude37SonnetLatest => "claude-3-7-sonnet-latest",
            ClaudeModel::Claude35SonnetLatest => "claude-3-5-sonnet-latest",
            ClaudeModel::Claude3Opus => "claude-3-opus-20240229",
            ClaudeModel::Claude3Sonnet => "claude-3-sonnet-20240229",
            ClaudeModel::Claude3Haiku => "claude-3-haiku-20240307",
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Using super::* is generally disallowed, but tests are an exception for the item under test.
    // Full paths are needed for other items like serde_json.

    #[test]
    fn test_model_serialization_deserialization() {
        // Test serialization
        let model = super::ClaudeModel::Claude35SonnetLatest;
        let serialized = serde_json::to_string(&model).unwrap();
        assert_eq!(serialized, "\"claude-3-5-sonnet-latest\"");

        // Test deserialization
        let deserialized: super::ClaudeModel = serde_json::from_str("\"claude-3-opus-20240229\"").unwrap();
        assert_eq!(deserialized, super::ClaudeModel::Claude3Opus);

        // Test default
        let default_model = super::ClaudeModel::default();
        assert_eq!(default_model, super::ClaudeModel::Claude35SonnetLatest);

        // Test as_str
        assert_eq!(super::ClaudeModel::Claude3Haiku.as_str(), "claude-3-haiku-20240307");
    }
}
