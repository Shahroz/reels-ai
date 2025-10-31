//! Email service module for sending transactional emails.
//!
//! Provides email functionality for authentication flows including
//! magic links and OAuth user guidance. Uses Postmark for email delivery.

pub mod get_postmark_client;
pub mod send_magic_link_email;
pub mod send_oauth_user_guidance_email;
pub mod send_organization_invitation_email;
pub mod templates;

pub use get_postmark_client::get_postmark_client;
pub use send_magic_link_email::send_magic_link_email;
pub use send_oauth_user_guidance_email::send_oauth_user_guidance_email;
pub use send_organization_invitation_email::send_organization_invitation_email;

