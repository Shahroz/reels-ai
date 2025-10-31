//! Handler for creating marketing content from property description with studio journey integration.
//!
//! This endpoint extends the basic property contents creation to immediately create a Content Studio
//! journey with pending document nodes for all 9 marketing content types. Users are then directed
//! to the studio to watch real-time generation progress with visual indicators.

use actix_web::post;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreatePropertyContentsWithStudioJourneyRequest {
    /// ID of the source document containing property description
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub source_document_id: uuid::Uuid,
    
    /// Whether to start in studio mode with journey creation
    #[schema(example = true)]
    pub start_in_studio: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CreatePropertyContentsWithStudioJourneyResponse {
    /// List of created marketing documents (may include pending documents)
    pub documents: std::vec::Vec<crate::db::documents::Document>,
    /// Total number of documents created (including pending)
    pub total_documents: usize,
    /// Source document that was used for generation
    pub source_document: crate::db::documents::Document,
    /// Content Studio journey ID (if start_in_studio is true)
    pub journey_id: std::option::Option<uuid::Uuid>,
    /// Root document ID for the journey (same as source_document_id)
    pub journey_root_document_id: uuid::Uuid,
    /// Whether generation is happening in the background (all documents complete together)
    pub is_async_generation: bool,
}

/// Content type definitions for the 9 marketing content types
#[derive(std::fmt::Debug, std::clone::Clone)]
pub struct ContentTypeDefinition {
    pub content_type: std::string::String,
    pub title: std::string::String,
    pub platform_name: std::string::String,
    pub description: std::string::String,
}

/// Get all 9 content type definitions
pub fn get_content_type_definitions() -> std::vec::Vec<ContentTypeDefinition> {
    vec![
        ContentTypeDefinition {
            content_type: "mls_public_remarks".to_string(),
            title: "MLS Public Remarks".to_string(),
            platform_name: "MLS Platform".to_string(),
            description: "Public remarks for MLS listing".to_string(),
        },
        ContentTypeDefinition {
            content_type: "portal_full_description".to_string(),
            title: "Real Estate Portal Description".to_string(),
            platform_name: "Real Estate Portal".to_string(),
            description: "Full property description for real estate portals".to_string(),
        },
        ContentTypeDefinition {
            content_type: "headline_bullets".to_string(),
            title: "Property Headlines & Bullets".to_string(),
            platform_name: "Marketing Materials".to_string(),
            description: "Property highlights with bullet points".to_string(),
        },
        ContentTypeDefinition {
            content_type: "instagram_feed_caption".to_string(),
            title: "Instagram Feed Caption".to_string(),
            platform_name: "Instagram".to_string(),
            description: "Instagram post caption for property listing".to_string(),
        },
        ContentTypeDefinition {
            content_type: "reels_tiktok_caption".to_string(),
            title: "Reels/TikTok Caption".to_string(),
            platform_name: "Reels/TikTok".to_string(),
            description: "Short-form video caption for Reels and TikTok".to_string(),
        },
        ContentTypeDefinition {
            content_type: "facebook_post".to_string(),
            title: "Facebook Post".to_string(),
            platform_name: "Facebook".to_string(),
            description: "Facebook social media post for property".to_string(),
        },
        ContentTypeDefinition {
            content_type: "google_business_profile_post".to_string(),
            title: "Google Business Profile Post".to_string(),
            platform_name: "Google Business Profile".to_string(),
            description: "Google Business Profile post for property listing".to_string(),
        },
        ContentTypeDefinition {
            content_type: "email_newsletter_listing".to_string(),
            title: "Email Newsletter Listing".to_string(),
            platform_name: "Email Newsletter".to_string(),
            description: "Email newsletter property listing content".to_string(),
        },
        ContentTypeDefinition {
            content_type: "sms_whatsapp_message".to_string(),
            title: "SMS/WhatsApp Message".to_string(),
            platform_name: "SMS/WhatsApp".to_string(),
            description: "Text message content for SMS and WhatsApp".to_string(),
        },
    ]
}

#[utoipa::path(
    post,
    path = "/api/property-contents/with-studio-journey",
    tag = "Property Contents",
    request_body = CreatePropertyContentsWithStudioJourneyRequest,
    responses(
        (status = 200, description = "Marketing content creation started with studio journey", body = CreatePropertyContentsWithStudioJourneyResponse),
        (status = 400, description = "Bad Request - Invalid document ID or missing property description"),
        (status = 404, description = "Source document not found or access denied"),
        (status = 500, description = "Internal Server Error - Content generation failed")
    ),
    security(("user_auth" = []))
)]
#[post("/with-studio-journey")]
pub async fn create_property_contents_with_studio_journey(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    req: actix_web::web::Json<CreatePropertyContentsWithStudioJourneyRequest>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let source_document_id = req.source_document_id;
    let start_in_studio = req.start_in_studio;

    log::info!("Creating property contents with studio journey for user {} from document {}, studio mode: {}", 
               user_id, source_document_id, start_in_studio);

    match execute_property_contents_with_studio_workflow(&pool, user_id, source_document_id, start_in_studio).await {
        std::result::Result::Ok(response) => {
            log::info!("Successfully created {} documents (including pending) for user {} from document {}, journey: {:?}", 
                       response.total_documents, user_id, source_document_id, response.journey_id);
            actix_web::HttpResponse::Ok().json(response)
        }
        std::result::Result::Err(e) => {
            log::error!("Property contents with studio journey creation failed for user {} and document {}: {}", 
                        user_id, source_document_id, e);
            
            if e.contains("not found") || e.contains("access denied") {
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: e,
                    }
                )
            } else if e.contains("Insufficient content") || e.contains("Invalid document") {
                actix_web::HttpResponse::BadRequest().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: e,
                    }
                )
            } else {
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: "Property contents with studio journey creation failed".into(),
                    }
                )
            }
        }
    }
}

