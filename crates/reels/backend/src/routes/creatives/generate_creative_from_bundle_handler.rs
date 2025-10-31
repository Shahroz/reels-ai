//! Handler for generating and saving a new creative from a bundle using LLM.
//!
//! POST /api/creatives/generate_from_bundle
//! Fetches a Bundle, then its associated style, assets, and documents.
//! Combines bundle documents with optional documents from the payload.
//! Fetches creative formats based on payload IDs.
//! Constructs a prompt, calls LLM, saves the creative (linking it to the bundle), and returns it.

use crate::db::assets::Asset;
use crate::db::creatives::Creative;
use crate::db::custom_creative_formats::CustomCreativeFormat;
use crate::db::styles::Style;
use crate::middleware::auth::AuthenticatedUser;
use crate::queries::user_credit_allocation::{deduct_user_credits_with_transaction, CreditChangesParams};
use bigdecimal::BigDecimal;
use crate::routes::creatives::creative_asset_utils::upload_creative_assets;
use crate::routes::creatives::generate_creative_from_bundle_request::GenerateCreativeFromBundleRequest;
use crate::routes::creatives::responses::CreativeResponse;
use crate::routes::error_response::ErrorResponse;
use crate::services::gcs::parse_gcs_url::parse_gcs_url;
use llm::llm_typed_unified::vendor_model::VendorModel;
use llm::vendors::gemini::gemini_model::GeminiModel;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// Internal temporary struct to hold combined format data for context generation
// Copied from generate_creative.rs
#[derive(Debug)]
struct CombinedFormatInfo {
    id: Uuid,
    name: String,
    description: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    metadata: Option<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/api/creatives/generate_from_bundle",
    request_body = GenerateCreativeFromBundleRequest,
    params(
        ("x-organization-id" = Option<String>, Header, description = "Optional organization ID to deduct credits from organization instead of user")
    ),
    responses(
        (status = 201, description = "Creative generated successfully from bundle", body = CreativeResponse),
        (status = 400, description = "Bad request (e.g., invalid IDs, missing assets, empty format list)", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden (e.g., bundle not owned by user)", body = ErrorResponse),
        (status = 404, description = "Resource not found (bundle, style, assets, document, format)", body = ErrorResponse),
        (status = 500, description = "Internal error (DB or LLM failure)", body = ErrorResponse)
    ),
    tag = "Creatives",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/generate_from_bundle")]
#[tracing::instrument(skip(pool, gcs_client, payload, user, req), fields(user_id = tracing::field::Empty, bundle_id = %payload.bundle_id))]
pub async fn generate_creative_from_bundle(
    pool: actix_web::web::Data<PgPool>,
    gcs_client: actix_web::web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    payload: actix_web::web::Json<GenerateCreativeFromBundleRequest>,
   user: actix_web::web::ReqData<AuthenticatedUser>,
    req: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let user_id = match &*user {
        AuthenticatedUser::Jwt(claims) => claims.user_id,
        AuthenticatedUser::ApiKey(id) => *id,
    };
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    // Extract organization_id from request headers, fallback to payload
    let organization_id = crate::services::credits_service::extract_organization_id_from_headers(&req)
        .or(payload.organization_id);

    // 1. Fetch Bundle and Verify Ownership
    let bundle_result =
        crate::queries::bundles::find_bundle_by_id::find_bundle_by_id(pool.get_ref(), payload.bundle_id).await;

    let bundle = match bundle_result {
        Ok(Some(b)) => {
            if b.user_id != user_id {
                log::warn!(
                    "User {} attempted to use bundle {} owned by {}",
                    user_id,
                    payload.bundle_id,
                    b.user_id
                );
                return actix_web::HttpResponse::Forbidden().json(ErrorResponse {
                    error: "Bundle does not belong to the authenticated user.".to_string(),
                });
            }
            b
        }
        Ok(None) => {
            log::warn!("Bundle not found for ID: {}", payload.bundle_id);
            return actix_web::HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Bundle not found: {}", payload.bundle_id),
            });
        }
        Err(e) => {
            log::error!("DB error fetching bundle {}: {:?}", payload.bundle_id, e);
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch bundle".to_string(),
            });
        }
    };

    // 2. Gather Context from Bundle
    // 2a. Fetch Style from bundle.style_id

    #[derive(sqlx::FromRow, Debug)]
    struct StyleWithAccessDetails {
        id: Uuid,
        user_id: Option<Uuid>,
        name: String,
        html_url: String,
        screenshot_url: String,
        is_public: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let style_result = sqlx::query_as!(
        StyleWithAccessDetails,
        r#"SELECT
            s.id, s.user_id, s.name, s.html_url, s.screenshot_url, s.is_public,
            s.created_at, s.updated_at
           FROM styles s WHERE s.id = $1 AND (s.user_id = $2 OR s.is_public = true)"#,
        bundle.style_id,
        Some(user_id)
    )
    .fetch_optional(pool.get_ref())
    .await;

    let style = match style_result {
        Ok(Some(s)) => Style {
            id: s.id,
            user_id: s.user_id,
            name: s.name,
            html_url: s.html_url,
            screenshot_url: s.screenshot_url,
            is_public: s.is_public,
            created_at: s.created_at,
            updated_at: s.updated_at,
        },
        Ok(None) => {
            log::warn!(
                "Style {} (from bundle {}) not found or not owned by user {}",
                bundle.style_id,
                bundle.id,
                user_id
            );
            return actix_web::HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Style not found: {}", bundle.style_id),
            });
        }
        Err(e) => {
            log::error!("DB error fetching style {}: {:?}", bundle.style_id, e);
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch style".to_string(),
            });
        }
    };

    let style_html = match parse_gcs_url(&style.html_url) {
        Ok((bucket_name, object_name)) => {
            match gcs_client.get_ref().as_ref().download_object_as_string(&bucket_name, &object_name).await {
                Ok(html) => html,
                Err(e) => {
                    log::error!("Failed to download style HTML from GCS bucket '{bucket_name}', object '{object_name}': {e:?}");
                    return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to read style HTML from storage".to_string(),
                    });
                }
            }
        }
        Err(e) => {
            log::error!("Failed to parse style HTML URL '{}': {}", style.html_url, e);
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Invalid style HTML URL format".to_string(),
            });
        }
    };

    // 2b. Fetch Assets from bundle.asset_ids
    let assets_result = sqlx::query_as!(
        Asset,
        "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = ANY($1) AND user_id = $2",
        &bundle.asset_ids,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await;

    let assets = match assets_result {
        Ok(a) => {
            if a.len() != bundle.asset_ids.len() {
                let found_ids: HashSet<Uuid> = a.iter().map(|asset| asset.id).collect();
                let missing_ids: Vec<String> = bundle
                    .asset_ids
                    .iter()
                    .filter(|id| !found_ids.contains(id))
                    .map(|id| id.to_string())
                    .collect();
                log::warn!(
                    "Assets (from bundle {}) not found or not owned by user {}: {}",
                    bundle.id,
                    user_id,
                    missing_ids.join(", ")
                );
                return actix_web::HttpResponse::NotFound().json(ErrorResponse {
                    error: format!("Assets not found: {}", missing_ids.join(", ")),
                });
            }
            a
        }
        Err(e) => {
            log::error!("DB error fetching assets for bundle {}: {:?}", bundle.id, e);
            return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch assets".to_string(),
            });
        }
    };

    // 2c. Determine and Fetch Documents
    // Combine document IDs from the bundle and the payload (additive, unique)
    let mut combined_document_ids_set = std::collections::HashSet::new();
    for doc_id in &bundle.document_ids {
        combined_document_ids_set.insert(*doc_id);
    }
    if let Some(payload_doc_ids) = &payload.document_ids {
        for doc_id in payload_doc_ids {
            combined_document_ids_set.insert(*doc_id);
        }
    }
    let document_ids_to_fetch: std::vec::Vec<sqlx::types::Uuid> =
        combined_document_ids_set.into_iter().collect();

    let mut document_context = String::new();
    let mut fetched_document_ids_for_creative: Option<Vec<Uuid>> = None;

    if !document_ids_to_fetch.is_empty() {
        #[derive(sqlx::FromRow, Debug)]
        struct DocumentForBundleCreative {
            id: Uuid,
            title: String,
            content: String,
        }
        let documents_result = sqlx::query_as!(
            DocumentForBundleCreative,
            r#"SELECT
                d.id, d.title, d.content
               FROM documents d WHERE d.id = ANY($1) AND (d.user_id = $2 OR (d.is_public = TRUE AND d.user_id IS NULL))"#,
            &document_ids_to_fetch,
            user_id
        )
        .fetch_all(pool.get_ref())
        .await;

        match documents_result {
            Ok(docs) => {
                if docs.len() != document_ids_to_fetch.len() {
                    let found_ids: HashSet<Uuid> = docs.iter().map(|d| d.id).collect();
                    let missing_ids: Vec<String> = document_ids_to_fetch
                        .iter()
                        .filter(|id| !found_ids.contains(id))
                        .map(|id| id.to_string())
                        .collect();
                    log::warn!(
                        "Documents not found or not accessible by user {}: {}",
                        user_id,
                        missing_ids.join(", ")
                    );
                    return actix_web::HttpResponse::NotFound().json(ErrorResponse {
                        error: format!("Documents not found: {}", missing_ids.join(", ")),
                    });
                }
                for doc in &docs {
                    document_context.push_str(&format!(
                        "\n---\nDocument ID: {}\nTitle: {}\nContent:\n{}\n---\n",
                        doc.id, doc.title, doc.content
                    ));
                }
                fetched_document_ids_for_creative = Some(docs.iter().map(|d| d.id).collect());
            }
            Err(e) => {
                log::error!("DB error fetching documents: {e:?}");
                return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to fetch documents".to_string(),
                });
            }
        }
    }

   // 3. Fetch Creative Formats (now solely from bundle.format_ids)
   // 3. Fetch Creative Formats from bundle.format_ids
   // This logic is adapted from generate_creative.rs and modified to use bundle.format_ids
   if bundle.format_ids.is_empty() {
       log::warn!("Bundle {} has an empty format_ids list.", bundle.id);
       return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
           error: "Bundle must have at least one creative format ID specified in its format_ids.".to_string(),
       });
   }

   let requested_format_ids_set: HashSet<Uuid> =
       bundle.format_ids.iter().cloned().collect();
   let mut found_formats_map: HashMap<Uuid, CombinedFormatInfo> = HashMap::new();
   let mut missing_ids = requested_format_ids_set.clone();

    // Fetch from Custom Formats (user-specific or public)
    if !missing_ids.is_empty() {
        let missing_ids_vec: Vec<Uuid> = missing_ids.iter().cloned().collect();
       let custom_formats_result = sqlx::query_as!(
            CustomCreativeFormat,
            r#"SELECT id, user_id, name, description, width, height, creative_type AS "creative_type: _", json_schema, metadata, created_at, updated_at, is_public
               FROM custom_creative_formats
               WHERE id = ANY($1) AND (is_public = TRUE OR user_id = $2)"#,
            &missing_ids_vec,
            user_id
        )
        .fetch_all(pool.get_ref())
        .await;

        match custom_formats_result {
            Ok(custom_formats) => {
                for format in custom_formats {
                    if missing_ids.remove(&format.id) {
                        found_formats_map.insert(
                            format.id,
                            CombinedFormatInfo {
                                id: format.id,
                                name: format.name,
                                description: format.description,
                                width: format.width,
                                height: format.height,
                                metadata: format.metadata,
                            },
                        );
                    }
                }
            }
            Err(e) => {
                log::error!("DB error fetching custom creative formats: {e:?}");
                // Proceed, final check below will catch missing IDs.
            }
        }
    }

    if !missing_ids.is_empty() {
        let still_missing_ids_str: Vec<String> =
            missing_ids.iter().map(|id| id.to_string()).collect();
        log::warn!(
            "Creative formats not found or not accessible by user {}: {}",
            user_id,
            still_missing_ids_str.join(", ")
        );
        return actix_web::HttpResponse::NotFound().json(ErrorResponse {
            error: format!(
                "Creative Formats not found: {}",
                still_missing_ids_str.join(", ")
           ),
       });
   }

   let mut ordered_found_formats: Vec<&CombinedFormatInfo> = Vec::with_capacity(bundle.format_ids.len());
   for requested_id in &bundle.format_ids {
       if let Some(format_info) = found_formats_map.get(requested_id) {
           ordered_found_formats.push(format_info);
       } else {
             log::error!("Logic error: Format ID {requested_id} was requested but not found in final map.");
             return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                 error: format!("Internal error processing format ID {requested_id}"),
             });
        }
    }

    let mut creative_format_context = String::new();
    for f_info in &ordered_found_formats {
        let mut context_part = format!(
            "Name: {}, Description: {}, Dimensions: {}x{}",
            f_info.name,
            f_info.description.as_deref().unwrap_or("N/A"),
            f_info.width.unwrap_or(0),
            f_info.height.unwrap_or(0)
        );
        if let Some(meta) = &f_info.metadata {
            context_part.push_str(&format!(
                "\nMetadata: {}",
                serde_json::to_string_pretty(meta).unwrap_or_else(|_| meta.to_string())
            ));
        }
        creative_format_context.push_str(&format!(
            "\n---\nFormat ID: {}\n{}\n---\n",
           f_info.id, context_part
       ));
   }

   // bundle.format_ids is guaranteed not empty due to the check above
   let primary_creative_format_id = bundle.format_ids[0]; // For DB insertion

   // 4. Construct LLM Prompt (adapted from generate_creative.rs)
   let assets_context = assets
        .iter()
        .map(|a| format!("- Asset Name: {}, Type: {}, URL: {}", a.name, a.r#type, a.url))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"Generate a complete, self-contained HTML creative based on the provided context.
The final output must be ONLY the raw HTML code, starting with <!DOCTYPE html> or <html> and ending with </html>.
Include all necessary CSS and JavaScript derived from the STYLE directly within the HTML (e.g., in <style> tags or inline styles).
Use the provided ASSET URLs for images or other resources.

CONTEXT:

<STYLE name="{}">
{}
</STYLE>

<ASSETS>
{}
</ASSETS>
{}
{}
TASK: Create the HTML output by following these instructions:
1.  **Style Guidance:** Use the provided `<STYLE>` block as the primary reference for stylistic choices. This includes color palettes, typography, layout principles, and any specific HTML components or CSS classes defined within the style's HTML content.
2.  **Asset Integration:** Incorporate the assets listed in `<ASSETS>` into the HTML structure appropriately. Use the provided URLs directly.
3.  **Content Foundation:** Base the textual and informational content of the creative primarily on the information provided in the `<DOCUMENT_CONTEXTS>` section, if present.
4.  **Format Adherence:** Ensure the final HTML structure and dimensions align with the requirements outlined in the `<CREATIVE_FORMAT_CONTEXTS>`. Pay attention to the specified name, description, dimensions (width/height), and any metadata hints.
5.  **Output Requirements:** Generate only the raw HTML code, starting with `<!DOCTYPE html>` or `<html>` and ending with `</html>`. Embed all necessary CSS and JavaScript within the HTML document (e.g., in `<style>` tags or inline styles derived from the STYLE context). Do not include any explanatory text or markdown formatting around the HTML code itself.

Create the HTML output"#,
        style.name,
        style_html,
        assets_context,
        if document_context.is_empty() {
            "".to_string()
        } else {
            format!("\n<DOCUMENT_CONTEXTS>\n{document_context}\n</DOCUMENT_CONTEXTS>")
        },
        if creative_format_context.is_empty() {
            "".to_string()
        } else {
            format!(
                "\n<CREATIVE_FORMAT_CONTEXTS>\n{creative_format_context}\n</CREATIVE_FORMAT_CONTEXTS>"
            )
        }
    );

    // 5. Call LLM Service (adapted from generate_creative.rs)
    let models = [VendorModel::Gemini(GeminiModel::Gemini25Pro),
        VendorModel::Gemini(GeminiModel::Gemini25ProPreview0325)];
    let mut validation_attempts = 0;
    const MAX_VALIDATION_ATTEMPTS: u32 = 3;
    log::info!("Sending prompt length: {}", prompt.len());

    loop {
        validation_attempts += 1;
        let model_idx = (validation_attempts as usize - 1) % models.len();
        let model_to_use = models[model_idx].clone();
        let llm_result =
            llm::llm_typed_unified::llm::llm(false, &prompt, vec![model_to_use], 1).await;

        match llm_result {
            Ok(html_content) => {
                let trimmed_content = html_content
                    .trim()
                    .trim_start_matches("```html")
                    .trim_end_matches("```")
                    .to_string();
                let is_long_enough = trimmed_content.len() >= 2000; // Validation criteria

                if is_long_enough {
                    let creative_id = Uuid::new_v4();
                    let html_content_bytes = trimmed_content.into_bytes();

                    let (html_url, screenshot_url) = match upload_creative_assets(
                        gcs_client.get_ref().as_ref(),
                        creative_id,
                        html_content_bytes,
                    )
                    .await
                    {
                        Ok(urls) => urls,
                        Err(e) => {
                            log::error!("Failed to upload creative assets for {creative_id}: {e}");
                            return actix_web::HttpResponse::InternalServerError()
                                .json(ErrorResponse { error: e });
                        }
                    };

                    // Prepare IDs for DB insertion
                    let bundle_asset_ids_slice: Option<Vec<Uuid>> = if bundle.asset_ids.is_empty() { None } else { Some(bundle.asset_ids.clone()) };


                    #[derive(sqlx::FromRow, Debug)]
                    struct NewCreativeDetails {
                        id: Uuid,
                        name: String,
                        collection_id: Option<Uuid>,
                        creative_format_id: Uuid,
                        style_id: Option<Uuid>,
                        document_ids: Option<Vec<Uuid>>,
                        asset_ids: Option<Vec<Uuid>>,
                        html_url: String,
                        draft_url: Option<String>,
                        bundle_id: Option<Uuid>,
                        screenshot_url: String,
                        is_published: bool,
                        publish_url: Option<String>,
                        created_at: chrono::DateTime<chrono::Utc>,
                        updated_at: chrono::DateTime<chrono::Utc>,
                        creator_email: Option<String>,
                        current_user_access_level: Option<String>,
                    }

                    let insert_result = sqlx::query_as!(
                        NewCreativeDetails,
                        r#"
                        INSERT INTO creatives (
                            id, name, collection_id, creative_format_id, style_id, document_ids,
                            asset_ids, html_url, screenshot_url, is_published, publish_url,
                            bundle_id, draft_url,
                            created_at, updated_at
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())
                        RETURNING
                            id, name, collection_id, creative_format_id, style_id, document_ids,
                            asset_ids, html_url, draft_url, screenshot_url, is_published,
                            publish_url, bundle_id, created_at, updated_at,
                            (SELECT u.email FROM users u JOIN collections col ON u.id = col.user_id WHERE col.id = $3) AS creator_email,
                            'owner' AS current_user_access_level
                        "#,
                        creative_id,
                        payload.name,
                        payload.collection_id, // From request payload
                        primary_creative_format_id, // From bundle.format_ids[0]
                        bundle.style_id,        // From fetched bundle
                        fetched_document_ids_for_creative.as_ref().map(|v| v.as_slice()), // Effective documents
                        bundle_asset_ids_slice.as_ref().map(|v| v.as_slice()), // From fetched bundle
                        html_url,
                        screenshot_url,
                        false,                  // Default is_published
                        None::<String>,         // Default publish_url
                        Some(bundle.id),        // bundle_id from fetched bundle
                        None::<String>          // Default draft_url
                    )
                    .fetch_one(pool.get_ref())
                    .await;

                    match insert_result {
                        Ok(details) => {
                            log::info!("Creative {} generated successfully from bundle {}", details.id, bundle.id);

                            // Consume credits before returning response
                            let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GENERATE_CREATIVE_FROM_BUNDLE;
                            let deduction_params = CreditChangesParams {
                                user_id,
                                organization_id, // Use the extracted organization_id from request headers or payload
                                credits_to_change: BigDecimal::from(credits_to_consume),
                                action_source: "api".to_string(),
                                action_type: "generate_creative_from_bundle".to_string(),
                                entity_id: Some(details.id.clone()),
                            };
                            if let Err(e) = deduct_user_credits_with_transaction(pool.get_ref(), deduction_params).await {
                                log::error!("Failed to deduct {} credits for user {} after generating creative from bundle: {}", credits_to_consume, user_id, e);
                            }

                            let response = CreativeResponse {
                                creative: Creative {
                                    id: details.id,
                                    name: details.name,
                                    collection_id: details.collection_id,
                                    creative_format_id: details.creative_format_id,
                                    style_id: details.style_id,
                                    document_ids: details.document_ids,
                                    asset_ids: details.asset_ids,
                                    html_url: details.html_url,
                                    draft_url: details.draft_url,
                                    bundle_id: details.bundle_id,
                                    screenshot_url: details.screenshot_url,
                                    is_published: details.is_published,
                                    publish_url: details.publish_url,
                                    created_at: details.created_at,
                                    updated_at: details.updated_at,
                                },
                                creator_email: details.creator_email,
                                current_user_access_level: details.current_user_access_level,
                            };
                            return actix_web::HttpResponse::Created().json(response);
                        }
                        Err(e) => {
                            log::error!("DB error saving creative from bundle {}: {:?}", bundle.id, e);
                            return actix_web::HttpResponse::InternalServerError().json(
                                ErrorResponse {
                                    error: "Failed to save generated creative to database"
                                        .to_string(),
                                },
                            );
                        }
                    }
                } else {
                    log::warn!(
                        "LLM output validation failed for bundle {} on attempt {}/{}. Length check (>=2000): {}. Response head: {:?}",
                        bundle.id,
                        validation_attempts,
                        MAX_VALIDATION_ATTEMPTS,
                        is_long_enough,
                        trimmed_content.chars().take(100).collect::<String>()
                    );
                    if validation_attempts >= MAX_VALIDATION_ATTEMPTS {
                        let error_message = if !is_long_enough {
                            "LLM generated content is too short after retries."
                        } else {
                            "LLM generated invalid HTML structure after retries."
                        };
                        return actix_web::HttpResponse::InternalServerError()
                            .json(ErrorResponse {
                                error: error_message.to_string(),
                            });
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "LLM generation call failed for bundle {} on attempt {}/{}: {:?}",
                    bundle.id,
                    validation_attempts,
                    MAX_VALIDATION_ATTEMPTS,
                    e
                );
                if validation_attempts >= MAX_VALIDATION_ATTEMPTS {
                    return actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                        error:
                            "Failed to generate creative HTML via LLM after multiple attempts."
                                .to_string(),
                    });
                }
            }
        }
        log::info!(
            "Retrying LLM call for bundle {}, attempt {} of {}.",
            bundle.id,
            validation_attempts + 1,
            MAX_VALIDATION_ATTEMPTS
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
