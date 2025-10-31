//! Error types for the Zyte client.
//!
//! Defines the main error enum used throughout the client library.
//! Reflects potential issues like environment variable errors, network problems,
//! JSON parsing failures, and API authentication issues.

//! Revision History
//! - 2025-04-15T15:06:21Z @AI: Created file based on one-item-per-file guideline.

#[derive(Debug, ::thiserror::Error)]
pub enum ZyteError {
    #[error("Environment variable error: {0}")]
    EnvVarError(String),
    
    #[error("Reqwest error: {0}")]
    ReqwestError(String),
    
    #[error("JSON error: {0}")]
    JsonError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
        #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Sitemap parsing error: {0}")]
    SitemapParseError(String),
    #[error("PDF decoding error: {0}")]
    PdfDecodeError(String),
    #[error("PDF parsing error: {0}")]
    PdfParseError(String),
    #[error("HTML cleaning error: {0}")]
    HtmlCleanError(String),

    #[error("Progress bar template error: {0}")]
    TemplateError(String),
    #[error("Placeholder error")]
    Placeholder,
    #[error("No browser html error")]
    NoBrowserHtml(String),
    #[error("Content extraction error")]
    ContentExtractionError,
}

impl From<::indicatif::style::TemplateError> for ZyteError {
    fn from(err: ::indicatif::style::TemplateError) -> Self {
        ZyteError::TemplateError(err.to_string())
    }
}
