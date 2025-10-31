//! Configures routes for managing items within user DB collections.
//!
//! This function groups all the CRUD operation handlers for items
//! under a common service scope, typically nested under `/{collection_id}/items`.
//! Adheres to 'one item per file' and FQN guidelines.

pub fn configure_user_db_collection_items_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/items") // Base path for this module is /{collection_id}/items
            .service(crate::routes::user_db_collections::items::create_user_db_collection_item::create_user_db_collection_item)
            .service(crate::routes::user_db_collections::items::list_user_db_collection_items::list_user_db_collection_items)
            .service(crate::routes::user_db_collections::items::get_user_db_collection_item::get_user_db_collection_item)
            .service(crate::routes::user_db_collections::items::update_user_db_collection_item::update_user_db_collection_item)
            .service(crate::routes::user_db_collections::items::delete_user_db_collection_item::delete_user_db_collection_item)
            .service(crate::routes::user_db_collections::items::query_user_db_collection_items::query_user_db_collection_items),
    );
}
