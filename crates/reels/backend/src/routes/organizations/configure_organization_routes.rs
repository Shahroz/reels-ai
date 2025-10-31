//! Configures the routes for organization-related endpoints.
//!
//! This function registers the handlers for creating and listing organizations
//! under the `/api/organizations` scope within the Actix web application setup.
//! Ensures organization endpoints are correctly mapped.

use actix_web::web;

use super::create_organization_handler::create_organization_handler;
use super::get_organization_handler::get_organization_handler;
use super::list_organizations_handler::list_organizations_handler;
use super::update_organization_handler::update_organization_handler;
use super::delete_organization_handler::delete_organization_handler;
use super::list_members_handler::list_members_handler;
use super::get_organization_members_for_credits::get_organization_members_handler;
use super::remove_member_handler::remove_member_handler;
use super::invite_member_handler::invite_member_handler;
use super::list_sent_invitations_handler::list_sent_invitations_handler;

/// Registers organization routes with the Actix application.
pub fn configure_organization_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_organization_handler)
       .service(list_organizations_handler)
       .service(get_organization_handler)
       .service(update_organization_handler)
       .service(delete_organization_handler)
       .service(list_members_handler)
       .service(get_organization_members_handler)
       .service(remove_member_handler)
       .service(
            web::resource("/{organization_id}/members")
                .route(web::post().to(invite_member_handler))
       )
       .service(list_sent_invitations_handler);
}
