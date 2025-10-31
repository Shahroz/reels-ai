//! Handler for generating and saving a new creative using LLM.
//!
//! POST /api/creatives/generate
//! Fetches style, assets, optional document, and format data (public or custom) based on provided IDs.
//! Constructs a prompt with this context, calls a unified LLM pool to generate HTML.
//! Saves the generated creative(s) to the database and returns the new record(s).
//! Adheres to coding standards, using fully qualified paths.

use llm::llm_typed_unified::vendor_model::VendorModel;
use actix_web::HttpResponse;
use crate::db::assets::Asset;
use crate::db::custom_creative_formats::CustomCreativeFormat;
use crate::db::creatives::Creative;
use crate::db::styles::Style;
use crate::auth::tokens::Claims; // For extracting user_id
use crate::routes::creatives::generate_creative_request::GenerateCreativeRequest;
use crate::routes::error_response::ErrorResponse;
use crate::services::creative_generation_service::process_single_creative_format_for_generation;
use crate::types::expanded_bundle::ExpandedBundle; // For bundle processing
use crate::queries::bundles::fetch_expanded_bundles_by_ids::fetch_expanded_bundles_by_ids;

use crate::services::gcs::parse_gcs_url::parse_gcs_url;
use std::collections::{HashMap, HashSet}; // Added HashSet
use llm::vendors::gemini::gemini_model::GeminiModel;

use futures::future;
use std::sync::Arc;
use uuid::Uuid;
use tracing;

