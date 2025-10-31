//! Module for handling object shares.

pub mod configure_shares_routes;
pub mod create_share;
pub mod list_shares;
pub mod delete_share;

// Potentially request/response structs if not co-located with handlers
pub mod create_share_request;
// list_shares might define its own request/response structs internally or in a separate file. 