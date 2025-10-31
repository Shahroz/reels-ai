//! Module for invitation-related API endpoints (accept/reject).

pub mod accept_invitation_handler;
pub mod reject_invitation_handler;
pub mod list_pending_invitations_handler;
// pub mod list_pending_invitations_handler; // Placeholder

use actix_web::web;

/// Configures the routes for invitation-related endpoints.
pub fn configure_invitation_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(accept_invitation_handler::accept_invitation_handler)
            .service(reject_invitation_handler::reject_invitation_handler)
            .service(list_pending_invitations_handler::list_pending_invitations_handler);
            // .service(list_pending_invitations_handler::list_pending_invitations_handler) 
} 