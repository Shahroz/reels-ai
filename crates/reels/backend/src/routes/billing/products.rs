use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use serde::Deserialize;

use crate::services::billing::billing_config::BillingConfig;
use crate::services::billing::stripe_client::StripeProductWithPrices;
use crate::services::billing::billing_factory::create_billing_service;

#[derive(Debug, Deserialize)]
pub struct GetProductsQuery {
    meta_product_type: Option<String>,
    active: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/billing/products",
    tag = "Billing",
    params(
        ("meta_product_type" = Option<String>, Query, description = "Optional metadata key to filter products by"),
        ("active" = Option<bool>, Query, description = "Optional boolean to filter products by active status")
    ),
    responses(
        (status = 200, description = "Products retrieved successfully", body = Vec<StripeProductWithPrices>),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/products")]
#[instrument(skip(_pool))]
pub async fn get_products(
    _pool: web::Data<PgPool>,
    query: web::Query<GetProductsQuery>,
) -> impl Responder {
    let meta_product_type = query.meta_product_type.as_deref();
    let active = query.active;

    let config = BillingConfig::from_env();
    match create_billing_service(&config) {
        Ok(billing_service) => {
            match billing_service.get_products(meta_product_type, active).await {
                Ok(products) => {
                    log::info!("Successfully retrieved {} products (filtered by meta_product_type: {:?}, active: {:?})", 
                              products.len(), meta_product_type, active);
                    HttpResponse::Ok().json(products)
                }
                Err(e) => {
                    log::error!("Failed to retrieve products: {e}");
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to retrieve products",
                        "message": e.to_string()
                    }))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create billing service: {e}");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create billing service",
                "message": e
            }))
        }
    }
}
