//! Configures routes for managing user DB collections.
//!
//! This function groups all the CRUD operation handlers for user-defined
//! database collections under a common service scope.
//! Adheres to 'one item per file' and FQN guidelines.

pub fn configure_user_db_collections_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("") // Base path for collections is /api/user-db-collections
            .service(crate::routes::user_db_collections::create_user_db_collection::create_user_db_collection)
            .service(crate::routes::user_db_collections::list_user_db_collections::list_user_db_collections)
            .service(crate::routes::user_db_collections::get_user_db_collection::get_user_db_collection)
            .service(crate::routes::user_db_collections::update_user_db_collection::update_user_db_collection)
            .service(crate::routes::user_db_collections::delete_user_db_collection::delete_user_db_collection)
            .service(crate::routes::user_db_collections::update_user_db_collection_schema::update_user_db_collection_schema)
            .service(crate::routes::user_db_collections::copy_predefined_collection::copy_predefined_collection)
            .service(crate::routes::user_db_collections::get_or_create_user_collection_by_predefined::get_or_create_user_collection_by_predefined)
            .service(crate::routes::user_db_collections::get_or_create_user_collection_by_predefined_name::get_or_create_user_collection_by_predefined_name)
            // Nested scope for items, configured by its own function
            .service(
                actix_web::web::scope("/{collection_id}")
                    .configure(crate::routes::user_db_collections::items::configure_user_db_collection_items_routes::configure_user_db_collection_items_routes)
            ),
    );
}