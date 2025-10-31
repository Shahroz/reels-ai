//! Handler for listing vocal tours with expanded document and asset details.
//!
//! Defines the GET `/api/vocal-tour` endpoint handler.
//! Fetches a paginated, sorted, and searchable list of vocal tours for the authenticated user.

use crate::auth::tokens::Claims;
use crate::db::{assets::Asset, documents::Document};
use crate::routes::error_response::ErrorResponse;
use crate::routes::vocal_tour::list_vocal_tours_params::ListVocalToursParams;
use crate::routes::vocal_tour::list_vocal_tours_response::{ExpandedVocalTour, ListVocalToursResponse};
use crate::sql_utils::count_sql_results::TotalCount;
use actix_web::{get, web, HttpResponse, Responder};
use indexmap::IndexMap;
use sqlx_conditional_queries::conditional_query_as;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/vocal-tour",
    tag = "Vocal Tour",
    params(
        ListVocalToursParams
    ),
    responses(
        (status = 200, description = "List of vocal tours with expanded details", body = ListVocalToursResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body = ErrorResponse),
    ),
    security(
        ("user_auth" = [])
    )
)]
#[get("")]
pub async fn list_vocal_tours(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<Claims>,
    params: web::Query<ListVocalToursParams>,
) -> impl Responder {
    let user_id = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;

    let sort_by = params
        .sort_by
        .as_deref()
        .unwrap_or("created_at");
    let sort_order = params
        .sort_order
        .as_deref()
        .unwrap_or("desc");

    let search_pattern = params
        .search
        .as_ref()
        .map(|s| format!("%{s}%"))
        .unwrap_or_else(|| "%".to_string());

    let total_count_query = conditional_query_as!(
        TotalCount,
        r#"
        SELECT COUNT(DISTINCT vt.id) as "count?"
        FROM vocal_tours vt
        INNER JOIN documents d ON vt.document_id = d.id
        WHERE vt.user_id = {user_id} AND d.title ILIKE {search_pattern}
        "#,
        #user_id = match &user_id { _ => "{user_id}" },
        #search_pattern = match &search_pattern { _ => "{search_pattern}" }
    )
    .fetch_one(&**pool)
    .await;

    let total_count = match total_count_query {
        Ok(total_count) => total_count.count.unwrap_or(0),
        Err(e) => {
            log::error!("Error counting vocal tours for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count vocal tours".into(),
            });
        }
    };

    #[derive(sqlx::FromRow, Debug)]
    struct VocalTourRow {
        vocal_tour_id: Uuid,
        user_id: Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        document_id: Uuid,
        document_user_id: Option<Uuid>,
        title: String,
        content: String,
        sources: Vec<String>,
        status: String,
        document_created_at: chrono::DateTime<chrono::Utc>,
        document_updated_at: chrono::DateTime<chrono::Utc>,
        is_public: bool,
        is_task: bool,
        include_research: Option<String>,
        collection_id: Option<Uuid>,
        asset_id: Option<Uuid>,
        asset_user_id: Option<Uuid>,
        asset_name: Option<String>,
        asset_type: Option<String>,
        gcs_object_name: Option<String>,
        asset_url: Option<String>,
        asset_collection_id: Option<Uuid>,
        asset_created_at: Option<chrono::DateTime<chrono::Utc>>,
        asset_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    // For now, let's use a two-step approach to work around the conditional_query_as limitations
    // First get the vocal tour IDs with proper pagination
    let vocal_tour_ids_result = sqlx::query!(
        r#"
        SELECT vt.id
        FROM vocal_tours vt
        INNER JOIN documents d ON vt.document_id = d.id
        WHERE vt.user_id = $1 AND d.title ILIKE $2
        ORDER BY 
            CASE WHEN $5 = 'title' AND $6 = 'asc' THEN d.title END ASC,
            CASE WHEN $5 = 'title' AND $6 = 'desc' THEN d.title END DESC,
            CASE WHEN $5 = 'updated_at' AND $6 = 'asc' THEN vt.updated_at END ASC,
            CASE WHEN $5 = 'updated_at' AND $6 = 'desc' THEN vt.updated_at END DESC,
            CASE WHEN $5 NOT IN ('title', 'updated_at') AND $6 = 'asc' THEN vt.created_at END ASC,
            CASE WHEN $5 NOT IN ('title', 'updated_at') AND $6 = 'desc' THEN vt.created_at END DESC
        LIMIT $3 OFFSET $4
        "#,
        user_id,
        search_pattern,
        limit,
        offset,
        sort_by,
        sort_order
    )
    .fetch_all(&**pool)
    .await;

    let vocal_tour_ids_vec = match vocal_tour_ids_result {
        Ok(rows) => rows.into_iter().map(|row| row.id).collect::<Vec<Uuid>>(),
        Err(e) => {
            log::error!("Error fetching vocal tour IDs for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch vocal tour IDs".into(),
            });
        }
    };

    let vocal_tour_ids: &[Uuid] = vocal_tour_ids_vec.as_slice();

    if vocal_tour_ids.is_empty() {
        return HttpResponse::Ok().json(ListVocalToursResponse { 
            items: Vec::new(), 
            total_count 
        });
    }

    // Then get the full data for these vocal tours
    let items_query = conditional_query_as!(
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
            vt.id = ANY({vocal_tour_ids})
        ORDER BY
            {#sort_by_sql_literal} {#sort_order_sql_literal}
        "#,
        #vocal_tour_ids = match &vocal_tour_ids { _ => "{vocal_tour_ids}" },
        #user_id = match &user_id { _ => "{user_id}" },
        #search_pattern = match &search_pattern { _ => "{search_pattern}" },
        #limit = match &limit { _ => "{limit}" },
        #offset = match &offset { _ => "{offset}" },
        #sort_by_sql_literal = match sort_by {
            "title" => "d.title",
            "updated_at" => "vt.updated_at",
            _ => "vt.created_at",
        },
        #sort_order_sql_literal = match sort_order {
            "asc" => "ASC",
            _ => "DESC",
        }
    )
        .fetch_all(&**pool)
        .await;

    let rows = match items_query {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("Error fetching vocal tours for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch vocal tours".into(),
            });
        }
    };

    let mut vocal_tours_map: IndexMap<Uuid, ExpandedVocalTour> = IndexMap::new();

    for row in rows {
        let vocal_tour_id = row.vocal_tour_id;
        
        let asset: Option<Asset> = if let Some(asset_id) = row.asset_id {
            Some(Asset {
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
            .entry(vocal_tour_id)
            .or_insert_with(|| ExpandedVocalTour {
                id: row.vocal_tour_id,
                user_id: row.user_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                document: Document {
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
            if let Some(tour) = vocal_tours_map.get_mut(&vocal_tour_id) {
                tour.assets.push(asset);
            }
        }
    }

    let items: Vec<ExpandedVocalTour> = vocal_tours_map.into_values().collect();

    HttpResponse::Ok().json(ListVocalToursResponse { items, total_count })
}
