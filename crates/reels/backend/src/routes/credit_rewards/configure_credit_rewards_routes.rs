//! Configures all credit rewards endpoints.
//!
//! Registers handlers under the /credit-rewards scope.
use actix_web::web;

pub fn configure_credit_rewards_routes(cfg: &mut web::ServiceConfig) {
    log::info!("Configuring Credit Rewards routes");
    cfg.service(
        web::scope("")
        .service(crate::routes::credit_rewards::get_reward_definitions::get_reward_definitions)
        .service(crate::routes::credit_rewards::get_user_rewards::get_user_credit_rewards)
        .service(crate::routes::credit_rewards::claim_reward::claim_reward)
    );
}
