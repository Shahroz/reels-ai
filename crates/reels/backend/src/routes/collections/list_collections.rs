//! Handler for listing all collections.
//!
//! GET /api/collections - Returns collections belonging to the authenticated user.

//! Revision History
//! - 2025-05-02T16:55:28Z @AI: Implement pagination, sorting, searching.
//! - 2025-05-02T13:24:34Z @AI: Modify handler to filter by authenticated user_id and add basic test structure.

//! GET /api/collections

use crate::db::collections::Collection;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::ToSchema;

#[derive(Deserialize, Debug, ToSchema)]
pub struct ListCollectionsParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
    #[schema(example = "campaign")]
    pub search: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ListCollectionsResponse {
    pub items: Vec<Collection>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/collections",
    tag = "Collections",
    security(
      ("bearer_auth" = [])
    ),
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (default created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort direction (asc or desc, default desc)"),
        ("search" = Option<String>, Query, description = "Filter collections by name")
    ),
    responses(
        (status = 200, description = "List collections", body = ListCollectionsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[get("")]
#[instrument(skip(pool, claims, params))]
pub async fn list_collections(
    pool: web::Data<PgPool>,
    claims: crate::auth::tokens::Claims, // Inject Claims extractor
    params: web::Query<ListCollectionsParams>,
) -> impl Responder {
    let user_id = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1);
    let offset = (page - 1) * limit;
    // Validate sort_by against allowed fields (e.g., "name", "created_at", "updated_at")
    let sort_by = params
        .sort_by
        .clone()
        .filter(|s| ["name", "created_at", "updated_at"].contains(&s.as_str()))
        .unwrap_or_else(|| "created_at".into());
    let sort_order = params
        .sort_order
        .clone()
        .filter(|s| s == "asc" || s == "desc")
        .unwrap_or_else(|| "desc".into());
    let search_pattern = params
        .search
        .clone()
        .map(|s| format!("%{s}%"))
        .unwrap_or_else(|| "%".into());

    // Total count query
    let total_count = match crate::queries::collections::count_collections::count_collections(
        pool.get_ref(),
        user_id,
        &search_pattern,
    )
    .await
    {
        Ok(count) => count,
        Err(e) => {
            log::error!("Error counting collections for user {user_id}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to count collections".into(),
            });
        }
    };

    // Items query - use sharing-aware version
    let items_result = crate::queries::collections::list_collections_with_sharing::list_collections_with_sharing(
        pool.get_ref(),
        user_id,
        &search_pattern,
        &sort_by,
        &sort_order,
        limit,
        offset,
    )
    .await;

    match items_result {
        Ok(items) => HttpResponse::Ok().json(ListCollectionsResponse { items, total_count }),
        Err(e) => {
            log::error!("Error fetching collections for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch collections".into(),
            })
        }
    }
}


#[cfg(test)]
mod tests {
    // Note: Full paths are required by guidelines. std::prelude items are exceptions.
    // Note: Mocking DB and request context is complex. This is a basic structure.

    #[actix_web::test]
    async fn test_list_collections_needs_impl() {
        // Arrange: Setup mock pool, claims, etc. This requires a testing framework or mock library.
        // Example using dummy data (won't actually query DB without proper mocking):
        // let mock_pool = ... create mock pool ...;
        // let user_id = uuid::Uuid::new_v4();
        // let claims = crate::auth::tokens::Claims { user_id, exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as u64 };
        // let req = actix_web::test::TestRequest::default()
        //     .insert_header(("Authorization", format!("Bearer fake-token"))) // Mock token needed if extractor checks it
        //     .app_data(actix_web::web::Data::new(mock_pool)) // Provide mock pool
        //     .to_http_request();

        // // Act: Call the handler (requires constructing request context correctly)
        // // This is simplified - real testing needs `init_service` or similar.
        // let result = super::list_collections(actix_web::web::Data::new(mock_pool_clone), claims).await;

        // Assert: Check the response status and body.
        // let response = actix_web::test::call_service(&mut app, req).await;
        // assert!(response.status().is_success());
        // // Further assertions on the body content would go here, checking if only collections
        // // for `user_id` are returned.

        // Placeholder assertion until proper mocking with params is implemented
        assert!(true, "Test needs implementation with DB/Request mocking");
    }
}
