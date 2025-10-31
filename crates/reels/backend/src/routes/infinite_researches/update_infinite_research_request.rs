//! Defines the request body structure for updating an infinite research task.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// Request payload for updating an infinite research task.
#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateInfiniteResearchRequest {
    /// A user-defined name for the research task.
    #[schema(example = "Daily Tech Stock Analysis")]
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// The detailed prompt for the research agent.
    #[schema(example = "Analyze the top 5 tech stocks (AAPL, MSFT, GOOG, AMZN, NVDA) and provide a summary.")]
    #[validate(length(min = 1))]
    pub prompt: String,

    /// The CRON schedule for when the task should run.
    #[schema(example = "0 9 * * 1-5")] // Example: Every weekday at 9 AM
    #[validate(length(min = 1, max = 255))]
    pub cron_schedule: String,

    /// Whether the task is currently enabled and should run on its schedule.
    #[schema(example = true)]
    pub is_enabled: bool,
}