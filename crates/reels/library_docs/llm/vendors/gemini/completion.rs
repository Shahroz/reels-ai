use crate::llm::unified::ImageData;
use crate::llm::vendors::gemini::GeminiModel;
use anyhow::Result;
use tracing::instrument;

#[instrument(skip(prompt, images))]
pub async fn generate_gemini_response(
    prompt: &str,
    temperature: f64,
    gemini_model: GeminiModel,
    support_images: bool,
    images: Option<Vec<ImageData>>
) -> Result<String> {
    if let Some(imgs) = images {
        // Process images if provided, supporting image requests irrespective of the support_images flag.
        return Ok(format!("Gemini response for prompt: {} with {} images processed", prompt, imgs.len()));
    }
    Ok(format!("Gemini response for prompt: {}", prompt))
}
