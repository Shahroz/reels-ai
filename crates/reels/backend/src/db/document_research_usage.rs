//! Defines the `DocumentResearchUsage` enum for controlling research utilization.
//!
//! This enum specifies how research documents associated with a task document
//! should be handled within a research workflow.

/// Enum specifying how research documents are used in tasks.
use sqlx::Type;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, utoipa::ToSchema, schemars::JsonSchema, Type)]
#[serde(rename_all = "PascalCase")] // For JSON serialization/deserialization
#[sqlx(type_name = "text", rename_all = "PascalCase")]
pub enum DocumentResearchUsage {
    /// Never use associated research documents.
    Never,
    /// Usage of research documents depends on the task's specific context or instructions.
    TaskDependent,
    /// Always use associated research documents.
    Always,
}

impl std::fmt::Display for DocumentResearchUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentResearchUsage::Never => write!(f, "Never"),
            DocumentResearchUsage::TaskDependent => write!(f, "TaskDependent"),
            DocumentResearchUsage::Always => write!(f, "Always"),
        }
    }
}

impl std::str::FromStr for DocumentResearchUsage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Never" => Ok(DocumentResearchUsage::Never),
            "TaskDependent" => Ok(DocumentResearchUsage::TaskDependent),
            "Always" => Ok(DocumentResearchUsage::Always),
            _ => Err(format!("Invalid DocumentResearchUsage value: {s}")),
        }
    }
}

impl From<String> for DocumentResearchUsage {
    fn from(s: String) -> Self {
        s.parse().unwrap_or(DocumentResearchUsage::TaskDependent)
    }
}

#[cfg(test)]
mod tests {
    // Using super:: to access the item in the parent module (the file scope).
    // Fully qualified paths for other items (e.g. std::string::String if needed).

    #[test]
    fn test_display_trait() {
        assert_eq!(super::DocumentResearchUsage::Never.to_string(), "Never");
        assert_eq!(
            super::DocumentResearchUsage::TaskDependent.to_string(),
            "TaskDependent"
        );
        assert_eq!(super::DocumentResearchUsage::Always.to_string(), "Always");
    }

    #[test]
    fn test_from_str_trait_ok() {
        assert_eq!("Never".parse(), Ok(super::DocumentResearchUsage::Never));
        assert_eq!(
            "TaskDependent".parse(),
            Ok(super::DocumentResearchUsage::TaskDependent)
        );
        assert_eq!("Always".parse(), Ok(super::DocumentResearchUsage::Always));
    }

    #[test]
    fn test_from_str_trait_err() {
        let result: Result<super::DocumentResearchUsage, String> = "InvalidValue".parse();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid DocumentResearchUsage value: InvalidValue"
        );

        let empty_result: Result<super::DocumentResearchUsage, String> = "".parse();
        assert!(empty_result.is_err());
        assert_eq!(
            empty_result.unwrap_err(),
            "Invalid DocumentResearchUsage value: "
        );
    }
}
