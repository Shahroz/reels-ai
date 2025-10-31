//! Handler for listing object shares.
use crate::auth::tokens::Claims;
use crate::auth::permissions::check_is_org_owner_or_admin;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::db::shares::{AccessLevel, EntityType, ObjectShare};
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use utoipa::{ToSchema, IntoParams};
use uuid::Uuid;
use std::str::FromStr;
use sqlx::QueryBuilder;

#[derive(Deserialize, Debug, ToSchema, IntoParams)]
pub struct ListSharesParams {
    #[schema(example = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx")]
    pub object_id: Option<Uuid>,
    #[schema(example = "style")]
    pub object_type: Option<String>,
    #[schema(example = "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy")]
    pub entity_id: Option<Uuid>,
    #[schema(example = "user")]
    pub entity_type: Option<String>,
    // TODO: Add pagination and sorting if needed in the future
}

#[derive(Serialize, ToSchema)]
pub struct ListSharesResponse {
    items: Vec<ObjectShare>,
}

// Helper to check if user can list shares for a specific object.
// (Owner of the object or has an 'editor' share on it)
async fn can_user_list_shares_for_object(
    pool: &PgPool,
    user_id: Uuid,
    object_id: Uuid,
    object_type: &str,
) -> Result<bool, HttpResponse> {
    // Check direct ownership (re-uses logic from create_share - could be refactored)
    let is_owner = match object_type {
        "style" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM styles WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking style ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        "creative" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM creatives c JOIN collections col ON c.collection_id = col.id WHERE c.id = $1 AND col.user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking creative ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        "document" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking document ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        // "research" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM research WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking research ownership: {}", e); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        // "research_workflow" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM research_workflows WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking research_workflow ownership: {}", e); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        "custom_format" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM custom_creative_formats WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking custom_format ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        "asset" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM assets WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking asset ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        "collection" => sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM collections WHERE id = $1 AND user_id = $2)", object_id, user_id).fetch_one(pool).await.map_err(|e| { log::error!("DB error checking collection ownership: {e}"); HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership")) })?.unwrap_or(false),
        _ => return Err(HttpResponse::BadRequest().json(ErrorResponse::from("Unsupported object type for share listing."))),
    };

    if is_owner {
        return Ok(true);
    }

    // Check for 'editor' share
    let org_memberships = find_active_memberships_for_user(pool, user_id).await.map_err(|e|{
        log::error!("DB error fetching org memberships for share listing: {e}");
        HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify permissions for share listing"))
    })?;
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice: &[Uuid] = if org_ids.is_empty() { &[] } else { &org_ids };

    let has_editor_share = sqlx::query_scalar!(
        r#"SELECT EXISTS (
            SELECT 1 FROM object_shares
            WHERE object_id = $1 AND object_type = $2 AND access_level = $3
            AND (
                (entity_type = 'user' AND entity_id = $4) OR 
                (entity_type = 'organization' AND entity_id = ANY($5))
            )
        )"#, 
        object_id, 
        object_type,
        AccessLevel::Editor as AccessLevel,
        user_id,
        org_ids_slice
    )
    .fetch_one(pool).await.map_err(|e| {
        log::error!("DB error checking editor share for listing: {e}");
        HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify share permissions for listing"))
    })?.unwrap_or(false);

    Ok(has_editor_share)
}


