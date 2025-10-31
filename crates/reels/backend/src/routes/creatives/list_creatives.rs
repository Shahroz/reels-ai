//! Handler for listing all creatives.
//!
//! GET /api/creatives

use crate::auth::tokens::Claims;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::routes::error_response::ErrorResponse;
use crate::sql_utils::count_sql_results::TotalCount;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};
use sqlx::{FromRow, PgPool};
use sqlx_conditional_queries::conditional_query_as;
use utoipa::ToSchema;
use log;
use tracing::instrument;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ... (Uuids enum and de_uuid_vec function as is) ...
#[derive(Deserialize)]
#[serde(untagged)]
enum Uuids {
    Seq(Vec<Uuid>),
    Str(String),
}
fn de_uuid_vec<'de, D>(deserializer: D) -> Result<Option<Vec<Uuid>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Uuids>::deserialize(deserializer)?;
    if let Some(uuids) = opt {
        let vec = match uuids {
            Uuids::Seq(v) => v,
            Uuids::Str(s) => {
                if s.trim().is_empty() {
                    return Ok(None);
                }
                s.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|part| Uuid::parse_str(part).map_err(de::Error::custom))
                    .collect::<Result<Vec<_>, D::Error>>()?
            }
        };
        Ok(Some(vec))
    } else {
        Ok(None)
    }
}

// ... (ListCreativesParams, CreativeListItem, ListCreativesResponse structs as is) ...
#[derive(Deserialize, Debug, ToSchema)]
pub struct ListCreativesParams {
    #[schema(example = 1)]
    pub page: Option<i64>,
    #[schema(example = 10)]
    pub limit: Option<i64>,
    #[schema(example = "updated_at")]
    pub sort_by: Option<String>,
    #[schema(example = "desc")]
    pub sort_order: Option<String>,
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub collection_id: Option<Uuid>,
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub style_id: Option<Uuid>,
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub creative_format_id: Option<Uuid>,
    #[schema(nullable = true)] // Uuid vector schema handled by ToSchema on Uuid
    #[serde(default, alias = "research_ids", deserialize_with = "de_uuid_vec")]
    pub document_ids: Option<Vec<Uuid>>,
    #[schema(example = "My Creative Search", nullable = true)]
    pub search: Option<String>, // From sharing branch
    #[schema(example = true, nullable = true)]
    pub is_favorite: Option<bool>,
}

