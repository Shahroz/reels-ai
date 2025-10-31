use actix_web::{get, web, HttpResponse, Responder};

#[utoipa::path(
    get,
    path = "/api/assets/lineage/graph",
    tag = "Assets",
    params( (
        "asset_id" = String, Query, description = "Asset ID to build lineage graph for"
    )),
    responses(
        (status = 200, description = "Lineage graph", body = crate::queries::assets::lineage::types::StudioGraph),
        (status = 404, description = "Asset not found or not owned by user"),
        (status = 400, description = "Invalid asset id")
    ),
    security(("user_auth" = []))
)]
#[get("/lineage/graph")]
pub async fn get_lineage_graph(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    q: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user_id = claims.user_id;
    let Some(asset_id_str) = q.get("asset_id") else {
        return HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse { error: "missing asset_id".into() })
    };
    let Ok(asset_id) = uuid::Uuid::parse_str(asset_id_str) else {
        return HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse { error: "invalid asset_id".into() })
    };

    match crate::queries::assets::lineage::get_graph_for_asset::get_graph_for_asset(&pool, user_id, asset_id).await {
        Ok(mut graph) => {
            // Ensure studio journey exists in database
            match crate::queries::assets::lineage::get_or_create_journey::get_or_create_journey(&pool, user_id, graph.root_asset_id).await {
                Ok(journey) => {
                    // Add journey_id to the graph response
                    graph.journey_id = Some(journey.id);
                    HttpResponse::Ok().json(graph)
                },
                Err(e) => {
                    log::error!("get_or_create_journey error: {}", e);
                    HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse { error: "internal error".into() })
                }
            }
        },
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse { error: "Asset not found".into() }),
        Err(e) => {
            log::error!("get_lineage_graph error: {}", e);
            HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse { error: "internal error".into() })
        }
    }
}


