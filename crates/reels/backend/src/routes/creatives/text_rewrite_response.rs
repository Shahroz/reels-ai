//! Defines the structured response expected from the LLM for text rewrite operations.
//!
//! This struct encapsulates the rewritten text provided by the language model.
//! It is designed for use with the `llm_typed` function, ensuring that the
//! LLM output conforms to a predefined schema.
//! The rewritten text should be the direct output, without any extra explanations or markdown.

// Adhering to rust_guidelines.md for fully qualified paths where appropriate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TextRewriteResponse {
    #[schemars(description="The rewritten text as requested by the instruction.")]
    pub rewritten_text: std::string::String,
}

impl llm::few_shots_traits::FewShotsOutput<TextRewriteResponse> for TextRewriteResponse {
    fn few_shots() -> std::vec::Vec<TextRewriteResponse> {
        std::vec![
            TextRewriteResponse {
                rewritten_text: std::string::String::from(
                    "This is an example of perfectly rewritten concise text.",
                ),
            },
            TextRewriteResponse {
                rewritten_text: std::string::String::from(
                    "<html><head><title>Updated Page</title></head><body><h1>Welcome</h1><p>This is the rewritten HTML content, now with a friendly tone.</p></body></html>",
                ),
            },
            TextRewriteResponse {
                rewritten_text: std::string::String::from(
                    "The quick brown fox jumps over the lazy dog.",
                ),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    // Adhering to rust_guidelines.md for tests.
    // Using fully qualified paths or `super::*` as appropriate.

    use llm::few_shots_traits::FewShotsOutput;

    #[test]
    fn test_struct_definition_and_few_shots() {
        //! Ensures the struct can be instantiated and few_shots provides examples.
        let example_response = super::TextRewriteResponse {
            rewritten_text: std::string::String::from("Test text."),
        };
        assert_eq!(example_response.rewritten_text, "Test text.");

        let few_shots = super::TextRewriteResponse::few_shots();
        assert!(!few_shots.is_empty());
        assert!(few_shots.iter().all(|fs| !fs.rewritten_text.is_empty()));
    }
}