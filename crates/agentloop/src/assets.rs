//! Defines the struct for embedding static frontend assets.
//!
//! Uses `rust-embed` to embed the contents of the `frontend/dist` directory
//! into the application binary. This allows the frontend to be served directly
//! by the Actix-web server without needing separate file hosting.
//! Adheres strictly to the project's Rust coding standards.

/// Embeds the frontend assets from the `frontend/dist` directory.
///
/// The `folder` attribute points to the relative path from the crate root.
/// The `prefix` attribute removes the specified prefix from the embedded file paths,
/// making them accessible relative to the server root (e.g., `dist/index.html` becomes `index.html`).
#[derive(rust_embed::RustEmbed)]
#[folder = "frontend/dist/"]
pub struct Assets;

// No tests needed for this simple embedding definition file.
