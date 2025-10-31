use anyhow::{anyhow, Context};
use std::env;
use schemars::JsonSchema;
use tokio::sync::mpsc::channel;
use url::Url;
use gennodes_common::traits::llm_traits::FewShotsOutput;
use gennodes_image_transform::analyze_image_from_base64;
use crate::integrations::zyte::ZyteClient;
use crate::style_analysis::divmagic_v1::ProcessMessage;
use crate::integrations::llm_typed_static::llm_typed_static_simple;
use crate::integrations::gpt::OpenAIModel;
use crate::integrations::sentinel::SentinelTypedResponse;
use crate::style_analysis::divmagic_v2::{extract_style_from_screenshot, visual_feedback};

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct CssStyleSummary {
    pub summary: String,
}

impl FewShotsOutput<CssStyleSummary> for CssStyleSummary {
    fn few_shots() -> Vec<CssStyleSummary> {
        vec![
            CssStyleSummary {
                summary: r#"Minimalistic Corporate Tech Style:

Layout structure: Utilizes a 12-column grid system with flexbox for responsive sections and absolute positioning for overlays.

Typography details: Primary font is 'Open Sans' (sizes: 16px for body, 24px for headings, weights: 400 and 600), line height of 1.6. Secondary font 'Montserrat' for buttons and highlights.

Color scheme: Primary color #2C3E50, secondary #18BC9C, background #ECF0F1, text #34495E.

Spacing: Consistent 20px margins between sections, 15px paddings for containers, 10px gaps between inline elements.

Unique styling features: Smooth 0.3s transitions on hover for links and buttons, subtle box shadows on cards, and a fixed navigation bar.

Google Fonts used: <link href='https://fonts.googleapis.com/css?family=Open+Sans|Montserrat' rel='stylesheet'>

CSS frameworks or libraries: Custom CSS with some utility classes resembling Tailwind CSS."#.to_owned(),
            },
        ]
    }
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct VisualStyleSummary {
    pub summary: String,
}

impl FewShotsOutput<VisualStyleSummary> for VisualStyleSummary {
    fn few_shots() -> Vec<VisualStyleSummary> {
        vec![
            VisualStyleSummary {
                summary: r#"Minimalistic Corporate Tech Visual Style:

Header design: Fixed top navigation with a centered logo and right-aligned menu items. Dropdown menus on hover with subtle animations.

Hero section: Full-width background image with a semi-transparent overlay, featuring a large headline in 'Montserrat' 36px, a subheading in 'Open Sans' 18px, and a prominent call-to-action button.

General layout and composition: 12-column grid system with content centered and padded. Sections alternate between full-width and contained widths for visual interest.

Use of whitespace and visual hierarchy: Generous padding around sections (50px top and bottom), with clear spacing between elements to guide the eye. Important elements like CTAs have extra margin to stand out.

Imagery and iconography style: High-quality photographs with a consistent filter for cohesion. Icons are simple line icons in the primary color.

Typography usage: 'Open Sans' for body text (16px), 'Montserrat' for headings (24px for h2, 18px for h3), and buttons (14px uppercase).

Distinctive design patterns or elements: Cards with subtle shadows and rounded corners, buttons with hover effects that change background color and add a slight scale, and a sticky footer.

Overall visual impression and aesthetic: Clean, professional, and modern, with a focus on usability and readability."#.to_owned(),
            },
        ]
    }
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ColorPaletteSummary {
    pub summary: String,
}

impl FewShotsOutput<ColorPaletteSummary> for ColorPaletteSummary {
    fn few_shots() -> Vec<ColorPaletteSummary> {
        vec![
            ColorPaletteSummary {
                summary: r#"Color Palette Analysis:

Dominant colors: #2C3E50 (navy blue), #18BC9C (teal), #ECF0F1 (light gray), #34495E (dark gray).

Overall color scheme: A cool, professional palette with a primary triad of navy, teal, and light gray, creating a harmonious and trustworthy atmosphere.

Color contrasts and harmonies: High contrast between #ECF0F1 and #34495E ensures readability, while the teal provides a vibrant accent that complements the navy.

Possible usage: #ECF0F1 as the main background, #2C3E50 for headers and footers, #34495E for body text, and #18BC9C for buttons, links, and accents."#.to_owned(),
            },
        ]
    }
}

#[derive(Debug, Clone, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct TailwindConversionSummary {
    pub summary: String,
}

impl FewShotsOutput<TailwindConversionSummary> for TailwindConversionSummary {
    fn few_shots() -> Vec<TailwindConversionSummary> {
        vec![
            TailwindConversionSummary {
                summary: r#"Example Conversion for Inline Styles to Tailwind CSS using exact colors and reusable component templates:

1. **Button Component**:
   - Tailwind classes: `bg-[#18BC9C] text-[#FFFFFF] py-2 px-4 rounded transition-colors duration-300 hover:bg-[#159a85]`
   - Usage: Reusable for primary actions across the site.

2. **Link Component**:
   - Tailwind classes: `text-[#2C3E50] no-underline transition-colors duration-300 hover:text-[#1F2E3D]`
   - Usage: Standard link styling for navigation and in-content links.

3. **Features Block Component**:
   - Template:
     ```html
     <div class="bg-[#F9FAFB] shadow-lg rounded-lg p-6">
         <h2 class="text-[#2C3E50] font-bold text-xl mb-2">Feature Title</h2>
         <p class="text-[#34495E] text-base">Description text for this feature block. This component can be reused across multiple sections.</p>
     </div>
     ```
   - Usage: Designed to display feature sections with consistent spacing, background, and shadow settings.

4. **Card Component with Distinct Shadow Settings**:
   - Template:
     ```html
     <div class="bg-white rounded-lg shadow-md hover:shadow-xl p-4">
         <img class="w-full rounded-t-lg" src="image.jpg" alt="Card Image">
         <div class="p-4">
             <h3 class="text-[#2C3E50] font-semibold text-lg">Card Title</h3>
             <p class="text-[#34495E] text-sm">Card description text goes here.</p>
         </div>
     </div>
     ```
   - Usage: A versatile card component that can be employed in galleries or listings, with a clear elevation change on hover.

If the original HTML contains repeated patterns for features sections or blocks with distinct styling (e.g., different shadow settings), each should be identified and converted into a corresponding reusable template."#
                    .to_owned(),
            },
        ]
    }
}

/// Extracts a screenshot from the website using the ZyteClient.
async fn extract_screenshot(tx: &tokio::sync::mpsc::Sender<ProcessMessage>, style_website: &Url, full_page: bool) -> anyhow::Result<String> {
    let api_key = env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set for tests");
    let client = ZyteClient::new(api_key);
    let screenshot = client.screenshot_website(style_website.to_string().as_str(), full_page).await?;
    tx.send(ProcessMessage::WebsiteScreenshot(screenshot.clone())).await?;
    Ok(screenshot)
}

/// Extracts a color palette from the screenshot.
async fn extract_palette(tx: &tokio::sync::mpsc::Sender<ProcessMessage>, screenshot: String) -> anyhow::Result<Option<String>> {
    tx.send(ProcessMessage::Progress("Analyzing color palette".to_string())).await?;
    let image_analysis = analyze_image_from_base64(&screenshot).ok();
    if let Some(image_analysis) = image_analysis {
        let palette_json = serde_json::to_string(&image_analysis.dominant_colors)?;
        let palette_summary = llm_typed_static_simple::<ColorPaletteSummary>(
            format!(
                r#"
Analyze the following color palette extracted from a website screenshot:

{palette_json}

Provide a detailed summary including:
1. Dominant colors: List the top colors with their hex codes.
2. Overall color scheme: Describe the type of color scheme and its mood or tone.
3. Color contrasts and harmonies: Note any significant contrasts or harmonious combinations.
4. Possible usage: Infer where these colors might be applied based on typical design patterns.
"#,
                palette_json = palette_json
            ),
            5,
            Some(OpenAIModel::Gpto3mini20250131),
        )
            .await
            .ok()
            .map(|res| res.content.summary);
        tx.send(ProcessMessage::ColorPalette(image_analysis.dominant_colors.clone())).await?;
        Ok(palette_summary)
    } else {
        Ok(None)
    }
}

/// Extracts a CSS style summary from the website.
async fn extract_css_style(style_website: &Url) -> Option<CssStyleSummary> {
    let api_key = env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set for screenshot");
    let client = ZyteClient::new(api_key);
    let style = client.extract_styles(style_website.to_string().as_str()).await;
    if let Ok(style) = style {
        let style_summary: anyhow::Result<SentinelTypedResponse<CssStyleSummary>> = llm_typed_static_simple(
            format!(
                r#"
Analyze the provided CSS styles and provide a detailed summary including:

1. **Layout structure**: Describe the primary layout techniques used (e.g., grid, flexbox, absolute/relative positioning).
2. **Typography details**: List font families (including Google Fonts in a format like `<link href='https://fonts.googleapis.com/css?family=FontName' rel='stylesheet'>`), sizes, weights, and line heights.
3. **Color scheme**: Identify primary colors, secondary colors, background colors, and text colors with their hex codes where applicable.
4. **Spacing**: Detail the use of margins, paddings, and gaps (e.g., consistent spacing patterns or specific values).
5. **Unique styling features**: Highlight any animations, transitions, hover effects, or other distinctive styling techniques.
6. **Google Fonts used**: Provide a list in a format ready to paste into the HTML header (e.g., `<link href='https://fonts.googleapis.com/css?family=Roboto' rel='stylesheet'>`).
7. **CSS frameworks or libraries**: If detectable, note any frameworks like Bootstrap or Tailwind CSS based on class names or patterns.

Ensure the summary is comprehensive, covering all major aspects of the website’s styling, and retain the original style values as passed.

<STYLE>
{style:?}
</STYLE>
"#,
                style = style
            ),
            5,
            Some(OpenAIModel::Gpto3mini20250131),
        )
            .await;
        style_summary.map(|summary| summary.content).ok()
    } else {
        None
    }
}

/// Extracts a visual style summary from the screenshot.
async fn extract_style_summary(tx: &tokio::sync::mpsc::Sender<ProcessMessage>, screenshot: &anyhow::Result<String>) -> Option<VisualStyleSummary> {
    if let Ok(screenshot_ref) = screenshot {
        let _ = tx.send(ProcessMessage::WebsiteScreenshot(screenshot_ref.clone())).await;
    }
    if let Ok(screenshot) = screenshot {
        let visual_analysis: Option<SentinelTypedResponse<VisualStyleSummary>> = llm_typed_static_simple(
            r#"
Use reasoning

Examine the provided website screenshot and provide a comprehensive visual style summary, including:

1. **Header design**: Describe the navigation structure, logo placement, and any interactive elements.
2. **Hero section** (if present): Detail its layout, imagery, typography, and call-to-action elements.
3. **General layout and composition**: Analyze the grid system, alignment (e.g., centered, justified), and balance of elements.
4. **Use of whitespace and visual hierarchy**: Note how spacing and element placement guide the user’s eye.
5. **Imagery and iconography style**: Describe the type, quality, and stylistic approach of images and icons (e.g., flat, realistic).
6. **Typography usage**: Summarize font styles, sizes, and their application across headings, body text, and buttons.
7. **Distinctive design patterns or elements**: Highlight features like cards, modals, buttons, or shadows.
8. **Overall visual impression and aesthetic**: Provide a holistic view of the design’s tone (e.g., modern, minimalist, playful).

This summary should give a clear, detailed picture of the website’s design language and user experience.

[Website screenshot is provided]
"#.to_string(),
            5,
            Some(OpenAIModel::Gpto3mini20250131),
        )
            .await
            .ok();
        visual_analysis.map(|v| v.content)
    } else {
        None
    }
}

/// Converts inline styles from the given URL to Tailwind CSS classes using exact colors.
pub async fn convert_inline_styles_to_tailwind(url: &str) -> anyhow::Result<String> {
    // Initialize ZyteClient with API key from environment variables.
    let api_key = env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set");
    let client = ZyteClient::new(api_key);

    // Extract the HTML with inline styles using the Zyte client.
    let html = client.extract_inline_styles_v2(url).await.unwrap();

    // Build the prompt instructing the LLM to convert inline styles into Tailwind CSS classes.
    let prompt = format!(
        r#"
Analyze the following HTML that contains inline styles:

HTML:
{}

Your task is to extract the color values and generate equivalent Tailwind CSS classes using exact colors (using bracket notation). The original inline style values are not important; instead, focus on identifying repeated design patterns and creating reusable component templates.

Focus on:

1. **Buttons**: Identify inline styles applied to buttons and convert them into a reusable Tailwind CSS component. Ensure that the exact background and text colors are preserved and include hover/active state variations if applicable.
2. **Links**: Identify inline styles for links and convert them into a reusable Tailwind CSS component. Provide exact color values for normal and hover/active states.
3. **Other Reusable Components**: If the HTML contains repeated patterns—such as feature sections, cards, or blocks with distinct shadow or spacing settings—generate reusable Tailwind CSS templates for these components. Include guidelines for spacing, shadows, and any additional styling elements, along with brief usage descriptions.

For each identified component type, output the resulting Tailwind CSS class template along with a brief description of its intended usage, without referencing the original inline styles.
"#,
        html
    );

    // Use the LLM helper to convert the inline styles to Tailwind CSS.
    let tailwind_conversion = llm_typed_static_simple::<TailwindConversionSummary>(
        prompt,
        5,
        Some(OpenAIModel::Gpto3mini20250131),
    )
        .await
        .context("Failed to convert inline styles to Tailwind CSS")?;

    // Return the conversion summary.
    Ok(tailwind_conversion.content.summary)
}

/// Combines all the strategies into one summary, including CSS styles, color palette, visual style, and Tailwind conversion.
pub async fn extract_style_from_url(url: Url) -> anyhow::Result<String> {
    let (tx, _rx) = channel(10);

    // Step 1: Extract CSS style, screenshots, and inline style conversion in parallel.
    let css_future = extract_css_style(&url);
    let visible_screenshot_future = extract_screenshot(&tx, &url, false);
    let full_screenshot_future = extract_screenshot(&tx, &url, true);
    let tailwind_conversion_future = convert_inline_styles_to_tailwind(url.as_str());

    let (css_result, visible_screenshot_result, full_screenshot_result, tailwind_conversion_result) = tokio::join!(
        css_future,
        visible_screenshot_future,
        full_screenshot_future,
        tailwind_conversion_future,
    );

    // Step 2: Analyze visible screenshot (palette and visual feedback) in parallel.
    let (visible_palette_result, visible_visual_result) = if let Ok(screenshot) = visible_screenshot_result {
        tokio::join!(
            extract_palette(&tx, screenshot.clone()),
            extract_style_from_screenshot(&tx, &screenshot)
        )
    } else {
        (
            Err(anyhow!("Failed to extract visible screenshot")),
            Err(anyhow!("Failed to extract visible screenshot"))
        )
    };

    // Step 3: Analyze full page screenshot (palette and visual feedback) in parallel.
    let (full_palette_result, full_visual_result) = if let Ok(screenshot) = full_screenshot_result {
        tokio::join!(
            extract_palette(&tx, screenshot.clone()),
            extract_style_from_screenshot(&tx, &screenshot)
        )
    } else {
        (
            Err(anyhow!("Failed to extract full screenshot")),
            Err(anyhow!("Failed to extract full screenshot"))
        )
    };

    // Step 4: Generate descriptions for each part, handling missing data gracefully.
    let css_description = if let Some(css) = css_result {
        format!("CSS Style Summary:\n{}\n", css.summary)
    } else {
        "CSS Style Summary: Not available.\n".to_string()
    };

    let visible_palette_description = match visible_palette_result {
        Ok(Some(palette)) => format!("Visible Part Color Palette Analysis:\n{}\n", palette),
        _ => "Visible Part Color Palette Analysis: Not available.\n".to_string(),
    };

    let visible_visual_description = match visible_visual_result {
        Ok(visual) => format!("Visible Part Visual Style Summary:\n{}\n", visual),
        Err(_) => "Visible Part Visual Style Summary: Not available.\n".to_string(),
    };

    let full_palette_description = match full_palette_result {
        Ok(Some(palette)) => format!("Full Page Color Palette Analysis:\n{}\n", palette),
        _ => "Full Page Color Palette Analysis: Not available.\n".to_string(),
    };

    let full_visual_description = match full_visual_result {
        Ok(visual) => format!("Full Page Visual Style Summary:\n{}\n", visual),
        Err(_) => "Full Page Visual Style Summary: Not available.\n".to_string(),
    };

    let tailwind_description = match tailwind_conversion_result {
        Ok(summary) => format!("Tailwind Conversion Summary:\n{}\n", summary),
        Err(_) => "Tailwind Conversion Summary: Not available.\n".to_string(),
    };

    // Step 5: Combine all descriptions into a single final summary.
    let final_description = format!(
        "Website Style Description for {}:\n\n{}{}{}{}{}{}",
        url,
        css_description,
        visible_palette_description,
        visible_visual_description,
        full_palette_description,
        full_visual_description,
        tailwind_description
    );

    Ok(final_description)
}

#[cfg(FALSE)]
mod tests {
    use std::str::FromStr;
    use super::*;
    use dotenvy::dotenv;

    #[tokio::test]
    async fn test_extract_style_summary() {
        let _ = dotenv();
        let url = Url::from_str("https://bounti.ai").unwrap();
        let visual_summary = extract_style_from_url(url).await.unwrap();
        println!("{}", visual_summary);
    }
}