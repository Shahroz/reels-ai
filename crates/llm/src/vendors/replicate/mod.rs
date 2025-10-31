//! Module aggregating Replicate API functionalities following one-item-per-file.
//!
//! This module organizes the components related to interacting with the Replicate API.
//! It declares sub-modules for the model definition and the API calling function.
//! Re-exports the primary public items for convenient use by parent modules.

//! Revision History
//! - 2025-04-15T15:27:38Z @AI: Initial creation during refactor.

pub mod call_replicate_api;
pub mod replicate_model;

// Re-export public items for easier access.
pub use self::call_replicate_api::call_replicate_api;
pub use self::replicate_model::ReplicateModel;
