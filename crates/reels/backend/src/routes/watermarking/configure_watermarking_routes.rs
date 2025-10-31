//! Configures all watermarking endpoints.
//!
//! Registers handlers under the /watermark scope.

pub fn configure_watermarking_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(crate::routes::watermarking::apply_batch_watermark::apply_batch_watermark);
}
