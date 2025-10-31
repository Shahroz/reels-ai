//! Configures all request handling routes under the `/api/requests` scope.
//!
//! Registers the list, retrieve, and delete endpoints.

pub fn configure_requests_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("")
            .service(crate::routes::requests::list_requests::list_requests)
            .service(crate::routes::requests::get_request_by_id::get_request_by_id)
            .service(crate::routes::requests::delete_request::delete_request),
    );
}
