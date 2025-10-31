//! Provides the CORS middleware configuration.
//!
//! This middleware handles Cross-Origin Resource Sharing settings.
//! It is currently configured to be permissive, allowing requests from any origin.
//! This simplifies development but should be reviewed for production environments.
//! Ensures basic CORS headers are set without strict restrictions.

use actix_cors::Cors;
// Note: zino_core imports removed as config reading is bypassed.
// Note: actix_web imports like Method, HeaderName removed as they are no longer used.

/// CORS middleware - currently forced to permissive.
pub(crate) fn cors_middleware() -> Cors {
    // Original logic reading from config is bypassed.
    // Always return a permissive configuration.
    Cors::permissive()
}
