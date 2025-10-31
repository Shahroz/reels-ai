//! Handles the editing of an existing creative's HTML content using LLM instructions.
//!
//! This endpoint allows users to submit textual instructions to modify the HTML
//! of a creative they own. The system fetches the current HTML, sends it along
//! with the instruction to an LLM, validates the LLM's output, saves the new
//! HTML to GCS as a draft, and updates the creative's record in the database.

use actix_web::{post, web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use llm::llm_typed_unified::llm::llm;
use llm::llm_typed_unified::vendor_model::VendorModel;
use llm::vendors::gemini::gemini_model::GeminiModel;
use crate::auth::tokens::Claims as ValidatedClaims; // Using Claims as it implements FromRequest
use crate::queries::organizations::find_active_memberships_for_user;
use crate::routes::creatives::edit_creative_request::EditCreativeRequest;
use crate::routes::creatives::responses::CreativeResponse;
use crate::routes::error_response::ErrorResponse;
use crate::utils::sanitize_llm_html_output;
use crate::services::gcs::parse_gcs_url::parse_gcs_url;

const LLM_RETRY_ATTEMPTS: usize = 3;
const MIN_HTML_LENGTH: usize = 100;

#[utoipa::path(
    post,
    path = "/api/creatives/{id}/edit",
    tag = "creatives",
    request_body = EditCreativeRequest,
    params(
        ("id" = uuid::Uuid, Path, description = "Creative ID")
    ),
    responses(
        (status = 200, description = "Creative edited successfully", body = CreativeResponse),
        (status = 404, description = "Creative not found or access denied", body = ErrorResponse),
        (status = 500, description = "Server error during creative editing process", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(
    skip(request, pool, gcs_client, claims),
    fields(creative_id = %path.as_ref(), user_id = %claims.user_id)
)]
#[post("/{id}/edit")]
pub async fn edit_creative(
    path: web::Path<Uuid>,
    request: web::Json<EditCreativeRequest>,
    pool: web::Data<PgPool>,
    gcs_client: web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    claims: ValidatedClaims,
) -> impl actix_web::Responder {
    let creative_id = path.into_inner();
    let user_id = claims.user_id;

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
            // No need to rollback tx here as this operation is outside of it.
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to retrieve necessary user data"));
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    // 1. Fetch Creative Info and Verify Permissions (Owner or Editor) (within the transaction)
    struct PermissionCheckResult {
        creative_id: Uuid,
        name: String,
        collection_id: Option<Uuid>,
        html_url: String,
        draft_url: Option<String>,
        screenshot_url: String,
        is_published: bool,
        publish_url: Option<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        creative_format_id: Uuid, 
        style_id: Option<Uuid>, 
        document_ids: Option<Vec<Uuid>>, 
        asset_ids: Option<Vec<Uuid>>,
        owner_user_id: Uuid,
        creator_email_val: Option<String>, // Renamed to avoid conflict
        effective_access_level: Option<String>,
        bundle_id: Option<Uuid>,
    }

    let perm_check_result = match sqlx::query_as!(
        PermissionCheckResult,
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
                        WHEN 'viewer' THEN 2
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
            c.id as creative_id, c.name, c.collection_id, c.html_url, c.draft_url, c.screenshot_url, 
            c.is_published, c.publish_url, c.created_at,
            c.creative_format_id, c.style_id, c.document_ids, c.asset_ids, 
            col.user_id AS owner_user_id, 
            u_creator.email AS creator_email_val,
            COALESCE(
                creative_share.access_level::TEXT,
                collection_share.access_level::TEXT
            ) AS effective_access_level,
            c.bundle_id
        FROM creatives c
        INNER JOIN collections col ON c.collection_id = col.id
        LEFT JOIN users u_creator ON col.user_id = u_creator.id
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
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction: {}", rb_err);
            }
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Creative not found".to_string(),
            });
        }
        Err(e) => {
            tracing::error!("Permission check query failed for creative {}: {}", creative_id, e);
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction: {}", rb_err);
            }
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify creative access"));
       }
   };

    let determined_access_level: String;
    if perm_check_result.owner_user_id == user_id {
        determined_access_level = "owner".to_string();
    } else if let Some(access_level) = &perm_check_result.effective_access_level {
        if access_level == "editor" {
            determined_access_level = "editor".to_string();
        } else {
            tracing::warn!(
                "User {} does not have editor permission for creative {}. Access level: {}. Denying edit.",
                user_id, creative_id, access_level
            );
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction: {}", rb_err);
            }
            return HttpResponse::Forbidden().json(ErrorResponse::from("Permission denied to edit creative"));
        }
    } else {
        tracing::warn!(
            "User {} is not owner and no share found for creative {}. Denying edit.",
            user_id, creative_id
        );
        if let Err(rb_err) = tx.rollback().await {
            tracing::error!("Failed to rollback transaction: {}", rb_err);
        }
        return HttpResponse::Forbidden().json(ErrorResponse::from("Permission denied to edit creative"));
    }

   // 2. Fetch Existing HTML (prioritize request.html_content, then draft_url, then html_url)
   let existing_html_content = if let Some(content) = &request.html_content {
       content.clone() // Use content from request if provided
   } else {
       let source_html_url = perm_check_result.draft_url.as_deref().unwrap_or(&perm_check_result.html_url);
       match parse_gcs_url(source_html_url) {
           Ok((bucket_name, object_name)) => {
               match gcs_client.get_ref().as_ref().download_object_as_string(&bucket_name, &object_name).await {
                   Ok(html) => html,
                   Err(e) => {
                       tracing::error!("Failed to download HTML content from GCS bucket '{}', object '{}': {:?}", bucket_name, object_name, e);
                       return HttpResponse::InternalServerError().json(ErrorResponse {
                           error: "Failed to read existing creative HTML from storage".to_string(),
                       });
                   }
               }
           }
           Err(e) => {
               tracing::error!("Failed to parse HTML URL '{}': {}", source_html_url, e);
               return HttpResponse::InternalServerError().json(ErrorResponse {
                   error: "Invalid creative HTML URL format".to_string(),
               });
           }
       }
  };

  let final_html_to_upload: std::string::String;

  if request.instruction.trim().is_empty() {
      tracing::info!(
          "Instruction is empty for creative_id {}. Skipping LLM processing and using existing/provided HTML.",
          creative_id
      );
      final_html_to_upload = existing_html_content;
  } else {
  // 3. Construct LLM Prompt
  let user_instruction = &request.instruction;
    let prompt = format!(
        "<EXISTING_HTML>\n{existing_html_content}\n</EXISTING_HTML>\n<INSTRUCTION_TEXT>\n{user_instruction}\n</INSTRUCTION_TEXT>\n<TASK>Based on the EXISTING_HTML, apply the INSTRUCTION_TEXT and return ONLY the modified raw HTML. Do not include comments or explanations. Ensure the output is a complete, self-contained HTML document. The response must be pure HTML, without any markdown code fences like ```html or similar.</TASK>"
    );

    // 4. Call LLM and Validate
    let models_to_try: std::vec::Vec<VendorModel> = vec![
        VendorModel::Gemini(GeminiModel::Gemini25Flash),
        VendorModel::Gemini(GeminiModel::Gemini25Pro),
    ];

    let mut validated_html: Option<std::string::String> = None;

    for attempt in 0..LLM_RETRY_ATTEMPTS {
        tracing::info!(
            "LLM attempt {}/{} for creative_id {} with instruction: \"{}\"",
            attempt + 1,
           LLM_RETRY_ATTEMPTS,
            creative_id,
            user_instruction
       );
        match llm(false, &prompt, models_to_try.clone(), 5).await {
            Ok(modified_html_string) => {
                // Use the new sanitizer function
                // The sanitize_llm_html_output function ensures that if Some(html) is returned,
                // html is a non-empty string that starts with '<' and ends with '>'.
                match sanitize_llm_html_output::sanitize_llm_html_output(&modified_html_string) {
                    Some(sanitized_content) => {
                        // Sanitizer ensures basic HTML structure. Now, apply business-specific validation.
                        if sanitized_content.len() >= MIN_HTML_LENGTH {
                            validated_html = Some(sanitized_content);
                            break; // Exit LLM retry loop
                        } else {
                            tracing::warn!(
                                "Sanitized LLM output failed length check for creative_id {}. Length: {}, Required: {}. Output: {:.100}",
                                creative_id,
                                sanitized_content.len(),
                                MIN_HTML_LENGTH,
                                sanitized_content
                            );
                        }
                    }
                    None => { // Sanitization returned None, meaning it couldn't be processed into valid-looking HTML
                        tracing::warn!(
                            "LLM output sanitization failed (sanitize_llm_html_output returned None) for creative_id {}. Original Output: {:.100}",
                            creative_id,
                            modified_html_string
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!("LLM call failed for creative_id {}: {}", creative_id, e);
            }
        }
    }

    final_html_to_upload = match validated_html {
        Some(html) => html,
        None => {
            tracing::error!("Failed to generate valid HTML after {} attempts for creative_id {}", LLM_RETRY_ATTEMPTS, creative_id);
            // NARRATIV: Rollback transaction before returning
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction after LLM failure: {}", rb_err);
            }
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to generate valid HTML after multiple attempts".to_string(),
            });
        }
    };
  }

    // 5. Save to GCS
    let bucket_name = match std::env::var("GCS_BUCKET") {
        Ok(name) => name,
        Err(_) => {
            tracing::error!("GCS_BUCKET environment variable not set.");
            // NARRATIV: Rollback transaction before returning
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction due to GCS_BUCKET env var missing: {}", rb_err);
            }
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Server configuration error: GCS bucket not specified".to_string(),
            });
        }
    };
   // Generate a random suffix for the draft filename to avoid CDN caching issues.
   // Takes the first 8 characters of a new v4 UUID.
   let random_suffix = uuid::Uuid::new_v4().to_string().chars().take(8).collect::<std::string::String>();
   let object_name = format!("creatives/{creative_id}/draft_{random_suffix}.html");

    let draft_gcs_url = match gcs_client.get_ref().as_ref()
        .upload_raw_bytes(
            &bucket_name,
            &object_name,
            "text/html",
            final_html_to_upload.as_bytes().to_vec(),
            true,
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic
        )
        .await
    {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("Failed to upload to GCS for creative_id {}: {}", creative_id, e);
            // NARRATIV: Rollback transaction before returning
            if let Err(rb_err) = tx.rollback().await {
                tracing::error!("Failed to rollback transaction after GCS upload failure: {}", rb_err);
            }
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to save modified creative content".to_string(),
            });
        }
    };

    // 5. Update Database within the transaction
    let update_result = sqlx::query!(
        "UPDATE creatives SET draft_url = $1, updated_at = NOW() WHERE id = $2",
        draft_gcs_url,
        creative_id
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = update_result {
        tracing::error!("Failed to update creative with new draft_url: {}", e);
        if let Err(rb_err) = tx.rollback().await {
            tracing::error!("Failed to rollback transaction: {}", rb_err);
        }
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to update creative record"));
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to finalize creative update"));
    }
    
    // 6. Return the final Creative object in the response
    let final_creative_response = CreativeResponse {
        creative: crate::db::creatives::Creative {
            id: perm_check_result.creative_id,
            name: perm_check_result.name,
            collection_id: perm_check_result.collection_id,
            creative_format_id: perm_check_result.creative_format_id,
            style_id: perm_check_result.style_id,
            document_ids: perm_check_result.document_ids,
            asset_ids: perm_check_result.asset_ids,
            html_url: perm_check_result.html_url,
            draft_url: Some(draft_gcs_url), // The new draft URL
            bundle_id: perm_check_result.bundle_id,
            screenshot_url: perm_check_result.screenshot_url,
            is_published: perm_check_result.is_published,
            publish_url: perm_check_result.publish_url,
            created_at: perm_check_result.created_at,
            updated_at: chrono::Utc::now(), // Reflect the update time
        },
        creator_email: perm_check_result.creator_email_val,
        current_user_access_level: Some(determined_access_level),
    };

    HttpResponse::Ok().json(final_creative_response)
}

#[cfg(test)]
mod tests {
    // Basic tests for handlers like this are typically integration tests.
    // Unit tests would require significant mocking of DB, GCS, LLM, and HTTP clients.
    // For now, we ensure the file compiles and basic structure is present.
    #[test]
    fn placeholder_test() {
        assert!(true);
    }
}