// Internal temporary struct to hold combined format data for context generation
#[derive(Debug, Clone)] // Added Clone
pub struct CombinedFormatInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[utoipa::path(
    post,
    path = "/api/creatives/generate",
    request_body = GenerateCreativeRequest,
    params(
        ("x-organization-id" = Option<String>, Header, description = "Optional organization ID to deduct credits from organization instead of user")
    ),
    responses(
        (status = 201, description = "Creative generated and saved successfully", body = [Creative]),
        (status = 400, description = "Bad request (e.g., invalid IDs, missing assets, empty format list)", body = ErrorResponse),
        (status = 404, description = "Resource not found (e.g., style, document, format)", body = ErrorResponse),
        (status = 500, description = "Internal error (DB or LLM failure)", body = ErrorResponse)
    ),
    tag = "Creatives",
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/generate")]
pub async fn generate_creative(
    pool: actix_web::web::Data<sqlx::PgPool>,
    gcs: actix_web::web::Data<std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>>,
    payload: actix_web::web::Json<GenerateCreativeRequest>,
    auth: actix_web::web::ReqData<Claims>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    let user_id = auth.user_id;
    let collection_id_to_use = payload.collection_id;

    // Extract organization_id from request headers, fallback to payload
    let organization_id = crate::services::credits_service::extract_organization_id_from_headers(&req)
        .or(payload.organization_id);

    tracing::info!(
        "generate_creative handler started for user_id: {}, collection_id: {}",
        user_id,
        collection_id_to_use
    );

    //
    // 0. COLLECTION PERMISSION CHECK (copied verbatim from SHARING branch)
    //
    struct CollectionPermissionResult {
        _owner_user_id: Uuid,
        _effective_access_level: Option<String>,
    }

    let org_memberships = match crate::queries::organizations::find_active_memberships_for_user(&pool, user_id).await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to fetch organization memberships for user {}: {}", user_id, e);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::from("Failed to retrieve necessary user data"));
        }
    };
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    let collection_perm_check = match sqlx::query_as!(
        CollectionPermissionResult,
        r#"
        WITH UserOrgMemberships_CTE AS (
            SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'
        ),
        RankedShares_CTE AS (
            SELECT
                os.object_id,
                os.access_level,
                ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                    CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'collection' AND os.object_id = $2
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
            es.access_level::TEXT AS _effective_access_level
        FROM collections col
        LEFT JOIN EffectiveShares_CTE es ON col.id = es.object_id
        WHERE col.id = $2
        "#,
        user_id,
        collection_id_to_use,
        &org_ids
    )
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(res)) => res,
        Ok(None) => {
            tracing::warn!("Target collection {} not found", collection_id_to_use);
            return HttpResponse::NotFound().json(ErrorResponse::from("Target collection not found"));
        }
        Err(e) => {
            tracing::error!(
                "Collection permission check query failed for collection {}: {}",
                collection_id_to_use,
                e
            );
            return HttpResponse::InternalServerError().json(ErrorResponse::from("Failed to verify collection access"));
        }
    };

    if !(collection_perm_check._owner_user_id == user_id
        || collection_perm_check._effective_access_level.as_deref() == Some("editor"))
    {
        tracing::warn!(
            "User {} does not have permission for collection {}. Owner: {}, Share: {:?}",
            user_id,
            collection_id_to_use,
            collection_perm_check._owner_user_id,
            collection_perm_check._effective_access_level
        );
        return HttpResponse::Forbidden().json(ErrorResponse::from("Permission denied for the target collection"));
    }

    //
    // 1. STYLE DETERMINATION (from MAIN branch)
    //
    let mut fetched_bundles_cache: Option<Vec<ExpandedBundle>> = None;
    let mut final_style_object: Option<Style> = None;

    // 1a. If payload.style_id is Some, fetch it and ensure user owns it or has access
    if let Some(direct_style_id) = payload.style_id {
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

        match sqlx::query_as!(
            StyleWithAccessDetails,
            r#"
            WITH RankedShares_CTE AS (
                SELECT
                    os.object_id,
                    os.access_level,
                    ROW_NUMBER() OVER (PARTITION BY os.object_id ORDER BY
                        CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                    ) as rn
                FROM object_shares os
                WHERE os.object_type = 'style' AND os.object_id = $1
                  AND (
                        (os.entity_type = 'user' AND os.entity_id = $2)
                        OR
                        (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
                    )
            ),
            EffectiveShares_CTE AS (
                SELECT object_id, access_level FROM RankedShares_CTE WHERE rn = 1
            )
            SELECT
                s.id, s.user_id, s.name, s.html_url, s.screenshot_url, s.is_public,
                s.created_at, s.updated_at
            FROM styles s
            LEFT JOIN EffectiveShares_CTE es ON s.id = es.object_id
            WHERE s.id = $1 AND (s.user_id = $2 OR s.is_public = true OR es.access_level IS NOT NULL)
            "#,
            direct_style_id,
            user_id,
            &org_ids
        )
        .fetch_optional(pool.get_ref())
        .await
        {
            Ok(Some(s)) => {
                tracing::info!("Using directly provided style_id: {}", direct_style_id);
                final_style_object = Some(Style {
                    id: s.id,
                    user_id: s.user_id,
                    name: s.name,
                    html_url: s.html_url,
                    screenshot_url: s.screenshot_url,
                    is_public: s.is_public,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                });
            }
            Ok(None) => {
                tracing::warn!(
                    "Directly provided style_id {} not found or not owned by user {}.",
                    direct_style_id,
                    user_id
                );
                return HttpResponse::NotFound().json(ErrorResponse {
                    error: format!("Style not found or not accessible: {direct_style_id}"),
                });
            }
            Err(e) => {
                tracing::error!("DB error fetching style {}: {:?}", direct_style_id, e);
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to fetch style".to_string(),
                });
            }
        }
    }

    // 1b. If no style from payload.style_id, try bundles
    if final_style_object.is_none() {
        if let Some(bundle_ids_from_payload) = &payload.bundle_ids {
            if !bundle_ids_from_payload.is_empty() {
                tracing::info!(
                    "payload.style_id is None. Attempting to use style from bundles: {:?}",
                    bundle_ids_from_payload
                );
                match fetch_expanded_bundles_by_ids(pool.get_ref(), user_id, bundle_ids_from_payload).await {
                    Ok(bundles) => {
                        if !bundles.is_empty() {
                            // Take style from first bundle
                            final_style_object = Some(bundles[0].style.clone());
                            tracing::info!(
                                "Using style {:?} from bundle {}",
                                bundles[0].style.id,
                                bundles[0].id
                            );
                        } else {
                            tracing::warn!("No bundles found for IDs {:?}.", bundle_ids_from_payload);
                        }
                        fetched_bundles_cache = Some(bundles);
                    }
                    Err(e) => {
                        tracing::error!(
                            "DB error fetching expanded bundles for style fallback: {:?}",
                            e
                        );
                        return HttpResponse::InternalServerError().json(ErrorResponse {
                            error: "Failed to fetch bundle data for style fallback".to_string(),
                        });
                    }
                }
            }
        }
    }

    let style = match final_style_object {
        Some(s) => s,
        None => {
            tracing::warn!("No style provided or found via bundles for user {}.", user_id);
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "A style must be provided either directly via 'style_id' or indirectly via 'bundle_ids'."
                    .to_string(),
            });
        }
    };

    // Aggregate IDs from payload and bundles
    let mut aggregated_asset_ids: HashSet<uuid::Uuid> = HashSet::new();
    let mut aggregated_document_ids: HashSet<uuid::Uuid> = HashSet::new();
    let mut aggregated_format_ids: HashSet<uuid::Uuid> = HashSet::new();

    if let Some(asset_ids_from_payload) = &payload.asset_ids {
        for asset_id in asset_ids_from_payload {
            aggregated_asset_ids.insert(*asset_id);
        }
    }
    if let Some(doc_ids_from_payload) = &payload.document_ids {
        for doc_id in doc_ids_from_payload {
            aggregated_document_ids.insert(*doc_id);
        }
    }
    for format_id in &payload.creative_format_ids {
        aggregated_format_ids.insert(*format_id);
    }

    // Aggregate from cached bundles (if they were fetched during style determination)
    if let Some(cached_bundles) = &fetched_bundles_cache {
        if !cached_bundles.is_empty() {
            log::info!("Aggregating assets, documents, and formats from {} cached bundles.", cached_bundles.len());
            for bundle_in_cache in cached_bundles {
                for asset in &bundle_in_cache.assets {
                    aggregated_asset_ids.insert(asset.id);
                }
                for document in &bundle_in_cache.documents {
                    aggregated_document_ids.insert(document.id);
                }
                for format_item in &bundle_in_cache.formats {
                    aggregated_format_ids.insert(format_item.id);
                }
            }
        }
    }

    let final_asset_ids_to_fetch: Vec<uuid::Uuid> = aggregated_asset_ids.into_iter().collect();
    let final_document_ids_to_fetch: Vec<uuid::Uuid> = aggregated_document_ids.into_iter().collect();
    let final_creative_format_ids_to_fetch: Vec<uuid::Uuid> = aggregated_format_ids.into_iter().collect();
    // Fetch style HTML content from its GCS URL using GCS client instead of reqwest
    let style_html = match parse_gcs_url(&style.html_url) {
        Ok((bucket_name, object_name)) => {
            match gcs.get_ref().as_ref().download_object_as_string(&bucket_name, &object_name).await {
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

    //
    // 3. FETCH ASSETS (from MAIN)
    //
    let assets = if !final_asset_ids_to_fetch.is_empty() {
        match sqlx::query_as!(
            Asset,
            "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = ANY($1)",
            &final_asset_ids_to_fetch
        )
        .fetch_all(pool.get_ref())
        .await
        {
            Ok(fetched_assets) => {
                if fetched_assets.len() != final_asset_ids_to_fetch.len() {
                    let found_ids: HashSet<Uuid> =
                        fetched_assets.iter().map(|a| a.id).collect();
                    let missing_ids: Vec<String> = final_asset_ids_to_fetch
                        .iter()
                        .filter(|id| !found_ids.contains(id))
                        .map(|id| id.to_string())
                        .collect();
                    tracing::warn!("Assets not found: {:?}", missing_ids);
                    return HttpResponse::NotFound().json(ErrorResponse {
                        error: format!("Assets not found: {}", missing_ids.join(", ")),
                    });
                }
                fetched_assets
            }
            Err(e) => {
                tracing::error!("DB error fetching assets: {:?}", e);
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to fetch assets".to_string(),
                });
            }
        }
    } else {
        Vec::new()
    };

    //
    // 4. FETCH DOCUMENTS (from MAIN)
    //
    let mut document_context_str = String::new();
    let mut actual_fetched_document_ids_for_creative_record: Option<Vec<Uuid>> = None;

    if !final_document_ids_to_fetch.is_empty() {
        tracing::info!(
            "Fetching document contents for IDs: {:?}",
            final_document_ids_to_fetch
        );

        #[derive(sqlx::FromRow, Debug)]
    struct DocumentForCreative {
        id: Uuid,
        title: String,
        content: String,
    }

    let docs_result = match sqlx::query_as!(
        DocumentForCreative,
        r#"SELECT
                d.id, d.title, d.content
            FROM documents d
            WHERE d.id = ANY($1)
            "#,
            &final_document_ids_to_fetch
        )
        .fetch_all(pool.get_ref())
        .await
        {
            Ok(docs) => docs,
            Err(e) => {
                tracing::error!("DB error fetching document items: {:?}", e);
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Failed to fetch document items".to_string(),
                });
            }
        };

        if docs_result.len() != final_document_ids_to_fetch.len() {
            tracing::warn!("Could not find all specified documents. Found {}, expected {}.", docs_result.len(), final_document_ids_to_fetch.len());
            return HttpResponse::NotFound().json(ErrorResponse::from("One or more specified documents were not found or are not accessible."));
        }

        for doc in &docs_result {
            document_context_str.push_str(&format!(
                "\n---\nDocument ID: {}\nTitle: {}\nContent:\n{}\n---\n",
                doc.id, doc.title, doc.content
            ));
        }
        actual_fetched_document_ids_for_creative_record =
            Some(final_document_ids_to_fetch.clone());
    }

    //
    // 5. FETCH CREATIVE FORMATS (Public & Custom) (from both branches)
    //
    if final_creative_format_ids_to_fetch.is_empty() {
        tracing::warn!("No creative_format_ids provided after aggregation.");
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "At least one creative format ID must be provided".to_string(),
        });
    }
    let requested_format_ids_set: HashSet<Uuid> =
        final_creative_format_ids_to_fetch.iter().cloned().collect();
    let mut found_formats_map: HashMap<Uuid, CombinedFormatInfo> = HashMap::new();
    let mut missing_ids: HashSet<Uuid> = requested_format_ids_set.clone();

    // 5a. Attempt to fetch from CustomCreativeFormat
    {
        let missing_ids_vec: Vec<Uuid> = missing_ids.iter().cloned().collect();
       match sqlx::query_as!(
           CustomCreativeFormat,
           r#"SELECT 
                 id, user_id, name, description, width, height,
                 creative_type AS "creative_type: _", json_schema, metadata, 
                 created_at, updated_at, is_public
               FROM custom_creative_formats 
               WHERE id = ANY($1)"#,
            &missing_ids_vec
        )
        .fetch_all(pool.get_ref())
        .await
        {
            Ok(custom_formats) => {
                for cf in custom_formats {
                    if missing_ids.remove(&cf.id) {
                        found_formats_map.insert(
                            cf.id,
                            CombinedFormatInfo {
                                id: cf.id,
                                name: cf.name,
                                description: cf.description,
                                width: cf.width,
                                height: cf.height,
                                metadata: cf.metadata,
                            },
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!(
                    "DB error fetching custom creative formats: {:?}. Will fail if any IDs still missing.",
                    e
                );
            }
        }
    }

    // 5b. Final check: any still missing?
    if !missing_ids.is_empty() {
        let still_missing_ids_str: Vec<String> =
            missing_ids.iter().map(|id| id.to_string()).collect();
        tracing::warn!(
            "Creative formats not found after checking custom table: {:?}",
            still_missing_ids_str
        );
        return HttpResponse::NotFound().json(ErrorResponse {
            error: format!("Creative Formats not found: {}", still_missing_ids_str.join(", ")),
        });
    }

    // 5c. Build ordered list in the original order
    let mut ordered_found_formats: Vec<&CombinedFormatInfo> =
        Vec::with_capacity(final_creative_format_ids_to_fetch.len());
    for &requested_id in &final_creative_format_ids_to_fetch {
        if let Some(f_info) = found_formats_map.get(&requested_id) {
            ordered_found_formats.push(f_info);
        } else {
            tracing::error!(
                "Logic error: Format ID {} was requested but not found in final map.",
                requested_id
            );
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Internal error processing format ID {requested_id}"),
            });
        }
    }

    //
    // 6. PREPARE CONTEXT STRINGS FOR ALL FORMATS
    //
    let assets_context = assets
        .iter()
        .map(|a| format!("- Asset Name: {}, Type: {}, URL: {}", a.name, a.r#type, a.url))
        .collect::<Vec<_>>()
        .join("\n");

    // (document_context was already built above)

    //
    // 7. DEFINE MODEL POOL (merge SHARING's three models)
    //
    let models = vec![
        VendorModel::Gemini(GeminiModel::Gemini25Pro),
        VendorModel::Gemini(GeminiModel::Gemini25ProPreview0325),
        VendorModel::Gemini(GeminiModel::Gemini25Flash),
    ];
    let llm_models_arc = Arc::new(models);
    const MAX_VALIDATION_ATTEMPTS: u32 = 3;

    //
    // 8. SPAWN ONE TASK PER FORMAT (as MAIN does)
    //
    let mut tasks = Vec::with_capacity(ordered_found_formats.len());
    let style_id = style.id;
    let style_name = style.name.clone();
    let style_html_clone = style_html.clone();
    let assets_ctx_clone = assets_context.clone();
    let documents_ctx_clone = document_context_str.clone();

    let final_asset_ids_option = if final_asset_ids_to_fetch.is_empty() {
        None
    } else {
        Some(final_asset_ids_to_fetch.clone())
    };
    let final_document_ids_option = actual_fetched_document_ids_for_creative_record.clone();
    let task_organization_id = organization_id;

    for f_info_ref in &ordered_found_formats {
        let task_pool = pool.clone();
        let task_gcs = gcs.clone();
        let task_style_id = style_id;
        let task_style_name = style_name.clone();
        let task_style_html = style_html_clone.clone();
        let task_assets_ctx = assets_ctx_clone.clone();
        let task_docs_ctx = documents_ctx_clone.clone();
        let task_format_info: CombinedFormatInfo = (*f_info_ref).clone();
        let task_collection_id = collection_id_to_use;
        let task_asset_ids = final_asset_ids_option.clone();
        let task_document_ids = final_document_ids_option.clone();
        let task_models = llm_models_arc.clone();
        let task_creative_name = payload.name.clone();

        let task_user_id = user_id;
        let task_org_id = task_organization_id;
        tasks.push(async move {
            process_single_creative_format_for_generation(
                task_pool,
                task_gcs,
                task_style_id,
                task_style_name,
                task_style_html,
                task_assets_ctx,
                task_docs_ctx,
                task_format_info,
                task_collection_id,
                task_asset_ids,
                task_document_ids,
                task_models,
                MAX_VALIDATION_ATTEMPTS,
                task_creative_name,
                task_user_id,
                task_org_id,
            )
            .await
        });
    }

    let task_results = future::join_all(tasks).await;
    let mut generated_creatives = Vec::new();
    for result in task_results {
        match result {
            Ok(created) => generated_creatives.push(created),
            Err(e) => {
                // Log but don't immediately fail; we want to collect any that did succeed.
                tracing::error!("Failed to generate a creative for one format: {}", e);
            }
        }
    }

    if generated_creatives.is_empty() {
        tracing::error!("No creatives were generated successfully.");
        HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to generate any creatives. LLM or validation issues persisted.".to_string(),
        })
    } else {
        HttpResponse::Created().json(generated_creatives)
    }
}
