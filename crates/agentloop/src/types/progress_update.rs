//! Defines the structure for progress updates and the callback type.

/// Represents a single progress update from the research loop.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressUpdate {
    pub sender: String, // e.g., "user", "agent", "tool"
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type alias for the async progress callback function.
pub type ProgressCallback =
    Box<dyn FnMut(ProgressUpdate) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;