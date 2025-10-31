use anyhow::{Result, anyhow};
use tracing::instrument;

use crate::llm::vendors::gemini::{generate_gemini_response, GeminiModel, InlineData};

/// Iteratively adjusts the provided HTML style using a feedback loop with the Gemini model, incorporating image data for visual reference.
/// 
/// # Arguments
/// 
/// * `initial_html` - The starting HTML content whose style needs to be refined.
/// * `images` - A vector of InlineData representing image data to be sent to Gemini for visual reference.
/// * `max_iterations` - Maximum number of iterations to perform.
/// 
/// # Returns
/// 
/// A Result containing the final refined HTML string or an error.
/// 
/// The function validates that image data is provided and in each iteration constructs a prompt instructing the Gemini model
/// to improve the visual style of the HTML using the provided images as reference for layout, colors, typography, and overall aesthetics.
/// The loop exits early if no change is detected between iterations.
#[instrument(skip(initial_html, images))]
pub async fn iterative_style_adjustment(initial_html: &str, images: Vec<InlineData>, max_iterations: usize) -> Result<String> {
    if images.is_empty() {
        return Err(anyhow!("No image data provided. Please supply at least one image for style adjustment."));
    }

    let mut current_html = initial_html.to_string();
    let temperature = 0.7;

    for i in 0..max_iterations {
        let prompt = format!(
            "Iteration {}: Improve the visual style of the following HTML. Use the provided image data as visual reference to enhance layout, colors, typography, and overall modern aesthetics. Respond ONLY with the updated HTML code.\n\nCurrent HTML:\n{}",
            i + 1,
            current_html
        );

        let new_html = generate_gemini_response(
            &prompt,
            temperature,
            GeminiModel::Gemini25ProExp0325,
            false,
            Some(images.clone())
        ).await?;

        // Exit loop if no change is detected after trimming whitespace
        if new_html.trim() == current_html.trim() {
            break;
        }

        current_html = new_html;
    }

    Ok(current_html)
}
