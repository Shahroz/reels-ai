//! Handler for enhancing one or more existing assets with AI.
//!
//! Defines the `enhance_asset` HTTP handler under `/api/assets/enhance`.
//! This handler takes one or more existing asset IDs, validates each is an image owned by the user,
//! calls the retouch_images workflow to enhance them, and saves the enhanced results
//! as new assets in the database, returning a batched response.
//!
//! ## Process Flow
//!
//! 1. Validate each asset exists and user owns it
//! 2. Validate each asset is an image type (required for AI enhancement)
//! 3. Call the retouch_images workflow with the asset's GCS URL
//! 4. Extract enhanced image GCS URLs from the workflow response
//! 5. Save enhanced images as new assets in the database
//! 6. Return per-asset results and totals
//!
//! ## Security & Authorization
//!
//! - User must be authenticated (enforced by middleware)
//! - User must own the original asset
//! - Enhanced assets are created under the same user
//! - Collection association is preserved from original asset
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Refactored into smaller files, removed use statements

use bigdecimal::BigDecimal;

#[utoipa::path(
    post,
    path = "/api/assets/enhance",
    tag = "Assets",
    request_body = crate::routes::assets::enhance_asset_request::EnhanceAssetRequest,
    params(
        ("x-organization-id" = Option<String>, Header, description = "Optional organization ID to use organization credits instead of personal credits")
    ),
    responses(
        (status = 200, description = "Asset enhanced successfully", body = crate::routes::assets::enhance_asset_response::EnhanceAssetResponse),
        (status = 400, description = "Bad Request - Invalid asset ID or asset not an image"),
        (status = 402, description = "Payment Required - Insufficient credits"),
        (status = 403, description = "Forbidden - Not a member of the specified organization"),
        (status = 404, description = "Asset not found or access denied"),
        (status = 500, description = "Internal Server Error - Enhancement or DB Error")
    ),
    security(("user_auth" = []))
)]
#[actix_web::post("/enhance")]
#[tracing::instrument(skip(pool, claims, req, http_req, session_manager))]
pub async fn enhance_asset(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    req: actix_web::web::Json<crate::routes::assets::enhance_asset_request::EnhanceAssetRequest>,
    http_req: actix_web::HttpRequest,
    session_manager: actix_web::web::Data<std::sync::Arc<crate::services::session_manager::HybridSessionManager>>,
) -> impl actix_web::Responder {
    let processing_start = std::time::Instant::now();
    let user_id = claims.user_id;
    let crate::routes::assets::enhance_asset_request::EnhanceAssetRequest {
        asset_ids,
        retouch_prompt,
        vocal_tour_id,
        is_regenerate,
    } = req.into_inner();
    
    // Extract organization_id from header (if present)
    let organization_id = http_req
        .headers()
        .get("x-organization-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    
    // Extract request context for event tracking  
    #[cfg(feature = "events")]
    let request_context = crate::routes::assets::extract_request_context_for_enhancement::extract_request_context_for_enhancement(&http_req, user_id, &session_manager).await;

    // Log regenerate clicked event if this is a regenerate action
    #[cfg(feature = "events")]
    if is_regenerate {
        let asset_id = asset_ids.first().and_then(|id| uuid::Uuid::parse_str(id).ok());
        let _ = crate::services::events_service::studio_events::log_regenerate_clicked(
            &pool,
            user_id,
            asset_id,
            &request_context,
        ).await;
    }

    // Validate and fetch assets
    let assets_to_enhance = match crate::routes::assets::validate_and_fetch_assets::validate_and_fetch_assets(&pool, &asset_ids, user_id).await {
        Ok(assets) => assets,
        Err(error) => return actix_web::HttpResponse::from(error),
    };

    // Calculate required credits (1 credit per asset)
    let required_credits = BigDecimal::from(assets_to_enhance.len() as i32 * crate::app_constants::credits_constants::CreditsConsumption::RETOUCH_IMAGES);
    
    // Prepare credit changes params for tool handler
    let credit_changes_params = crate::queries::user_credit_allocation::CreditChangesParams {
        user_id,
        organization_id,
        credits_to_change: required_credits.clone(),
        action_source: "api".to_string(),
        action_type: "retouch_images".to_string(),
        entity_id: if asset_ids.len() == 1 {
            uuid::Uuid::parse_str(&asset_ids[0]).ok()
        } else {
            None
        },
    };

    // Prepare retouch parameters
    let photos = assets_to_enhance.iter().map(|asset| asset.url.clone()).collect::<std::vec::Vec<std::string::String>>();
    let retouch_params = crate::agent_tools::tool_params::retouch_images_params::RetouchImagesParams {
        photos: photos.clone(),
        retouch_prompt: retouch_prompt.clone(),
        user_id: Some(user_id),
        organization_id,
        credit_changes_params: Some(credit_changes_params),
    };

    // Call retouch handler
    let (full_response, _user_response) = match crate::agent_tools::handlers::handle_retouch_images::handle_retouch_images(retouch_params).await {
        Ok(responses) => responses,
        Err(e) => {
            log::error!("Retouch workflow failed for assets {:?} (user {user_id}): {e}", photos);
            return actix_web::HttpResponse::from(crate::routes::assets::enhance_asset_error::EnhanceAssetError::EnhancementFailed(e.to_string()));
        }
    };

    // Extract GCS URIs from response
    let enhanced_gcs_uris = match crate::routes::assets::gcs_uri_extractor::extract_gcs_uris_from_response(&full_response.response) {
        Ok(uris) => uris,
        Err(e) => {
            log::error!("Failed to extract GCS URIs from enhancement response for assets {:?} (user {user_id}): {e}", photos);
            return actix_web::HttpResponse::from(crate::routes::assets::enhance_asset_error::EnhanceAssetError::ProcessingFailed(e.to_string()));
        }
    };

    log::info!("Successfully extracted {} enhanced GCS URI(s) for assets {:?} (user: {})", 
                enhanced_gcs_uris.len(), photos, user_id);

    // Compute final names based on root original name + short label
    let short_label = crate::routes::assets::derive_short_label_from_prompt::derive_short_label_from_prompt(retouch_prompt.as_deref().unwrap_or(""));
    let final_names = match crate::routes::assets::compute_final_names::compute_final_names(&pool, user_id, &assets_to_enhance, &enhanced_gcs_uris, &short_label).await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to compute final names, falling back: {}", e);
            enhanced_gcs_uris.iter().enumerate().map(|(i, uri)| {
                let orig = assets_to_enhance.get(i);
                let ext = uri.split('.').last().and_then(|e| if e.len()<=5 { Some(e.to_string()) } else { None })
                    .or_else(|| orig.and_then(|o| o.name.split('.').last().map(|s| s.to_string())))
                    .unwrap_or_else(|| "jpg".to_string());
                format!("Derived - {}.{}", short_label, ext)
            }).collect()
        }
    };

    // Prepare enhanced assets data
    let assets_to_save = match crate::routes::assets::prepare_enhanced_assets_data::prepare_enhanced_assets_data(&enhanced_gcs_uris, &assets_to_enhance, Some(&final_names)) {
        Ok(assets) => assets,
        Err(error) => return actix_web::HttpResponse::from(error),
    };

    // Save enhanced assets to database with original owner
    let original_owner_id = assets_to_enhance.first()
        .and_then(|asset| asset.user_id)
        .unwrap_or(user_id);
    
    let enhanced_assets = match crate::routes::assets::save_assets_from_gcs::save_assets_from_gcs_urls(&pool, original_owner_id, assets_to_save, false).await {
        Ok(assets) => assets,
        Err(e) => {
            log::error!("Failed to save enhanced assets for original assets {:?} (user {user_id}): {e}", photos);
            return actix_web::HttpResponse::from(crate::routes::assets::enhance_asset_error::EnhanceAssetError::SaveFailed(e.to_string()));
        }
    };

    // Record asset derivations and studio nodes
    for (idx, new_asset) in enhanced_assets.iter().enumerate() {
        if let Some(original) = assets_to_enhance.get(idx) {
            // Insert provenance edge
            let params = serde_json::json!({
                "type": "Retouch",
                "data": { "retouch_prompt": retouch_prompt.clone().unwrap_or_default() }
            });
            if let Err(e) = crate::queries::provenance::insert_provenance_edge::insert_provenance_edge(
                &pool,
                &crate::queries::provenance::insert_provenance_edge::NodeRef::Asset(original.id),
                &crate::queries::provenance::insert_provenance_edge::NodeRef::Asset(new_asset.id),
                "enhanced",
                &params,
            ).await {
                log::warn!("Failed to insert provenance edge from {} to {}: {}", original.id, new_asset.id, e);
            }

            // Inherit shares from parent asset
            if let Err(e) = crate::queries::assets::inherit_shares_from_asset::inherit_shares_from_asset_single(
                &pool,
                original.id,
                new_asset.id,
            ).await {
                log::warn!("Failed to inherit shares from parent asset {} to enhanced asset {}: {}", original.id, new_asset.id, e);
            }

            // Ensure journey & nodes
            if let Ok(journey) = crate::queries::assets::lineage::get_or_create_journey::get_or_create_journey(&pool, user_id, original.id).await {
                if let Ok(parent_node) = crate::queries::assets::lineage::get_or_create_node::get_or_create_node(&pool, journey.id, original.id, None).await {
                    if let Err(e) = crate::queries::assets::lineage::get_or_create_node::get_or_create_node(&pool, journey.id, new_asset.id, Some(parent_node.id)).await {
                        log::warn!("Failed to create studio node for derived asset {}: {}", new_asset.id, e);
                    }
                }
            }
        }
    }

    // If a vocal_tour_id is provided, associate the new assets with it
    if let Some(vocal_tour_id_str) = vocal_tour_id {
        log::info!("Associating enhanced assets with vocal tour {vocal_tour_id_str}");

        match uuid::Uuid::parse_str(&vocal_tour_id_str) {
            Ok(vocal_tour_uuid) => {
                match crate::queries::vocal_tours::get_vocal_tour_by_id::get_vocal_tour_by_id(&pool, vocal_tour_uuid).await {
                    Ok(Some(mut vocal_tour)) => {
                        if vocal_tour.user_id == user_id {
                            let new_asset_ids: std::vec::Vec<uuid::Uuid> = enhanced_assets.iter().map(|a| a.id).collect();
                            vocal_tour.asset_ids.extend(new_asset_ids);

                            if let Err(e) = crate::queries::vocal_tours::update_vocal_tour_asset_ids::update_vocal_tour_asset_ids(
                                &pool,
                                vocal_tour_uuid,
                                user_id,
                                &vocal_tour.asset_ids,
                            ).await {
                                log::error!("Failed to update vocal tour {vocal_tour_uuid} with new assets: {e}");
                            } else {
                                log::info!("Successfully associated {} new assets with vocal tour {vocal_tour_uuid}", enhanced_assets.len());
                            }
                        } else {
                            log::warn!("User {user_id} attempted to associate assets with vocal tour {vocal_tour_uuid} they do not own.");
                        }
                    },
                    Ok(None) => {
                        log::warn!("Vocal tour {vocal_tour_uuid} not found for association.");
                    },
                    Err(e) => {
                        log::error!("Database error fetching vocal tour {vocal_tour_uuid}: {e}");
                    }
                }
            },
            Err(e) => {
                log::warn!("Invalid vocal_tour_id format received: '{vocal_tour_id_str}': {e}");
            }
        }
    }

    // Update credit reward progress for enhanced assets
    let total_enhanced = enhanced_assets.len();
    if total_enhanced > 0 {
        if let Err(e) = crate::queries::credit_rewards::update_user_reward_progress(
            &pool,
            user_id,
            crate::app_constants::credits_constants::CreditRewardActionTypes::ENHANCE_ASSETS,
            total_enhanced as i32,
        ).await {
            log::warn!("Failed to update credit reward progress for user {}: {}", user_id, e);
        }
    }

    let response = crate::routes::assets::enhance_asset_response::EnhanceAssetResponse {
        original_assets: assets_to_enhance,
        enhanced_assets,
        total_enhanced,
    };

    // Calculate success rate 
    let success_rate = if response.original_assets.len() > 0 {
        (response.total_enhanced as f64 / response.original_assets.len() as f64) * 100.0
    } else {
        0.0
    };
    
    // Log asset enhancement completed event
    #[cfg(feature = "events")]
    {
        let original_asset_ids: std::vec::Vec<uuid::Uuid> = response.original_assets.iter().map(|a| a.id).collect();
        let enhanced_asset_ids: std::vec::Vec<uuid::Uuid> = response.enhanced_assets.iter().map(|a| a.id).collect();
        
        let _ = crate::services::events_service::asset_events::log_asset_enhancement_completed(
            &pool,
            user_id,
            &original_asset_ids,
            &enhanced_asset_ids,
            &retouch_prompt.clone().unwrap_or_default(),
            success_rate,
            &request_context,
            processing_start,
        ).await;
    }

    actix_web::HttpResponse::Ok().json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Integration test requiring database and external services
    async fn test_enhance_asset_handler_integration() {
        // This is a placeholder for integration tests
        // In a real scenario, you would:
        // 1. Set up test database with fixtures
        // 2. Create test user and assets
        // 3. Mock or use test gennodes service
        // 4. Call the handler
        // 5. Verify response and database state
    }

    #[test]
    fn test_success_rate_calculation() {
        let original_count = 5;
        let enhanced_count = 4;
        let success_rate = if original_count > 0 {
            (enhanced_count as f64 / original_count as f64) * 100.0
        } else {
            0.0
        };
        assert_eq!(success_rate, 80.0);
    }

    #[test]
    fn test_success_rate_with_zero_originals() {
        let original_count = 0;
        let enhanced_count = 0;
        let success_rate = if original_count > 0 {
            (enhanced_count as f64 / original_count as f64) * 100.0
        } else {
            0.0
        };
        assert_eq!(success_rate, 0.0);
    }
}
