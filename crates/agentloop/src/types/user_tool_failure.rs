//! Defines the structure for a user-facing tool failure response.
//!
//! This struct is used to present a concise summary of a tool's execution failure
//! to the user or for internal logging. It includes the tool's name and an error message.
//! Adheres to one-item-per-file and FQN standards from `rust_guidelines`.

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema, schemars::JsonSchema)]
pub struct UserToolFailure {
    /// A message describing the error or reason for failure.
    pub error: std::string::String,
}

#[cfg(test)]
mod tests {
    // No super:: needed as UserToolFailure is in this file's scope.
    // For serde_json etc., full paths are used if not in prelude.

    #[test]
    fn test_user_tool_failure_serde() {
        let failure = super::UserToolFailure {
            tool_name: std::string::String::from("error_prone_tool"),
            error: std::string::String::from("Something went terribly wrong."),
        };

        let serialized = serde_json::to_string(&failure).unwrap();
        let deserialized: super::UserToolFailure = serde_json::from_str(&serialized).unwrap();

        std::assert_eq!(deserialized.tool_name, "error_prone_tool");
        std::assert_eq!(deserialized.error, "Something went terribly wrong.");
    }
}