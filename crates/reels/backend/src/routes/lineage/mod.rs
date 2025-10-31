use actix_web::{get, web, HttpResponse, Responder};
use tracing::instrument;

#[derive(serde::Deserialize, Debug)]
pub struct LineageGraphParams {
    pub seed_type: String,
    pub seed_id: uuid::Uuid,
}

#[utoipa::path(
    get,
    path = "/api/lineage/graph",
    tag = "Assets",
    params(
        ("seed_type" = String, Query, description = "asset | document | video", example = "asset"),
        ("seed_id" = String, Query, description = "UUID of the seed entity", example = "550e8400-e29b-41d4-a716-446655440000"),
    ),
    responses(
        (status = 200, description = "Lineage graph computed successfully"),
        (status = 400, description = "Unsupported seed type or invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/graph")]
#[instrument(skip(pool, claims))]
pub async fn get_lineage_graph_unified(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    query: web::Query<LineageGraphParams>,
) -> impl Responder {
    let user_id = claims.user_id;
    match query.seed_type.as_str() {
        "asset" => {
            match crate::queries::assets::lineage::get_graph_for_asset::get_graph_for_asset(&pool, user_id, query.seed_id).await {
                Ok(graph) => HttpResponse::Ok().json(graph),
                Err(e) => {
                    log::error!("Failed to compute lineage graph for asset {}: {}", query.seed_id, e);
                    HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to compute graph"}))
                }
            }
        }
        _ => HttpResponse::BadRequest().json(serde_json::json!({"error": "Unsupported seed_type; only 'asset' is currently supported"})),
    }
}

pub fn configure_lineage_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("")
            .service(get_lineage_graph_unified)
    );
}


