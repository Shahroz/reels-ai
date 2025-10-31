//! Handler for creating a vocal tour from uploaded assets.
//!
//! Defines the `create_vocal_tour` HTTP handler under `/api/vocal-tour`.
//! This handler takes asset IDs, validates them, calls the vocal tour workflow,
//! creates a formatted document, and saves extracted images as new assets.
//!
//! ## Process Flow
//!
//! 1. Validate asset IDs exist and user owns them
//! 2. Call handle_vocal_tour with asset GCS URLs
//! 3. Parse response to extract property description and images
//! 4. Create formatted HTML document with property description
//! 5. Save extracted images as new assets
//! 6. Return created document and assets
//!
//! ## Security & Authorization
//!
//! - User must be authenticated (enforced by middleware)
//! - User must own all provided assets
//! - Created document and assets are owned by the same user

use actix_web::{post, web, HttpResponse, Responder};
use crate::routes::vocal_tour::create_vocal_tour_request::CreateVocalTourRequest;
use crate::routes::vocal_tour::create_vocal_tour_response::CreateVocalTourResponse;
use crate::routes::vocal_tour::document_template::VOCAL_TOUR_DOCUMENT_TEMPLATE;
use crate::routes::assets::save_assets_from_gcs::GcsAssetData;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/vocal-tour",
    tag = "Vocal Tour",
    request_body = CreateVocalTourRequest,
    responses(
        (status = 200, description = "Vocal tour created successfully", body = CreateVocalTourResponse),
        (status = 400, description = "Bad Request - Invalid asset IDs or no assets provided"),
        (status = 404, description = "Assets not found or access denied"),
        (status = 500, description = "Internal Server Error - Vocal tour generation or DB error")
    ),
    security(("user_auth" = []))
)]
#[post("")]
#[instrument(skip(pool, claims, req, http_req, session_manager))]
pub async fn create_vocal_tour(
    pool: web::Data<sqlx::PgPool>,
    claims: web::ReqData<crate::auth::tokens::Claims>,
    req: web::Json<CreateVocalTourRequest>,
    http_req: actix_web::HttpRequest,
    session_manager: web::Data<std::sync::Arc<crate::services::session_manager::HybridSessionManager>>,
) -> impl Responder {
    let user_id = claims.user_id;
    let CreateVocalTourRequest { asset_ids, collection_id } = req.into_inner();

    // 1. Validate we have assets to process
    if asset_ids.is_empty() {
        log::warn!("User {user_id} attempted to create vocal tour with no assets");
        return HttpResponse::BadRequest().json(
            crate::routes::assets::error_response::ErrorResponse {
                error: "At least one asset ID must be provided".into(),
            },
        );
    }

    // 2. Parse and validate collection_id if provided
    let collection_uuid = if let Some(collection_id_str) = collection_id {
        match uuid::Uuid::parse_str(&collection_id_str) {
            std::result::Result::Ok(id) => std::option::Option::Some(id),
            std::result::Result::Err(e) => {
                log::warn!("Invalid collection ID format '{collection_id_str}' for user {user_id}: {e}");
                return HttpResponse::BadRequest().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: format!("Invalid collection ID format: {collection_id_str}"),
                    },
                );
            }
        }
    } else {
        std::option::Option::None
    };

    // 3. Parse and validate asset IDs
    let mut asset_uuids = std::vec::Vec::new();
    for asset_id in &asset_ids {
        match uuid::Uuid::parse_str(asset_id) {
            std::result::Result::Ok(id) => asset_uuids.push(id),
            std::result::Result::Err(e) => {
                log::warn!("Invalid asset ID format '{asset_id}' for user {user_id}: {e}");
                return HttpResponse::BadRequest().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: format!("Invalid asset ID format: {asset_id}"),
                    },
                );
            }
        }
    }

    // 4. Fetch and validate all assets belong to the user
    let mut assets = std::vec::Vec::new();
    for asset_uuid in asset_uuids {
        match crate::queries::assets::get_asset_by_id::get_asset_by_id(&pool, asset_uuid).await {
            std::result::Result::Ok(std::option::Option::Some(asset)) if asset.user_id == Some(user_id) => {
                assets.push(asset);
            }
            std::result::Result::Ok(std::option::Option::Some(_)) => {
                log::warn!("User {user_id} attempted to use asset {asset_uuid} owned by another user");
                return HttpResponse::NotFound().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: format!("Asset {asset_uuid} not found or access denied"),
                    },
                );
            }
            std::result::Result::Ok(std::option::Option::None) => {
                log::warn!("Asset {asset_uuid} not found for user {user_id}");
                return HttpResponse::NotFound().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: format!("Asset {asset_uuid} not found"),
                    },
                );
            }
            std::result::Result::Err(e) => {
                log::error!("Database error fetching asset {asset_uuid} for user {user_id}: {e}");
                return HttpResponse::InternalServerError().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: "Failed to fetch assets".into(),
                    },
                );
            }
        }
    }

    // 5. Call the workflow orchestration logic
    match execute_vocal_tour_workflow(&pool, user_id, assets, collection_uuid, &http_req, &session_manager).await {
        std::result::Result::Ok(response) => {
            log::info!("Successfully created vocal tour for user {}: document '{}' with {} assets", 
                       user_id, response.document.title, response.created_assets.len());
            
            // TODO: Credit deduction is currently disabled for vocal tours
            // When re-enabling, use the organization-aware credit deduction pattern:
            //
            // 1. Extract organization_id from header (recommended) or add to request body:
            //    let organization_id = http_req
            //        .headers()
            //        .get("x-organization-id")
            //        .and_then(|h| h.to_str().ok())
            //        .and_then(|s| uuid::Uuid::parse_str(s).ok());
            //
            // 2. Use deduct_user_credits_with_transaction (supports both personal and org credits):
            //    let credits_to_consume = crate::app_constants::credits_constants::CreditsConsumption::VOCAL_TOUR;
            //    let deduction_params = crate::queries::user_credit_allocation::CreditChangesParams {
            //        user_id,
            //        organization_id,  // ← This enables org credit support
            //        credits_to_change: credits_to_consume,
            //        action_source: "api".to_string(),
            //        action_type: "vocal_tour".to_string(),
            //        entity_id: Some(response.document.id),
            //    };
            //    if let Err(e) = crate::queries::user_credit_allocation::deduct_user_credits_with_transaction(&pool, deduction_params).await {
            //        log::error!("Failed to deduct {} credits for user {}: {}", credits_to_consume, user_id, e);
            //    }
            //
            // See /api/assets/enhance for a working example of the header-based pattern.
            HttpResponse::Ok().json(response)
        }
        std::result::Result::Err(e) => {
            log::error!("Vocal tour workflow failed for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Vocal tour generation failed".into(),
                },
            )
        }
    }
}

