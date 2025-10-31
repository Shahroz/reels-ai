pub mod style_replication;

// // Placeholder struct to satisfy type errors (E0432) for HTMLResponse
// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
// struct HTMLResponse {
//     html: String,
// }
//
// // Placeholder structs to satisfy type errors (E0412)
// use crate::style_analysis::divmagic_v1::ProcessMessage;
// // These might need to be moved to a more appropriate module (e.g., integrations or a common types mod)
// #[derive(serde::Deserialize)] // Added Deserialize based on llm_typed_static_simple usage
// struct ColorPaletteSummary {
//     summary: String,
// }
// #[derive(serde::Deserialize)] // Added Deserialize based on llm_typed_static_simple usage
// struct TailwindConversionSummary {
//     summary: String,
// }
//
// // This module implements the core logic for extracting website styles.
//
// use anyhow::{anyhow, Context, Result};
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
// use sqlx::{types::Uuid, PgPool};
// use std::time::Instant;
// use tokio::sync::mpsc::{channel, Sender};
// use url::Url;
// // use gennodes_common::traits::llm_traits::FewShotsOutput;
//
// // Assuming Zyte and LLM clients are structured similarly to the library doc
// // Adjust these paths if the actual project structure differs.
// use crate::db::requests::{self, CreateRequestArgs, UpdateRequestArgs};
// use crate::integrations::extract_css_style;
// use crate::integrations::gpt::OpenAIModel;
// use crate::integrations::llm_typed_static::llm_typed_static_simple;
// use crate::integrations::zyte::ZyteClient;
// use crate::llm::traits::FewShotsOutput;
// // Placeholder import
// // use gennodes_image_transform::analyze_image_from_base64; // Added to resolve E0425
//
// // Re-using structures from library_docs/website_style/extract_style.rs
// // Ensure these are defined or imported correctly if they live elsewhere.
// // use crate::style_analysis::divmagic_v1::{ProcessMessage, HTMLResponse}; // Assuming this path based on library doc & added HTMLResponse
// // use crate::style_analysis::divmagic_v2::extract_style_from_screenshot; // Assuming this path based on library doc (Removed as unused)
//
// // Imports for visual feedback loop
// // use crate::integrations::llm_typed_image_description::{llm_describe_image_typed, GPTVisionModel};
// // use crate::integrations::sentinel::SentinelTypedResponse;
// // Assuming screenshot_html is moved or accessible via a path like this:
// // use crate::integrations::lighthouse::screenshot_html; // Assuming screenshot_html is moved here
//
// // --- Structs for Visual Feedback Loop --- //
//
// /// Helper struct for the visual score response (adapted from html_improvement.rs)
// #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
// struct VisualScore {
//     #[schemars(description = "Explain step by step your reasoning process")]
//     reasoning: String,
//     #[schemars(description = "Score between 1 (no design) and 100 (expert design)")]
//     visual_score: usize,
// }
//
// // Implement FewShotsOutput for VisualScore if needed for llm_describe_image_typed
// impl FewShotsOutput<VisualScore> for VisualScore {
//     fn few_shots() -> Vec<VisualScore> {
//         todo!()
//     }
// }
//
// // --- End Structs for Visual Feedback Loop --- //
//
// /// Parameters for the main style extraction function.
// #[derive(Debug, Clone)]
// pub struct ExtractStyleParams {
//     pub target_url: Url,
//     pub content_to_style: Option<String>, // Optional content to apply style to
//     pub what_to_create: Option<String>,   // Description of desired output
//     pub user_id: Option<Uuid>,
//     pub visual_feedback_enabled: bool,
//     pub db_pool: PgPool,
//     pub zyte_api_key: String,
//     // Add LLM client/config if needed beyond llm_typed_static_simple
//     // Parameters for the feedback loop
//     pub min_score: Option<usize>,
//     pub max_tries: Option<usize>,
// }
//
// /// Represents the final output of the style extraction process.
// #[derive(Debug, Clone)]
// pub struct StyleExtractionOutput {
//     pub request_id: i32,
//     pub final_description: String,
//     pub compressed_output_html: Option<Vec<u8>>, // Final styled HTML output
// }
//
// /// Scores the visual design based solely on the screenshot.
// /// Adapted from library_docs/website_style/visual_html_finetuning/html_improvement.rs::score_html
// async fn score_html_internal(_screenshot_base64: &str) -> Result<usize> {
//     let _prompt = r#"
// Evaluate the visual design of the website based solely on the provided screenshot.
// Rate the design on a scale from 1 (very poor, no style) to 100 (expert design).
// Return a JSON object with keys "reasoning" (string) and "visual_score" (integer).
// "#;
//
//     // Commented out due to unresolved llm_describe_image_typed
//     // // Using llm_describe_image_typed requires appropriate setup for LLM calls
//     // let result: SentinelTypedResponse<VisualScore> = llm_describe_image_typed::<VisualScore>(
//     //     prompt,
//     //     screenshot_base64,
//     //     GPTVisionModel::Gpt4o, // Or another appropriate vision model
//     //     3, // Retries
//     // )
//     // .await
//     // .context("LLM call failed during visual scoring")?;
//     // Ok(result.content.visual_score)
//     Ok(80) // Placeholder score
// }
//
// /// Generates a new HTML variation that improves the design.
// /// Adapted from library_docs/website_style/visual_html_finetuning/html_improvement.rs::generate_html_variation
// async fn generate_html_variation_internal(
//     html: &str,
//     _screenshot_base64: &str, /*, global_style: Option<&str> */
// ) -> Result<String> {
//     // let additional_style_prompt = if let Some(style) = global_style {
//     //     format!("Additionally, ensure that any modifications align with the following global style guidelines: {}.", style)
//     // } else {
//     //     String::new()
//     // };
//
//     let prompt = format!(
//         r#"
// HTML:
// {}
//
// Improve the design of the following HTML based on its screenshot.
// Enhance the visual aesthetics, layout, and user experience using modern design principles and Tailwind CSS.
// When it comes to images, only use the ones provided; if they are missing or unsuitable, please remove them.
// Return only the updated HTML as a JSON structure with "reasoning" (string) and "html" (string) as keys.
// Important:
// - preserve all content from HTML (all sections)
// "#,
//         html //, additional_style_prompt
//     );
//
//     // Commented out due to unresolved llm_describe_image_typed
//     // // Using llm_describe_image_typed requires appropriate setup for LLM calls
//     // let result: SentinelTypedResponse<HTMLResponse> = llm_describe_image_typed::<HTMLResponse>(
//     //     &prompt,
//     //     screenshot_base64,
//     //     GPTVisionModel::Gpto1, // Or another appropriate vision model
//     //     3, // Retries
//     // )
//     // .await
//     // .context("LLM call failed during HTML variation generation")?;
//     // Ok(result.content.html)
//     Ok(html.to_string()) // Placeholder variation
// }
//
// /// Improves HTML style by iteratively generating and scoring variations.
// /// Adapted from library_docs/website_style/visual_html_finetuning/html_improvement.rs::improve_html_style
// async fn replicate_style_with_feedback(
//     initial_html: &str,
//     max_tries: usize,
//     min_score: usize,
//     // global_style: Option<&str> // Optional: Add if needed later
// ) -> Result<String> {
//     let mut current_html = initial_html.to_string();
//     let mut best_score = 0; // Initialize best score
//
//     // Initial evaluation is helpful but optional here, we can start the loop directly
//     // Or evaluate the initial HTML first
//     // Commented out initial scoring due to unresolved screenshot_html
//     // match screenshot_html(&current_html, true).await {
//     //     Ok(screenshot) => {
//     //         match score_html_internal(&screenshot).await {
//     //             Ok(score) => {
//     //                 log::info!("Initial HTML score: {}", score);
//     //                 best_score = score;
//     //                 if best_score >= min_score {
//     //                     log::info!("Initial score meets minimum, returning initial HTML.");
//     //                     return Ok(current_html);
//     //                 }
//     //             }
//     //             Err(e) => {
//     //                 log::warn!("Failed to score initial HTML, proceeding with improvement loop: {:?}", e);
//     //             }
//     //         }
//     //     }
//     //     Err(e) => {
//     //          log::warn!("Failed to screenshot initial HTML, proceeding with improvement loop: {:?}", e);
//     //     }
//     // }
//
//     log::info!(
//         "Starting visual feedback loop: max_tries={}, min_score={}",
//         max_tries,
//         min_score
//     );
//     for i in 0..max_tries {
//         log::info!("Feedback loop iteration {}/{}...", i + 1, max_tries);
//         // Commented out feedback loop iteration logic due to unresolved dependencies
//         // let iteration_result: Result<(String, usize)> = async {
//         //     // 1. Screenshot current HTML
//         //     let current_screenshot = screenshot_html(&current_html, true).await
//         //         .context("Failed to screenshot current HTML in feedback loop")?;
//         //
//         //     // 2. Generate variation
//         //     let new_html_variation = generate_html_variation_internal(&current_html, &current_screenshot /*, global_style */).await
//         //         .context("Failed to generate HTML variation in feedback loop")?;
//         //
//         //     // 3. Screenshot new variation
//         //     let new_screenshot = screenshot_html(&new_html_variation, true).await
//         //          .context("Failed to screenshot new HTML variation in feedback loop")?;
//         //
//         //     // 4. Score new variation
//         //     let new_score = score_html_internal(&new_screenshot).await
//         //         .context("Failed to score new HTML variation in feedback loop")?;
//         //
//         //     Ok((new_html_variation, new_score))
//         // }.await;
//         let iteration_result: Result<(String, usize)> = Ok((current_html.clone(), best_score)); // Placeholder: No improvement
//
//         match iteration_result {
//             Ok((new_html, new_score)) => {
//                 log::info!("Iteration {} - New score: {}", i + 1, new_score);
//                 // Update current HTML only if the new score is strictly better
//                 // This avoids getting stuck on variations with the same score
//                 if new_score > best_score {
//                     best_score = new_score;
//                     current_html = new_html;
//                     log::info!(
//                         "Improved HTML with score: {}. Updated current HTML.",
//                         best_score
//                     );
//                 } else {
//                     log::info!(
//                         "New score ({}) is not better than best score ({}). Keeping previous HTML.",
//                         new_score,
//                         best_score
//                     );
//                 }
//
//                 // Early exit if score meets or exceeds minimum.
//                 if best_score >= min_score {
//                     log::info!(
//                         "Minimum score ({}) reached. Exiting feedback loop.",
//                         min_score
//                     );
//                     break;
//                 }
//             }
//             Err(e) => {
//                 log::warn!(
//                     "Feedback loop iteration {} failed: {:?}. Continuing to next iteration.",
//                     i + 1,
//                     e
//                 );
//                 // Optionally break if too many consecutive errors occur
//                 continue;
//             }
//         }
//     }
//     log::info!("Feedback loop finished. Final score: {}", best_score);
//     Ok(current_html)
// }
//
// /// Extracts style information from a URL, persists progress, and returns a summary.
// /// If `content_to_style` is provided, it also generates styled HTML using Zyte
// /// and potentially refines it with a visual feedback loop.
// pub async fn extract_and_persist_style(
//     params: ExtractStyleParams,
// ) -> anyhow::Result<StyleExtractionOutput> {
//     let start_time = Instant::now();
//
//     // Step 0: Create initial request record in DB
//     let create_args = CreateRequestArgs {
//         url: Some(params.target_url.to_string()),
//         content_to_style: params.content_to_style.clone(),
//         what_to_create: params.what_to_create.clone(),
//         status: "Initializing".to_string(),
//         user_id: params.user_id,
//         visual_feedback: Some(params.visual_feedback_enabled),
//     };
//     let request_id = requests::create_request(&params.db_pool, create_args).await?; // Added await
//     let _ = requests::update_request_status(&params.db_pool, request_id, "Processing").await; // Use await
//
//     // Setup for potential progress messages (though not fully utilized in this adaptation)
//     let (tx, _rx): (Sender<ProcessMessage>, _) = channel(10);
//
//     // --- Styling Generation (if content_to_style is provided) --- //
//     let mut final_styled_html_bytes: Option<Vec<u8>> = None;
//     if let Some(content_to_style) = &params.content_to_style {
//         log::info!(
//             "Content to style provided for request {}, proceeding with style generation.",
//             request_id
//         );
//         let _ = requests::update_request_status(
//             &params.db_pool,
//             request_id,
//             "Generating Initial Style",
//         )
//         .await;
//         let zyte_client = ZyteClient::new(params.zyte_api_key.clone());
//
//         // Call Zyte to get initial styling. IMPORTANT: Disable Zyte's internal refinement loop
//         // if we are using our own feedback loop based on visual_feedback_enabled.
//         let use_zyte_internal_refinement = false; // Let our loop handle refinement if enabled
//         match zyte_client
//             .extract_and_apply_styles(
//                 params.target_url.as_str(),
//                 content_to_style,
//                 use_zyte_internal_refinement, // Pass false here
//             )
//             .await
//         {
//             Ok(initial_styled_html) => {
//                 log::info!(
//                     "Successfully generated initial styled HTML for request {}.",
//                     request_id
//                 );
//                 let mut current_styled_html: String = initial_styled_html;
//
//                 // Apply our visual feedback loop if enabled
//                 if params.visual_feedback_enabled {
//                     log::info!(
//                         "Visual feedback enabled for request {}, starting refinement loop.",
//                         request_id
//                     );
//                     let _ = requests::update_request_status(
//                         &params.db_pool,
//                         request_id,
//                         "Refining Style (Visual Feedback)",
//                     )
//                     .await;
//                     let max_tries = params.max_tries.unwrap_or(5); // Default tries
//                     let min_score = params.min_score.unwrap_or(80); // Default minimum score
//
//                     match replicate_style_with_feedback(&current_styled_html, max_tries, min_score)
//                         .await
//                     {
//                         Ok(refined_html) => {
//                             log::info!(
//                                 "Visual feedback loop completed successfully for request {}.",
//                                 request_id
//                             );
//                             current_styled_html = refined_html;
//                         }
//                         Err(e) => {
//                             log::error!(
//                                 "Visual feedback loop failed for request {}: {:?}",
//                                 request_id,
//                                 e
//                             );
//                             // Keep the initial HTML, log error
//                             let _ = requests::update_request_status(
//                                 &params.db_pool,
//                                 request_id,
//                                 &format!("Feedback Loop Failed: {}", e),
//                             )
//                             .await;
//                         }
//                     }
//                 } else {
//                     log::info!(
//                         "Visual feedback disabled for request {}, skipping refinement loop.",
//                         request_id
//                     );
//                 }
//
//                 // Store the final result (initial or refined)
//                 final_styled_html_bytes = Some(current_styled_html.into_bytes()); // Ensure this is called on String
//                 log::info!(
//                     "Final styled HTML prepared for request {}. Size: {} bytes",
//                     request_id,
//                     final_styled_html_bytes.as_ref().map_or(0, |v| v.len())
//                 );
//             }
//             Err(e) => {
//                 log::error!(
//                     "Failed to generate initial styled HTML via Zyte for request {}: {:?}",
//                     request_id,
//                     e
//                 );
//                 // Update status and potentially fail the request here or let it proceed to description generation
//                 let _ = requests::update_request_status(
//                     &params.db_pool,
//                     request_id,
//                     &format!("Initial Styling Failed: {}", e),
//                 )
//                 .await;
//                 // Decide if failure here should stop the whole process
//                 // For now, let it continue to generate the description part
//             }
//         }
//     }
//
//     // Wrap the core *description* logic in a try block to easily update DB status on error
//     let description_result: anyhow::Result<String> = async {
//         let _ = requests::update_request_status(
//             &params.db_pool,
//             request_id,
//             "Extracting Styles Description",
//         )
//         .await;
//         // Step 1: Extract CSS style, screenshots, and inline style conversion in parallel.
//         let zyte_client = ZyteClient::new(params.zyte_api_key.clone()); // Assuming constructor
//
//         let css_future = extract_css_style(&params.target_url); // Uses LLM internally
//                                                                 // Commented out screenshot extraction due to unresolved dependencies
//                                                                 // let visible_screenshot_future = extract_screenshot(&tx, &params.target_url, false, &zyte_client); // Pass client
//                                                                 // let full_screenshot_future = extract_screenshot(&tx, &params.target_url, true, &zyte_client); // Pass client
//         let tailwind_conversion_future =
//             convert_inline_styles_to_tailwind(params.target_url.as_str(), &zyte_client); // Pass client
//
//         // Updated join to remove screenshot futures
//         let (css_result, tailwind_conversion_result) = tokio::join!(
//             css_future,
//             // visible_screenshot_future,
//             // full_screenshot_future,
//             tailwind_conversion_future,
//         );
//         // Define placeholder results for screenshots
//         let visible_screenshot_result: Result<String, anyhow::Error> =
//             Err(anyhow!("Screenshot extraction disabled"));
//         let full_screenshot_result: Result<String, anyhow::Error> =
//             Err(anyhow!("Screenshot extraction disabled"));
//
//         let _ =
//             requests::update_request_status(&params.db_pool, request_id, "Analyzing Screenshots")
//                 .await;
//         // Step 2: Analyze visible screenshot (palette and visual feedback) in parallel.
//         let (visible_palette_result, visible_visual_result) =
//             if let Ok(ref _screenshot) = visible_screenshot_result {
//             // Execute the async tasks and assign results directly inside the block
//             // Since the original join! calls are commented out, assign placeholder errors for now
//             // If the joins were active, they would need to return the results, not just ()
//              (
//                  Err(anyhow!("Visible palette extraction disabled")), // Placeholder error
//                  Err(anyhow!("Visible visual analysis disabled"))    // Placeholder error
//              )
//             // If join! was used and returned results:
//             // let (palette_res, visual_res) = tokio::join!(
//             //     extract_palette(&tx, screenshot.clone()),
//             //     extract_style_from_screenshot(&tx, screenshot)
//             // );
//             // (palette_res, visual_res) // Return the tuple from the if block
//         } else {
//             (
//                 Err(anyhow!("Failed to extract visible screenshot")), // Propagate error
//                 Err(anyhow!("Failed to extract visible screenshot"))  // Propagate error
//             )
//         };
//
//         // Step 3: Analyze full page screenshot (palette and visual feedback) in parallel.
//         let (full_palette_result, full_visual_result) =
//             if let Ok(ref _screenshot) = full_screenshot_result {
//             // Execute the async tasks and assign results directly inside the block
//             // Since the original join! calls are commented out, assign placeholder errors for now
//              (
//                  Err(anyhow!("Full palette extraction disabled")), // Placeholder error
//                  Err(anyhow!("Full visual analysis disabled"))    // Placeholder error
//              )
//             // If join! was used and returned results:
//             // let (palette_res, visual_res) = tokio::join!(
//             //     extract_palette(&tx, screenshot.clone()),
//             //     extract_style_from_screenshot(&tx, screenshot)
//             // );
//             // (palette_res, visual_res) // Return the tuple from the if block
//         } else {
//             (
//                 Err(anyhow!("Failed to extract full screenshot")), // Propagate error
//                 Err(anyhow!("Failed to extract full screenshot"))  // Propagate error
//             )
//         };
//
//         let _ = requests::update_request_status(
//             &params.db_pool,
//             request_id,
//             "Generating Summary Description",
//         )
//         .await;
//         // Step 4: Generate descriptions for each part, handling missing data gracefully.
//         let css_description = css_result.map_or_else(
//             |_| "CSS Style Summary: Not available.\n".to_string(),
//             |css| format!("CSS Style Summary:\n{}\n", css.summary),
//         );
//
//         let visible_palette_description = match visible_palette_result {
//             Ok(Some(palette)) => format!("Visible Part Color Palette Analysis:\n{}\n", palette),
//             Ok(None) => {
//                 "Visible Part Color Palette Analysis: Could not be generated.\n".to_string()
//             }
//             Err(_) => {
//                 "Visible Part Color Palette Analysis: Not available due to screenshot error.\n"
//                     .to_string()
//             }
//         };
//
//         let visible_visual_description = match visible_visual_result {
//             Ok(visual) => format!("Visible Part Visual Style Summary:\n{}\n", visual.summary), // Assuming visual has .summary
//             Err(_) => "Visible Part Visual Style Summary: Not available.\n".to_string(),
//         };
//
//         let full_palette_description = match full_palette_result {
//             Ok(Some(palette)) => format!("Full Page Color Palette Analysis:\n{}\n", palette),
//             Ok(None) => "Full Page Color Palette Analysis: Could not be generated.\n".to_string(),
//             Err(_) => "Full Page Color Palette Analysis: Not available due to screenshot error.\n"
//                 .to_string(),
//         };
//
//         let full_visual_description = match full_visual_result {
//             Ok(visual) => format!("Full Page Visual Style Summary:\n{}\n", visual.summary), // Assuming visual has .summary
//             Err(_) => "Full Page Visual Style Summary: Not available.\n".to_string(),
//         };
//
//         let tailwind_description = match tailwind_conversion_result {
//             Ok(summary) => format!("Tailwind Conversion Summary:\n{}\n", summary),
//             Err(e) => format!("Tailwind Conversion Summary: Not available. Error: {}\n", e),
//         };
//
//         // Step 5: Combine all descriptions into a single final summary.
//         let final_description = format!(
//             "Website Style Description for {}:\n\n{}{}{}{}{}{}",
//             params.target_url,
//             css_description,
//             visible_palette_description,
//             visible_visual_description,
//             full_palette_description,
//             full_visual_description,
//             tailwind_description
//         );
//
//         Ok(final_description)
//     }
//     .await;
//
//     let elapsed_ms = start_time.elapsed().as_millis() as i32;
//
//     // Step 6: Update DB record with final status and results
//     match description_result {
//         Ok(final_description) => {
//             let status = if final_styled_html_bytes.is_some() {
//                 "Completed (with styled HTML)".to_string()
//             } else if params.content_to_style.is_some() {
//                 "Completed (HTML styling failed)".to_string()
//             } else {
//                 "Completed (description only)".to_string()
//             };
//
//             let update_args = UpdateRequestArgs {
//                 // Placeholder for actual compressed content if generated
//                 compressed_style_website_content: None,
//                 compressed_output_html: final_styled_html_bytes.clone(), // Use the generated bytes
//                 status,
//                 execution_time_ms: Some(elapsed_ms),
//                 credits_used: Some(1), // Placeholder: Calculate actual credits used
//             };
//             requests::update_request_completion(&params.db_pool, request_id, update_args).await?;
//
//             Ok(StyleExtractionOutput {
//                 request_id,
//                 final_description,
//                 compressed_output_html: final_styled_html_bytes.clone(),
//             })
//         }
//         Err(e) => {
//             log::error!(
//                 "Style description generation failed for request {}: {:?}",
//                 request_id,
//                 e
//             );
//             let error_message = format!("Description Failed: {}", e);
//             let update_args = UpdateRequestArgs {
//                 compressed_style_website_content: None,
//                 compressed_output_html: final_styled_html_bytes.clone(), // Still store HTML if generated before description error
//                 status: error_message.chars().take(255).collect(),       // Truncate error for DB
//                 execution_time_ms: Some(elapsed_ms),
//                 credits_used: Some(0),
//             };
//             // Attempt to update DB even on failure
//             let _ =
//                 requests::update_request_completion(&params.db_pool, request_id, update_args).await;
//             // Propagate the original error
//             Err(e).context("Style description extraction process failed")
//         }
//     }
// }
//
// // Helper function adaptations (ensure dependencies like `analyze_image_from_base64` are available)
// // These might need adjustments based on actual client implementations
//
// // Commented out due to unresolved dependencies
// // /*
// // async fn extract_screenshot(
// //     tx: &Sender<ProcessMessage>,
// //     style_website: &Url,
// //     full_page: bool,
// //     client: &ZyteClient, // Pass client explicitly
// // ) -> anyhow::Result<String> {
// //     let screenshot = client.screenshot_website(style_website.to_string().as_str(), full_page).await?;
// //     // Sending message might fail if receiver dropped, ignore error for now
// //     let _ = tx.send(ProcessMessage::WebsiteScreenshot(screenshot.clone())).await;
// //     Ok(screenshot)
// // }
// // */
// // Commented out due to unresolved analyze_image_from_base64
// async fn extract_palette(
//     tx: &Sender<ProcessMessage>,
//     _screenshot: String,
// ) -> anyhow::Result<Option<String>> {
//     let _ = tx
//         .send(ProcessMessage::Progress(
//             "Analyzing color palette".to_string(),
//         ))
//         .await;
//     // let image_analysis = analyze_image_from_base64(&screenshot).ok(); // <-- This function is unresolved
//     // Placeholder namespace required if gennodes_image_transform was used for type
//     mod gennodes_image_transform {
//         pub struct ImageAnalysisResult {
//             pub dominant_colors: Vec<String>,
//         }
//     }
//     let image_analysis: Option<gennodes_image_transform::ImageAnalysisResult> = None; // Placeholder
//     if let Some(image_analysis) = image_analysis {
//         let palette_json = serde_json::to_string(&image_analysis.dominant_colors)?;
//         let palette_summary = llm_typed_static_simple::<ColorPaletteSummary>(
//             format!(
//                 r#"
// Analyze the following color palette extracted from a website screenshot:
//
// {palette_json}
//
// Provide a detailed summary including:
// 1. Dominant colors.
// 2. Overall color scheme.
// 3. Contrasts and harmonies.
// 4. Possible usage.
// "#,
//                 palette_json = palette_json
//             ),
//             5,
//             Some(OpenAIModel::Gpto3mini20250131), // Use appropriate model
//         )
//         .await
//         .ok()
//         .map(|res| res.content.summary);
//         // Sending message might fail
//         let _ = tx
//             .send(ProcessMessage::ColorPalette(
//                 image_analysis.dominant_colors.clone(),
//             ))
//             .await;
//         Ok(palette_summary)
//     } else {
//         Ok(None)
//     }
// }
//
// async fn convert_inline_styles_to_tailwind(
//     url: &str,
//     client: &ZyteClient,
// ) -> anyhow::Result<String> {
//     // This function seems unrelated to the main flow modification, keeping as is.
//     let html = client
//         .extract_inline_styles_v2(url)
//         .await
//         .context("Failed to extract HTML with inline styles via Zyte")?;
//
//     let prompt = format!(
//         r#"
// Analyze the following HTML with inline styles:
//
// HTML:
// {}
//
// Convert inline styles to Tailwind CSS classes using exact colors (bracket notation). Focus on identifying repeated design patterns (buttons, links, cards, etc.) and create reusable component templates. Output the Tailwind templates and usage notes.
// "#,
//         html
//     );
//
//     let tailwind_conversion = llm_typed_static_simple::<TailwindConversionSummary>(
//         prompt,
//         5,
//         Some(OpenAIModel::Gpto3mini20250131), // Use appropriate model
//     )
//     .await
//     .context("LLM failed to convert inline styles to Tailwind CSS")?;
//
//     Ok(tailwind_conversion.content.summary)
// }
//
// // Note: extract_css_style and extract_style_summary from the library doc rely on LLMs.
// // Make sure llm_typed_static_simple or equivalent is correctly configured and available.
// // The library doc implementations are assumed here.
//
// // TODO: Add unit/integration tests for this module, especially the feedback loop.
// // TODO: Ensure LIGHTHOUSE_URL and LIGHTHOUSE_API_KEY env vars are properly handled/documented for deployment.
// // TODO: Refactor LLM calls (scoring, generation) into dedicated functions if reused elsewhere.
// // TODO: Consider moving helper structs (VisualScore, HTMLResponse) to a shared types module.
// // TODO: Add proper error handling and status updates within the feedback loop itself.
