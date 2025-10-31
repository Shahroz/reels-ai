//! Configures the routes for asset management under `/api/assets`.
//!
//! Registers HTTP handlers for asset operations in the Actix web application.

pub fn configure_assets_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("")
            .service(crate::routes::assets::list_assets::list_assets)
            .service(crate::routes::assets::get_asset_by_id::get_asset_by_id)
            .service(crate::routes::assets::create_asset::create_asset)
            // Register specific routes BEFORE generic ones to avoid path conflicts
            .service(crate::routes::assets::attach_assets::attach_assets)
            .service(crate::routes::assets::detach_assets::detach_assets)
            .service(crate::routes::assets::get_upload_url::get_upload_url)
            .service(crate::routes::assets::confirm_upload::confirm_upload)
            .service(crate::routes::assets::quick_enhance_image::quick_enhance_image)
            .service(crate::routes::assets::studio_graph::get_lineage_graph)
            // Generic routes with path parameters come after specific routes
            .service(crate::routes::assets::delete_asset::delete_asset)
            .service(crate::routes::assets::patch_asset::patch_asset)
            // Asset enhancement endpoint - handles credits in route handler to support org context
            .service(crate::routes::assets::enhance_asset::enhance_asset)
    );
}
