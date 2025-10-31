//! Handler for creating or updating an object share.
use crate::auth::tokens::Claims;
use crate::db::shares::{AccessLevel, EntityType, ObjectShare};
use crate::routes::error_response::ErrorResponse;
use crate::routes::shares::create_share_request::CreateShareRequest;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;
use std::str::FromStr;

// Helper struct to fetch only user_id
#[derive(sqlx::FromRow)]
struct UserIdRow {
    id: Uuid,
}

// Helper to check if user can manage shares for an object (owner or editor)
async fn can_user_manage_object_shares(
    pool: &PgPool, 
    user_id: Uuid,
    object_id: Uuid,
    object_type: &str,
) -> Result<bool, HttpResponse> {
    let is_owner = match object_type {
        "style" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM styles WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking style ownership: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership"))
                })?.unwrap_or(false)
        }
        "creative" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM creatives c JOIN collections col ON c.collection_id = col.id WHERE c.id = $1 AND col.user_id = $2)", object_id, user_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking creative ownership: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership"))
                })?.unwrap_or(false)
        }
        "document" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking document ownership: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership"))
                })?.unwrap_or(false)
        }
        "custom_format" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM custom_creative_formats WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking custom_format ownership: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership"))
                })?.unwrap_or(false)
        }
        "asset" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM assets WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking asset ownership: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership"))
                })?.unwrap_or(false)
        }
        "collection" => {
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM collections WHERE id = $1 AND user_id = $2)", object_id, user_id)
                .fetch_one(pool).await.map_err(|e| {
                    log::error!("DB error checking collection ownership: {e}");
                    HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to check ownership"))
                })?.unwrap_or(false)
        }
        _ => return Err(HttpResponse::BadRequest().json(ErrorResponse::from("Unsupported object type for sharing management."))),
    };

    if is_owner {
        return Ok(true);
    }

    let org_memberships = crate::queries::organizations::find_active_memberships_for_user(pool, user_id).await.map_err(|e|{
        log::error!("DB error fetching org memberships for share management: {e}");
        HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify permissions"))
    })?;
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let org_ids_slice: &[Uuid] = if org_ids.is_empty() { &[] } else { &org_ids };

    let has_editor_share = sqlx::query_scalar!(
        r#"SELECT EXISTS (
            SELECT 1 FROM object_shares
            WHERE object_id = $1 AND object_type = $2 AND access_level = $3
            AND (
                (entity_type = 'user'::object_share_entity_type AND entity_id = $4) OR 
                (entity_type = 'organization'::object_share_entity_type AND entity_id = ANY($5))
            )
        )"#, 
        object_id, 
        object_type,
        AccessLevel::Editor as AccessLevel,
        user_id,
        org_ids_slice
    )
    .fetch_one(pool).await.map_err(|e| {
        log::error!("DB error checking editor share: {e}");
        HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify share permissions"))
    })?.unwrap_or(false);

    Ok(has_editor_share)
}

