//! Configures all Creative-related routes.
//!
//! Mounted under /api/creatives with JWT authentication.

use actix_web::web; // Keep this import
use crate::middleware::credits_guard::require_generate_creative;
use super::{delete_creative, get_creative_by_id, list_creatives, generate_creative, edit_creative, publish_draft, get_creative_content, discard_draft, text_rewrite, generate_creative_from_bundle_handler, update_creative_name, duplicate_creative};

/// Sets up endpoints for Creative operations within the /api/creatives scope.
/// Note: list_webflow_creatives is now registered directly under /api.
pub fn configure_creatives_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_creatives::list_creatives)
       .service(get_creative_by_id::get_creative_by_id)
       .service(delete_creative::delete_creative)
       .service(edit_creative::edit_creative)
       .service(publish_draft::publish_draft)
       .service(get_creative_content::get_creative_content)
       .service(discard_draft::discard_draft)
       .service(text_rewrite::text_rewrite_handler)
       .service(update_creative_name::update_creative_name) // Added update creative name route
       .service(duplicate_creative::duplicate_creative)
       // Creative generation endpoints require credits
       .service(
           web::scope("")
               .wrap(require_generate_creative())
               .service(generate_creative::generate_creative)
               .service(generate_creative_from_bundle_handler::generate_creative_from_bundle)
       );
}

// Make components visible for openapi documentation generation
pub mod openapi {
}
