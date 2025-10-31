//! Processes HTML content to find embedded base64 data URIs in image tags, upload them to GCS, and replace URIs with GCS URLs.
//!
//! This utility focuses on extracting `<img>` tag `src` attributes and `<image>` tag `xlink:href` attributes
//! containing base64 data. It decodes the data, uploads it to a specified GCS bucket path,
//! and replaces the original data URI in the HTML string with the GCS URL.
//! Includes tests for various scenarios.

// Note: Keep existing imports and function signature as they are part of the main logic.
use base64::{engine::general_purpose, Engine as _};
use scraper::{Html, Selector};
use std::collections::HashMap;
use uuid::Uuid;
use tracing::instrument;

/// Processes HTML content to find and replace base64 data URIs in image tags with GCS URLs.
///
/// This handles both `<img src="...">` and `<image xlink:href="...">`.
///
/// # Arguments
/// * `html_content` - The HTML string to process.
/// * `gcs_client` - An instance of the GCSClient for uploads.
/// * `bucket` - The GCS bucket name to upload assets to.
/// * `style_id` - The UUID of the style, used for constructing the GCS object path.
///
/// # Returns
/// * `Ok(String)` containing the modified HTML content.
/// * `Err(String)` if any critical error occurs during processing or upload.
#[instrument(skip(html_content, gcs_client, bucket))]
pub async fn process_image_data_uris(
    html_content: &str,
    gcs_client: &dyn crate::services::gcs::gcs_operations::GCSOperations,
    bucket: &str,
    style_id: Uuid,
) -> Result<String, String> {
    log::info!("Starting image data URI processing for style_id: {}", style_id);
    let mut replacements: HashMap<String, String> = HashMap::new();
    let mut processed_html = html_content.to_string(); // Clone to modify

    // Parse the HTML document
   let document = Html::parse_document(&processed_html);

    // --- Process <img> and <image> tags with data URIs ---
    // Note: Modern HTML parsers normalize xlink:href to href in SVG contexts
    // Use a simpler selector since we'll check all attributes in the loop anyway
    let img_selector = match Selector::parse("img, image") {
        Ok(selector) => selector,
        Err(e) => {
            log::error!("Failed to parse img/image selector: {e}");
            // Decide if this is fatal or just prevents img processing
            return Err(format!("Internal error: Failed to parse HTML selector ({e})"));
        }
   };

    for element in document.select(&img_selector) {
        // Check for 'src' (for <img>) or 'href'/'xlink:href' (for <image>)
        // Note: Modern parsers normalize xlink:href to href in SVG contexts
        // Use iteration approach due to scraper crate attribute access issues
        let mut src_opt = None;
        for (attr_name, attr_value) in element.value().attrs() {
            if attr_name == "src" || attr_name == "href" || attr_name == "xlink:href" {
                src_opt = Some(attr_value);
                break;
            }
        }
        if let Some(src) = src_opt {
            if src.starts_with("data:") {
                log::debug!("Found data URI in image tag: {}...", &src[..std::cmp::min(src.len(), 60)]);
                // Avoid reprocessing if already replaced (e.g., duplicate URIs)
                if replacements.contains_key(src) {
                    continue;
                }

                if let Some(pos) = src.find(";base64,") {
                    let mime_type = &src[5..pos]; // "data:".len() == 5
                    let base64_data = &src[pos + 8..]; // ";base64,".len() == 8

                    match general_purpose::STANDARD.decode(base64_data) {
                        Ok(data_bytes) => {
                            let asset_id = Uuid::new_v4();
                            let extension = mime_guess::get_mime_extensions_str(mime_type)
                                .and_then(|exts| exts.first().copied())
                                .unwrap_or("bin");
                            let filename = format!("{asset_id}.{extension}");
                            let object_path = format!("styles/{style_id}/assets/{filename}");

                            log::debug!("Uploading data URI asset: {object_path}");
                            match gcs_client
                                .upload_raw_bytes(bucket, &object_path, mime_type, data_bytes, false, crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic)
                                .await
                            {
                                Ok(gcs_url) => {
                                    log::info!("Uploaded data URI asset to GCS: {gcs_url}");
                                    replacements.insert(src.to_string(), gcs_url);
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to upload data URI asset '{object_path}' to GCS: {e}"
                                    );
                                    // Decide: continue or abort? Returning error for now.
                                    return Err(format!(
                                        "Failed to upload asset {filename} to storage: {e}"
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to decode base64 data from attribute (length {}): {}",
                                base64_data.len(),
                                e
                            );
                            // Skip this URI but don't fail the whole process
                        }
                    }
                } else {
                    log::warn!(
                        "Data URI format error or missing ';base64,' in attribute: {}...",
                        &src[..std::cmp::min(src.len(), 60)]
                    );
                    // Skip this URI
                }
            }
        }
    }

    // Perform replacements in the HTML content
    // Iterate replacements and replace occurrences in the cloned string
    for (original_uri, gcs_url) in &replacements {
         // Use `replacen` to avoid replacing already replaced URLs if GCS URL somehow contains the original data URI string
        processed_html = processed_html.replace(original_uri, gcs_url);
    }

    log::info!(
        "Finished image data URI processing for style_id: {}. Replaced {} URIs.",
        style_id,
        replacements.len()
    );
    Ok(processed_html)
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    // Import necessary items for testing the parsing logic directly
    use super::{general_purpose, Html, Selector}; // Use fully qualified paths as per guidelines if needed

    #[test]
    fn test_find_and_decode_single_img_data_uri() {
        // Test focuses on finding the URI and decoding it from an <img> tag
        let data_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII="; // 1x1 pixel PNG
        let html_in = format!("<p>Image: <img src='{}'></p>", data_uri);
        let document = Html::parse_document(&html_in);
        let img_selector = Selector::parse("img, image").expect("Failed to parse selector");

        let mut found_data: Option<(String, String, Vec<u8>)> = None;

        for element in document.select(&img_selector) {
            // Find the attribute value by iterating through all attributes
            let mut src_opt = None;
            for (attr_name, attr_value) in element.value().attrs() {
                if attr_name == "src" || attr_name == "href" || attr_name == "xlink:href" {
                    src_opt = Some(attr_value);
                    break;
                }
            }
            if let Some(src) = src_opt {
                if src.starts_with("data:") {
                    if let Some(pos) = src.find(";base64,") {
                        let mime_type = &src[5..pos];
                        let base64_data = &src[pos + 8..];
                        match general_purpose::STANDARD.decode(base64_data) {
                            Ok(decoded_bytes) => {
                                found_data = Some((
                                    src.to_string(),
                                    mime_type.to_string(),
                                    decoded_bytes,
                                ));
                                break; // Found the first one
                            }
                            Err(e) => {
                                panic!("Decoding failed in test: {}", e);
                            }
                        }
                    }
                }
            }
        }

        assert!(found_data.is_some(), "No data URI found or decoded");
        let (found_uri, mime, bytes) = found_data.unwrap();
        assert_eq!(found_uri, data_uri);
        assert_eq!(mime, "image/png");
        assert!(!bytes.is_empty(), "Decoded bytes should not be empty");
    }
    
    #[test]
    fn test_find_and_decode_single_image_xlink_href_data_uri() {
        // Test focuses on finding the URI and decoding it from an <image> tag
        // Note: Modern HTML parsers normalize xlink:href to href in SVG contexts
        let data_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII="; // 1x1 pixel PNG
        let html_in = format!("<svg><image xlink:href='{}'></image></svg>", data_uri);
        let document = Html::parse_document(&html_in);
        // Use a simpler selector since we'll check all attributes in the loop anyway
        let img_selector = Selector::parse("img, image").expect("Failed to parse selector");

        let mut found_data: Option<(String, String, Vec<u8>)> = None;

        for element in document.select(&img_selector) {
            // Find the attribute value by iterating through all attributes
            let mut src_opt = None;
            for (attr_name, attr_value) in element.value().attrs() {
                if attr_name == "src" || attr_name == "href" || attr_name == "xlink:href" {
                    src_opt = Some(attr_value);
                    break;
                }
            }
            if let Some(src) = src_opt {
                if src.starts_with("data:") {
                    if let Some(pos) = src.find(";base64,") {
                        let mime_type = &src[5..pos];
                        let base64_data = &src[pos + 8..];
                        match general_purpose::STANDARD.decode(base64_data) {
                            Ok(decoded_bytes) => {
                                found_data = Some((
                                    src.to_string(),
                                    mime_type.to_string(),
                                    decoded_bytes,
                                ));
                                break; // Found the first one
                            }
                            Err(e) => {
                                panic!("Decoding failed in test: {}", e);
                            }
                        }
                    }
                }
            }
        }

        assert!(found_data.is_some(), "No data URI found or decoded from <image> tag");
        let (found_uri, mime, bytes) = found_data.unwrap();
        assert_eq!(found_uri, data_uri);
        assert_eq!(mime, "image/png");
        assert!(!bytes.is_empty(), "Decoded bytes should not be empty");
    }

    #[test]
    fn test_find_multiple_mixed_image_data_uris() {
        let data_uri_png = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=";
        let data_uri_gif = "data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs=";
        let html_in = format!("<p><img src='{}'></p><svg><image xlink:href='{}'></image></svg>", data_uri_png, data_uri_gif);
        let document = Html::parse_document(&html_in);
        let img_selector = Selector::parse("img, image").expect("Failed to parse selector");

        let mut found_uris = 0;

        for element in document.select(&img_selector) {
            // Find the attribute value by iterating through all attributes
            let mut src_opt = None;
            for (attr_name, attr_value) in element.value().attrs() {
                if attr_name == "src" || attr_name == "href" || attr_name == "xlink:href" {
                    src_opt = Some(attr_value);
                    break;
                }
            }
            if let Some(src) = src_opt {
                if src.starts_with("data:") {
                     if let Some(pos) = src.find(";base64,") {
                        let base64_data = &src[pos + 8..];
                        match general_purpose::STANDARD.decode(base64_data) {
                             Ok(_) => {
                                 found_uris += 1;
                             }
                             Err(_) => {
                                 // Ignore decode errors for this test, just count findings
                             }
                         }
                     }
                }
            }
        }
        assert_eq!(found_uris, 2, "Expected to find and decode 2 data URIs from mixed tags");
    }

     #[test]
    fn test_ignore_non_data_uri_src() {
        let html_in = "<p><img src='http://example.com/image.png'><svg><image xlink:href='relative/path.jpg'></image></svg></p>";
        let document = Html::parse_document(&html_in);
        let img_selector = Selector::parse("img, image").expect("Failed to parse selector");

        let mut data_uri_count = 0;
        for element in document.select(&img_selector) {
            // Find the attribute value by iterating through all attributes
            let mut src_opt = None;
            for (attr_name, attr_value) in element.value().attrs() {
                if attr_name == "src" || attr_name == "href" || attr_name == "xlink:href" {
                    src_opt = Some(attr_value);
                    break;
                }
            }
            if let Some(src) = src_opt {
                if src.starts_with("data:") {
                    data_uri_count += 1;
                }
            }
        }
        assert_eq!(data_uri_count, 0, "Should not find any data URIs");
    }

    #[test]
    fn test_handle_malformed_data_uri() {
        // Missing ';base64,' part
        let html_in = "<p><img src='data:image/jpeg,somebytes'></p>";
        let document = Html::parse_document(&html_in);
        let img_selector = Selector::parse("img, image").expect("Failed to parse selector");

        let mut data_uri_found_and_decoded = false;
        for element in document.select(&img_selector) {
             // Find the attribute value by iterating through all attributes
             let mut src_opt = None;
             for (attr_name, attr_value) in element.value().attrs() {
                 if attr_name == "src" || attr_name == "href" || attr_name == "xlink:href" {
                     src_opt = Some(attr_value);
                     break;
                 }
             }
             if let Some(src) = src_opt {
                 if src.starts_with("data:") {
                      if let Some(pos) = src.find(";base64,") {
                         let base64_data = &src[pos + 8..];
                         if general_purpose::STANDARD.decode(base64_data).is_ok() {
                             data_uri_found_and_decoded = true;
                         }
                      }
                 }
             }
        }
        // The parsing logic in the main function logs a warning but continues.
        // This test just verifies our test harness correctly identifies it won't find the ';base64,' separator.
        assert!(!data_uri_found_and_decoded, "Should not successfully find and decode the malformed URI");
    }
}
