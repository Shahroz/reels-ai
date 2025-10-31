use llm::vendors::gemini::gemini_output::GeminiOutput;
use anyhow::Result;
use tracing::instrument;
use llm::vendors::gemini::completion::{generate_gemini_response};
use llm::vendors::gemini::gemini_model::GeminiModel;

/// Generates HTML INSTRUCTION in the style of a given URL using assets and provided INSTRUCTION.
///
/// # Arguments
///
/// * `url` - The URL of the website whose style should be replicated.
/// * `assets` - A string containing information about assets (e.g., image URLs, descriptions).
/// * `INSTRUCTION` - The INSTRUCTION (text, structure description) to be styled.
///
/// # Returns
///
/// A `Result` containing the generated HTML string or an error.
#[instrument(skip(source_html, assets, instruction))]
pub async fn replicate_style(source_html: &str, assets: &str, instruction: &str) -> Result<String> {
    log::info!("Starting INSTRUCTION repurposeing");

    // 2. Construct the prompt for Gemini
    //    Based on the test `page_in_the_style`
    let prompt = format!(
        r#"
<SOURCE_STYLE_HTML>
{source_html}
</SOURCE_STYLE_HTML>

<ASSETS>
{assets}
</ASSETS>

<INSTRUCTION>
{instruction}
</INSTRUCTION>

TASK:
Create HTML code for the provided <INSTRUCTION> that closely matches the style found in the <SOURCE_STYLE_HTML>.
Use the <ASSETS> where appropriate (e.g., for images).
Important:
- remove all comments from the original HTML
- remove all tracking scripts
- preserve only style related scripts but not linking to the original site (unless the user wants to reuse the existing content)

Respond **only** with the generated HTML code, ensuring it is well-formed and ready to be rendered.
Do not include any explanations, markdown formatting, or ```html fences. Just the raw HTML.
"#
    );
    log::debug!("Generated prompt for Gemini (length: {})", prompt.len());

    // 3. Call Gemini to generate the styled HTML
    //    Using settings similar to the test `page_in_the_style`
    log::info!("Calling Gemini to generate styled HTML...");
    let temperature = 0.7;
    // todo! use unified llm function with this model asn Gemini20Pro as fallback
    let model = GeminiModel::Gemini25ProPreview0325; // As used in the test
    let gemini_output = generate_gemini_response(&prompt, temperature, model, None)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let styled_html = match gemini_output {
        GeminiOutput::Text(text) => text,
        _ => {
            return Err(anyhow::anyhow!(
                "Expected text output from Gemini, but received a different output type."
            ))
        }
    };
    log::info!(
        "Successfully received styled HTML from Gemini (length: {})",
        styled_html.len()
    );
    Ok(styled_html)
}

// Optional: Add tests similar to `page_in_the_style`
#[cfg(test)] // Changed from FALSE
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    #[ignore] // Ignored by default as it requires API keys and network access
    async fn test_replicate_style_basic() -> Result<()> {
        dotenv().ok();
        let _ = env_logger::builder().is_test(true).try_init();

        let url = "https://vercel.com"; // Example URL
        let assets = r#"
- Logo: https://example.com/logo.png (Description: Company logo)
- Hero Image: https://example.com/hero.jpg (Description: Abstract background)
"#;
        let instruction = r#"
My Awesome Product

Headline: The Future of Web Development
Sub-headline: Build faster, scale automatically.
Button: Get Started Now

Features Section:
- Feature 1: Instant Previews
- Feature 2: Global CDN
- Feature 3: Serverless Functions
"#;

        let result_html = replicate_style(url, assets, instruction).await?;

        assert!(!result_html.is_empty());
        println!("Generated HTML:\n{}", result_html);

        // Optionally write to a file for inspection
        let mut file = File::create("replicated_style_output.html")?;
        write!(file, "{}", result_html)?;
        println!("Output saved to replicated_style_output.html");

        Ok(())
    }
}
