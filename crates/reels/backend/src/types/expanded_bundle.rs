//! Defines the `ExpandedBundle` struct, a version of `Bundle` with resolved foreign key IDs.
//!
//! This struct represents a `Bundle` where fields like `style_id`, `document_ids`,
//! and `format_ids` (now `formats`) are replaced with their corresponding full data structures.
//! Note: Assets and creatives have been removed from the application.
//! Adheres to Rust coding standards, including one item per file and file preamble.

/// Represents a bundle with its associated resources fully expanded.
#[derive(serde::Serialize, utoipa::ToSchema, std::fmt::Debug, std::clone::Clone)]
pub struct ExpandedBundle {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub user_id: uuid::Uuid,
    #[schema(example = "My Awesome Expanded Bundle")]
    pub name: std::string::String,
    #[schema(example = "A description of what this bundle contains, with expanded details.", nullable = true)]
    pub description: Option<std::string::String>,
    
    /// The fully resolved style object associated with this bundle.
    // pub style: crate::db::styles::Style, // db module deleted 
    
    /// A list of fully resolved document objects associated with this bundle.
    // pub documents: std::vec::Vec<crate::db::documents::Document>, // db module deleted
    
    /// Assets and creatives removed from application

    #[schema(value_type = String, format = "date-time", example = "2024-05-29T10:00:00Z")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-05-29T12:00:00Z")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
