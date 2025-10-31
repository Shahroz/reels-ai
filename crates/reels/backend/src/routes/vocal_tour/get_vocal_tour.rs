//! Handler for retrieving a single vocal tour with expanded details.
//!
//! Defines the GET `/api/vocal-tour/{id}` endpoint handler.
//! Fetches a vocal tour by its ID for the authenticated user, including its
//! associated document and assets.

use crate::routes::vocal_tour::list_vocal_tours_response::ExpandedVocalTour;
use crate::routes::error_response::ErrorResponse;

#[utoipa::path(
    get,
    path = "/api/vocal-tour/{id}",
    tag = "Vocal Tour",
    params(
        ("id" = uuid::Uuid, Path, description = "The ID of the vocal tour to retrieve.")
    ),
    responses(
        (status = 200, description = "Vocal tour with expanded details", body = ExpandedVocalTour),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Vocal tour not found", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[actix_web::get("/{id}")]
pub async fn get_vocal_tour(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    path: actix_web::web::Path<uuid::Uuid>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let vocal_tour_id = path.into_inner();

    #[derive(sqlx::FromRow, Debug)]
    struct VocalTourRow {
        vocal_tour_id: uuid::Uuid,
        user_id: uuid::Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        document_id: uuid::Uuid,
        document_user_id: Option<uuid::Uuid>,
        title: String,
        content: String,
        sources: Vec<String>,
        status: String,
        document_created_at: chrono::DateTime<chrono::Utc>,
        document_updated_at: chrono::DateTime<chrono::Utc>,
        is_public: bool,
        is_task: bool,
        include_research: Option<String>,
        collection_id: Option<uuid::Uuid>,
        asset_id: Option<uuid::Uuid>,
        asset_user_id: Option<uuid::Uuid>,
        asset_name: Option<String>,
        asset_type: Option<String>,
        gcs_object_name: Option<String>,
        asset_url: Option<String>,
        asset_collection_id: Option<uuid::Uuid>,
        asset_created_at: Option<chrono::DateTime<chrono::Utc>>,
        asset_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let items_query = sqlx::query_as!(
        VocalTourRow,
        r#"
        SELECT
            vt.id as vocal_tour_id, vt.user_id, vt.created_at, vt.updated_at,
            d.id as document_id, d.user_id as document_user_id, d.title, d.content, d.sources, d.status,
            d.created_at as document_created_at, d.updated_at as document_updated_at, d.is_public, d.is_task, d.include_research, d.collection_id,
            a.id as "asset_id?", a.user_id as "asset_user_id?", a.name as "asset_name?", a.type as "asset_type?", a.gcs_object_name as "gcs_object_name?",
            a.url as "asset_url?", a.collection_id as "asset_collection_id?", a.created_at as "asset_created_at?", a.updated_at as "asset_updated_at?"
        FROM
            vocal_tours vt
        INNER JOIN
            documents d ON vt.document_id = d.id
        LEFT JOIN
            assets a ON a.id = ANY(vt.asset_ids)
        WHERE
            vt.user_id = $1 AND vt.id = $2
        "#,
        user_id,
        vocal_tour_id
    )
    .fetch_all(&**pool)
    .await;

    let rows = match items_query {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("Error fetching vocal tour {vocal_tour_id} for user {user_id}: {e}");
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: "Failed to fetch vocal tour".into(),
            });
        }
    };

    if rows.is_empty() {
        return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
            error: "Vocal tour not found".into(),
        });
    }

    let mut vocal_tours_map: indexmap::IndexMap<uuid::Uuid, crate::routes::vocal_tour::list_vocal_tours_response::ExpandedVocalTour> = indexmap::IndexMap::new();

    for row in rows {
        let asset: Option<crate::db::assets::Asset> = if let Some(asset_id) = row.asset_id {
            Some(crate::db::assets::Asset {
                id: asset_id,
                user_id: Some(row.asset_user_id.unwrap()),
                name: row.asset_name.unwrap(),
                r#type: row.asset_type.unwrap(),
                gcs_object_name: row.gcs_object_name.unwrap(),
                url: row.asset_url.unwrap(),
                collection_id: row.asset_collection_id,
                metadata: None, // Asset metadata not available in this query
                created_at: row.asset_created_at,
                updated_at: row.asset_updated_at,
                is_public: false, // Default for existing assets
            })
        } else {
            None
        };

        vocal_tours_map
            .entry(row.vocal_tour_id)
            .or_insert_with(|| crate::routes::vocal_tour::list_vocal_tours_response::ExpandedVocalTour {
                id: row.vocal_tour_id,
                user_id: row.user_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                document: crate::db::documents::Document {
                    id: row.document_id,
                    user_id: row.document_user_id,
                    title: row.title,
                    content: row.content,
                    sources: row.sources,
                    status: row.status,
                    created_at: row.document_created_at,
                    updated_at: row.document_updated_at,
                    is_public: row.is_public,
                    is_task: row.is_task,
                    include_research: row.include_research.and_then(|s| s.parse().ok()),
                    collection_id: row.collection_id,
                },
                assets: Vec::new(),
            });

        if let Some(asset) = asset {
            if let Some(tour) = vocal_tours_map.get_mut(&row.vocal_tour_id) {
                tour.assets.push(asset);
            }
        }
    }

    let vocal_tour = vocal_tours_map.into_values().next().unwrap(); // Should be safe due to is_empty check

    actix_web::HttpResponse::Ok().json(vocal_tour)
}

#[cfg(test)]
mod tests {
    // Unit tests for API handlers are complex due to dependencies.
    // Integration tests are preferred for this layer.
}