#[derive(Serialize, Debug, ToSchema, FromRow)]
pub struct CreativeListItem {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid, 
    #[schema(value_type = String, nullable = true)]
    pub name: Option<String>,
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub collection_id: Option<Uuid>, 
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub creative_format_id: Option<Uuid>, 
    #[schema(value_type = String, format = "uuid", nullable = true)]
    pub style_id: Option<Uuid>,
    #[schema(nullable = true)] 
    pub document_ids: Option<Vec<Uuid>>,
    #[schema(nullable = true)] 
    pub asset_ids: Option<Vec<Uuid>>,
    #[schema(nullable = true)]
    pub html_url: Option<String>,
    #[schema(nullable = true)]
    pub draft_url: Option<String>,
    pub screenshot_url: String,
    pub is_published: bool,
    #[schema(nullable = true)]
    pub publish_url: Option<String>,
    #[schema(value_type = String, format = "date-time", nullable = true)]
    pub created_at: Option<DateTime<Utc>>,
    #[schema(value_type = String, format = "date-time", nullable = true)]
    pub updated_at: Option<DateTime<Utc>>,
    #[schema(nullable = true)]
    pub style_name: Option<String>,
    #[schema(nullable = true)] 
    pub document_names: Option<Vec<String>>,
    #[schema(nullable = true)]
    pub creative_format_name: Option<String>,
    #[schema(nullable = true)]
    pub collection_name: Option<String>,
    #[schema(example = "editor", value_type = Option<String>, nullable = true)]
    pub current_user_access_level: Option<String>,
    #[schema(example = "user@example.com", value_type = Option<String>, nullable = true)]
    pub creator_email: Option<String>,
    #[schema(example = true)]
    pub is_favorite: bool,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ListCreativesResponse {
    pub items: Vec<CreativeListItem>,
    pub total_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/creatives",
    responses(
        (status = 200, description = "List creatives with sharing information", body = ListCreativesResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("page" = Option<i64>, Query, description = "Page number (default 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default 10)"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by (e.g., updated_at, name, created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc or desc)"),
        ("collection_id" = Option<String>, Query, description = "Filter by collection ID (UUID string)"),
        ("style_id" = Option<String>, Query, description = "Filter by style ID (UUID string)"),
        ("creative_format_id" = Option<String>, Query, description = "Filter by creative format ID (UUID string)"),
        ("document_ids" = Option<Vec<String>>, Query, description = "Document IDs (comma-separated or repeated)"),
        ("search" = Option<String>, Query, description = "Search term for collection name or creative HTML URL"),
        ("is_favorite" = Option<bool>, Query, description = "Filter by favorite status (true for favorited, false for non-favorited)")
    ),
    security(
        ("jwt_token" = [])
    ),
    tag = "Creatives",
)]
#[get("")]
#[instrument(skip(pool, claims, params), fields(user_id = %claims.user_id))]
pub async fn list_creatives(
    pool: web::Data<PgPool>,
    claims: Claims,
    params: web::Query<ListCreativesParams>,
) -> impl Responder {
    // --- Parameter Preparation ---
    let user_id_param = claims.user_id;
    let page = params.page.unwrap_or(1).max(1);
    let limit_param = params.limit.unwrap_or(10).max(1);
    let offset_param = (page - 1) * limit_param;

    let sort_by_param_str = params.sort_by.clone().unwrap_or_else(|| "updated_at".to_string());
    let sort_order_param_str = params.sort_order.clone().unwrap_or_else(|| "desc".to_string());

    // --- Transformed Filter Parameters for SQL Binding ---
    let collection_id_for_sql_str: String = params.collection_id.map_or("".to_string(), |id| id.to_string());
    let style_id_for_sql_str: String = params.style_id.map_or("".to_string(), |id| id.to_string());
    let creative_format_id_for_sql_str: String = params.creative_format_id.map_or("".to_string(), |id| id.to_string());

    let document_ids_for_sql_vec_str: Vec<String> = params.document_ids
        .as_ref()
        .filter(|v| !v.is_empty()) 
        .map_or_else(Vec::new, |v_uuid| v_uuid.iter().map(Uuid::to_string).collect());
    // Create a slice for binding:
    let document_ids_for_sql_slice: &[String] = document_ids_for_sql_vec_str.as_slice();


    let search_for_sql_str: String = params.search
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map_or("".to_string(), |s| format!("%{}%", s.to_lowercase()));
    
    let is_favorite_filter = params.is_favorite; // Option<bool>

    // --- Organization and Sharing Setup ---
    let org_memberships = match find_active_memberships_for_user(pool.get_ref(), user_id_param).await {
        Ok(memberships) => memberships,
        Err(e) => {
            log::error!("Failed to fetch organization memberships for user {user_id_param}: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve necessary user data."));
        }
    };
    let org_ids_for_sql_binding: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice: &[Uuid] = org_ids_for_sql_binding.as_slice();
    
    // --- Total Count Query ---
    let total_count_query = conditional_query_as!(
        TotalCount, 
        r#"
        WITH UserOrgMemberships_CTE AS (
            SELECT organization_id FROM organization_members WHERE user_id = {user_id_param} AND status = 'active'
        ),
        RankedShares_CTE AS (
            SELECT
                os.object_id,
                os.access_level,
                ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'creative'
              AND (
                    (os.entity_type = 'user' AND os.entity_id = {user_id_param})
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY({org_ids_slice}))
                )
        ),
        EffectiveShares_CTE AS ( SELECT object_id, access_level FROM RankedShares_CTE WHERE rn = 1 ),
        CollectionShares_CTE AS (
            SELECT
                os.object_id as collection_id,
                os.access_level,
                ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'collection'
              AND (
                    (os.entity_type = 'user' AND os.entity_id = {user_id_param})
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY({org_ids_slice}))
                )
        ),
        EffectiveCollectionShares_CTE AS ( SELECT collection_id, access_level FROM CollectionShares_CTE WHERE rn = 1 )
        SELECT COUNT(DISTINCT c.id) AS "count?" 
        FROM creatives c
        INNER JOIN collections col ON c.collection_id = col.id
        LEFT JOIN EffectiveShares_CTE es ON c.id = es.object_id
        LEFT JOIN EffectiveCollectionShares_CTE ecs ON c.collection_id = ecs.collection_id
        WHERE (col.user_id = {user_id_param} OR es.access_level IS NOT NULL OR ecs.access_level IS NOT NULL)
            AND CASE WHEN {collection_id_for_sql_str} != '' THEN c.collection_id = {collection_id_for_sql_str}::uuid ELSE TRUE END
            AND CASE WHEN {style_id_for_sql_str} != '' THEN c.style_id = {style_id_for_sql_str}::uuid ELSE TRUE END
            AND CASE WHEN {creative_format_id_for_sql_str} != '' THEN c.creative_format_id = {creative_format_id_for_sql_str}::uuid ELSE TRUE END
            AND CASE WHEN array_length({document_ids_for_sql_slice}::TEXT[], 1) > 0 THEN c.document_ids && ({document_ids_for_sql_slice}::uuid[]) ELSE TRUE END
            AND CASE WHEN {search_for_sql_str} != '' THEN (LOWER(col.name) LIKE {search_for_sql_str} OR LOWER(c.html_url) LIKE {search_for_sql_str}) ELSE TRUE END
            AND CASE WHEN {is_favorite_filter}::BOOLEAN IS NOT NULL THEN EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {user_id_param} AND entity_id = c.id AND entity_type = 'creative') = {is_favorite_filter} ELSE TRUE END
        "#,
        #user_id_param = match &user_id_param { _ => "{user_id_param}" }, 
        #org_ids_slice = match &org_ids_slice { _ => "{org_ids_slice}" }, 
        
        #collection_id_for_sql_str = match &collection_id_for_sql_str { _ => "{collection_id_for_sql_str}" },
        #style_id_for_sql_str = match &style_id_for_sql_str { _ => "{style_id_for_sql_str}" },
        #creative_format_id_for_sql_str = match &creative_format_id_for_sql_str { _ => "{creative_format_id_for_sql_str}" },
        #document_ids_for_sql_slice = match &document_ids_for_sql_slice { _ => "{document_ids_for_sql_slice}" }, // MODIFIED to use slice for binding
        #search_for_sql_str = match &search_for_sql_str { _ => "{search_for_sql_str}" },
        #is_favorite_filter = match &is_favorite_filter { _ => "{is_favorite_filter}" }
    );
    
    let total_count_result = total_count_query.fetch_one(pool.get_ref()).await;
    let total_count = match total_count_result {
        Ok(tc) => tc.count.unwrap_or(0),
        Err(e) => {
            log::error!("Error counting creatives (full CASE WHEN with slice): {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to count creatives."));
        }
    };
    
    let default_documents_slice: &[String] = &[];

    #[derive(FromRow, Debug)]
    struct CreativeListItemWithExtras {
        id: Uuid,
        name: Option<String>,
        collection_id: Option<Uuid>, 
        creative_format_id: Option<Uuid>, 
        style_id: Option<Uuid>,
        document_ids: Option<Vec<Uuid>>,
        asset_ids: Option<Vec<Uuid>>,
        html_url: Option<String>,
        draft_url: Option<String>,
        screenshot_url: String,
        is_published: bool,
        publish_url: Option<String>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
        style_name: Option<String>,
        document_names: Option<Vec<String>>,
        creative_format_name: Option<String>,
        collection_name: Option<String>,
        current_user_access_level: Option<String>,
        creator_email: Option<String>,
        is_favorite: Option<bool>,
    }

    // --- Items Query ---
    let items_query = conditional_query_as!(
        CreativeListItemWithExtras,
        r#"
        WITH UserOrgMemberships_CTE AS (
            SELECT organization_id FROM organization_members WHERE user_id = {user_id_param} AND status = 'active'
        ),
        RankedShares_CTE AS (
            SELECT
                os.object_id,
                os.access_level,
                ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'creative'
              AND (
                    (os.entity_type = 'user' AND os.entity_id = {user_id_param})
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY({org_ids_slice}))
                )
        ),
        EffectiveShares_CTE AS ( SELECT object_id, access_level FROM RankedShares_CTE WHERE rn = 1 ),
        CollectionShares_CTE AS (
            SELECT
                os.object_id as collection_id,
                os.access_level,
                ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'collection'
              AND (
                    (os.entity_type = 'user' AND os.entity_id = {user_id_param})
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY({org_ids_slice}))
                )
        ),
        EffectiveCollectionShares_CTE AS ( SELECT collection_id, access_level FROM CollectionShares_CTE WHERE rn = 1 )
        SELECT
            c.id AS "id",
            c.name AS "name?",
            c.collection_id AS "collection_id",
            c.creative_format_id AS "creative_format_id",
            c.style_id AS "style_id?",
            c.document_ids AS "document_ids?",
            c.asset_ids AS "asset_ids?",
            c.html_url AS "html_url?",
            c.draft_url AS "draft_url?",
            c.screenshot_url AS "screenshot_url",
            c.is_published AS "is_published",
            c.publish_url AS "publish_url?",
            c.created_at AS "created_at?",
            c.updated_at AS "updated_at?",
            s.name AS "style_name?",
            COALESCE(
               (SELECT array_agg(ri.title ORDER BY ri.title)
               FROM unnest(c.document_ids) AS rid(id)
               JOIN documents ri ON ri.id = rid.id),
               {default_documents_slice} 
           ) AS "document_names?",
            COALESCE(cf.name, ccf.name) AS "creative_format_name?",
            col.name AS "collection_name?",
            u_creator.email AS "creator_email?",
            CASE
                WHEN col.user_id = {user_id_param} THEN 'owner'::text
                WHEN es.access_level IS NOT NULL THEN es.access_level::text
                ELSE ecs.access_level::text
            END AS "current_user_access_level?",
            COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {user_id_param} AND entity_id = c.id AND entity_type = 'creative')), false) AS is_favorite
        FROM creatives c
        INNER JOIN collections col ON c.collection_id = col.id
        LEFT JOIN users u_creator ON col.user_id = u_creator.id
        LEFT JOIN styles s ON c.style_id = s.id
        LEFT JOIN creative_formats cf ON c.creative_format_id = cf.id
        LEFT JOIN custom_creative_formats ccf ON c.creative_format_id = ccf.id
        LEFT JOIN EffectiveShares_CTE es ON c.id = es.object_id
        LEFT JOIN EffectiveCollectionShares_CTE ecs ON c.collection_id = ecs.collection_id
        WHERE (col.user_id = {user_id_param} OR es.access_level IS NOT NULL OR ecs.access_level IS NOT NULL)
            AND CASE WHEN {collection_id_for_sql_str} != '' THEN c.collection_id = {collection_id_for_sql_str}::uuid ELSE TRUE END
            AND CASE WHEN {style_id_for_sql_str} != '' THEN c.style_id = {style_id_for_sql_str}::uuid ELSE TRUE END
            AND CASE WHEN {creative_format_id_for_sql_str} != '' THEN c.creative_format_id = {creative_format_id_for_sql_str}::uuid ELSE TRUE END
            AND CASE WHEN array_length({document_ids_for_sql_slice}::TEXT[], 1) > 0 THEN c.document_ids && ({document_ids_for_sql_slice}::uuid[]) ELSE TRUE END
            AND CASE WHEN {search_for_sql_str} != '' THEN (LOWER(col.name) LIKE {search_for_sql_str} OR LOWER(c.html_url) LIKE {search_for_sql_str}) ELSE TRUE END
            AND CASE WHEN {is_favorite_filter}::BOOLEAN IS NOT NULL THEN EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {user_id_param} AND entity_id = c.id AND entity_type = 'creative') = {is_favorite_filter} ELSE TRUE END
        ORDER BY {#sort_by_sql_literal} {#sort_order_sql_literal}
        LIMIT {limit_param} OFFSET {offset_param}
        "#,
        // Bind parameters
        #user_id_param = match &user_id_param { _ => "{user_id_param}" },
        #org_ids_slice = match &org_ids_slice { _ => "{org_ids_slice}" },
        
        #collection_id_for_sql_str = match &collection_id_for_sql_str { _ => "{collection_id_for_sql_str}" },
        #style_id_for_sql_str = match &style_id_for_sql_str { _ => "{style_id_for_sql_str}" },
        #creative_format_id_for_sql_str = match &creative_format_id_for_sql_str { _ => "{creative_format_id_for_sql_str}" },
        #document_ids_for_sql_slice = match &document_ids_for_sql_slice { _ => "{document_ids_for_sql_slice}" }, // MODIFIED to use slice for binding
        #search_for_sql_str = match &search_for_sql_str { _ => "{search_for_sql_str}" },
        #is_favorite_filter = match &is_favorite_filter { _ => "{is_favorite_filter}" },

        #default_documents_slice = match &default_documents_slice { _ => "{default_documents_slice}" },

        // SQL Literal substitutions for sorting (these create variants)
        #sort_by_sql_literal = match (sort_by_param_str.as_str(), sort_order_param_str.as_str()) {
            ("name", _) => "LOWER(col.name)", 
            ("created_at", _) => "c.created_at",
            ("updated_at", _) => "c.updated_at",
            _ => "c.updated_at", 
        },
        #sort_order_sql_literal = match sort_order_param_str.as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => "DESC", 
        },
        
        #limit_param = match &limit_param { _ => "{limit_param}" },
        #offset_param = match &offset_param { _ => "{offset_param}" }
    );

    let items_result = items_query.fetch_all(&**pool).await;

    let items = match items_result {
        Ok(list) => list,
        Err(e) => {
            log::error!("Error fetching creatives (full CASE WHEN with slice fix): {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to list creatives.".into(),
            });
        }
    };

    let response_items = items
        .into_iter()
        .map(|item| CreativeListItem {
            id: item.id,
            name: item.name,
            collection_id: item.collection_id,
            creative_format_id: item.creative_format_id,
            style_id: item.style_id,   
            document_ids: item.document_ids,
            asset_ids: item.asset_ids,
            html_url: item.html_url,
            draft_url: item.draft_url,
            screenshot_url: item.screenshot_url,
            is_published: item.is_published,
            publish_url: item.publish_url,
            created_at: item.created_at,
            updated_at: item.updated_at,
            style_name: item.style_name,
            document_names: item.document_names,
            creative_format_name: item.creative_format_name,
            collection_name: item.collection_name,
            current_user_access_level: item.current_user_access_level,
            creator_email: item.creator_email,
            is_favorite: item.is_favorite.unwrap_or(false),
        })
        .collect();

    log::info!("Successfully listed creatives with full CASE WHEN filter optimization (slice fix).");
    HttpResponse::Ok().json(ListCreativesResponse {
        items: response_items,
        total_count,
    })
}