//! Standalone CLI application for processing sitemaps or single pages using the zyte_client library.
//!
//! Parses command-line arguments using clap and orchestrates the processing workflow.
//! Calls the core logic from the `zyte_client` library modules.
//! Uses a Tokio runtime for asynchronous operations.

//! Revision History
//! - 2025-04-24T08:27:21Z @AI: Extract Serper logic into serper::client module.
//! - 2025-04-24T07:15:48Z @AI: Add SinglePage subcommand, refactor main for subcommands.
//! - 2025-04-24T07:05:53Z @AI: Update CLI args (jobs default), refine preamble per instruction.
//! - 2025-04-24T06:40:15Z @AI: Implement CLI argument parsing and main workflow.

use api_tools::zyte::sitemap_processor;
use api_tools::serper::client;
use api_tools::zyte::fetch_and_save_text;

/// Main CLI application structure.
#[derive(clap::Parser, Debug)]
#[command(author, version, about = "CLI tool for interacting with multiple APIs: Zyte sitemap, Zyte single-page, and Serper search")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Enum defining the available subcommands.
#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Process URLs from a sitemap (local file or URL).
    Sitemap {
        /// Source of the sitemap (local file path or URL).
        #[arg(short = 's', long)]
        sitemap_source: String,

        /// Path to the directory where output files will be saved.
        #[arg(short = 'o', long)]
        output_dir: String,

        /// Optional query string to filter URLs from the sitemap. Only URLs containing this string will be processed.
        #[arg(long)]
        filter: Option<String>,

        /// Optional number of parallel jobs (download threads) to use.
        #[arg(short = 'j', long, default_value_t = 10)]
        jobs: usize,
    },
    /// Fetch and save the text content of a single URL (Zyte API).
    Browse {
        /// The URL to fetch.
        #[arg(long)]
        url: String,

        /// The file path to save the extracted text to.
        #[arg(long)]
        output: String,
    },
    /// Search using the Serper API.
    Search {
        /// Query string for the Serper search.
        #[arg(short = 'q', long)]
        query: String,
    },
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let cli = <Cli as clap::Parser>::parse(); // Use FQP for trait method

    // Execute the appropriate command
    match cli.command {
        Commands::Sitemap { sitemap_source, output_dir, filter, jobs } => {
            std::println!(
                "Starting sitemap processing: Source='{}', Output='{}', Filter='{}', Jobs={}",
                sitemap_source,
                output_dir,
                filter.as_deref().unwrap_or("None"), // Nicer printing for Option
                jobs
            );

            // Call the core sitemap processing function
            sitemap_processor::process_sitemap(
                sitemap_source,
                filter, // Pass the Option<String> directly
                output_dir,
                jobs,
                None, // No minimum last modification date filter
            )
            .await?; // Await and propagate errors using ?

            std::println!("Sitemap processing completed successfully.");
        }
        Commands::Browse { url, output } => {
            std::println!(
                "Starting single page processing: URL='{}', Output='{}'",
                url,
                output
            );

            // Call the single page fetching function
            fetch_and_save_text::fetch_and_save_text(&url, &output).await?; // Await and propagate errors

            std::println!("Single page processing completed successfully.");
        }
        Commands::Search { query } => {
            std::println!("Starting Serper search: query='{}'", query);
            // Call the dedicated Serper client function
            let search_result = client::search(&query).await?;
            std::println!("{}", search_result); // Print the result from the search function
        }
    }

    Ok(()) // Return Ok if execution reaches here without errors
}

// No tests typically needed for the main CLI entry point itself.
// Integration tests involving running the compiled binary would cover this flow.