/// Executes the property contents workflow with studio journey integration.
/// 
/// This function orchestrates the creation of property marketing content with real-time
/// studio experience:
/// 1. Validates user ownership and fetches source document
/// 2. Creates Content Studio journey with source document as root
/// 3. Pre-creates 9 pending document nodes for all content types
/// 4. Starts GenNodes workflow asynchronously
/// 5. Updates documents from pending to complete as content is generated
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `user_id` - ID of the authenticated user
/// * `source_document_id` - ID of the source document containing property description
/// * `start_in_studio` - Whether to create studio journey and pending documents
/// 
/// # Returns
/// 
/// A `Result` containing the response with journey information on success.
async fn execute_property_contents_with_studio_workflow(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    source_document_id: uuid::Uuid,
    start_in_studio: bool,
) -> std::result::Result<CreatePropertyContentsWithStudioJourneyResponse, std::string::String> {
    // Step 1: Validate user ownership and fetch source document
    log::info!("Fetching source document {} for user {}", source_document_id, user_id);
    
    let source_document = match crate::queries::documents::find_document_by_id_and_user::find_document_by_id_and_user(
        pool, 
        source_document_id, 
        user_id
    ).await {
        std::result::Result::Ok(std::option::Option::Some(doc)) => doc,
        std::result::Result::Ok(std::option::Option::None) => {
            return std::result::Result::Err("Source document not found or access denied".to_string());
        }
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to fetch source document: {}", e));
        }
    };

    // Extract collection_id for inheritance
    let source_collection_id = source_document.collection_id;
    log::info!("Source document has collection_id: {:?}", source_collection_id);

    if !start_in_studio {
        // Fall back to original workflow without studio journey
        return execute_original_workflow(pool, user_id, source_document, source_collection_id).await;
    }

    // Step 2: Create Content Studio journey with source document as root
    log::info!("🎬 CREATE_STUDIO_JOURNEY START: user_id={}, source_document_id={}", user_id, source_document_id);
    
    let journey_id = match create_studio_journey_for_property_contents(
        pool, 
        user_id, 
        source_document_id
    ).await {
        std::result::Result::Ok(id) => {
            log::info!("✅ JOURNEY CREATED: journey_id={}, user_id={}, source_document_id={}", id, user_id, source_document_id);
            id
        },
        std::result::Result::Err(e) => {
            log::error!("💥 JOURNEY CREATION FAILED: user_id={}, source_document_id={}, error={}", user_id, source_document_id, e);
            return std::result::Result::Err(format!("Failed to create studio journey: {}", e));
        }
    };

    // Step 2.1: Create a node for the root document (source document)
    log::info!("🌳 CREATE_ROOT_NODE START: source_document_id={}, journey_id={}", source_document_id, journey_id);
    
    let root_node_id = match crate::queries::documents::lineage::get_or_create_document_node::get_or_create_document_node(
        pool,
        journey_id,
        source_document_id,
        std::option::Option::None, // No parent for root node
        std::option::Option::None, // No custom prompt for source document
    ).await {
        std::result::Result::Ok(node) => {
            log::info!("✅ ROOT_NODE CREATED: root_node_id={}, source_document_id={}, journey_id={}", node.id, source_document_id, journey_id);
            node.id
        },
        std::result::Result::Err(e) => {
            log::error!("💥 ROOT_NODE CREATION FAILED: source_document_id={}, journey_id={}, error={}", source_document_id, journey_id, e);
            return std::result::Result::Err(format!("Failed to create root node: {}", e));
        }
    };

    log::info!("✅ STUDIO_JOURNEY SETUP COMPLETE: journey_id={}, root_node_id={}", journey_id, root_node_id);

    // Step 3: Pre-create pending document nodes for all 9 content types
    let content_types = get_content_type_definitions();
    let mut created_documents = std::vec::Vec::new();
    
    log::info!("📝 CREATE_PENDING_DOCS START: count={}, journey_id={}", content_types.len(), journey_id);
    
    let mut tx = match pool.begin().await {
        std::result::Result::Ok(tx) => tx,
        std::result::Result::Err(e) => {
            log::error!("💥 TRANSACTION START FAILED: journey_id={}, error={}", journey_id, e);
            return std::result::Result::Err(format!("Failed to start database transaction: {}", e));
        }
    };

    // Create pending documents for each content type and add them to the journey
    for content_type_def in &content_types {
        log::info!("📄 CREATE_PENDING_DOC: content_type={}, title={}", content_type_def.content_type, content_type_def.title);
        match create_pending_marketing_document(
            &mut tx,
            user_id,
            &content_type_def,
            source_collection_id,
        ).await {
            std::result::Result::Ok(document) => {
                log::info!("✅ PENDING_DOC CREATED: document_id={}, title={}, content_type={}", 
                          document.id, content_type_def.title, content_type_def.content_type);
                created_documents.push(document.clone());
                
                // Add this document to the journey as a child of the root node
                log::info!("🔗 ADD_TO_JOURNEY: document_id={}, journey_id={}, parent_node_id={}", 
                          document.id, journey_id, root_node_id);
                match add_document_to_studio_journey(
                    pool,
                    journey_id,
                    document.id,
                    std::option::Option::Some(root_node_id), // Parent is the root node
                ).await {
                    std::result::Result::Ok(_) => {
                        log::info!("✅ ADDED_TO_JOURNEY: document_id={}, content_type={}, parent_node_id={}", 
                                  document.id, content_type_def.content_type, root_node_id);
                    }
                    std::result::Result::Err(e) => {
                        log::error!("💥 ADD_TO_JOURNEY FAILED: document_id={}, journey_id={}, error={}", 
                                   document.id, journey_id, e);
                        let _ = tx.rollback().await;
                        return std::result::Result::Err(format!("Failed to add document {} to journey: {}", 
                                   document.id, e));
                    }
                }
            }
            std::result::Result::Err(e) => {
                log::error!("💥 PENDING_DOC CREATION FAILED: content_type={}, error={}", 
                           content_type_def.content_type, e);
                let _ = tx.rollback().await;
                return std::result::Result::Err(format!("Failed to create pending document '{}': {}", 
                           content_type_def.title, e));
            }
        }
    }

    // Commit pending documents transaction
    if let std::result::Result::Err(e) = tx.commit().await {
        return std::result::Result::Err(format!("Failed to commit pending documents transaction: {}", e));
    }

    // Documents were already added to journey inside the transaction loop above
    // No need to add them again here

    log::info!("Successfully created {} pending marketing documents", created_documents.len());

    // Step 4: Return response immediately so user can see pending documents in studio
    // GenNodes will run asynchronously in the background
    log::info!("Returning response immediately with {} pending documents", created_documents.len());
    
    // Step 5: Start GenNodes workflow asynchronously (fire and forget)
    let pool_clone = pool.clone();
    let created_documents_clone = created_documents.clone();
    tokio::spawn(async move {
        log::info!("Starting async GenNodes workflow for {} pending documents", created_documents_clone.len());
        
        match execute_gennodes_and_update_pending_documents(
            &pool_clone,
            user_id,
            source_document_id,
            source_collection_id,
            &created_documents_clone,
        ).await {
            std::result::Result::Ok(_) => {
                log::info!("Async GenNodes workflow completed successfully, all pending documents updated");
            }
            std::result::Result::Err(e) => {
                log::error!("Async GenNodes workflow failed: {}", e);
                // Documents remain in pending state - user can see error state in studio
            }
        }
    });
    
    // Step 6: Return response with journey information immediately
    let response = CreatePropertyContentsWithStudioJourneyResponse {
        documents: created_documents,
        total_documents: content_types.len(),
        source_document,
        journey_id: std::option::Option::Some(journey_id),
        journey_root_document_id: source_document_id,
        is_async_generation: true,
    };

    std::result::Result::Ok(response)
}

