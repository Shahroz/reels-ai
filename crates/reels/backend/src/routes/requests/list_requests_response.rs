//! Response for listing user requests.
//!
//! Contains a vector of `RequestRecord` retrieved for the authenticated user.

use crate::db::requests::RequestRecord;

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ListRequestsResponse {
    #[schema(value_type = RequestRecord)]
    pub requests: std::vec::Vec<crate::db::requests::RequestRecord>,
}