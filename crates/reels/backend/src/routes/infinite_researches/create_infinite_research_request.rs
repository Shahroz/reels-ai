//! Defines the request body structure for creating a new infinite research task.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// Request payload for creating a new infinite research task.
#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateInfiniteResearchRequest {
    /// A user-defined name for the research task.
    #[schema(example = "Daily Market Analysis")]
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// The detailed prompt for the research agent.
    #[schema(example = "Analyze the top 5 tech stocks and provide a summary.")]
    #[validate(length(min = 1))]
    pub prompt: String,

    /// The CRON schedule for when the task should run.
    #[schema(example = "0 9 * * *")] // Example: Every day at 9 AM
    #[validate(length(min = 1, max = 255))]
    pub cron_schedule: String,
}