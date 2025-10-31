//! Configures auth route handlers within the /auth scope.
//!
//! Registers each endpoint with Actix-web.
pub fn configure_auth_routes(cfg: &mut actix_web::web::ServiceConfig) {
   cfg.service(crate::routes::auth::register::register)
       .service(crate::routes::auth::login::login)
       .service(crate::routes::auth::google_login::google_login)
       .service(crate::routes::auth::google_callback::google_callback)
       .service(crate::routes::auth::password_reset::password_reset)
       .service(crate::routes::auth::reset_password::reset_password)
       .service(crate::routes::auth::logout::logout)
        .service(crate::routes::auth::verify_token::verify_token)
        .service(crate::routes::auth::admin_password_reset::admin_password_reset)
        .service(crate::routes::auth::change_password::change_password)
        .service(crate::routes::auth::request_magic_link::request_magic_link)
        .service(crate::routes::auth::verify_magic_link_token::verify_magic_link_token);
}
