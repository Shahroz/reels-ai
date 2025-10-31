pub mod actix_server;
pub mod helpers;

// Re-export the clean, parallel-safe API
pub use helpers::{
    // Primary user management API (recommended)
    TestUser,

    // JWT helpers
    generate_test_jwt,
    Claims,
    create_jwt,

    // Document and API key types
    CreateDocumentRequest,
    Document,
};
