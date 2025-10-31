//! Assets database queries module.
//!
//! This module contains all database query functions related to assets,
//! including CRUD operations and specialized queries.

pub mod create_asset;
pub mod get_asset_by_id;
pub mod get_asset_by_id_with_collection;
pub mod inherit_shares_from_asset;
pub mod list_assets_with_collection;
pub mod update_asset_collection;
pub mod count_assets;
pub mod delete_asset;

pub use create_asset::create_asset;
pub use get_asset_by_id::get_asset_by_id;
pub use get_asset_by_id_with_collection::get_asset_by_id_with_collection;
pub use list_assets_with_collection::list_assets_with_collection;
pub use update_asset_collection::update_asset_collection;
pub use count_assets::count_assets;
pub use delete_asset::delete_asset;

pub mod lineage;