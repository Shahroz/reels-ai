use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use serde::Deserialize;

use crate::services::billing::billing_config::BillingConfig;
use crate::services::billing::stripe_client::StripePlanWithProduct;
use crate::services::billing::billing_factory::create_billing_service;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GetPlansQuery {
    active: Option<bool>,
    meta_product_type: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/billing/plans",
    tag = "Billing",
    params(
        ("active" = Option<bool>, Query, description = "Optional boolean to filter plans by active status"),
        ("meta_product_type" = Option<String>, Query, description = "Optional metadata key to filter plans by product type")
    ),
    responses(
        (status = 200, description = "Plans retrieved successfully", body = Vec<StripePlanWithProduct>),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/plans")]
#[instrument(skip(_pool))]
pub async fn get_plans(
    _pool: web::Data<PgPool>,
    query: web::Query<GetPlansQuery>,
) -> impl Responder {
    let active = query.active;
    let meta_product_type = query.meta_product_type.as_deref();

    let config = BillingConfig::from_env();
    match create_billing_service(&config) {
        Ok(billing_service) => {
            match billing_service.get_plans(meta_product_type, active).await {
                Ok(plans) => {
                    log::info!("Successfully retrieved {} plans (filtered by active: {:?}, meta_product_type: {:?})",
                              plans.len(), active, meta_product_type);
                    HttpResponse::Ok().json(plans)
                }
                Err(e) => {
                    log::error!("Failed to retrieve plans: {e}");
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to retrieve plans",
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
