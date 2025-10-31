//! Response for retrieving a single user request.
//!
//! Contains the `RequestRecord` for the requested ID.

use crate::db::requests::RequestRecord;

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GetRequestResponse {
    #[schema(value_type = RequestRecord)]
    pub request: crate::db::requests::RequestRecord,
}