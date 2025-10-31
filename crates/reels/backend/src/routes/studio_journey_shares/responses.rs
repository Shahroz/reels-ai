//! Defines response bodies for the Studio Journey Shares API.

use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct CreateShareResponse {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type=String)]
    pub share_token: Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct GetShareResponse {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type=String)]
    pub share_token: Uuid,
    pub is_active: bool,
}