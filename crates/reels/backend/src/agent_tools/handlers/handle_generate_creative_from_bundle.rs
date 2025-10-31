//! Handles the 'generate_creative_from_bundle' agent tool action.
//!
//! This function takes `GenerateCreativeFromBundleParams`, fetches the specified

use std::collections::{HashMap, HashSet};
use crate::db::assets::Asset;
use crate::db::creatives::Creative;
use crate::queries::bundles::fetch_expanded_bundles_by_ids::fetch_expanded_bundles_by_ids;
use crate::routes::creatives::generate_creative::CombinedFormatInfo;
use crate::services::creative_generation_service::process_single_creative_format_for_generation;
use llm::llm_typed_unified::vendor_model::VendorModel;
use llm::vendors::gemini::gemini_model::GeminiModel;

// Function length justification:
// Similar to `handle_generate_creative`, this is a high-level orchestrator.
// It performs data aggregation from a bundle, which is a complex data structure,
// before proceeding with the same multi-step generation process.
// Maintaining it as one function preserves the logical flow.
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub async fn handle_generate_creative_from_bundle(    params: crate::agent_tools::tool_params::generate_creative_from_bundle_params::GenerateCreativeFromBundleParams,
    pool: &sqlx::PgPool,
    gcs: std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>,
) -> std::result::Result<(agentloop::types::full_tool_response::FullToolResponse, agentloop::types::user_tool_response::UserToolResponse), std::string::String> {
    let user_id = params.user_id.ok_or("User id should be provided".to_owned())?;
    let organization_id = params.organization_id;
    let collection_id_to_use = params.collection_id;

    // Check credit availability before proceeding
    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::GENERATE_CREATIVE_FROM_BUNDLE;
    
    if let Err(error) = crate::services::credits_service::check_credits_availability_by_user_or_organization(
        pool,
        user_id,
        credits_to_consume,
        organization_id,
    ).await {
        return std::result::Result::Err(error.message);
    }

    // 0. COLLECTION PERMISSION CHECK (Same as handle_generate_creative)
    struct CollectionPermissionResult { _owner_user_id: uuid::Uuid, _effective_access_level: Option<String> }
    let org_memberships = match crate::queries::organizations::find_active_memberships_for_user(pool, user_id).await {
        Ok(m) => m,
        Err(e) => return std::result::Result::Err(format!("Failed to fetch organization memberships: {e}")),
    };
    let org_ids: std::vec::Vec<uuid::Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();
    let collection_perm_check = match sqlx::query_as!(
        CollectionPermissionResult,
        "SELECT c.user_id AS _owner_user_id, os.access_level::TEXT AS _effective_access_level FROM collections c LEFT JOIN (SELECT object_id, access_level FROM (SELECT os.object_id, os.access_level, ROW_NUMBER() OVER(PARTITION BY os.object_id ORDER BY CASE os.access_level WHEN 'editor' THEN 1 ELSE 2 END) as rn FROM object_shares os WHERE os.object_type = 'collection' AND os.object_id = $2 AND ((os.entity_type = 'user' AND os.entity_id = $1) OR (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[])))) ranked_shares WHERE rn = 1) os ON c.id = os.object_id WHERE c.id = $2",
        user_id, collection_id_to_use, &org_ids
    ).fetch_optional(pool).await {
        Ok(Some(res)) => res,
        Ok(None) => return std::result::Result::Err("Target collection not found".to_string()),
        Err(e) => return std::result::Result::Err(format!("Failed to verify collection access: {e}")),
    };
    if !(collection_perm_check._owner_user_id == user_id || collection_perm_check._effective_access_level.as_deref() == Some("editor")) {
        return std::result::Result::Err("Permission denied for the target collection".to_string());
    }

    // 1. Fetch Bundle and Extract info
    let bundles = match fetch_expanded_bundles_by_ids(pool, user_id, &[params.bundle_id]).await {
        Ok(b) if !b.is_empty() => b,
        Ok(_) => return std::result::Result::Err(format!("Bundle not found or not accessible: {}", params.bundle_id)),
        Err(e) => return std::result::Result::Err(format!("Failed to fetch bundle: {e}")),
    };
    let bundle = &bundles[0];
    let style = bundle.style.clone();

    // Aggregate non public IDs from bundle and params
    let asset_ids: HashSet<uuid::Uuid> = bundle.assets.iter().filter(|a| !a.is_public).map(|a| a.id).collect();
    let mut doc_ids: HashSet<uuid::Uuid> = bundle.documents.iter().map(|d| d.id).collect();
    if let Some(param_docs) = &params.document_ids {
        doc_ids.extend(param_docs);
    }
    let format_ids: HashSet<uuid::Uuid> = bundle.formats.iter().map(|f| f.id).collect();

    let final_asset_ids: Vec<uuid::Uuid> = asset_ids.into_iter().collect();
    let final_doc_ids: Vec<uuid::Uuid> = doc_ids.into_iter().collect();
    let final_format_ids: Vec<uuid::Uuid> = format_ids.into_iter().collect();

    // The rest of the logic is identical to handle_generate_creative
    let (bucket, object) = crate::services::gcs::parse_gcs_url::parse_gcs_url(&style.html_url).map_err(|e| e.to_string())?;
    let style_html = gcs.download_object_as_string(&bucket, &object).await.map_err(|e| format!("Failed to read style HTML: {e}"))?;

    let assets = if !final_asset_ids.is_empty() {
        sqlx::query_as!(Asset, "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = ANY($1) AND is_public = FALSE", &final_asset_ids).fetch_all(pool).await.map_err(|e| format!("Failed to fetch assets: {e}"))?
    } else { vec![] };

    let mut doc_context = String::new();
    if !final_doc_ids.is_empty() {
        #[derive(sqlx::FromRow)] struct DocInfo { id: uuid::Uuid, title: String, content: String }
        let docs = sqlx::query_as!(DocInfo, "SELECT id, title, content FROM documents WHERE id = ANY($1)", &final_doc_ids).fetch_all(pool).await.map_err(|e| format!("Failed to fetch documents: {e}"))?;
        for doc in docs {
            doc_context.push_str(&format!("\n---\nID: {}\nTitle: {}\n{}\n---\n", doc.id, doc.title, doc.content));
        }
    }

    if final_format_ids.is_empty() { return std::result::Result::Err("Bundle contains no creative formats.".to_string()); }
    let format_map: HashMap<_, _> = bundle.formats.iter().map(|f| (f.id, f.clone())).collect();

    let assets_context = assets.iter().map(|a| format!("- {}: {}, URL: {}", a.name, a.r#type, a.url)).collect::<Vec<_>>().join("\n");
    let models = std::sync::Arc::new(vec![VendorModel::Gemini(GeminiModel::Gemini25Pro)]);

    let mut tasks = Vec::new();
    for format_id in &final_format_ids {
        let format = match format_map.get(format_id) {
            Some(f) => f,
            None => continue,
        };
        let cfi = CombinedFormatInfo { id: format.id, name: format.name.clone(), description: format.description.clone(), width: format.width, height: format.height, metadata: format.metadata.clone() };
        tasks.push(process_single_creative_format_for_generation(
            actix_web::web::Data::from(std::sync::Arc::new(pool.clone())),
            actix_web::web::Data::new(gcs.clone()),
            style.id, style.name.clone(), style_html.clone(),
            assets_context.clone(), doc_context.clone(), cfi,
            collection_id_to_use, Some(final_asset_ids.clone()), Some(final_doc_ids.clone()),
            models.clone(), 3, params.name.clone(), user_id, organization_id
        ));
    }

    let results = futures::future::join_all(tasks).await;
    let creatives: std::vec::Vec<Creative> = results.into_iter().filter_map(|r| r.ok().map(|cr| cr.creative)).collect();

    if creatives.is_empty() {
        std::result::Result::Err("Failed to generate any creatives.".to_string())
    } else {
        let tool_name = "generate_creative_from_bundle";
        let full_response = agentloop::types::full_tool_response::FullToolResponse {
            tool_name: tool_name.to_string(),
            response: serde_json::to_value(&creatives)
                .map_err(|e| format!("Failed to serialize creatives: {e}"))?,
        };
        let user_response = agentloop::types::user_tool_response::UserToolResponse {
            tool_name: tool_name.to_string(),
            summary: format!("Successfully generated {} creatives from bundle.", creatives.len()),
            icon: None,
            data: Some(full_response.response.clone()),
        };
        std::result::Result::Ok((full_response, user_response))
    }
}
