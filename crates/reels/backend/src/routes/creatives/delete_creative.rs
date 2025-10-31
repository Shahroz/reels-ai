//! Handler for deleting a creative.
//!
//! DELETE /api/creatives/{id}
//! Requires authentication and checks user ownership via the associated collection.

use crate::routes::error_response::ErrorResponse; // Import the standard error response struct
use crate::auth::tokens::Claims; // Add Claims for auth context
use tracing::instrument;

// NARRATIV: Added for permission check and transaction
use crate::queries::organizations::find_active_memberships_for_user;
use sqlx::PgPool;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/api/creatives/{id}",
    responses(
        (status = 204, description = "No Content"),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "Creatives",
    security( // Add security requirement
        ("bearer_auth" = [])
    )
)]
#[actix_web::delete("/{id}")]
#[instrument(skip(pool, id, claims))]
pub async fn delete_creative(
    pool: actix_web::web::Data<PgPool>,
    id: actix_web::web::Path<Uuid>,
    claims: Claims,
) -> impl actix_web::Responder {
    let creative_id = *id;
    let user_id = claims.user_id;

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!("Failed to begin database transaction: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to start database operation"));
        }
    };

    // Fetch organization memberships using the pool, not the transaction
    let org_memberships = match find_active_memberships_for_user(&pool, user_id).await {
        Ok(memberships) => memberships,
        Err(e) => {
            tracing::error!("Failed to fetch organization memberships for user {}: {}", user_id, e);
            // No need to rollback tx here as this operation is outside of it.
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve necessary user data"));
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    struct PermissionQueryResult {
        _owner_user_id: Uuid, // Used for internal check, not directly returned
        _effective_access_level: Option<String>,
    }

    let permission_query_result = match sqlx::query_as!(
        PermissionQueryResult,
        r#"
        WITH UserOrgMemberships_CTE AS (
            SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'
        ),
        RankedShares_CTE AS (
            SELECT
                os.object_id,
                os.access_level,
                ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                    CASE os.access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2 -- Though viewer can't delete, good for CTE structure
                        ELSE 3
                    END
                ) as rn
            FROM object_shares os
            WHERE (
                    (os.object_type = 'creative' AND os.object_id = $2)
                    OR 
                    (os.object_type = 'collection' AND os.object_id = (SELECT collection_id FROM creatives WHERE id = $2))
                )
              AND (
                    (os.entity_type = 'user' AND os.entity_id = $1)
                    OR
                    (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
                )
        ),
        EffectiveShares_CTE AS (
            SELECT object_id, access_level FROM RankedShares_CTE WHERE rn = 1
        )
        SELECT 
            col.user_id AS _owner_user_id, 
            COALESCE(
                creative_share.access_level::TEXT,
                collection_share.access_level::TEXT
            ) AS _effective_access_level
        FROM creatives c
        INNER JOIN collections col ON c.collection_id = col.id
        LEFT JOIN EffectiveShares_CTE creative_share ON c.id = creative_share.object_id
        LEFT JOIN EffectiveShares_CTE collection_share ON col.id = collection_share.object_id
        WHERE c.id = $2
        "#,
        user_id,      // $1
        creative_id,  // $2
        &org_ids      // $3
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(res)) => res,
        Ok(None) => {
            if let Err(rb_err) = tx.rollback().await { tracing::error!("Transaction rollback failed: {}", rb_err); }
            return actix_web::HttpResponse::NotFound().json(ErrorResponse::from("Creative not found"));
        }
        Err(e) => {
            tracing::error!("Permission check query failed for creative {}: {}", creative_id, e);
            if let Err(rb_err) = tx.rollback().await { tracing::error!("Transaction rollback failed: {}", rb_err); }
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify creative access"));
       }
   };

    if !(permission_query_result._owner_user_id == user_id || permission_query_result._effective_access_level.as_deref() == Some("editor")) {
        tracing::warn!(
            "User {} does not have permission to delete creative {}. Owner: {}, Share: {:?}. Denying.",
            user_id, creative_id, permission_query_result._owner_user_id, permission_query_result._effective_access_level
        );
        if let Err(rb_err) = tx.rollback().await { tracing::error!("Transaction rollback failed: {}", rb_err); }
        return actix_web::HttpResponse::Forbidden().json(ErrorResponse::from("Permission denied to delete creative"));
    }

    // TODO: GCS Deletion - Add logic to delete associated files from GCS (e.g., html_url, draft_url, screenshot_url)
    // This might involve fetching the creative's URLs before deleting its record.
    // For now, proceeding with DB deletion only.

    // 2. Delete from object_shares
    if let Err(e) = sqlx::query!(
        "DELETE FROM object_shares WHERE object_id = $1 AND object_type = 'creative'",
        creative_id
    )
    .execute(&mut *tx)
    .await {
        tracing::error!("Failed to delete object shares for creative {}: {}", creative_id, e);
        if let Err(rb_err) = tx.rollback().await { tracing::error!("Transaction rollback failed: {}", rb_err); }
        return actix_web::HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to delete creative sharing links"));
    }

    // 3. Delete the creative itself
    match sqlx::query!(
        "DELETE FROM creatives WHERE id = $1",
        creative_id
    )
    .execute(&mut *tx)
    .await
    {
        Ok(exec_result) if exec_result.rows_affected() > 0 => {
            // Commit the transaction
            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for deleting creative {}: {}", creative_id, e);
                return actix_web::HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to finalize creative deletion"));
            }
            actix_web::HttpResponse::NoContent().finish()
        }
        Ok(_) => { // Should have been caught by permission check if not found
            tracing::warn!("Creative {} not found during final delete step, though permission check passed.", creative_id);
            if let Err(rb_err) = tx.rollback().await { tracing::error!("Transaction rollback failed: {}", rb_err); }
            actix_web::HttpResponse::NotFound().json(ErrorResponse::from("Creative not found"))
        }
        Err(e) => {
            tracing::error!("Failed to delete creative {}: {}", creative_id, e);
            if let Err(rb_err) = tx.rollback().await { tracing::error!("Transaction rollback failed: {}", rb_err); }
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to delete creative"))
        }
    }
}