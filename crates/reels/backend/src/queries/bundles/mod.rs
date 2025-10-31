//! Declares modules for bundle-related database query functions.
//!
//! This module organizes individual query functions, each in its own file,
//! adhering to the project's coding standards for modularity.
//! Functions here interact with the `bundles` table in the database.

// Revision History
// - 2025-05-29T18:15:36Z @AI: Initial creation of this module file.

pub mod create_bundle;
pub mod delete_bundle;
pub mod fetch_expanded_bundles_by_ids;
pub mod find_bundle_by_id;
 pub mod list_expanded_bundles_for_user;
 pub mod count_bundles_for_user;
pub mod update_bundle;

pub use fetch_expanded_bundles_by_ids::fetch_expanded_bundles_by_ids;
 pub use list_expanded_bundles_for_user::list_expanded_bundles_for_user;
 pub use count_bundles_for_user::count_bundles_for_user;
