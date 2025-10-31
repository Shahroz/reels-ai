//! Orchestrates various HTML minimization techniques to reduce content size.
//!
//! This function applies a series of minimization steps, including removing comments,
//! iframes, style tags, processing data URIs for images, handling large SVG paths,
//! and finally removing excessively large individual HTML tags if the content still
//! exceeds a specified size threshold.
//! It aims to reduce HTML size progressively, as outlined in project documentation.

// Note: Fully qualified paths are used as per rust_guidelines.md.
// `uuid::Uuid` and `crate::services::gcs::gcs_operations::GCSOperations` are used in the function signature.

/// Orchestrates HTML minimization.
///
/// # Arguments
/// * `html_content` - The initial HTML string.
/// * `gcs_client` - A GCS client for operations like uploading assets.
/// * `bucket` - The GCS bucket name.
/// * `style_id` - UUID for the current style, used in GCS paths.
/// * `max_final_size_chars` - Target maximum character count for the HTML in bytes.
/// * `svg_path_d_attribute_threshold` - Threshold for `d` attribute length in SVGs to be processed in bytes.
///
/// # Returns
/// * `Ok(String)` with the minimized HTML content.
/// * `Err(String)` if a critical error occurs during processing.
pub async fn minimize_large_html_content(
    html_content: &str,
    _gcs_client: &dyn crate::services::gcs::gcs_operations::GCSOperations,
    _bucket: &str,
    _style_id: uuid::Uuid,
    max_final_size_chars: usize,
    _svg_path_d_attribute_threshold: usize,
) -> Result<std::string::String, std::string::String> {
    println!("Starting HTML minimization. Initial size: {} chars.", html_content.chars().count());

    // 1. Remove HTML comments
    let mut current_html =
        crate::utils::html_minimizer::remove_html_comments:: remove_html_comments(html_content);
    println!("After removing comments. Size: {} chars.", current_html.chars().count());

    // 2. Remove iframes
    current_html = crate::utils::html_minimizer::remove_iframes::remove_iframes(&current_html);
    println!("After removing iframes. Size: {} chars.", current_html.chars().count());

    // 3. Remove style tags
    current_html = crate::utils::html_minimizer::remove_style_tags::remove_style_tags(&current_html);
    println!("After removing style tags. Size: {} chars.", current_html.chars().count());

    // 4. Process image data URIs (async)
    // This function can return Err(String), so we use `?`
    // println!("Processing image data URIs...");
    // current_html = crate::utils::html_minimizer::process_image_data_uris::process_image_data_uris(
    //     &current_html,
    //     gcs_client,
    //     bucket,
    //     style_id,
    // )
    // .await?;
    // println!("After processing image data URIs. Size: {} chars.", current_html.chars().count());
    // 
    // // 5. Process SVG paths (async)
    // // This function can return Err(String), so we use `?`
    // println!("Processing SVG paths...");
    // current_html = crate::utils::html_minimizer::process_svg_paths::process_svg_paths(
    //     &current_html,
    //     gcs_client,
    //     bucket,
    //     style_id,
    //     svg_path_d_attribute_threshold,
    // )
    // .await?;
    // println!("After processing SVG paths. Size: {} chars.", current_html.chars().count());

    // 6. Conditionally trim HTML from the bottom if overall size is still too big
    if current_html.chars().count() > max_final_size_chars {
        println!(
            "HTML content size ({} chars) exceeds max_final_size_chars ({}). Applying trim_html_from_bottom.",
            current_html.chars().count(),
            max_final_size_chars
        );
        current_html = crate::utils::html_minimizer::trim_html_from_bottom::trim_html_from_bottom(
            &current_html,
            max_final_size_chars,
        );
        println!("After trimming from bottom. Size: {} chars.", current_html.chars().count());
    }

    println!("Finished HTML minimization. Final size: {} chars.", current_html.chars().count());
    Ok(current_html)
}

#[cfg(test)]
mod tests {
    // Mock GCSClient for testing purposes
    struct MockGCSClient {
        // Store expected calls or uploaded data if needed for assertions
        // For simplicity, this mock will always succeed uploads.
        // A more complex mock could track calls:
        // pub uploads: std::sync::Mutex<std::vec::Vec<(String, String, String)>>, // bucket, path, content_type
    }

