//! Placeholder for Sender enum.

use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ToSchema, Default)]
pub enum Sender {
    #[default]
    User,
    Agent,
    Tool,
    System,
    Assistant,
}