#[utoipa::path(
    post,
    path = "/api/shares",
    request_body = CreateShareRequest,
    responses(
        (status = 200, description = "Share created/updated", body = ObjectShare),
        (status = 201, description = "Share created/updated", body = ObjectShare),
        (status = 400, description = "Invalid request payload or parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - User cannot manage shares for this object", body = ErrorResponse),
        (status = 404, description = "Object to be shared or target user not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Shares",
    security(("user_auth" = []))
)]
#[post("")]
#[instrument(skip(pool, claims, req))]
pub async fn create_share(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateShareRequest>,
) -> impl Responder {
    let authenticated_user_id = claims.user_id;
    let request_data = req.into_inner();

    log::info!("logremove - Received create_share request. Authenticated User ID: {}, Raw Request Data: {:?}", authenticated_user_id, request_data);

    match request_data.object_type.as_str() {
        "style" | "creative" | "document" | "custom_format" | "asset" | "collection" => (),
        _ => return HttpResponse::BadRequest().json(ErrorResponse::from(format!("Unsupported object_type: {}", request_data.object_type))),
    }

    match can_user_manage_object_shares(&pool, authenticated_user_id, request_data.object_id, &request_data.object_type).await {
        Ok(true) => (),
        Ok(false) => {
            log::warn!("User {} forbidden to manage shares for object {} ({})", authenticated_user_id, request_data.object_id, request_data.object_type);
            return HttpResponse::Forbidden().json(ErrorResponse::from("You do not have permission to share this object or manage its shares."));
        }
        Err(resp) => return resp,
    }
    
    let entity_type_enum = match EntityType::from_str(&request_data.entity_type.to_lowercase()) {
        Ok(et) => et,
        Err(_) => return HttpResponse::BadRequest().json(ErrorResponse::from(format!("Invalid entity_type: {}", request_data.entity_type))),
    };
    let access_level_enum = match AccessLevel::from_str(&request_data.access_level.to_lowercase()) {
        Ok(al) => al,
        Err(_) => return HttpResponse::BadRequest().json(ErrorResponse::from(format!("Invalid access_level: {}", request_data.access_level))),
    };

    let final_entity_id: Uuid;

    match entity_type_enum {
        EntityType::User => {
            match (request_data.entity_id, request_data.entity_email) {
                (Some(id), None) => {
                    final_entity_id = id;
                }
                (None, Some(email)) => {
                    if email.trim().is_empty() {
                        return HttpResponse::BadRequest().json(ErrorResponse::from("User email cannot be empty when entity_id is not provided."));
                    }
                    match sqlx::query_as!(UserIdRow, "SELECT id FROM users WHERE email = $1", email)
                        .fetch_optional(&**pool)
                        .await
                    {
                        Ok(Some(user_row)) => {
                            final_entity_id = user_row.id;
                        }
                        Ok(None) => {
                            log::warn!("User not found for email: {email}");
                            return HttpResponse::NotFound().json(ErrorResponse::from(format!("User with email '{email}' not found.")));
                        }
                        Err(e) => {
                            log::error!("DB error fetching user by email {email}: {e}");
                            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to resolve user email."));
                        }
                    }
                }
                (Some(_), Some(_)) => {
                    return HttpResponse::BadRequest().json(ErrorResponse::from("Provide either entity_id or entity_email for user share, not both."));
                }
                (None, None) => {
                    return HttpResponse::BadRequest().json(ErrorResponse::from("Either entity_id or entity_email must be provided for user share."));
                }
            }
        }
        EntityType::Organization => {
            if let Some(id) = request_data.entity_id {
                final_entity_id = id;
                if request_data.entity_email.is_some() {
                    return HttpResponse::BadRequest().json(ErrorResponse::from("entity_email should not be provided for organization share."));
                }
            } else {
                return HttpResponse::BadRequest().json(ErrorResponse::from("entity_id must be provided for organization share."));
            }
        }
    }

    let mut tx = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(e) => {
            log::error!("Failed to start transaction for creating share: {e}");
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Database error"));
        }
    };

    log::info!(
        "Attempting to create/update share. Object ID: {}, Type: {}, Entity ID: {}, Entity Type: {:?}, Access: {:?}",
        request_data.object_id,
        request_data.object_type,
        final_entity_id, // Log the resolved/validated entity ID
        entity_type_enum,
        access_level_enum
    );

    let result = sqlx::query_as!(
        ObjectShare,
        r#"
        INSERT INTO object_shares (object_id, object_type, entity_id, entity_type, access_level)
        VALUES ($1, $2, $3, $4::object_share_entity_type, $5::object_share_access_level)
        ON CONFLICT (object_id, object_type, entity_id, entity_type) 
        DO UPDATE SET access_level = EXCLUDED.access_level, updated_at = NOW()
        RETURNING id, object_id, object_type, entity_id, entity_type AS "entity_type!: EntityType", access_level AS "access_level!: AccessLevel", created_at, updated_at,
        NULL::text AS entity_name
        "#,
        request_data.object_id,
        request_data.object_type,
        final_entity_id, // Use the resolved ID
        entity_type_enum as EntityType,
        access_level_enum as AccessLevel
    )
    .fetch_one(&mut *tx)
    .await;

    match result {
        Ok(share) => {
            if let Err(e) = tx.commit().await {
                log::error!("Failed to commit transaction for creating share: {e}");
                return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to finalize share creation."));
            }
            HttpResponse::Ok().json(share)
        }
        Err(e) => {
            if let Err(rb_err) = tx.rollback().await { 
                log::error!("Failed to rollback transaction for creating share: {rb_err}");
            }
            log::error!("DB error creating/updating share: {e}");
            if let Some(db_err) = e.as_database_error() {
                // Example: Check for a specific constraint name related to object_id existence.
                // The actual constraint name might differ based on your schema.
                if db_err.constraint().is_some_and(|c| c.ends_with("_object_id_fkey")) {
                    return HttpResponse::NotFound().json(ErrorResponse::from("Object to be shared not found."));
                }
            }
            HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to create or update share."))
        }
    }
} 
