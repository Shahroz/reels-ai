//! Processes a sitemap, fetches specified URLs, cleans content, and saves to disk.
//!
//! Handles fetching/parsing sitemaps, filtering URLs, parallel downloads via Zyte,
//! basic HTML cleaning, and progress reporting. Adheres to one-item-per-file.
//! Uses fully qualified paths. Intended for integration into the zyte_client crate.

//! Revision History
//! - 2025-04-24T07:01:59Z @AI: Initial implementation of main parallel processing logic.

pub async fn process_sitemap(
    sitemap_source: String,
    filter_query: Option<String>,
    output_dir: String,
    n_jobs: usize,
    min_last_mod: Option<chrono::DateTime<chrono::Utc>>, // Added: Minimum last modification date filter
) -> std::result::Result<(), crate::zyte::error::ZyteError> {
    use crate::zyte::fetch_and_parse_sitemap::fetch_and_parse_sitemap;
    use crate::zyte::filter_sitemap_urls::filter_sitemap_urls;
    use crate::zyte::fetch_clean_and_save_page::fetch_clean_and_save_page;
    use crate::zyte::client::ZyteClient;
    use indicatif::{ProgressBar, ProgressStyle};
    use futures::stream::{self, StreamExt};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Step 1: Fetch and parse the sitemap
    let urls = fetch_and_parse_sitemap(&sitemap_source, min_last_mod)
        .await
        .map_err(|e| {
            println!("Error fetching and parsing sitemap: {e:?}");
            e
        })?;

    // Step 2: Filter URLs if a filter query is provided
    let filtered_urls = filter_sitemap_urls(&urls, filter_query.as_deref());

    // Step 3: Ensure the output directory exists
    tokio::fs::create_dir_all(&output_dir).await.map_err(|e| {
        println!("Error ensuring output directory exists: {e:?}");
        crate::zyte::error::ZyteError::IoError(e.to_string())
    })?;

    // Step 4: Initialize the ZyteClient
    let client = ZyteClient::new().map_err(|e| {
        println!("Error initializing ZyteClient: {e:?}");
        e
    })?;

    // Step 5: Setup the progress bar
    let pb = ProgressBar::new(filtered_urls.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
        .progress_chars("#>-")
    );
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    // Step 6-8: Create a stream, process URLs in parallel, collect results
    let output_dir_clone = Arc::new(output_dir);
    stream::iter(filtered_urls)
        .map(|url| {
            let client = client.clone();
            let output_dir = Arc::clone(&output_dir_clone);
            let pb = pb.clone();
            let success_count = Arc::clone(&success_count);
            let failure_count = Arc::clone(&failure_count);

            tokio::spawn(async move {
                let result = fetch_clean_and_save_page(&client, &url, &output_dir).await;
                pb.inc(1); // Increment progress bar after each task completes
                match result {
                    Ok(_) => {
                        success_count.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        println!("Error processing URL {url}: {e:?}");
                        failure_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            })
        })
        .buffer_unordered(n_jobs)
        .for_each(|_| async {})
        .await;

    // Step 10: Finish progress bar
    pb.finish_with_message("Processing completed.");

    // Step 11: Log results and return
    let successes = success_count.load(Ordering::Relaxed);
    let failures = failure_count.load(Ordering::Relaxed);
    println!("Processing finished: {successes} successes, {failures} failures");

    if failures > 0 {
        println!("Some tasks failed. Review the logs for more details.");
    }

    Ok(())
}