/// Creates the final response from document data and created assets.
fn create_final_response(
    vocal_tour_id: uuid::Uuid,
    document_data: crate::queries::documents::insert_document_entry::InsertedDocumentData,
    created_assets: std::vec::Vec<crate::db::assets::Asset>,
) -> std::result::Result<CreateVocalTourResponse, std::string::String> {
    // Convert InsertedDocumentData to Document
    let document = crate::db::documents::Document {
        id: document_data.id,
        user_id: document_data.user_id,
        title: document_data.title,
        content: document_data.content,
        sources: document_data.sources,
        status: document_data.status,
        created_at: document_data.created_at,
        updated_at: document_data.updated_at,
        is_public: document_data.is_public,
        is_task: document_data.is_task,
        include_research: document_data.include_research.and_then(|s| s.parse().ok()),
        collection_id: document_data.collection_id, // Use the collection_id from document_data
    };

    let response = CreateVocalTourResponse {
        vocal_tour_id,
        document,
        total_assets: created_assets.len(),
        created_assets,
    };

    std::result::Result::Ok(response)
}

/// Executes the vocal tour workflow orchestration logic.
/// 
/// This private function contains the core step-by-step logic of the vocal tour workflow:
/// 1. Calls handle_vocal_tour with asset GCS URLs
/// 2. Parses the response to extract property information
/// 3. Creates a formatted HTML document
/// 4. Saves extracted images as new assets
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `user_id` - ID of the authenticated user
/// * `assets` - Vector of validated assets owned by the user
/// * `collection_id` - Optional collection ID to attach the document and assets to
/// 
/// # Returns
/// 
/// A `Result` containing the `CreateVocalTourResponse` on success, or an error message.
async fn execute_vocal_tour_workflow(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    assets: std::vec::Vec<crate::db::assets::Asset>,
    collection_id: std::option::Option<uuid::Uuid>,
    http_req: &actix_web::HttpRequest,
    session_manager: &std::sync::Arc<crate::services::session_manager::HybridSessionManager>,
) -> std::result::Result<CreateVocalTourResponse, std::string::String> {
    // Step 1: Analyze Content using handle_vocal_tour
    log::info!("Starting vocal tour analysis for user {} with {} assets", user_id, assets.len());

    // Categorize assets by type for the vocal tour parameters
    let mut photos = std::vec::Vec::new();
    let mut videos = std::vec::Vec::new();
    let mut documents = std::vec::Vec::new();

    for asset in &assets {
        if asset.r#type.starts_with("image/") {
            photos.push(asset.url.clone());
        } else if asset.r#type.starts_with("video/") {
            videos.push(asset.url.clone());
        } else {
            documents.push(asset.url.clone());
        }
    }

    // Create VocalTourParams
    let vocal_tour_params = crate::agent_tools::tool_params::vocal_tour_params::VocalTourParams {
        documents: if documents.is_empty() { std::option::Option::None } else { std::option::Option::Some(documents) },
        photos: if photos.is_empty() { std::option::Option::None } else { std::option::Option::Some(photos) },
        videos: if videos.is_empty() { std::option::Option::None } else { std::option::Option::Some(videos) },
        retouch_prompt: std::option::Option::None, // Per instructions, don't specify retouch_prompt
        user_id: std::option::Option::Some(user_id),
        organization_id: std::option::Option::None,
    };

    // Call handle_vocal_tour and measure processing time
    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
    let vocal_tour_start = std::time::Instant::now();
    let (full_response, _user_response) = match crate::agent_tools::handlers::handle_vocal_tour::handle_vocal_tour(pool, vocal_tour_params, user_id).await {
        std::result::Result::Ok(responses) => responses,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Vocal tour analysis failed from agent: {e}"));
        }
    };
    let vocal_tour_processing_time = vocal_tour_start.elapsed();

    // Debug: Print the actual response structure
    log::info!("📊 VOCAL TOUR RESPONSE STRUCTURE:");
    log::info!("{}", serde_json::to_string_pretty(&full_response.response).unwrap_or_else(|_| "Failed to serialize".to_string()));
    
    // Also print to stdout for test visibility
    println!("📊 VOCAL TOUR RESPONSE STRUCTURE:");
    println!("{}", serde_json::to_string_pretty(&full_response.response).unwrap_or_else(|_| "Failed to serialize".to_string()));

    // Note: Vocal tour analytics event will be logged at the end with complete outcome metrics

    // Step 2: Create Document
    log::info!("Parsing vocal tour response for user {user_id}");

    // Parse the response to extract property information
    let property_data = match parse_vocal_tour_response(&full_response.response) {
        std::result::Result::Ok(data) => data,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to parse vocal tour response: {e}"));
        }
    };

    // Format the final HTML document
    let final_html = VOCAL_TOUR_DOCUMENT_TEMPLATE
        .replace("{title}", &property_data.title)
        .replace("{body}", &property_data.formatted_body);

    // Create the document in database
    let mut tx = match pool.begin().await {
        std::result::Result::Ok(tx) => tx,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to start database transaction: {e}"));
        }
    };

    let document_data = match crate::queries::documents::insert_document_entry::insert_document_entry(
        &mut tx,
        std::option::Option::Some(user_id),
        &std::string::String::from("Walkthru Description"),
        &final_html,
        &[std::string::String::from("vocal-tour")],
        false, // is_public
        false, // is_task
        std::option::Option::Some(crate::db::document_research_usage::DocumentResearchUsage::Never),
        collection_id, // collection_id
    ).await {
        std::result::Result::Ok(data) => data,
        std::result::Result::Err(e) => {
            let _ = tx.rollback().await;
            return std::result::Result::Err(format!("Failed to create document: {e}"));
        }
    };

    // Commit document transaction
    if let std::result::Result::Err(e) = tx.commit().await {
        return std::result::Result::Err(format!("Failed to commit document transaction: {e}"));
    }

    // Step 3: Save Assets
    log::info!("Saving {} extracted images as assets for user {}", property_data.image_urls.len(), user_id);

    let mut assets_to_save = std::vec::Vec::new();
    for (index, image_data) in property_data.image_urls.iter().enumerate() {
        // Extract GCS object name from URL
        let gcs_object_name = match extract_object_name_from_gcs_url(&image_data.url) {
            std::result::Result::Ok(name) => name,
            std::result::Result::Err(e) => {
                log::warn!("Failed to extract object name from GCS URL '{}': {}", image_data.url, e);
                continue; // Skip this asset and continue with others
            }
        };

        // Generate asset name following the convention: {property_tour_title}_{number}__{image_title}
        let sanitized_title = property_data.title
            .replace(" ", "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
        let sanitized_image_title = image_data.title
            .replace(" ", "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
        
        let asset_name = format!("{}_{}__{}", sanitized_title, index + 1, sanitized_image_title);

        let asset_data = GcsAssetData {
            name: asset_name,
            r#type: std::string::String::from("image/webp"), // Most vocal tour images are webp
            gcs_url: image_data.url.clone(),
            gcs_object_name,
            collection_id: collection_id.map(|id| id.to_string()), // Convert Uuid to String
        };

        assets_to_save.push(asset_data);
    }

    // Save all assets
    let created_assets = match crate::routes::assets::save_assets_from_gcs::save_assets_from_gcs_urls(pool, user_id, assets_to_save, true).await {
        std::result::Result::Ok(assets) => assets,
        std::result::Result::Err(e) => {
            log::error!("Failed to save vocal tour assets for user {user_id}: {e}");
            // Don't fail the entire operation if asset saving fails - the document was created successfully
            std::vec::Vec::new()
        }
    };

    // Step 4: Create Vocal Tour Entity
    log::info!("Creating vocal tour entity for user {} with document {} and {} assets", user_id, document_data.id, created_assets.len());
    
    // Collect asset IDs
    let asset_ids: std::vec::Vec<uuid::Uuid> = created_assets.iter().map(|asset| asset.id).collect();
    
    // Create vocal tour entity in database
    let mut vocal_tour_tx = match pool.begin().await {
        std::result::Result::Ok(tx) => tx,
        std::result::Result::Err(e) => {
            log::error!("Failed to start vocal tour transaction for user {user_id}: {e}");
            // Don't fail the entire operation if vocal tour creation fails - document and assets were created successfully
            log::warn!("Continuing without vocal tour entity creation");
            return std::result::Result::Err(format!("Vocal tour analysis failed: {e}"));
        }
    };

    match crate::queries::vocal_tours::insert_vocal_tour::insert_vocal_tour(
        &mut vocal_tour_tx,
        user_id,
        document_data.id,
        &asset_ids,
    ).await {
        std::result::Result::Ok(vocal_tour) => {
            if let std::result::Result::Err(e) = vocal_tour_tx.commit().await {
                log::error!("Failed to commit vocal tour transaction for user {user_id}: {e}");
                log::warn!("Continuing without vocal tour entity - document and assets were created successfully");
                std::result::Result::Err(format!("Failed to commit the transaction to create the vocal tour: {e}"))
            } else {
                log::info!("Successfully created vocal tour entity {} for user {}", vocal_tour.id, user_id);
                
                // Step 4.5: Log Complete Vocal Tour Analytics Event (if events feature enabled)
                #[cfg(feature = "events")]
                {
                    // Extract real request context from HTTP request and get session
                    let request_context = match extract_request_context_from_http_request(http_req, user_id, session_manager).await {
                        std::result::Result::Ok(context) => context,
                        std::result::Result::Err(e) => {
                            log::warn!("Failed to extract request context for vocal tour analytics: {}", e);
                            // Fallback to minimal context if extraction fails
                            crate::services::events_service::request_context::RequestData {
                                method: "POST".to_string(),
                                path: "/api/vocal-tour".to_string(),
                                full_url: "unknown".to_string(),
                                query_string: "".to_string(),
                                headers: std::collections::HashMap::new(),
                                query_params: serde_json::json!({}),
                                user_agent: Some("unknown".to_string()),
                                ip_address: Some("unknown".to_string()),
                                real_ip: None,
                                forwarded_for: None,
                                scheme: "https".to_string(),
                                host: "unknown".to_string(),
                                port: None,
                                http_version: "HTTP/1.1".to_string(),
                                content_type: Some("application/json".to_string()),
                                content_length: None,
                                content_encoding: None,
                                accept_language: None,
                                accept_encoding: None,
                                request_body: None,
                                request_body_size: None,
                                request_body_truncated: false,
                                user_registration_date: None,
                                cookies: std::collections::HashMap::new(),
                                request_id: uuid::Uuid::new_v4().to_string(),
                                timestamp: chrono::Utc::now(),
                                user_id: Some(user_id),
                                session_id: None,
                            }
                        }
                    };
                    
                    // Extract created asset IDs
                    let created_asset_ids: Vec<uuid::Uuid> = created_assets.iter().map(|asset| asset.id).collect();
                    
                    let _ = crate::services::events_service::vocal_tour_events::log_vocal_tour_gennodes_response(
                        pool, 
                        user_id, 
                        &full_response.response,
                        &request_context,
                        vocal_tour_processing_time.as_millis() as u64,
                        // NEW: Outcome metrics
                        Some(document_data.id),
                        &created_asset_ids,
                        Some(vocal_tour.id),
                    ).await;
                }
                
                // Step 5: Return Results
                create_final_response(vocal_tour.id, document_data, created_assets)
            }
        }
        std::result::Result::Err(e) => {
            let _ = vocal_tour_tx.rollback().await;
            log::error!("Failed to create vocal tour entity for user {user_id}: {e}");
            std::result::Result::Err(format!("Failed to create vocal tour entity for user {user_id}: {e}"))
        }
    }
}

/// Parses the vocal tour response to extract property data.
fn parse_vocal_tour_response(response: &serde_json::Value) -> std::result::Result<PropertyData, std::string::String> {
    // Parse the vocal tour response to extract property description
    let property_data = response
        .get("data")
        .and_then(|data| data.get("PropertyDescription"))
        .ok_or_else(|| "Missing PropertyDescription in response".to_string())?;

    let title = property_data
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| std::string::String::from("Missing or invalid title in PropertyDescription"))?
        .to_string();

    let body = property_data
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| std::string::String::from("Missing or invalid body in PropertyDescription"))?;

    let voice_over_transcript = property_data
        .get("voiceOverTranscript")
        .and_then(|v| v.as_str());

    // Extract image URLs from the body (look for GCS URLs in href attributes)
    let image_urls = extract_image_urls_from_html(body)?;

    // Format the body with transcript if present
    let mut formatted_body = format!("<h1>{title}</h1>\n{body}");
    
    if let std::option::Option::Some(transcript) = voice_over_transcript {
        formatted_body.push_str(&format!(
            r#"

<h2>Voiceover Transcript</h2>
<div class="transcript">
<p>{transcript}</p>
</div>"#
        ));
    }

    // Ensure all image links open in new tab by adding target="_blank" if not present
    formatted_body = formatted_body.replace("<a href=\"https://storage.googleapis.com", "<a href=\"https://storage.googleapis.com");
    if !formatted_body.contains("target=\"_blank\"") {
        formatted_body = formatted_body.replace("<a href=\"", "<a href=\"").replace("\">", "\" target=\"_blank\">");
    }

    std::result::Result::Ok(PropertyData {
        title,
        formatted_body,
        image_urls,
    })
}

