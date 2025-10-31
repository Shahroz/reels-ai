//! Asset and collection ownership validation functions.
//!
//! This module provides reusable validation functions for verifying user ownership
//! of assets and collections across route handlers. Each validation function returns
//! standardized HTTP responses that can be used directly in API endpoints,
//! eliminating code duplication while maintaining transaction compatibility.
//! Functions are organized following the one-item-per-file principle.

pub mod validate_asset_ownership;
pub mod validate_collection_ownership;
pub mod validate_bulk_asset_ownership;