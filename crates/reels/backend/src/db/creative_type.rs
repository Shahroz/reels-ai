//! Defines the types of creatives supported.
//!
//! This enum represents the different categories or formats a creative can belong to.
//! It corresponds to the `creative_type` text column in format-related tables.
//! Adheres to Rust guidelines: one item per file, preamble docs.

use serde::{Deserialize, Serialize};
use sqlx::Type; // Needed for deriving sqlx::Type
use utoipa::ToSchema;

/// Represents the type of a creative format.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "text")] // Map to PostgreSQL text type
#[serde(rename_all = "snake_case")] // Example serialization format
pub enum CreativeType {
    // Add specific creative types as needed based on application logic.
    // Examples:
    Website,
    Banner,
    Email,
    SocialPost,
    Video,
    Unknown,
}

// Optional: Implement standard traits like Display, FromStr if needed for parsing/displaying.
impl std::fmt::Display for CreativeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreativeType::Banner => write!(f, "banner"),
            CreativeType::Email => write!(f, "email"),
            CreativeType::SocialPost => write!(f, "social_post"),
            CreativeType::Video => write!(f, "video"),
            CreativeType::Unknown => write!(f, "unknown"),
            CreativeType::Website => write!(f, "website"),
        }
    }
}

impl std::str::FromStr for CreativeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "banner" => Ok(CreativeType::Banner),
            "email" => Ok(CreativeType::Email),
            "social_post" => Ok(CreativeType::SocialPost),
            "social post" => Ok(CreativeType::SocialPost), // Handle variations
            "video" => Ok(CreativeType::Video),
            "website" => Ok(CreativeType::Website),
            _ => Ok(CreativeType::Unknown), // Default or return Err("Invalid creative type")
        }
    }
}

// No tests needed for a simple enum definition unless complex logic (like FromStr) is added.