/// Parsed property data from vocal tour response
struct PropertyData {
    title: std::string::String,
    formatted_body: std::string::String,
    image_urls: std::vec::Vec<ImageData>,
}

/// Image data extracted from vocal tour response
struct ImageData {
    url: std::string::String,
    title: std::string::String,
}

/// Extracts image URLs and titles from HTML content.
/// 
/// Looks for anchor tags containing GCS URLs and extracts both the URL and the link text.
/// 
/// # Arguments
/// 
/// * `html` - HTML content to parse
/// 
/// # Returns
/// 
/// A `Result` containing a vector of `ImageData` or an error message.
fn extract_image_urls_from_html(html: &str) -> std::result::Result<std::vec::Vec<ImageData>, std::string::String> {
    let mut image_urls = std::vec::Vec::new();
    
    // Simple regex to find GCS URLs in href attributes
    // Look for pattern: <a href="https://storage.googleapis.com/..."><b>Title</b></a>
    let lines = html.lines();
    for line in lines {
        if line.contains("storage.googleapis.com") && line.contains("<a href=") {
            // Extract URL from href attribute
            if let std::option::Option::Some(url_start) = line.find("href=\"") {
                let url_start = url_start + 6; // Skip 'href="'
                if let std::option::Option::Some(url_end) = line[url_start..].find("\"") {
                    let url = line[url_start..url_start + url_end].to_string();
                    
                    // Extract title from <b>Title</b> or just the link text
                    let title = if let std::option::Option::Some(title_start) = line.find("<b>") {
                        let title_start = title_start + 3; // Skip '<b>'
                        if let std::option::Option::Some(title_end) = line[title_start..].find("</b>") {
                            line[title_start..title_start + title_end].to_string()
                        } else {
                            format!("Image {}", image_urls.len() + 1)
                        }
                    } else {
                        format!("Image {}", image_urls.len() + 1)
                    };
                    
                    image_urls.push(ImageData { url, title });
                }
            }
        }
    }
    
    std::result::Result::Ok(image_urls)
}

