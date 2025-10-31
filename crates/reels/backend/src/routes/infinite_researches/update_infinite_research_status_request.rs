//! Request body for updating the status of an infinite research task.

/// Request payload for updating the enabled/disabled status of an infinite research task.
#[derive(serde::Deserialize, validator::Validate, utoipa::ToSchema, Debug)]
pub struct UpdateInfiniteResearchStatusRequest {
    /// The new status for the task.
    #[schema(example = true)]
    pub is_enabled: bool,
}