/// Creates a Content Studio journey for property contents generation.
async fn create_studio_journey_for_property_contents(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    root_document_id: uuid::Uuid,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    match crate::queries::documents::lineage::get_or_create_document_journey::get_or_create_document_journey(
        pool,
        user_id,
        root_document_id,
        std::option::Option::None, // No custom name
    ).await {
        std::result::Result::Ok(journey) => std::result::Result::Ok(journey.id),
        std::result::Result::Err(e) => {
            std::result::Result::Err(format!("Failed to create document journey: {}", e))
        }
    }
}

/// Adds a document to the studio journey as a child node.
async fn add_document_to_studio_journey(
    pool: &sqlx::PgPool,
    journey_id: uuid::Uuid,
    document_id: uuid::Uuid,
    parent_node_id: std::option::Option<uuid::Uuid>,
) -> std::result::Result<uuid::Uuid, std::string::String> {
    match crate::queries::documents::lineage::get_or_create_document_node::get_or_create_document_node(
        pool,
        journey_id,
        document_id,
        parent_node_id,
        std::option::Option::None, // No custom prompt for generated content
    ).await {
        std::result::Result::Ok(node) => std::result::Result::Ok(node.id),
        std::result::Result::Err(e) => {
            std::result::Result::Err(format!("Failed to create document node: {}", e))
        }
    }
}

