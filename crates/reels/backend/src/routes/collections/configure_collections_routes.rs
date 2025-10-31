//! Configures all collections endpoints.
//!
//! Registers handlers under the /collections scope.

pub fn configure_collections_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(crate::routes::collections::list_collections::list_collections)
        .service(crate::routes::collections::get_collection_by_id::get_collection_by_id)
        .service(crate::routes::collections::get_collection_with_assets::get_collection_with_assets)
        .service(crate::routes::collections::create_imageboard_board::create_imageboard_board)
        .service(crate::routes::collections::create_collection::create_collection)
        .service(crate::routes::collections::update_collection::update_collection)
        .service(crate::routes::collections::delete_collection::delete_collection);
}
