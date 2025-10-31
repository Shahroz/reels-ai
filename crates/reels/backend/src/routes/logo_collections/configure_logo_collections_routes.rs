//! Configures all logo collections endpoints.
//!
//! Registers handlers under the /logo-collections scope.

pub fn configure_logo_collections_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(crate::routes::logo_collections::list_logo_collections::list_logo_collections)
        .service(crate::routes::logo_collections::get_logo_collection_by_id::get_logo_collection_by_id)
        .service(crate::routes::logo_collections::create_logo_collection::create_logo_collection)
        .service(crate::routes::logo_collections::update_logo_collection::update_logo_collection)
        .service(crate::routes::logo_collections::delete_logo_collection::delete_logo_collection)
        .service(crate::routes::logo_collections::add_asset_to_logo_collection::add_asset_to_logo_collection)
        .service(crate::routes::logo_collections::remove_asset_from_logo_collection::remove_asset_from_logo_collection);
}
