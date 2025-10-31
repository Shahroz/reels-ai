//! Request payload for creating a collection.
//!
//! Used by POST /api/collections.


use serde_json::Value;
use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub metadata: Option<Value>,
    #[schema(format = "uuid", value_type = Option<String>)]
    pub organization_id: Option<Uuid>,
}