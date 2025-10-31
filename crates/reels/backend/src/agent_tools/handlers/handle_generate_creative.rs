//! Handles the 'generate_creative' agent tool action.
//!
//! This function takes `GenerateCreativeParams` and orchestrates the creative
//! generation process by leveraging the `creative_generation_service`. It adapts
//! the logic from the main `generate_creative` route.

use crate::db::assets::Asset;
use crate::db::creatives::Creative;
use crate::db::custom_creative_formats::CustomCreativeFormat;
use crate::db::styles::Style;
use crate::queries::bundles::fetch_expanded_bundles_by_ids::fetch_expanded_bundles_by_ids;
use crate::routes::creatives::generate_creative::CombinedFormatInfo;
use crate::services::creative_generation_service::process_single_creative_format_for_generation;
use crate::types::expanded_bundle::ExpandedBundle;
use llm::llm_typed_unified::vendor_model::VendorModel;
use llm::vendors::gemini::gemini_model::GeminiModel;
use std::collections::{HashMap, HashSet};

// Function length justification:
// This function is a high-level orchestrator for a complex multi-step process:
// permission checks, data aggregation from multiple sources (DB, bundles),
// context preparation, and spawning concurrent tasks for LLM generation.
// It mirrors the complexity of the corresponding API route and is best kept as a
// single, cohesive unit.
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub async fn handle_generate_creative(
    params: crate::agent_tools::tool_params::generate_creative_params::GenerateCreativeParams,
    pool: &sqlx::PgPool,
    gcs: std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>,
) -> std::result::Result<
    (
        agentloop::types::full_tool_response::FullToolResponse,
        agentloop::types::user_tool_response::UserToolResponse,
    ),
    std::string::String,
