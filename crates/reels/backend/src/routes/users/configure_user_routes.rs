// crates/narrativ/backend/src/routes/users/configure_user_routes.rs

use actix_web::web;
use crate::routes::users::user_credit_handlers::{claim_daily_credits_handler, get_user_credits_handler};
use crate::routes::users::delete_user_handler::delete_current_user_handler;
use crate::routes::users::get_current_user_handler::get_current_user_handler;
use crate::routes::users::get_credit_usage_history::get_credit_usage_history_handler;
use crate::routes::users::get_action_type_breakdown::get_action_type_breakdown_handler;
use crate::routes::users::get_organization_user_credit_breakdown::get_organization_user_credit_breakdown_handler;
use crate::routes::users::subscriptions::get_current_user_subscription;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
        .service(claim_daily_credits_handler)
        .service(get_user_credits_handler)
        .service(get_credit_usage_history_handler) // GET /api/users/credit-usage-history
        .service(get_action_type_breakdown_handler) // GET /api/users/action-type-breakdown
        .service(get_organization_user_credit_breakdown_handler) // GET /api/users/organization-credit-breakdown
        .service(get_current_user_handler) // GET /api/users/me
        .service(delete_current_user_handler) // DELETE /api/users/me
        .service(get_current_user_subscription) // GET /api/users/subscriptions/me
    );
} 