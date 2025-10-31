use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use serde_json::{json, Value};
use std::env;
use gennodes_common::traits::llm_traits::FewShotsOutput;
// Assuming these types and functions are defined elsewhere in your project:
use crate::integrations::llm_typed_static::llm_typed_static_simple;
use crate::integrations::gpt::OpenAIModel;
use crate::integrations::llm_typed_image_description::{llm_describe_image_typed, GPTVisionModel};
use crate::style_analysis::divmagic_v1::HTMLResponse;
use crate::integrations::sentinel::SentinelTypedResponse;
use crate::tools::website_style::visual_html_finetuning::screenshot_html;
use tracing::instrument;

/// A helper struct for the visual score response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct VisualScore {
    #[schemars(description="Explain step by step your reasoning process")]
    reasoning: String,
    #[schemars(description="Score between 1 (no design) and 100 (expert design)")]
    visual_score: usize,
}

impl FewShotsOutput<VisualScore> for VisualScore {}

/// Generates a new HTML variation that improves the design.
/// Uses LLM with both HTML in the prompt and a screenshot as an image argument.
/// (html, screenshot, global_style) -> new_html
#[instrument(skip(html, screenshot, global_style))]
async fn generate_html_variation(html: &str, screenshot: &str, global_style: Option<&str>) -> Result<String> {
    let additional_style_prompt = if let Some(style) = global_style {
        format!("Additionally, ensure that any modifications align with the following global style guidelines: {}.", style)
    } else {
        String::new()
    };

    let prompt = format!(
        r#"
HTML:
{}

Style guidelines:
{}

Improve the design of the following HTML based on its screenshot.
Enhance the visual aesthetics, layout, and user experience using modern design principles and Tailwind CSS.
When it comes to images, only use the ones provided; if they are missing or unsuitable, please remove them.
Return only the updated HTML as a JSON structure with "reasoning" and "html" as keys.
Important:
- preserve all content from HTML (all sections)
"#,
        html, additional_style_prompt
    );

    let result: SentinelTypedResponse<HTMLResponse> =
        llm_describe_image_typed::<HTMLResponse>(
            &prompt,
            screenshot,
            GPTVisionModel::Gpto1,
            5
        )
            .await?;
    Ok(result.content.html)
}

/// Scores the visual design based solely on the screenshot.
/// (screenshot) -> score
/// Returns a numeric score from 1 (very poor) to 100 (expert).
#[instrument(skip(screenshot))]
pub async fn score_html(screenshot: &str) -> Result<usize> {
    let prompt = r#"
Evaluate the visual design of the website based solely on the provided screenshot.
Rate the design on a scale from 1 (very poor, no style) to 100 (expert design).
Return a JSON object with a key "visual_score" containing the numeric score.
"#;

    let result: SentinelTypedResponse<VisualScore> =
        llm_describe_image_typed::<VisualScore>(
            prompt,
            screenshot,
            GPTVisionModel::Gpt4o,
            5
        )
            .await?;
    Ok(result.content.visual_score)
}

/// Improves HTML style by iteratively generating and scoring variations.
/// In each iteration, a new HTML variant is generated and then scored.
/// If the new score meets or exceeds `min_score`, the process exits early.
/// Returns the original HTML if improvements fail completely.
///
/// Added parameter:
/// - `global_style`: Optional guidelines to ensure that the new HTML variations remain consistent with the website's overall style.
#[instrument(skip(html, global_style))]
pub async fn improve_html_style(html: &str, max_tries: usize, min_score: usize, global_style: Option<&str>) -> String {
    let mut current_html = html.to_string();

    // Evaluate the initial HTML.
    let initial_result: anyhow::Result<usize> = async {
        let screenshot = screenshot_html::screenshot_html(&current_html, true).await?;
        let score = score_html(&screenshot).await?;
        Ok(score)
    }.await;

    let mut best_score = match initial_result {
        Ok(score) => {
            println!("Initial score: {}", score);
            if score >= min_score {
                return current_html;
            }
            score
        }
        Err(e) => {
            println!("Failed to evaluate initial HTML: {:?}", e);
            return current_html; // Return original on initial failure
        }
    };

    // Iterative improvement.
    for _ in 0..max_tries {
        let iteration_result: anyhow::Result<(String, usize)> = async {
            let screenshot = screenshot_html::screenshot_html(&current_html, true).await?;
            let new_html = generate_html_variation(&current_html, &screenshot, global_style).await?;
            let new_screenshot = screenshot_html::screenshot_html(&new_html, true).await?;
            let new_score = score_html(&new_screenshot).await?;
            Ok((new_html, new_score))
        }.await;

        match iteration_result {
            Ok((new_html, new_score)) => {
                println!("New score: {}", new_score);
                // Update current HTML if the new score is higher.
                if new_score >= best_score {
                    best_score = new_score;
                    current_html = new_html;
                    println!("Improved HTML with score: {}", best_score);
                    println!("Current HTML: {}", current_html);
                }

                // Early exit if score meets or exceeds minimum.
                if best_score >= min_score {
                    break;
                }
            }
            Err(e) => {
                println!("Iteration failed: {:?}", e);
                // Continue to next iteration instead of failing completely.
                continue;
            }
        }
    }

    current_html
}

#[cfg(FALSE)]
mod tests {
    use dotenvy::dotenv;
    use super::*;

    #[tokio::test]
    async fn test_improve_html() {
        let _ = dotenv();
        let html = r#""#;
        let improved_html = improve_html_style(html, 5, 90, None).await;
        println!("{:?}", improved_html);
        // You can use further assertions to validate the improved HTML.
    }
}