#[utoipa::path(
    get,
    path = "/api/shares",
    params(ListSharesParams),
    responses(
        (status = 200, description = "List of shares", body = ListSharesResponse),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden to list shares with these parameters", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Shares",
    security(("user_auth" = []))
)]
#[get("")]
#[instrument(skip_all)]
pub async fn list_shares(
    pool: web::Data<PgPool>,
    claims: Option<web::ReqData<Claims>>,
    params: web::Query<ListSharesParams>,
) -> impl Responder {
    log::info!("logremove - list_shares handler invoked. Query params: {:?}", params.0);

    let object_id = params.object_id;
    let object_type = params.object_type.clone();
    let entity_id = params.entity_id;
    let entity_type = params.entity_type.clone();

    log::info!("logremove - Extracted params: object_id={:?}, object_type={:?}, entity_id={:?}, entity_type={:?}", object_id, &object_type, entity_id, &entity_type);

    let mut query_builder = QueryBuilder::new(
        r#"SELECT 
            os.id, os.object_id, os.object_type, os.entity_id, os.entity_type, os.access_level, 
            os.created_at, os.updated_at,
            CASE
                WHEN os.entity_type = 'user'::object_share_entity_type THEN u.email
                WHEN os.entity_type = 'organization'::object_share_entity_type THEN o.name
                ELSE NULL
            END AS entity_name
        FROM object_shares os
        LEFT JOIN users u ON os.entity_id = u.id AND os.entity_type = 'user'::object_share_entity_type
        LEFT JOIN organizations o ON os.entity_id = o.id AND os.entity_type = 'organization'::object_share_entity_type
        WHERE "#
    );

    let mut conditions_added = false;

    if let (Some(obj_id), Some(obj_type)) = (object_id, &object_type) {
        let user_id = match claims {
            Some(c) => {
                log::info!("logremove - Claims present, user_id: {}", c.user_id);
                c.user_id
            }
            None => {
                log::info!("logremove - No claims present, but required for listing shares by object.");
                return HttpResponse::Unauthorized().json(ErrorResponse::from("Authentication required to list shares for an object."));
            }
        };
        // Listing shares for a specific object
        match can_user_list_shares_for_object(&pool, user_id, obj_id, obj_type).await {
            Ok(true) => (),
            Ok(false) => {
                log::warn!("User {user_id} forbidden to list shares for object {obj_id} ({obj_type})");
                return HttpResponse::Forbidden().json(ErrorResponse::from("You do not have permission to view shares for this object."));
            }
            Err(resp) => return resp,
        }
        log::info!("logremove - Listing shares by object: object_id={obj_id}, object_type={obj_type}");
        query_builder.push("os.object_id = ");
        query_builder.push_bind(obj_id);
        query_builder.push(" AND os.object_type = ");
        query_builder.push_bind(obj_type.clone());
        conditions_added = true;
    } else if let (Some(ent_id), Some(ent_type)) = (entity_id, &entity_type) {
        if let Some(ref c) = claims {
             log::info!("logremove - Claims present, user_id: {} for entity listing.", c.user_id);
            // Listing objects shared with a specific entity
            let entity_type = match EntityType::from_str(&ent_type.to_lowercase()){
                Ok(et) => et,
                Err(_) => return HttpResponse::BadRequest().json(ErrorResponse::from(format!("Invalid entity_type for filtering: {ent_type}"))),
            };

            match entity_type {
                EntityType::User => {
                    if ent_id != c.user_id {
                        log::warn!("User {} forbidden to list shares for user {}", c.user_id, ent_id);
                        return HttpResponse::Forbidden().json(ErrorResponse::from("You can only list shares directly granted to yourself."));
                    }
                }
                EntityType::Organization => {
                    // User must be an owner/admin of the organization to see what's shared with it.
                    // (Or a member, if we decide members can see all org shares - current plan says owner/admin)
                    // For now, using owner/admin check.
                    match check_is_org_owner_or_admin(&pool, c.user_id, ent_id).await {
                        Ok(true) => (),
                        Ok(false) => {
                             log::warn!("User {} forbidden to list shares for organization {}", c.user_id, ent_id);
                             return HttpResponse::Forbidden().json(ErrorResponse::from("You do not have permission to view shares for this organization."));
                        }
                        Err(_e) => {
                            log::error!("DB error checking org ownership for listing shares.");
                            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify permissions"));
                        }
                    }
                }
            }
            log::info!("logremove - Listing shares by entity: entity_id={ent_id}, entity_type={ent_type}");
            query_builder.push("os.entity_id = ");
            query_builder.push_bind(ent_id);
            query_builder.push(" AND os.entity_type = ");
            query_builder.push_bind(ent_type.as_str());
            conditions_added = true;
        } else {
            log::info!("logremove - No claims present for entity listing. This might be a public listing or an error if specific entity claims are normally expected.");
        }
    } else {
        log::warn!("logremove - Invalid parameters for listing shares. Neither (object_id, object_type) nor (entity_id, entity_type) were fully provided.");
        return HttpResponse::BadRequest().json(ErrorResponse::from(
            "Either (object_id, object_type) or (entity_id, entity_type) must be provided.",
        ));
    }

    if !conditions_added {
        // This case should ideally be caught by the logic above, but as a safeguard:
        log::warn!("logremove - No query conditions were added. Aborting.");
        return HttpResponse::BadRequest().json(ErrorResponse::from("Invalid query parameters."));
    }
    
    log::info!("logremove - Final query SQL before execution: {:?}", query_builder.sql());

    let shares_result = query_builder
        .build_query_as::<ObjectShare>()
        .fetch_all(&**pool)
        .await;

    match shares_result {
        Ok(shares) => {
            log::info!("logremove - Successfully fetched {} shares from DB.", shares.len());
            let response = ListSharesResponse { items: shares };
            log::info!("logremove - Returning 200 OK with ListSharesResponse.");
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("logremove - DB error fetching shares: {e:?}");
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to list shares."))
        }
    }
} 