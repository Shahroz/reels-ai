//! Module for handling user favorites.

pub mod configure_user_favorites_routes;
pub mod create_favorite;
pub mod list_favorites;
pub mod delete_favorite;
pub mod toggle_favorite;
pub mod add_favorite_prompt;
pub mod list_favorite_prompts;
pub mod remove_favorite_prompt;

// Request/response structs
pub mod create_favorite_request;
pub mod entity_relations;
pub mod favorite_with_entity; 