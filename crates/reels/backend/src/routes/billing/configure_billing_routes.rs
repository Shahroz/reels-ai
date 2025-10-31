//! Configures all Billing-related routes.
//!
//! Mounted under /api/billing with JWT authentication and BillingAccessGuard.

use actix_web::web;

use super::{checkout, portal, products, status, plans};

/// Sets up endpoints for Billing operations within the /api/billing scope.
pub fn configure_billing_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(checkout::create_checkout_session)
        .service(checkout::update_checkout_session)
        .service(portal::create_customer_portal_session)
        .service(products::get_products)
        .service(plans::get_plans)
        .service(status::get_billing_status_handler)
        .service(status::get_payment_status_handler);
}
