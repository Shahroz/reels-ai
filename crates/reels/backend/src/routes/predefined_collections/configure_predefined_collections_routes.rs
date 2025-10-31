//! Configures routes for managing predefined collections.
//!
//! This function groups all the CRUD operation handlers for predefined
//! collections under a common service scope.
//! Adheres to 'one item per file' and FQN guidelines.

pub fn configure_predefined_collections_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("") // Base path for predefined collections is /api/predefined-collections
            .service(crate::routes::predefined_collections::create_predefined_collection::create_predefined_collection)
            .service(crate::routes::predefined_collections::list_predefined_collections::list_predefined_collections)
            .service(crate::routes::predefined_collections::get_predefined_collection::get_predefined_collection)
            .service(crate::routes::predefined_collections::update_predefined_collection::update_predefined_collection)
            .service(crate::routes::predefined_collections::delete_predefined_collection::delete_predefined_collection)
    );
} 