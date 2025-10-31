# API Clients Crate

This crate provides Rust clients for interacting with various web APIs.

## Features

- **Zyte API Client (`zyte` module):**
    - Fetch raw HTML, browser-rendered HTML, and screenshots.
    - Perform browser actions (e.g., scrolling).
    - Process sitemaps (fetch, parse, filter URLs).
    - Fetch and clean text content from multiple URLs in parallel.
    - Configurable via environment variables (`ZYTE_API_KEY`).
- **Serper API Client (`serper` module):**
    - Perform Google searches via the Serper API.
    - Configurable via environment variables (`SERPER_API_KEY`).
- **Command-Line Tool (`tool` binary):**
    - Provides CLI access to sitemap processing, single-page fetching (Zyte), and Serper search functionality.

## Structure

The crate follows a "one item per file" convention within its modules and uses fully qualified paths internally.

- `src/zyte/`: Contains all components related to the Zyte API.
- `src/serper/`: Contains all components related to the Serper API.
- `src/lib.rs`: Declares the main library modules.
- `src/main.rs`: Implements the command-line tool binary.

## Usage (Library)

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
api_clients = { path = "path/to/crates/api_clients" } # Adjust path as needed
# Or specify version if published
```

Example usage:

```rust
// Requires ZYTE_API_KEY and SERPER_API_KEY in environment or .env file
use api_clients::zyte;
use api_clients::serper;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Zyte Example: Fetch single page text ---
    let zyte_url = "http://books.toscrape.com/";
    let output_path = "books_toscrape.txt";
    match zyte::fetch_and_save_text::fetch_and_save_text(zyte_url, output_path).await {
        Ok(_) => println!("Successfully fetched text from {} to {}", zyte_url, output_path),
        Err(e) => eprintln!("Zyte fetch failed: {}", e),
    }

    // --- Serper Example: Perform search ---
    let search_query = "Rust programming language";
    match serper::client::search(search_query).await {
        Ok(results) => println!("Serper search results for '{}':\n{}", search_query, results),
        Err(e) => eprintln!("Serper search failed: {}", e),
    }

    Ok(())
}
```

## Usage (CLI Tool)

Build the tool: `cargo build --bin tool`

Run with `--help` to see options: `./target/debug/tool --help`

Examples:

```bash
# Process a sitemap using Zyte API (requires ZYTE_API_KEY env var)
./target/debug/tool sitemap -s https://example.com/sitemap.xml -o ./output_dir -j 5 --filter "product"

# Fetch a single page using Zyte API (requires ZYTE_API_KEY env var)
./target/debug/tool single-page --url https://example.com/page --output ./page.txt

# Perform a Serper search (requires SERPER_API_KEY env var)
./target/debug/tool serper -q "latest Rust features"
```

## Configuration

The clients require API keys set as environment variables:

- `ZYTE_API_KEY`: Your API key for the Zyte API.
- `SERPER_API_KEY`: Your API key for the Serper API (serper.dev).

You can place these in a `.env` file in the crate root directory. The library uses `dotenvy` to load them automatically.
