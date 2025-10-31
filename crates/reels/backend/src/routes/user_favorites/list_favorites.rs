//! Handler for listing user favorites with entity relation data.
use crate::auth::tokens::Claims;
use crate::db::favorites::{FavoriteEntityType, UserFavorite};
use crate::db::{styles::Style, creatives::Creative, documents::Document};
use crate::db::document_research_usage::DocumentResearchUsage;
use crate::routes::error_response::ErrorResponse;
use crate::routes::user_favorites::entity_relations::EntityData;
use crate::routes::user_favorites::favorite_with_entity::FavoriteWithEntity;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use utoipa::{ToSchema, IntoParams};
use std::str::FromStr;
use sqlx::QueryBuilder;

#[derive(Deserialize, Debug, ToSchema, IntoParams)]
pub struct ListFavoritesParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "creative")]
    pub entity_type: Option<String>,
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ListFavoritesResponse {
    pub items: Vec<FavoriteWithEntity>,
    pub total_count: i64,
}

// Helper function to fetch entity data based on entity_id and entity_type
async fn fetch_entity_data(
    pool: &PgPool,
    _user_id: Uuid,
    entity_id: Uuid,
    entity_type: &FavoriteEntityType,
) -> Option<EntityData> {
    match entity_type {
        FavoriteEntityType::Style => {
            // Check if style exists
            let style_result = sqlx::query_as!(
                Style,
                r#"
                SELECT id, user_id, name, html_url, screenshot_url, is_public, created_at, updated_at
                FROM styles
                WHERE id = $1
                "#,
                entity_id
            )
            .fetch_optional(pool)
            .await;

            match style_result {
                Ok(Some(style)) => Some(EntityData::Style(style)),
                _ => None,
            }
        }
        FavoriteEntityType::Creative => {
            // Check if creative exists
            let creative_result = sqlx::query_as!(
                Creative,
                r#"
                SELECT
                    id, name, collection_id, creative_format_id, style_id, document_ids, asset_ids,
                    html_url, draft_url, bundle_id, screenshot_url, is_published, publish_url,
                    created_at, updated_at
                FROM creatives
                WHERE id = $1
                "#,
                entity_id
            )
            .fetch_optional(pool)
            .await;

            match creative_result {
                Ok(Some(creative)) => Some(EntityData::Creative(creative)),
                _ => None,
            }
        }
        FavoriteEntityType::Document => {
            // Temporary struct for database query result
            #[derive(Debug, Clone, sqlx::FromRow)]
            struct DocumentRow {
                pub id: sqlx::types::Uuid,
                pub user_id: Option<sqlx::types::Uuid>,
                pub title: String,
                pub content: String,
                pub sources: Vec<String>,
                pub status: String,
                pub created_at: chrono::DateTime<chrono::Utc>,
                pub updated_at: chrono::DateTime<chrono::Utc>,
                pub is_public: bool,
                pub is_task: bool,
                pub include_research: Option<String>,
                pub collection_id: Option<sqlx::types::Uuid>,
            }

            // Check if document exists
            let document_result = sqlx::query_as!(
                DocumentRow,
                r#"
                SELECT
                    id, user_id, title, content, sources, status, created_at, updated_at,
                    is_public, is_task, include_research, collection_id
                FROM documents
                WHERE id = $1
                "#,
                entity_id
            )
            .fetch_optional(pool)
            .await;

            match document_result {
                Ok(Some(document_row)) => {
                    // Convert DocumentRow to Document with proper parsing
                    let include_research = document_row.include_research
                        .and_then(|s| s.parse::<DocumentResearchUsage>().ok());

                    let document = Document {
                        id: document_row.id,
                        user_id: document_row.user_id,
                        title: document_row.title,
                        content: document_row.content,
                        sources: document_row.sources,
                        status: document_row.status,
                        created_at: document_row.created_at,
                        updated_at: document_row.updated_at,
                        is_public: document_row.is_public,
                        is_task: document_row.is_task,
                        include_research,
                        collection_id: document_row.collection_id,
                    };

                    Some(EntityData::Document(document))
                }
                _ => None,
            }
        }
        FavoriteEntityType::Prompt => {
            // TODO: Implement prompt favorites in future
            // For now, prompts don't have a dedicated entity table
            // They will be stored in a separate favorited_prompts table or as JSONB metadata
            None
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/user-favorites",
    params(ListFavoritesParams),
    responses(
        (status = 200, description = "List user favorites with entity data", body = ListFavoritesResponse),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    security(
        ("jwt_token" = [])
    ),
    tag = "User Favorites",
)]
#[get("")]
#[instrument(skip_all)]
pub async fn list_favorites(
    pool: web::Data<PgPool>,
    claims: Claims,
    params: web::Query<ListFavoritesParams>,
) -> impl Responder {
    log::info!("list_favorites handler invoked. Query params: {:?}", params.0);

    let user_id = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).max(1).min(100);
    let offset = (page - 1) * limit;

    let entity_type = params.entity_type.clone();
    let sort_by = params.sort_by.clone().unwrap_or_else(|| "created_at".to_string());
    let sort_order = params.sort_order.clone().unwrap_or_else(|| "desc".to_string());

    log::info!("list_favorites - Extracted params: user_id={}, page={}, limit={}, entity_type={:?}, sort_by={}, sort_order={}",
               user_id, page, limit, &entity_type, sort_by, sort_order);

    // Validate sort parameters
    let valid_sort_fields = ["created_at", "updated_at"];
    if !valid_sort_fields.contains(&sort_by.as_str()) {
        log::warn!("list_favorites - Invalid sort_by parameter: {sort_by}");
        return HttpResponse::BadRequest().json(ErrorResponse::from("Invalid sort_by parameter"));
    }

    let valid_sort_orders = ["asc", "desc"];
    if !valid_sort_orders.contains(&sort_order.as_str()) {
        log::warn!("list_favorites - Invalid sort_order parameter: {sort_order}");
        return HttpResponse::BadRequest().json(ErrorResponse::from("Invalid sort_order parameter"));
    }

    // Parse entity_type filter if provided
    let entity_type_filter = if let Some(entity_type) = &entity_type {
        match entity_type.as_str() {
            "style" | "creative" | "document" => {
                let entity_type_enum = match FavoriteEntityType::from_str(entity_type) {
                    Ok(et) => et,
                    Err(_) => {
                        log::warn!("list_favorites - Invalid entity_type parameter: {entity_type}");
                        return HttpResponse::BadRequest().json(ErrorResponse::from("Invalid entity_type parameter"));
                    }
                };
                Some(entity_type_enum)
            }
            _ => {
                log::warn!("list_favorites - Invalid entity_type parameter: {entity_type}");
                return HttpResponse::BadRequest().json(ErrorResponse::from("Invalid entity_type parameter"));
            }
        }
    } else {
        None
    };

    // Get total count using QueryBuilder for consistency
    let mut count_query_builder = QueryBuilder::new(
        "SELECT COUNT(*) FROM user_favorites WHERE user_id = "
    );
    count_query_builder.push_bind(user_id);

    // Add entity_type filter if provided
    if let Some(entity_type) = &entity_type_filter {
        count_query_builder.push(" AND entity_type = ");
        count_query_builder.push_bind(*entity_type);
    }

    log::info!("list_favorites - Count query SQL: {:?}", count_query_builder.sql());

    let total_count = match count_query_builder
        .build_query_scalar::<i64>()
        .fetch_one(&**pool)
        .await
    {
        Ok(count) => count,
        Err(e) => {
            log::error!("list_favorites - DB error counting favorites: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to count favorites"));
        }
    };

    log::info!("list_favorites - Total count: {}", total_count);

    // Build query with QueryBuilder for better control
    let mut query_builder = QueryBuilder::new(
        r#"SELECT id, user_id, entity_id, entity_type as "entity_type: FavoriteEntityType", created_at, updated_at
        FROM user_favorites
        WHERE user_id = "#
    );
    query_builder.push_bind(user_id);

    // Add entity_type filter if provided
    if let Some(entity_type) = &entity_type_filter {
        query_builder.push(" AND entity_type = ");
        query_builder.push_bind(*entity_type);
    }

    // Add ordering
    query_builder.push(" ORDER BY ");
    query_builder.push(sort_by);
    query_builder.push(" ");
    query_builder.push(sort_order.to_uppercase());

    // Add pagination
    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    log::info!("list_favorites - Final query SQL before execution: {:?}", query_builder.sql());

    // Execute query
    let favorites_result = query_builder
        .build_query_as::<UserFavorite>()
        .fetch_all(&**pool)
        .await;

    let favorites = match favorites_result {
        Ok(favorites) => {
            log::info!("list_favorites - Successfully fetched {} favorites from DB.", favorites.len());
            favorites
        }
        Err(e) => {
            log::error!("list_favorites - DB error fetching favorites: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to fetch favorites"));
        }
    };

    // Fetch entity data for each favorite
    let mut favorites_with_entities = Vec::new();

    for favorite in favorites {
        let entity_data = if let Some(filter_type) = &entity_type_filter {
            // If entity_type filter is provided, only fetch data for that type
            if &favorite.entity_type == filter_type {
                fetch_entity_data(&pool, user_id, favorite.entity_id, &favorite.entity_type).await
            } else {
                None
            }
        } else {
            // If no filter, fetch data for all types
            fetch_entity_data(&pool, user_id, favorite.entity_id, &favorite.entity_type).await
        };

        let favorite_with_entity = FavoriteWithEntity {
            id: favorite.id,
            user_id: favorite.user_id,
            entity_id: favorite.entity_id,
            entity_type: favorite.entity_type,
            created_at: favorite.created_at,
            updated_at: favorite.updated_at,
            entity_data,
        };

        favorites_with_entities.push(favorite_with_entity);
    }

    let response = ListFavoritesResponse {
        items: favorites_with_entities,
        total_count,
    };

    log::info!("list_favorites - Returning 200 OK with {} favorites.", response.items.len());
    HttpResponse::Ok().json(response)
}