//! Database model for favorited prompts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;

/// A user's favorited enhancement prompt
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct FavoritedPrompt {
    /// Unique identifier for this favorited prompt
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    /// User who favorited this prompt
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    /// The enhancement prompt text
    #[schema(example = "make it brighter and more vibrant")]
    pub prompt_text: String,
    
    /// Optional user-defined title for organizing prompts
    #[schema(example = "Bright & Vibrant", nullable = true)]
    pub title: Option<String>,
    
    /// When this prompt was favorited
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: DateTime<Utc>,
}