> {
    let user_id = params
        .user_id
        .ok_or("The user_id should be provided".to_owned())?;
    let organization_id = params.organization_id;
    let collection_id_to_use = params.collection_id;

    // Check credit availability before proceeding
    let credits_to_consume =
        crate::app_constants::credits_constants::CreditsConsumption::GENERATE_CREATIVE;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        organization_id,
    ).await {
        return std::result::Result::Err(error.message);
    }
    
    // 0. COLLECTION PERMISSION CHECK
    struct CollectionPermissionResult {
        _owner_user_id: uuid::Uuid,
        _effective_access_level: Option<String>,
    }

    let org_memberships = match crate::queries::organizations::find_active_memberships_for_user(
        pool, user_id,
    )
    .await
    {
        Ok(m) => m,
        Err(e) => {
            let err_msg =
                format!("Failed to fetch organization memberships for user {user_id}: {e}");
            log::error!("{err_msg}");
            return std::result::Result::Err(err_msg);
        }
    };
    let org_ids: std::vec::Vec<uuid::Uuid> = org_memberships
        .into_iter()
        .map(|m| m.organization_id)
        .collect();

    let collection_perm_check = match sqlx::query_as!(
        CollectionPermissionResult,
        r#"
        SELECT c.user_id AS _owner_user_id, os.access_level::TEXT AS _effective_access_level
        FROM collections c
        LEFT JOIN (
            SELECT object_id, access_level
            FROM (
                SELECT os.object_id, os.access_level,
                       ROW_NUMBER() OVER(PARTITION BY os.object_id ORDER BY CASE os.access_level WHEN 'editor' THEN 1 ELSE 2 END) as rn
                FROM object_shares os
                WHERE os.object_type = 'collection' AND os.object_id = $2
                  AND ((os.entity_type = 'user' AND os.entity_id = $1) OR (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[])))
            ) ranked_shares
            WHERE rn = 1
        ) os ON c.id = os.object_id
        WHERE c.id = $2
        "#,
        user_id,
        collection_id_to_use,
        &org_ids
    )
    .fetch_optional(pool)
    .await
    {
        Ok(Some(res)) => res,
        Ok(None) => return std::result::Result::Err("Target collection not found".to_string()),
        Err(e) => return std::result::Result::Err(format!("Failed to verify collection access: {e}")),
    };

    if !(collection_perm_check._owner_user_id == user_id
        || collection_perm_check._effective_access_level.as_deref() == Some("editor"))
    {
        return std::result::Result::Err("Permission denied for the target collection".to_string());
    }

    // 1. STYLE DETERMINATION
    let mut fetched_bundles_cache: Option<Vec<ExpandedBundle>> = None;
    let mut final_style_object: Option<Style> = None;

    if let Some(direct_style_id) = params.style_id {
        // Fetch style and check access
        match sqlx::query_as!(
            Style,
            "SELECT s.* FROM styles s LEFT JOIN object_shares os ON s.id = os.object_id AND os.object_type = 'style' WHERE s.id = $1 AND (s.is_public = true OR s.user_id = $2 OR (os.entity_type = 'user' AND os.entity_id = $2) OR (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[])))",
            direct_style_id, user_id, &org_ids
        )
        .fetch_optional(pool).await {
            Ok(Some(s)) => final_style_object = Some(s),
            Ok(None) => return std::result::Result::Err(format!("Style not found or not accessible: {direct_style_id}")),
            Err(e) => return std::result::Result::Err(format!("Failed to fetch style: {e}")),
        }
    }

    if final_style_object.is_none() {
        if let Some(bundle_ids) = &params.bundle_ids {
            if !bundle_ids.is_empty() {
                match fetch_expanded_bundles_by_ids(pool, user_id, bundle_ids).await {
                    Ok(bundles) => {
                        if !bundles.is_empty() {
                            final_style_object = Some(bundles[0].style.clone());
                        }
                        fetched_bundles_cache = Some(bundles);
                    }
                    Err(e) => {
                        return std::result::Result::Err(format!(
                            "Failed to fetch bundle data for style fallback: {e}"
                        ))
                    }
                }
            }
        }
    }

    let style = match final_style_object {
        Some(s) => s,
        None => {
            return std::result::Result::Err(
                "A style must be provided either directly or via 'bundle_ids'.".to_string(),
            )
        }
    };

    // Aggregate IDs
    let mut asset_ids: HashSet<uuid::Uuid> = params
        .asset_ids
        .clone()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let mut doc_ids: HashSet<uuid::Uuid> = params
        .document_ids
        .clone()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let mut format_ids: HashSet<uuid::Uuid> =
        params.creative_format_ids.clone().into_iter().collect();

    if let Some(bundles) = &fetched_bundles_cache {
        for bundle in bundles {
            asset_ids.extend(bundle.assets.iter().map(|a| a.id));
            doc_ids.extend(bundle.documents.iter().map(|d| d.id));
            format_ids.extend(bundle.formats.iter().map(|f| f.id));
        }
    }

    let final_asset_ids: Vec<uuid::Uuid> = asset_ids.into_iter().collect();
    let final_doc_ids: Vec<uuid::Uuid> = doc_ids.into_iter().collect();
    let final_format_ids: Vec<uuid::Uuid> = format_ids.into_iter().collect();

    // Fetch style HTML
    let (bucket, object) = crate::services::gcs::parse_gcs_url::parse_gcs_url(&style.html_url)
        .map_err(|e| e.to_string())?;
    let style_html = gcs
        .download_object_as_string(&bucket, &object)
        .await
        .map_err(|e| format!("Failed to read style HTML: {e}"))?;

    // Fetch Assets, Docs, Formats
    let assets = if !final_asset_ids.is_empty() {
        sqlx::query_as!(Asset, "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = ANY($1) AND is_public = FALSE", &final_asset_ids).fetch_all(pool).await.map_err(|e| format!("Failed to fetch assets: {e}"))?
    } else {
        vec![]
    };

    let mut doc_context = String::new();
    if !final_doc_ids.is_empty() {
        #[derive(sqlx::FromRow)]
        struct DocInfo {
            id: uuid::Uuid,
            title: String,
            content: String,
        }
        let docs = sqlx::query_as!(
            DocInfo,
            "SELECT id, title, content FROM documents WHERE id = ANY($1)",
            &final_doc_ids
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch documents: {e}"))?;
        for doc in docs {
            doc_context.push_str(&format!(
                "\n---\nID: {}\nTitle: {}\n{}\n---\n",
                doc.id, doc.title, doc.content
            ));
        }
    }

    if final_format_ids.is_empty() {
        return std::result::Result::Err(
            "At least one creative format ID must be provided".to_string(),
        );
    }
    let formats = sqlx::query_as!(CustomCreativeFormat, "SELECT id, user_id, name, description, width, height, creative_type AS \"creative_type: _\", json_schema, metadata, created_at, updated_at, is_public FROM custom_creative_formats WHERE id = ANY($1)", &final_format_ids).fetch_all(pool).await.map_err(|e| format!("Failed to fetch formats: {e}"))?;
    let format_map: HashMap<_, _> = formats.into_iter().map(|f| (f.id, f)).collect();

    // Prepare for generation
    let assets_context = assets
        .iter()
        .map(|a| format!("- {}: {}, URL: {}", a.name, a.r#type, a.url))
        .collect::<Vec<_>>()
        .join("\n");
    let models = std::sync::Arc::new(vec![VendorModel::Gemini(GeminiModel::Gemini25Pro)]);

    let mut tasks = Vec::new();
    for format_id in &final_format_ids {
        let format = match format_map.get(format_id) {
            Some(f) => f,
            None => continue,
        };
        let cfi = CombinedFormatInfo {
            id: format.id,
            name: format.name.clone(),
            description: format.description.clone(),
            width: format.width,
            height: format.height,
            metadata: format.metadata.clone(),
        };
        tasks.push(process_single_creative_format_for_generation(
            actix_web::web::Data::from(std::sync::Arc::new(pool.clone())),
            actix_web::web::Data::new(gcs.clone()),
            style.id,
            style.name.clone(),
            style_html.clone(),
            assets_context.clone(),
            doc_context.clone(),
            cfi,
            collection_id_to_use,
            Some(final_asset_ids.clone()),
            Some(final_doc_ids.clone()),
            models.clone(),
            3,
            params.name.clone(),
            user_id,
            organization_id,
        ));
    }

    let results = futures::future::join_all(tasks).await;
    let creatives: std::vec::Vec<Creative> = results
        .into_iter()
        .filter_map(|r| r.ok().map(|cr| cr.creative))
        .collect();

    if creatives.is_empty() {
        std::result::Result::Err("Failed to generate any creatives.".to_string())
    } else {
        let tool_name = "generate_creative";
        let full_response = agentloop::types::full_tool_response::FullToolResponse {
            tool_name: tool_name.to_string(),
            response: serde_json::to_value(&creatives)
                .map_err(|e| format!("Failed to serialize creatives: {e}"))?,
        };
        let user_response = agentloop::types::user_tool_response::UserToolResponse {
            tool_name: tool_name.to_string(),
            summary: format!("Successfully generated {} creatives.", creatives.len()),
            icon: None,
            data: Some(full_response.response.clone()),
        };
        std::result::Result::Ok((full_response, user_response))
    }
}
