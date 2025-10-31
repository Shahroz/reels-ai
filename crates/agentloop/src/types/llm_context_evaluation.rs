//! Holds the structured response from the LLM for context evaluation tasks.
//!
//! This struct defines the expected format for the LLM's assessment of whether
//! the current context is sufficient to proceed with a task. It mirrors the
//! fields needed for `ContextEvaluatorFeedback` but is specifically designed
//! for direct deserialization from the LLM output. It implements FewShotsOutput.

#[derive(std::fmt::Debug, std::clone::Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, utoipa::ToSchema)]
pub struct LlmContextEvaluation {
    /// Flag indicating if the context is sufficient for the task.
    pub is_sufficient: bool,
    /// Suggested next steps for the agent based on the context evaluation.
    pub next_steps: std::vec::Vec<std::string::String>,
    /// Details about any missing information needed to proceed. None if sufficient.
    pub missing_information: std::option::Option<std::string::String>,
}

impl llm::few_shots_traits::FewShotsOutput<LlmContextEvaluation> for LlmContextEvaluation {
    fn few_shots() -> std::vec::Vec<LlmContextEvaluation> {
        std::vec![
            // Example 1: Context is insufficient
            LlmContextEvaluation {
                is_sufficient: false,
                next_steps: std::vec![
                    std::string::String::from("Ask the user for the specific file path."),
                    std::string::String::from("Request clarification on the desired code modification.")
                    ],
                missing_information: std::option::Option::Some(std::string::String::from(
                    "The user mentioned modifying a file but did not specify which file or the exact changes needed.",
                )),
            },
            // Example 2: Context is sufficient
            LlmContextEvaluation {
                is_sufficient: true,
                next_steps: std::vec![
                    std::string::String::from("Generate the Python script as requested."),
                    std::string::String::from("Create the 'output' directory if it doesn't exist."),
                    std::string::String::from("Save the script to 'output/data_processor.py'.")
                    ],
                missing_information: std::option::Option::None,
            },
        ]
    }
}