/// Creates a pending marketing document for a specific content type.
async fn create_pending_marketing_document(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
    content_type_def: &ContentTypeDefinition,
    collection_id: std::option::Option<uuid::Uuid>,
) -> std::result::Result<crate::db::documents::Document, std::string::String> {
    // Create placeholder content with metadata about the pending generation
    let placeholder_content = format!(
        r#"<div class="property-content pending-generation" data-content-type="{}">
    <div class="header">
        <h1>🤖 Generating {}</h1>
        <div class="platform-badge">{}</div>
    </div>
    
    <div class="content-section">
        <div class="generation-status">
            <div class="loading-indicator">
                <div class="robot-animation">🤖</div>
                <p>AI is creating your {} content...</p>
                <div class="progress-bar">
                    <div class="progress-fill"></div>
                </div>
            </div>
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> {}</p>
        <p><strong>Platform:</strong> {}</p>
        <p><strong>Status:</strong> Generating...</p>
        <p><strong>Description:</strong> {}</p>
    </div>
</div>"#,
        content_type_def.content_type,
        content_type_def.title,
        content_type_def.platform_name,
        content_type_def.platform_name,
        content_type_def.title,
        content_type_def.platform_name,
        content_type_def.description
    );

    // Create document with pending status using direct SQL
    let document_record = match sqlx::query!(
        r#"
        INSERT INTO documents (user_id, title, content, sources, status, is_public, is_task, include_research)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING 
            id, user_id, title, content, sources, status, created_at, updated_at, 
            is_public, is_task, include_research
        "#,
        std::option::Option::Some(user_id),
        content_type_def.title.clone(),
        placeholder_content,
        &std::vec::Vec::<std::string::String>::new(), // Empty sources initially
        "pending", // Mark as pending
        false, // Not public
        false, // Not a task
        std::option::Option::None::<&str>
    )
    .fetch_one(&mut **tx)
    .await {
        std::result::Result::Ok(record) => record,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to create pending document: {}", e));
        }
    };

    // Update with collection_id if source had one
    if let std::option::Option::Some(coll_id) = collection_id {
        match sqlx::query!(
            "UPDATE documents SET collection_id = $1, updated_at = NOW() WHERE id = $2",
            coll_id,
            document_record.id
        )
        .execute(&mut **tx)
        .await {
            std::result::Result::Ok(_) => {
                log::debug!("Set collection_id {} for pending document {}", coll_id, document_record.id);
            }
            std::result::Result::Err(e) => {
                return std::result::Result::Err(format!("Failed to set collection_id: {}", e));
            }
        }
    }

    // Note: Document will be added to journey after transaction commits

    // Convert to full Document structure
    let document = crate::db::documents::Document {
        id: document_record.id,
        user_id: document_record.user_id,
        title: document_record.title,
        content: document_record.content,
        sources: document_record.sources,
        status: document_record.status,
        created_at: document_record.created_at,
        updated_at: document_record.updated_at,
        is_public: document_record.is_public,
        is_task: document_record.is_task,
        include_research: document_record.include_research.map(|s| s.parse().unwrap_or(crate::db::document_research_usage::DocumentResearchUsage::TaskDependent)),
        collection_id,
    };

    std::result::Result::Ok(document)
}

