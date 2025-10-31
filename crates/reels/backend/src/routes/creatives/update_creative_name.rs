//! Handles updating the name of an existing creative.
//!
//! This endpoint allows users to update the name of a creative they own or have editor access to.
//! The system verifies permissions, updates the name in the database, and returns the updated creative.

use actix_web::{patch, web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use crate::auth::tokens::Claims as ValidatedClaims;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::routes::creatives::update_creative_name_request::UpdateCreativeNameRequest;
use crate::routes::creatives::responses::CreativeResponse;
use crate::routes::error_response::ErrorResponse;
use crate::db::creatives::Creative;

#[utoipa::path(
    patch,
    path = "/api/creatives/{id}/name",
    tag = "creatives",
    request_body = UpdateCreativeNameRequest,
    params(
        ("id" = uuid::Uuid, Path, description = "Creative ID")
    ),
    responses(
        (status = 200, description = "Creative name updated successfully", body = CreativeResponse),
        (status = 400, description = "Bad request - invalid name", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - no permission to edit", body = ErrorResponse),
        (status = 404, description = "Creative not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(
    skip(request, pool, claims),
    fields(creative_id = %path.as_ref(), user_id = %claims.user_id)
)]
#[patch("/{id}/name")]
pub async fn update_creative_name(
    path: web::Path<Uuid>,
    request: web::Json<UpdateCreativeNameRequest>,
    pool: web::Data<PgPool>,
    claims: ValidatedClaims,
) -> impl actix_web::Responder {
    let creative_id = path.into_inner();
    let user_id = claims.user_id;
    let new_name = request.name.trim();

    // Validate name
    if new_name.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse::from("Creative name cannot be empty"));
    }

    if new_name.len() > 255 {
        return HttpResponse::BadRequest().json(ErrorResponse::from("Creative name cannot exceed 255 characters"));
    }

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin database transaction: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to start database operation"));
        }
    };

    // Fetch organization memberships using the pool, not the transaction
    let org_memberships = match find_active_memberships_for_user(&pool, user_id).await {
        Ok(memberships) => memberships,
        Err(e) => {
            tracing::error!("Failed to fetch organization memberships for user {}: {}", user_id, e);
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve necessary user data"));
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    // Atomic update with permission check - prevents race conditions
    let update_result = match sqlx::query!(
        r#"
        UPDATE creatives 
        SET name = $1, updated_at = NOW()
        WHERE id = $2 
          AND EXISTS (
              SELECT 1 FROM collections col 
              WHERE col.id = creatives.collection_id 
                AND (
                    -- Owner check
                    col.user_id = $3
                    OR
                    -- Direct creative editor share
                    EXISTS (
                        SELECT 1 FROM object_shares os 
                        WHERE os.object_type = 'creative' AND os.object_id = $2
                          AND os.access_level = 'editor'
                          AND (
                                (os.entity_type = 'user' AND os.entity_id = $3) 
                                OR 
                                (os.entity_type = 'organization' AND os.entity_id = ANY($4::UUID[]))
                          )
                    )
                    OR
                    -- Collection editor share
                    EXISTS (
                        SELECT 1 FROM object_shares os 
                        WHERE os.object_type = 'collection' AND os.object_id = col.id
                          AND os.access_level = 'editor'
                          AND (
                                (os.entity_type = 'user' AND os.entity_id = $3) 
                                OR 
                                (os.entity_type = 'organization' AND os.entity_id = ANY($4::UUID[]))
                          )
                    )
                )
          )
        "#,
        new_name,      // $1
        creative_id,   // $2  
        user_id,       // $3
        &org_ids       // $4
    )
    .execute(&mut *tx)
    .await
    {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Failed to update creative name for creative {}: {}", creative_id, e);
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction: {}", rb_err);
            }
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to update creative name"));
        }
    };

    // Check if the update actually affected any rows
    if update_result.rows_affected() == 0 {
        if let Err(rb_err) = tx.rollback().await {
            tracing::error!("Failed to rollback transaction: {}", rb_err);
        }
        return HttpResponse::NotFound().json(ErrorResponse::from("Creative not found or permission denied"));
    }

    // Fetch the updated creative and related data for the response
    let creative_data = match sqlx::query!(
        r#"
        SELECT 
            c.id, c.name, c.collection_id, c.html_url, c.draft_url, c.screenshot_url, 
            c.is_published, c.publish_url, c.created_at, c.updated_at,
            c.creative_format_id, c.style_id, c.document_ids, c.asset_ids, c.bundle_id,
            col.user_id AS owner_user_id, 
            u_creator.email AS "creator_email?",
            CASE
                WHEN col.user_id = $2 THEN 'owner'::text
                ELSE COALESCE(
                    (SELECT os.access_level::text FROM object_shares os 
                     WHERE os.object_type = 'creative' AND os.object_id = $1
                       AND ((os.entity_type = 'user' AND os.entity_id = $2) 
                            OR (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[])))
                     ORDER BY CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                     LIMIT 1),
                    (SELECT os.access_level::text FROM object_shares os 
                     WHERE os.object_type = 'collection' AND os.object_id = col.id
                       AND ((os.entity_type = 'user' AND os.entity_id = $2) 
                            OR (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[])))
                     ORDER BY CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                     LIMIT 1)
                )
            END AS current_user_access_level
        FROM creatives c
        INNER JOIN collections col ON c.collection_id = col.id
        LEFT JOIN users u_creator ON col.user_id = u_creator.id
        WHERE c.id = $1
        "#,
        creative_id,  // $1
        user_id,      // $2
        &org_ids      // $3
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to fetch updated creative data for creative {}: {}", creative_id, e);
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction: {}", rb_err);
            }
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve updated creative"));
        }
    };

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to save changes"));
    }

    // Build the response
    let creative = Creative {
        id: creative_data.id,
        name: creative_data.name,
        collection_id: creative_data.collection_id,
        creative_format_id: creative_data.creative_format_id,
        style_id: creative_data.style_id,
        document_ids: creative_data.document_ids,
        asset_ids: creative_data.asset_ids,
        html_url: creative_data.html_url,
        draft_url: creative_data.draft_url,
        bundle_id: creative_data.bundle_id,
        screenshot_url: creative_data.screenshot_url,
        is_published: creative_data.is_published,
        publish_url: creative_data.publish_url,
        created_at: creative_data.created_at,
        updated_at: creative_data.updated_at,
    };

    let response = CreativeResponse {
        creative,
        creator_email: creative_data.creator_email,
        current_user_access_level: creative_data.current_user_access_level,
    };

    HttpResponse::Ok().json(response)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Placeholder test to ensure the module compiles
        assert!(true);
    }
} 