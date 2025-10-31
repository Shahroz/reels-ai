//! Defines the query parameters for the list_vocal_tours endpoint.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema, utoipa::IntoParams)]
pub struct ListVocalToursParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
    #[schema(example = "property tour")]
    pub search: Option<String>,
}