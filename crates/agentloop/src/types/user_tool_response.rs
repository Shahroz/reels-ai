//! Defines the structure for a user-facing tool response, typically a summary.
//!
//! This struct is used to present a concise summary of a tool's execution
//! to the user. It includes the tool's name and a human-readable summary string.
//! It is designed for display purposes where a brief overview is sufficient.
//! Adheres to one-item-per-file and FQN standards.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, schemars::JsonSchema)]
pub struct UserToolResponse {
    pub tool_name: std::string::String,
    pub summary: std::string::String,
    pub icon: std::option::Option<std::string::String>,
    pub data: std::option::Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_user_tool_response_serde() {
        let response = super::UserToolResponse {
            tool_name: std::string::String::from("summary_tool"),
            summary: std::string::String::from("The tool completed successfully."),
            icon: std::option::Option::None,
            data: std::option::Option::None,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: super::UserToolResponse = serde_json::from_str(&serialized).unwrap();

        std::assert_eq!(deserialized.tool_name, "summary_tool");
        std::assert_eq!(deserialized.summary, "The tool completed successfully.");
        std::assert_eq!(deserialized.icon, std::option::Option::None);
        std::assert_eq!(deserialized.data, std::option::Option::None);
    }
}
