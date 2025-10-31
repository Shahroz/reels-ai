//! Response for deleting a user request.
//!
//! Contains a message indicating the deletion outcome.

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct DeleteRequestResponse {
    pub message: std::string::String,
}
