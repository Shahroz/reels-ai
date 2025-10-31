//! Auth routes module
//!
//! Contains one-item-per-file definitions for authentication.

pub mod change_password;
pub mod change_password_request;
pub mod configure_auth_routes;
pub mod google_login;
pub mod google_callback;
pub mod validate_callback_parameters;
pub mod process_oauth_token;
pub mod create_user_session;
pub mod is_mobile_device;
pub mod determine_redirect_url;
pub mod validation_limits;
pub mod registration_helpers;
pub mod login;
pub mod login_request;
pub mod login_response;
pub mod logout;
pub mod password_reset;
pub mod password_reset_request;
pub mod register;
pub mod register_request;
pub mod verify_token;
pub mod verify_token_response;
pub mod request_magic_link;
pub mod verify_magic_link_token;
pub mod verify_magic_link_token_request;
pub mod verify_magic_link_token_response;
pub mod handle_oauth_user_magic_link_request;
pub mod handle_password_user_magic_link_request;
pub mod build_magic_link_url;
pub mod generate_session_token_for_user;
pub mod track_magic_link_login_analytics;
pub mod extract_request_context;
pub mod verify_magic_link_claims;
pub mod fetch_user_for_magic_link;
pub mod verify_and_consume_token_version;
pub mod create_session_token;
pub mod verify_magic_link_core;
pub mod map_verification_error_to_response;
pub mod standard_logout_response;
pub mod stop_impersonation_response;
pub mod logout_response_body;
pub mod reset_password_request;
pub mod reset_password;
pub mod admin_password_reset;
