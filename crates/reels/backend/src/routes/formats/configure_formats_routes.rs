//! Configures all Creative Format related routes.
//!
//! Mounts handlers under `/api/formats`. Public formats are under `/api/formats/creative-formats`,
//! while user-specific custom formats are under `/api/formats/custom-creative-formats`
//! and require JWT authentication.

pub fn configure_formats_routes(cfg: &mut actix_web::web::ServiceConfig) {
    // Custom creative formats for authenticated users: under /api/formats/custom-creative-formats
    cfg.service(
        actix_web::web::scope("/custom-creative-formats")
            .service(crate::routes::formats::list_custom_creative_formats::list_custom_creative_formats)
            .service(crate::routes::formats::create_custom_creative_format::create_custom_creative_format)
            .service(crate::routes::formats::copy_custom_creative_format::copy_custom_creative_format)
            .service(crate::routes::formats::update_custom_creative_format::update_custom_creative_format)
            .service(crate::routes::formats::delete_custom_creative_format::delete_custom_creative_format)
            // .service(crate::routes::formats::transfer_custom_creative_format_from_organization::transfer_custom_creative_format_from_organization_handler),
    );
}
