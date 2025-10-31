//! Defines the requested reasoning effort level for certain OpenAI models.
//! 
//! Allows specifying low, medium, or high effort, potentially influencing
//! response quality, latency, or cost on models supporting this feature.
//! Used in OpenAIChatCompletionRequest.
//! Provides a hint to the model about computational budget.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ReasoningEffort {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}
