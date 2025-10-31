//! Processes HTML to find large SVG elements, upload them to GCS, and replace them with `<img>` tags.
//!
//! This function parses an HTML string, identifies `<svg>` elements containing `<path>` elements
//! with `d` attributes exceeding a specified length threshold. These large SVGs are serialized,
//! uploaded to Google Cloud Storage (GCS), and replaced in the HTML with an `<img>` tag
//! whose `src` points to the GCS URL. This helps reduce the size of the initial HTML document.
//! It uses the `kuchikiki` crate for HTML parsing and manipulation.
