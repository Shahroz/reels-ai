//! Configures imageboard-specific routes for the Actix web application.
//!
//! This function defines the `/imageboard` scope and registers
//! all sub-routes related to imageboard webhook functionalities.

use actix_web::web;
use crate::middleware::imageboard_webhook_guard::ImageboardWebhookGuard;

/// Configures imageboard-specific routes.
///
/// Adds the /imageboard scope and its sub-routes to the Actix web application.
/// This function registers handlers for:
/// - Getting organization balance
/// - Bulk deducting credit transactions
///
/// All routes are protected by ImageboardWebhookGuard middleware which:
/// - Extracts collection_id from path parameter
/// - Validates collection (board) existence
/// - Validates organization existence (via collection's organization_id)
///
/// # Arguments
///
/// * `cfg` - A mutable reference to `actix_web::web::ServiceConfig`.
pub fn configure_imageboard_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("/imageboard")
            .wrap(ImageboardWebhookGuard)
            .service(
                web::scope("/webhook")
                    .service(crate::routes::imageboard::get_organization_balance::get_organization_balance)
                    .service(crate::routes::imageboard::bulk_deduct_credit_transaction::bulk_deduct_credit_transaction)
            )
    );
}

