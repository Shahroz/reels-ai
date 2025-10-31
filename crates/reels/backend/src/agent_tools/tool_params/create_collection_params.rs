//! Defines parameters for the `create_collection` agent tool.
//!
//! This structure encapsulates the arguments for creating a new collection,
//! including the collection name and optional metadata.
//! The `user_id` is injected by the dispatcher and not expected from the agent.
//! Adheres to the project's Rust coding standards.


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct CreateCollectionParams {
    /// The name of the collection to create.
    #[schemars(example = "My New Collection")]
    pub name: String,
    /// Optional JSON metadata for the collection.
    #[schemars(skip)]
    pub metadata: Option<serde_json::Value>,
    /// Optional organization ID to associate the collection with.
    #[schemars(skip)]
    pub organization_id: Option<uuid::Uuid>,
    /// The user ID, injected by the application. Not expected from the agent.
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
}