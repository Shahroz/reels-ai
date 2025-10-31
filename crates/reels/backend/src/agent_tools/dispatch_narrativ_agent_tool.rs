//! Dispatches agent tool requests to appropriate handlers from the agentloop.
//!
//! This function acts as a ToolHandler for the reels_app backend,
//! routing ToolChoice requests to specific tool implementations within the
//! agentloop crate. It validates parameters and calls the respective agentloop tool.
//! Adheres strictly to project Rust coding standards: one item per file, fully qualified paths.

// Note: No 'use' statements allowed. All paths must be fully qualified.
// Assumes 'agentloop' is a crate dependency and its types are accessible via 'agentloop::...'.
// Assumes 'actix_web' and 'serde_json' are available dependencies.
// Removed: use crate::GLOBAL_POOL; // Will use crate::main::get_global_pool().await instead

pub fn dispatch_narrativ_agent_tool(
    tool_choice: agentloop::types::tool_choice::ToolChoice,
    app_state: actix_web::web::Data<agentloop::state::app_state::AppState>,
    session_id: agentloop::types::session_id::SessionId,
) -> std::pin::Pin<
    std::boxed::Box<
        dyn std::future::Future<
                Output = std::result::Result<
                    (
                        agentloop::types::full_tool_response::FullToolResponse,
                        agentloop::types::user_tool_response::UserToolResponse,
                    ),
                    std::string::String,
                >,
            > + Send,
    >,