/// Executes GenNodes workflow and updates all pending documents with generated content.
async fn execute_gennodes_and_update_pending_documents(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    source_document_id: uuid::Uuid,
    source_collection_id: std::option::Option<uuid::Uuid>,
    pending_documents: &std::vec::Vec<crate::db::documents::Document>,
) -> std::result::Result<(), std::string::String> {
    // Step 1: Get the source document to extract property description
    let source_document = match crate::queries::documents::find_document_by_id_and_user::find_document_by_id_and_user(
        pool, 
        source_document_id, 
        user_id
    ).await {
        std::result::Result::Ok(std::option::Option::Some(doc)) => doc,
        std::result::Result::Ok(std::option::Option::None) => {
            return std::result::Result::Err("Source document not found or access denied".to_string());
        }
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to fetch source document: {}", e));
        }
    };

    // Step 2: Extract property description from document content
    let property_description = match super::create_property_contents::extract_property_description(&source_document.content) {
        std::result::Result::Ok(desc) => desc,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(e);
        }
    };

    log::info!("Extracted property description: {} characters", property_description.len());

    // Step 3: Call PropertyDescriptionToContents GenNodes workflow
    let params = crate::agent_tools::tool_params::property_description_to_contents_params::PropertyDescriptionToContentsParams {
        property_info: property_description,
        user_id: std::option::Option::Some(user_id),
    };

    let (full_response, _user_response) = match crate::agent_tools::handlers::handle_property_description_to_contents::handle_property_description_to_contents(params).await {
        std::result::Result::Ok(responses) => responses,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("GenNodes workflow failed: {}", e));
        }
    };

    log::info!("GenNodes workflow completed successfully");

    // Step 4: Parse GenNodes response into marketing content items
    let marketing_contents = match super::parse_property_marketing_response::parse_property_marketing_response(
        &full_response.response, 
        source_collection_id
    ) {
        std::result::Result::Ok(contents) => contents,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to parse GenNodes response: {}", e));
        }
    };

    log::info!("Parsed {} marketing content items", marketing_contents.len());

    // Step 5: Update all pending documents with actual generated content
    let mut tx = match pool.begin().await {
        std::result::Result::Ok(tx) => tx,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to start database transaction: {}", e));
        }
    };

    for marketing_content in &marketing_contents {
        // Find the pending document that matches this content type
        let pending_doc = pending_documents.iter().find(|doc| {
            doc.title == marketing_content.title
        });

        if let std::option::Option::Some(doc) = pending_doc {
            // Format the content using the existing template system
            let formatted_content = super::property_content_templates::format_marketing_content(&marketing_content);

            // Update the document with actual content and mark as completed
            match sqlx::query!(
                "UPDATE documents SET content = $1, status = $2, updated_at = NOW() WHERE id = $3",
                formatted_content,
                "completed", // Change from "pending" to "completed"
                doc.id
            )
            .execute(&mut *tx)
            .await {
                std::result::Result::Ok(_) => {
                    log::debug!("Updated pending document {} ('{}') with generated content", doc.id, doc.title);
                }
                std::result::Result::Err(e) => {
                    let _ = tx.rollback().await;
                    return std::result::Result::Err(format!("Failed to update document '{}': {}", doc.title, e));
                }
            }
        } else {
            log::warn!("Could not find pending document for content type: {}", marketing_content.title);
        }
    }

    // Commit all document updates
    if let std::result::Result::Err(e) = tx.commit().await {
        return std::result::Result::Err(format!("Failed to commit document updates: {}", e));
    }

    log::info!("Successfully updated {} pending documents with generated content", marketing_contents.len());
    std::result::Result::Ok(())
}

/// Fallback to original workflow without studio journey integration.
async fn execute_original_workflow(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    source_document: crate::db::documents::Document,
    _source_collection_id: std::option::Option<uuid::Uuid>,
) -> std::result::Result<CreatePropertyContentsWithStudioJourneyResponse, std::string::String> {
    let source_document_id = source_document.id;
    
    // Call the original property contents creation workflow
    match super::create_property_contents::execute_property_contents_workflow(
        pool,
        user_id,
        source_document_id,
    ).await {
        std::result::Result::Ok(original_response) => {
            let response = CreatePropertyContentsWithStudioJourneyResponse {
                documents: original_response.documents,
                total_documents: original_response.total_documents,
                source_document,
                journey_id: std::option::Option::None,
                journey_root_document_id: source_document_id,
                is_async_generation: false,
            };
            std::result::Result::Ok(response)
        }
        std::result::Result::Err(e) => std::result::Result::Err(e),
    }
}
