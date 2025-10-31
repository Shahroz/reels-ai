//! Request payload for creating an imageboard board for a collection.
//!
//! Used by POST /api/collections/{collection_id}/imageboard.

use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema, Debug)]
pub struct CreateImageboardBoardRequest {
    pub organization_id: Uuid,
}