/// Extracts the GCS object name from a GCS URL.
/// 
/// Converts a URL like "https://storage.googleapis.com/bucket/path/file.jpg"
/// to just "path/file.jpg"
/// 
/// # Arguments
/// 
/// * `gcs_url` - The full GCS URL
/// 
/// # Returns
/// 
/// A `Result` containing the object name or an error message.
fn extract_object_name_from_gcs_url(gcs_url: &str) -> std::result::Result<std::string::String, std::string::String> {
    // Remove the base GCS URL part
    let base_url = "https://storage.googleapis.com/";
    if !gcs_url.starts_with(base_url) {
        return std::result::Result::Err(format!("Invalid GCS URL: {gcs_url}"));
    }
    
    let remaining = &gcs_url[base_url.len()..];
    
    // Find the first slash to separate bucket from object path
    if let std::option::Option::Some(slash_pos) = remaining.find('/') {
        let object_name = &remaining[slash_pos + 1..];
        if object_name.is_empty() {
            return std::result::Result::Err(format!("Empty object name in GCS URL: {gcs_url}"));
        }
        std::result::Result::Ok(object_name.to_string())
    } else {
        std::result::Result::Err(format!("Invalid GCS URL format: {gcs_url}"))
    }
}

/// Extracts request context from HTTP request for custom events
/// This function gets real IP, user agent, headers, and session ID
/// Only compiled when the "events" feature is enabled
#[cfg(feature = "events")]
async fn extract_request_context_from_http_request(
    http_req: &actix_web::HttpRequest,
    user_id: uuid::Uuid,
    session_manager: &std::sync::Arc<crate::services::session_manager::HybridSessionManager>,
) -> std::result::Result<crate::services::events_service::request_context::RequestData, std::string::String> {
    // Extract basic request info
    let method = http_req.method().to_string();
    let path = http_req.path().to_string();
    let query_string = http_req.query_string().to_string();
    let scheme = if http_req.connection_info().scheme() == "https" { "https" } else { "http" };
    let host = http_req.connection_info().host().to_string();
    let full_url = format!("{}://{}{}", scheme, host, path);
    
    // Extract headers
    let mut headers = std::collections::HashMap::new();
    for (name, value) in http_req.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }
    
    // Extract IP address
    let connection_info = http_req.connection_info();
    let ip_address = connection_info.realip_remote_addr()
        .or_else(|| connection_info.peer_addr())
        .map(|addr| addr.split(':').next().unwrap_or(addr).to_string());
    
    // Extract user agent
    let user_agent = headers.get("user-agent").cloned();
    
    // Get session ID using session manager
    let session_id = match session_manager.get_or_create_session(user_id).await {
        Ok(session) => Some(session),
        Err(e) => {
            log::warn!("Failed to get session for user {}: {}", user_id, e);
            None
        }
    };
    
    // Extract specific headers before moving headers map
    let content_type = headers.get("content-type").cloned();
    let content_length = headers.get("content-length")
        .and_then(|v| v.parse::<u64>().ok());
    let content_encoding = headers.get("content-encoding").cloned();
    let accept_language = headers.get("accept-language").cloned();
    let accept_encoding = headers.get("accept-encoding").cloned();

    Ok(crate::services::events_service::request_context::RequestData {
        method,
        path,
        full_url,
        query_string,
        headers,
        query_params: serde_json::json!({}), // Could parse query string if needed
        user_agent,
        ip_address,
        real_ip: None, // Could extract from X-Real-IP header if needed
        forwarded_for: None, // Could extract from X-Forwarded-For if needed
        scheme: scheme.to_string(),
        host,
        port: None, // Could extract port from host if needed
        http_version: format!("{:?}", http_req.version()),
        content_type,
        content_length,
        content_encoding,
        accept_language,
        accept_encoding,
        request_body: None, // Would need to be extracted during middleware processing
        request_body_size: None,
        request_body_truncated: false,
        user_registration_date: None, // Not needed for custom events
        cookies: std::collections::HashMap::new(), // Could parse Cookie header if needed
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        user_id: Some(user_id),
        session_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_object_name_from_gcs_url() {
        let url = "https://storage.googleapis.com/real-estate-videos/6f37170b-ab0a-423a-b959-54527fafc059_0_00_frame_1.webp";
        let result = extract_object_name_from_gcs_url(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "6f37170b-ab0a-423a-b959-54527fafc059_0_00_frame_1.webp");
    }

    #[test]
    fn test_extract_image_urls_from_html() {
        let html = r#"<li><a href="https://storage.googleapis.com/real-estate-videos/test.webp"><b>Test Image</b></a> - Description</li>"#;
        let result = extract_image_urls_from_html(html);
        assert!(result.is_ok());
        let images = result.unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].url, "https://storage.googleapis.com/real-estate-videos/test.webp");
        assert_eq!(images[0].title, "Test Image");
    }

    #[tokio::test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    async fn test_e2e_vocal_tour_workflow() {
        // Load environment variables from .env file
        dotenvy::dotenv().ok();
        
        println!("🧪 Testing END-TO-END vocal tour workflow with REAL calls!");
        println!("📋 This test will create actual database records and call real services");
        
        // Test asset ID provided by user
        let test_asset_id = "309c93e4-fd2b-4ff0-834b-b5384ba6ed7d";
        println!("🎯 Using asset ID: {}", test_asset_id);
        
        // Setup database connection
        let database_url = match std::env::var("DATABASE_URL") {
            std::result::Result::Ok(url) => {
                println!("✅ Found DATABASE_URL");
                url
            }
            std::result::Result::Err(_) => {
                println!("❌ DATABASE_URL environment variable not found");
                println!("💡 Please set DATABASE_URL in your .env file");
                panic!("Missing DATABASE_URL");
            }
        };
        
        println!("🔧 Connecting to database...");
        let pool = match sqlx::PgPool::connect(&database_url).await {
            std::result::Result::Ok(pool) => {
                println!("✅ Database connected successfully");
                pool
            }
            std::result::Result::Err(e) => {
                println!("❌ Failed to connect to database: {}", e);
                panic!("Database connection failed: {}", e);
            }
        };
        
        // Parse asset UUID
        let asset_uuid = match uuid::Uuid::parse_str(test_asset_id) {
            std::result::Result::Ok(uuid) => {
                println!("✅ Asset UUID parsed successfully: {}", uuid);
                uuid
            }
            std::result::Result::Err(e) => {
                println!("❌ Failed to parse asset UUID: {}", e);
                panic!("Invalid asset UUID: {}", e);
            }
        };
        
        // Fetch the asset to verify it exists and get its details
        println!("🔍 Fetching asset from database...");
        let asset = match crate::queries::assets::get_asset_by_id::get_asset_by_id(&pool, asset_uuid).await {
            std::result::Result::Ok(std::option::Option::Some(asset)) => {
                println!("✅ Asset found:");
                println!("   ID: {}", asset.id);
                println!("   Name: {}", asset.name);
                println!("   Type: {}", asset.r#type);
                println!("   URL: {}", asset.url);
                println!("   User ID: {:?}", asset.user_id);
                asset
            }
            std::result::Result::Ok(std::option::Option::None) => {
                println!("❌ Asset {} not found in database", test_asset_id);
                panic!("Asset not found");
            }
            std::result::Result::Err(e) => {
                println!("❌ Database error fetching asset: {}", e);
                panic!("Database error: {}", e);
            }
        };
        
        // Test the complete workflow orchestration
        println!("\n🚀 Starting vocal tour workflow execution...");
        let assets = vec![asset.clone()];
        
        println!("📊 Asset categorization:");
        let mut photos = std::vec::Vec::new();
        let mut videos = std::vec::Vec::new();
        let mut documents = std::vec::Vec::new();
        
        for test_asset in &assets {
            if test_asset.r#type.starts_with("image/") {
                photos.push(test_asset.url.clone());
                println!("   📸 Image: {}", test_asset.url);
            } else if test_asset.r#type.starts_with("video/") {
                videos.push(test_asset.url.clone());
                println!("   🎥 Video: {}", test_asset.url);
            } else {
                documents.push(test_asset.url.clone());
                println!("   📄 Document: {}", test_asset.url);
            }
        }
        
        // Execute the workflow (using None for http_req and session_manager in tests)
        match execute_vocal_tour_workflow(&pool, asset.user_id.expect("Asset should have a user_id"), assets, std::option::Option::None, &actix_web::test::TestRequest::get().to_http_request(), &std::sync::Arc::new(crate::services::session_manager::HybridSessionManager::new(pool.clone()))).await {
            std::result::Result::Ok(response) => {
                println!("\n🎉 VOCAL TOUR WORKFLOW COMPLETED SUCCESSFULLY!");
                println!("==========================================");
                
                // Print document details
                println!("📄 CREATED DOCUMENT:");
                println!("   ID: {}", response.document.id);
                println!("   Title: {}", response.document.title);
                println!("   User ID: {:?}", response.document.user_id);
                println!("   Sources: {:?}", response.document.sources);
                println!("   Status: {}", response.document.status);
                println!("   Is Public: {}", response.document.is_public);
                println!("   Is Task: {}", response.document.is_task);
                println!("   Created At: {}", response.document.created_at);
                
                // Print content preview (first 500 chars)
                let content_preview = if response.document.content.len() > 500 {
                    format!("{}...", &response.document.content[..500])
                } else {
                    response.document.content.clone()
                };
                println!("   Content Preview: {}", content_preview);
                
                // Print asset details
                println!("\n🖼️ CREATED ASSETS: {} total", response.created_assets.len());
                for (index, created_asset) in response.created_assets.iter().enumerate() {
                    println!("   Asset {}: ", index + 1);
                    println!("     ID: {}", created_asset.id);
                    println!("     Name: {}", created_asset.name);
                    println!("     Type: {}", created_asset.r#type);
                    println!("     URL: {}", created_asset.url);
                    println!("     GCS Object: {}", created_asset.gcs_object_name);
                    println!("     User ID: {:?}", created_asset.user_id);
                    println!("     Created At: {:?}", created_asset.created_at);
                }
                
                println!("\n📈 SUMMARY:");
                println!("   Total Assets Created: {}", response.total_assets);
                println!("   Document ID: {}", response.document.id);
                println!("   Document Title: {}", response.document.title);
                
                println!("\n✅ All data has been saved to the database successfully!");
                println!("🔗 You can verify the document in the documents table with ID: {}", response.document.id);
                println!("🔗 You can verify the assets in the assets table with user_id: {:?}", asset.user_id);
                
                // Additional verification - check if document exists in DB
                println!("\n🔍 Verifying document was saved to database...");
                match crate::queries::documents::find_document_by_id_and_user::find_document_by_id_and_user(
                    &pool, 
                    response.document.id, 
                    response.document.user_id.unwrap()
                ).await {
                    std::result::Result::Ok(std::option::Option::Some(_)) => {
                        println!("✅ Document verification successful - found in database");
                    }
                    std::result::Result::Ok(std::option::Option::None) => {
                        println!("⚠️ Document verification failed - not found in database");
                    }
                    std::result::Result::Err(e) => {
                        println!("⚠️ Document verification error: {}", e);
                    }
                }
                
                // Verify assets were saved
                println!("🔍 Verifying {} assets were saved to database...", response.created_assets.len());
                let mut verified_assets = 0;
                for created_asset in &response.created_assets {
                    match crate::queries::assets::get_asset_by_id::get_asset_by_id(&pool, created_asset.id).await {
                        std::result::Result::Ok(std::option::Option::Some(_)) => {
                            verified_assets += 1;
                        }
                        _ => {}
                    }
                }
                println!("✅ Asset verification: {}/{} assets found in database", verified_assets, response.created_assets.len());
                
            }
            std::result::Result::Err(e) => {
                println!("\n❌ VOCAL TOUR WORKFLOW FAILED:");
                println!("   Error: {}", e);
                panic!("Workflow failed: {}", e);
            }
        }
        
        println!("\n🏁 End-to-end test completed successfully!");
    }
} 