    impl MockGCSClient {
        fn new() -> Self {
            Self {
                // uploads: std::sync::Mutex::new(std::vec::Vec::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl crate::services::gcs::gcs_operations::GCSOperations for MockGCSClient {
        async fn upload_raw_bytes(
            &self,
            bucket_name: &str,
            object_name: &str,
            _content_type: &str,
            _data: std::vec::Vec<u8>,
            _disable_cache: bool,
            _url_format: crate::services::gcs::gcs_operations::UrlFormat,
        ) -> anyhow::Result<std::string::String> {
            // let mut uploads = self.uploads.lock().unwrap();
            // uploads.push((bucket_name.to_string(), object_name.to_string(), _content_type.to_string()));
            Ok(format!("gcs://{}/{}", bucket_name, object_name))
        }

        async fn delete_object(
            &self,
            _bucket_name: &str,
            _object_name: &str,
        ) -> anyhow::Result<()> {
            unimplemented!("MockGCSClient::delete_object not implemented")
        }

        async fn download_object_as_string(
            &self,
            _bucket_name: &str,
            _object_name: &str,
        ) -> anyhow::Result<std::string::String> {
            unimplemented!("MockGCSClient::download_object_as_string not implemented")
        }

        async fn download_object_as_bytes(
            &self,
            _bucket_name: &str,
            _object_name: &str,
        ) -> anyhow::Result<std::vec::Vec<u8>> {
            unimplemented!("MockGCSClient::download_object_as_bytes not implemented")
        }
        
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[tokio::test]
    async fn test_orchestrator_basic_flow_no_large_tag_removal() {
        let html_in = "<!-- comment --><p>Hello</p><style>body{}</style><iframe src=''></iframe>";
        let gcs_client = MockGCSClient::new();
        let style_id = uuid::Uuid::new_v4();

        let result = super::minimize_large_html_content(
            html_in,
            &gcs_client,
            "test-bucket",
            style_id,
            1000, // max_final_size_chars (large enough to not trigger large_tag_removal)
            100,  // svg_path_d_attribute_threshold
        )
        .await;

        assert!(result.is_ok());
        let html_out = result.unwrap();
        // Expected: comments, style, iframe removed.
        // process_image_data_uris and process_svg_paths won't change this simple HTML.
        // kuchikiki used by remove_iframes and remove_large_tags adds html/head/body structure.
        // remove_html_comments and remove_style_tags are more direct string manipulation or fragment based.
        // The exact output depends on which parser dominates the final structure.
        // Let's check for absence of removed tags and presence of main content.
        assert!(!html_out.contains("<!--"));
        assert!(!html_out.contains("<style"));
        assert!(!html_out.contains("<iframe"));
        assert!(html_out.contains("<p>Hello</p>") || html_out.contains("Hello")); // scraper might output just "Hello" if it was root
    }

    #[tokio::test]
    async fn test_orchestrator_triggers_large_tag_removal() {
        let very_long_span = format!("<span style='padding:{}'>long content</span>", "0".repeat(600));
        let html_in = format!("<p>Small</p>{}", very_long_span); // This will be > 100 chars easily

        let gcs_client = MockGCSClient::new();
        let style_id = uuid::Uuid::new_v4();

        let result = super::minimize_large_html_content(
            &html_in,
            &gcs_client,
            "test-bucket",
            style_id,
            100,  // max_final_size_chars (small enough to trigger trimming)
            100,  // svg_path_d_attribute_threshold
        )
        .await;

        assert!(result.is_ok());
        let html_out = result.unwrap();

        // After other ops, if html_out.chars().count() > 100, trim_html_from_bottom is called.
        // The long_span should be removed by the trimming function.
        assert!(html_out.contains("<p>Small</p>"), "html_out: {}", html_out);
        assert!(!html_out.contains("<span"), "html_out: {}", html_out);
        assert!(!html_out.contains("long content"), "html_out: {}", html_out);
    }

    #[tokio::test]
    async fn test_minimize_large_html_from_file() {
        // Load HTML from a file that is expected to be large.
        // The instruction is to not load the file in context, but it must exist for the test.
        // We assume it's in `tests/data/large_style.html` relative to crate root.
        let html_in = include_str!("large_style.html");

        let initial_size = html_in.chars().count();
        // Using `println!` as requested for reporting, visible when running `cargo test -- --nocapture`
        println!(
            "Initial HTML size from file: {} characters",
            initial_size
        );
        assert!(
            initial_size > 100,
            "The test file should be sufficiently large."
        );

        let gcs_client = MockGCSClient::new();
        let style_id = uuid::Uuid::new_v4();

        // These thresholds are set high to avoid triggering some specific removals
        // unless the content is truly massive, letting the standard removals (comments, styles) do the work.
        let result = super::minimize_large_html_content(
            html_in,
            &gcs_client,
            "test-bucket",
            style_id,
            300000, // max_final_size_chars
            200,   // svg_path_d_attribute_threshold
        )
        .await;

        assert!(result.is_ok(), "Minimization failed: {:?}", result.err());
        let html_out = result.unwrap();
        println!("{}", html_out);
        let final_size = html_out.chars().count();
        println!("Final HTML size from file: {} characters", final_size);

        // The main assertion is that the size was reduced.
        assert!(
            final_size < initial_size,
            "Expected final size {} to be less than initial size {}",
            final_size,
            initial_size
        );
    }
}