> {
    std::boxed::Box::pin(async move {
        // Extract user_id and organization_id with minimal lock duration
        let (user_id, organization_id) = {
            let sessions = app_state.sessions.lock().await;
            let session = sessions.get(&session_id).expect("Cannot find the session");
            (session.user_id, session.organization_id)
        }; // Lock released here

        // a small hack we inject user_id and organization_id into the JSON value
        // the agent does not have access to the user_id and organization_id and we would not want it to know it
        // either way - so we can inject it here from the session settings
        let mut parameters = tool_choice.parameters.clone();

        // 1. Get the outer map as a mutable reference
        if let Some(outer) = parameters.as_object_mut() {
            // 2. Grab the *first* value in that map (whatever the key is)
            if let Some(inner_val) = outer.values_mut().next() {
                // 3. Ensure it's an object and insert your user_id
                if let serde_json::Value::Object(inner_map) = inner_val {
                    inner_map.insert(
                        "user_id".to_string(),
                        serde_json::Value::String(user_id.to_string()),
                    );

                    // Also inject organization_id if present
                    if let Some(org_id) = organization_id {
                        inner_map.insert(
                            "organization_id".to_string(),
                            serde_json::Value::String(org_id.to_string()),
                        );
                    }
                }
            }
        }

        let narrativ_tool_params_result: std::result::Result<
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters,
            serde_json::Error,
        > = serde_json::from_value(parameters.clone());

        match narrativ_tool_params_result {
            std::result::Result::Ok(parsed_tool_params) => {
                // Successfully parsed into a specific tool's parameter structure.
                // Now, dispatch to the correct handler based on the enum variant.
                match parsed_tool_params {
                   crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::Search(params) => {
                       crate::agent_tools::handlers::handle_narrativ_search::handle_narrativ_search(params).await
                   }
                  crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::BrowseRaw(p) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    crate::agent_tools::handlers::handle_narrativ_browse_raw::handle_narrativ_browse_raw(p, user_id, pool).await
                  }
                  crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::BrowseWithQuery(p) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    crate::agent_tools::handlers::handle_narrativ_browse_with_query::handle_narrativ_browse_with_query(p, user_id, pool).await
                  }
                  crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GoogleSearchBrowse(p) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    crate::agent_tools::handlers::handle_google_search_browse::handle_google_search_browse(p, user_id, pool).await
                  }
                  crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::SaveContext(params) => {
                      crate::agent_tools::handlers::handle_narrativ_save_context::handle_narrativ_save_context(params, app_state.clone(), session_id).await
                  }
                  // --- BEGIN: New dispatch arms for User DB Collection and Item tools ---
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::CreateUserDbCollection(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_create_user_db_collection::handle_create_user_db_collection(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::DeleteUserDbCollection(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_delete_user_db_collection::handle_delete_user_db_collection(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GetUserDbCollection(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_get_user_db_collection::handle_get_user_db_collection(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListUserDbCollections(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_list_user_db_collections::handle_list_user_db_collections(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::UpdateUserDbCollection(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_update_user_db_collection::handle_update_user_db_collection(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::UpdateUserDbCollectionSchema(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_update_user_db_collection_schema::handle_update_user_db_collection_schema(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::CreateUserDbCollectionItem(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_create_user_db_collection_item::handle_create_user_db_collection_item(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::DeleteUserDbCollectionItem(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_delete_user_db_collection_item::handle_delete_user_db_collection_item(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GetUserDbCollectionItem(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_get_user_db_collection_item::handle_get_user_db_collection_item(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListUserDbCollectionItemsTool(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_list_user_db_collection_items_tool::handle_list_user_db_collection_items_tool(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::QueryUserDbCollectionItems(params) => {
                     let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                     let mut params = params.clone();
                     params.user_id = Some(user_id);
                     crate::agent_tools::handlers::handle_query_user_db_collection_items::handle_query_user_db_collection_items(params, pool).await
                 }
                 crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::UpdateUserDbCollectionItem(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_update_user_db_collection_item::handle_update_user_db_collection_item(params, pool).await
                }
                // --- BEGIN: New dispatch arms for Narrativ Document tools ---
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentCount(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_narrativ_document_count::handle_narrativ_document_count(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentDelete(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_narrativ_document_delete::handle_narrativ_document_delete(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentFetchList(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_narrativ_document_fetch_list::handle_narrativ_document_fetch_list(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentFindById(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_narrativ_document_find_by_id::handle_narrativ_document_find_by_id(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentInsert(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_narrativ_document_insert::handle_narrativ_document_insert(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentUpdate(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_narrativ_document_update::handle_narrativ_document_update(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListAssets(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_list_assets::handle_list_assets(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::SaveAsset(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_save_asset::handle_save_asset(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListBundles(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_list_bundles::handle_list_bundles(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListCollections(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_list_collections::handle_list_collections(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListFormats(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_list_formats::handle_list_formats(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListStyles(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_list_styles::handle_list_styles(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GenerateStyleFromUrl(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let gcs_client = crate::services::gcs::gcs_client::GCSClient::new();
                    let screenshot_config = crate::services::screenshot::screenshot_config::ScreenshotConfig::from_env();
                    let screenshot_service = crate::services::screenshot::service_factory::create_screenshot_service(&screenshot_config)
                        .map_err(|e| format!("Failed to create screenshot service: {e}"))?;
                   let mut params = params.clone();
                   params.user_id = Some(user_id);
                   params.organization_id = organization_id;
                   crate::agent_tools::handlers::handle_generate_style_from_url::handle_generate_style_from_url(pool, &gcs_client, screenshot_service.as_ref(), params, user_id).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GenerateCreative(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let gcs_client = std::sync::Arc::new(crate::services::gcs::gcs_client::GCSClient::new());
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_generate_creative::handle_generate_creative(params, pool, gcs_client).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GenerateCreativeFromBundle(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let gcs_client = std::sync::Arc::new(crate::services::gcs::gcs_client::GCSClient::new());
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_generate_creative_from_bundle::handle_generate_creative_from_bundle(params, pool, gcs_client).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::CreateCollection(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_create_collection::handle_create_collection(params, pool).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::PropertyResearch(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_property_research::handle_property_research(pool, params, user_id).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::PropertyDescriptionToContents(params) => {
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    crate::agent_tools::handlers::handle_property_description_to_contents::handle_property_description_to_contents(params).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::RetouchImages(params) => {
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_retouch_images::handle_retouch_images(params).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::VocalTour(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_vocal_tour::handle_vocal_tour(pool, params, user_id).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::QuickEnhanceImage(params) => {
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_quick_enhance_image::handle_quick_enhance_image(params).await
                }
                crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GenerateReel(params) => {
                    let pool: &sqlx::PgPool = &crate::db_pool::GLOBAL_POOL;
                    let gcs_client = crate::services::gcs::gcs_client::GCSClient::new();
                    let mut params = params.clone();
                    params.user_id = Some(user_id);
                    params.organization_id = organization_id;
                    crate::agent_tools::handlers::handle_generate_reel::handle_generate_reel(params, pool, &gcs_client).await
                }
               }
            }
            std::result::Result::Err(e) => {
                // Failed to deserialize parameters for a known tool.
                std::result::Result::Err(format!(
                    "Failed to parse parameters for tool : {}. Parameters (json): {}",
                    e,
                    serde_json::to_string(&tool_choice.parameters)
                        .unwrap_or_else(|_| std::string::String::from("Unserializable parameters"))
                ))
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    // For items from the parent module (this file), use `super::`.
    // For all other items (std::, agentloop::, crate:: for other modules), use FQN.
    // `actix_rt::test` is used for async tests. Ensure `actix-rt` is a dev-dependency.
    // `uuid` crate for generating test UUIDs. Ensure it's a dev-dependency.

    // Mocking/constructing agentloop types (AppState, SessionId, ToolChoice) is necessary for tests.
    // These helpers are used across multiple tests.
    // This assumes these types have public constructors or factory methods usable for testing.
    // For example, `AppState::new_mock()` or similar might be needed.
    // If `agentloop::types::tool_choice::ToolChoice` fields are not public, constructing it for tests will require a helper or public constructor.

    fn create_mock_app_state() -> actix_web::web::Data<agentloop::state::app_state::AppState> {
        // This is a placeholder. A real AppState might be complex.
        // Assuming AppState has a way to be instantiated for tests, e.g. a `new_mock()` method.
        // If AppState is just a struct with public fields, it can be constructed directly.
        // Replace with actual AppState construction logic for tests.
        // Using AppState::new with default/empty values.
        actix_web::web::Data::new(agentloop::state::app_state::AppState::new(
            agentloop::config::app_config::AppConfig::default(), // config
            None,                                                // tool_schemas
            None,                                                // tool_handler
        ))
    }

    fn create_mock_session_id() -> agentloop::types::session_id::SessionId {
        uuid::Uuid::nil() // Use Uuid::nil() for a mock session ID, as SessionId is likely a type alias for uuid::Uuid
    }

    fn create_mock_tool_choice(
        _name: &str,
        params: serde_json::Value,
    ) -> agentloop::types::tool_choice::ToolChoice {
        agentloop::types::tool_choice::ToolChoice { parameters: params }
    }
}
