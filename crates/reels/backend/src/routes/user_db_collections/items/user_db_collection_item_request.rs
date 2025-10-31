//! Defines the request body for creating or updating a user DB collection item.
//!
//! This struct contains the `item_data` which must conform to the schema
//! defined in the parent `UserDbCollection`.
//! Adheres to 'one item per file' and FQN guidelines.

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct UserDbCollectionItemRequest {
    #[schema(value_type = Object, example = json!({"title": "My Photo", "url": "https://example.com/photo.jpg"}))]
    pub item_data: serde_json::Value,
}
