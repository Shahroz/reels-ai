use anyhow::{anyhow, Result};
use gennodes_common::template::render_template_with_json;
use crate::integrations::llm_typed_image_description::{llm_describe_image_typed, GPTVisionModel};
use crate::style_analysis::divmagic_v1::HTMLResponse;
use crate::integrations::sentinel::SentinelTypedResponse;
use crate::tools::website_style::visual_html_finetuning::html_improvement::score_html;
use crate::tools::website_style::visual_html_finetuning::screenshot_html;
use crate::tools::website_style::visual_html_finetuning::wrap_tailwind_component::wrap_tailwind_component;
use tracing::instrument;

/// Uses visual GPT to generate an improved template based on the provided template snippet
/// and the screenshot of its rendered version. Global style guidelines are optional.
/// Expects the LLM to return JSON with keys "reasoning" and "html" (where "html" holds the improved template).
#[instrument(skip(template, screenshot, global_style))]
async fn generate_template_variation(
    template: &str,
    screenshot: &str,
    global_style: Option<&str>,
) -> Result<String> {
    let additional_style_prompt = if let Some(style) = global_style {
        format!("Additionally, ensure that any modifications align with the following global style guidelines: {}.", style)
    } else {
        String::new()
    };

    let prompt = format!(
        r#"
Template:
{}

Style guidelines:
{}

Improve the design of the provided Tailwind template based solely on the screenshot of its rendered page.
Apply modern design principles and Tailwind CSS best practices.
Modify only the template code and return the improved version as a JSON structure with keys "reasoning" and "html".
"#,
        template, additional_style_prompt
    );

    let result: SentinelTypedResponse<HTMLResponse> = llm_describe_image_typed::<HTMLResponse>(
        &prompt,
        screenshot,
        GPTVisionModel::Gpto1,
        5,
    )
        .await?;
    Ok(result.content.html)
}

/// Improves a Tailwind template snippet by iteratively generating and scoring variations.
///
/// # Parameters:
/// - `template`: The original Tailwind template snippet to improve.
/// - `json_context`: A JSON context used to render the template.
/// - `max_tries`: Maximum number of improvement iterations.
/// - `min_score`: Minimum visual score to stop early (range 1–100).
/// - `global_style`: Optional style guidelines for consistency.
///
/// # Returns:
/// The improved template snippet.
#[instrument(skip(template, json_context, global_style))]
pub async fn improve_html_template_style(
    template: &str,
    json_context: &serde_json::Value,
    max_tries: usize,
    min_score: usize,
    global_style: Option<&str>,
) -> String {
    let mut current_template = template.to_string();

    // Render the initial HTML from the template using the provided JSON context
    let initial_rendered_html = match render_template_with_json(&current_template, json_context, true) {
        Ok(html) => html,
        Err(e) => {
            println!("Failed to render initial HTML: {:?}", e);
            return current_template;
        }
    };

    // Screenshot and score the rendered HTML
    let initial_rendered_html_wrapped = wrap_tailwind_component(&initial_rendered_html);
    let initial_screenshot = match screenshot_html::screenshot_html(&initial_rendered_html_wrapped, true).await {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to screenshot initial rendered HTML: {:?}", e);
            return current_template;
        }
    };

    let mut best_score = match score_html(&initial_screenshot).await {
        Ok(score) => score,
        Err(e) => {
            println!("Failed to score initial template: {:?}", e);
            0
        }
    };

    // If the initial design already meets the minimum score, return the original template
    if best_score >= min_score {
        return current_template;
    }

    for _ in 0..max_tries {
        // Re-render the current template with the JSON context
        let rendered_html = match render_template_with_json(&current_template, json_context, true) {
            Ok(html) => html,
            Err(e) => {
                println!("Failed to render HTML: {:?}", e);
                continue;
            }
        };

        // Screenshot and score the current rendered HTML
        let wrapped_rendered_html = wrap_tailwind_component(&rendered_html);
        let screenshot = match screenshot_html::screenshot_html(&wrapped_rendered_html, true).await {
            Ok(s) => s,
            Err(e) => {
                println!("Screenshot failed: {:?}", e);
                continue;
            }
        };

        let current_score = match score_html(&screenshot).await {
            Ok(score) => score,
            Err(e) => {
                println!("Scoring failed: {:?}", e);
                continue;
            }
        };

        // If the current design meets the score threshold, exit early
        if current_score >= min_score {
            best_score = current_score;
            break;
        }

        // Generate a new template variation based on the current template and its screenshot
        let new_template = match generate_template_variation(&current_template, &screenshot, global_style).await {
            Ok(t) => t,
            Err(e) => {
                println!("Failed to generate template variation: {:?}", e);
                continue;
            }
        };

        // Re-render the new template using the same JSON context
        let new_rendered_html = match render_template_with_json(&new_template, json_context, true) {
            Ok(html) => html,
            Err(e) => {
                println!("Failed to render new template: {:?}", e);
                continue;
            }
        };

        // Screenshot and score the newly rendered HTML
        let new_rendered_html_wrapped = wrap_tailwind_component(&new_rendered_html);
        let new_screenshot = match screenshot_html::screenshot_html(&new_rendered_html_wrapped, true).await {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to screenshot new rendered HTML: {:?}", e);
                continue;
            }
        };

        let new_score = match score_html(&new_screenshot).await {
            Ok(score) => score,
            Err(e) => {
                println!("Failed to score new template: {:?}", e);
                continue;
            }
        };

        // If the new score is an improvement, update our current best template
        if new_score > best_score {
            best_score = new_score;
            current_template = new_template;
        }

        if best_score >= min_score {
            break;
        }
    }

    current_template
}

#[cfg(FALSE)]
mod tests {
    use serde_json::json;
    use dotenvy::dotenv;
    use super::*;

    #[tokio::test]
    async fn test_improvement() {
        let _ = dotenv();
        // Sample rendered HTML (could be used for manual comparison) is no longer needed as input.
        // Instead, we define a JSON context matching the template's placeholders.
        let json_context = json!({
            "headline": "Innovate with Redis & Bounti",
            "subheadline": "Redis drives innovation with a cutting-edge sales strategy.",
            "description": "Experience a modern, clean UI with bold typography and dynamic transitions that empower your business to scale and innovate.",
            "cta_url": "https://redis.com/innovate",
            "cta_text": "Explore More"
        });

        let template = r#"
<section class="body-font bg-gradient-to-r from-[#102000] to-[#F0F0F0] py-24">
  <div class="container mx-auto flex flex-col md:flex-row items-center px-5">
    <div class="flex flex-col md:items-start items-center text-center md:text-left mb-16 md:mb-0 md:w-1/2 lg:pr-24">
      <h1 class="mb-4 text-[61px] font-bold text-[rgb(249,250,245)]">
        {{ headline }}
        <br class="hidden lg:inline-block">{{ subheadline }}
      </h1>

      <p class="mb-8 leading-relaxed text-[rgb(13,20,7)] text-[21px] font-normal">
        {{ description }}
      </p>

      <div class="flex justify-center">
        <a href="{{ cta_url }}" class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 text-lg">
          {{ cta_text }}
        </a>
      </div>
    </div>
  </div>
</section>
        "#;

        let improved_template = improve_html_template_style(template, &json_context, 3, 90, None).await;
        println!("Improved Template:\n{}", improved_template);
    }
}