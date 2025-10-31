//! Provides the main function to convert `NarrativToolParameters` into `GeminiTools`.
//!
//! This module iterates through all variants of `NarrativToolParameters`,
//! extracts their names, descriptions, and parameter schemas (by generating
//! a JSON schema via `schemars` and then converting it), and assembles them
//! into the `GeminiTools` structure expected by the Gemini API.
//! Adheres strictly to project Rust coding standards.

use anyhow::anyhow;
use llm::vendors::gemini::function_declaration::FunctionDeclaration;

/// Converts all defined `NarrativToolParameters` into a `GeminiTools` structure.
///
/// This function iterates over each variant of `NarrativToolParameters`:
/// 1. Derives the tool name (snake_case).
/// 2. Retrieves the tool description.
/// 3. Generates a JSON schema for the parameters of that specific tool variant.
/// 4. Converts this JSON schema into the `GeminiSchema` format.
/// 5. Assembles these into `GeminiFunctionDeclaration` objects.
///
/// # Returns
/// A `Result` containing `GeminiTools` if successful, or an error string if any part of the
/// conversion fails (e.g., schema generation or parsing). The `Tool` struct contains all declarations.
#[allow(clippy::too_many_lines)] // The match statement is inherently long.
pub fn convert_narrativ_tools_to_gemini_format() -> anyhow::Result<Vec<FunctionDeclaration>> {
    let mut declarations = Vec::new();
   // Requires NarrativToolParameters to derive strum::IntoEnumIterator, ToString, EnumProperties
   for tool_variant in <crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters as strum::IntoEnumIterator>::iter() {
       let tool_name = tool_variant.to_string(); // Relies on #[strum(serialize_all = "snake_case")]
       let tool_description = strum::EnumProperty::get_str(&tool_variant, "description")
           .ok_or_else(|| {
               anyhow!("Missing description for tool variant: {:?}", tool_variant.as_ref())
           })?
            .to_string();

        // Generate JSON schema for the parameters of the specific variant.
        // This requires matching on the variant to know its specific parameter type.
        let params_json_schema_value = match tool_variant {
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::Search(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::search_params::SearchParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::BrowseRaw(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::browse_raw_params::BrowseRawParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::BrowseWithQuery(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::browse_with_query_params::BrowseWithQueryParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GoogleSearchBrowse(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::google_search_browse_params::GoogleSearchBrowseParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::SaveContext(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::save_context_params::SaveContextParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::CreateUserDbCollection(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::create_user_db_collection_params::CreateUserDbCollectionParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::DeleteUserDbCollection(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::delete_user_db_collection_params::DeleteUserDbCollectionParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GetUserDbCollection(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::get_user_db_collection_params::GetUserDbCollectionParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListUserDbCollections(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::list_user_db_collections_params::ListUserDbCollectionsParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::UpdateUserDbCollection(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::update_user_db_collection_params::UpdateUserDbCollectionParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::UpdateUserDbCollectionSchema(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::update_user_db_collection_schema_params::UpdateUserDbCollectionSchemaParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::CreateUserDbCollectionItem(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::create_user_db_collection_item_params::CreateUserDbCollectionItemParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::DeleteUserDbCollectionItem(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::delete_user_db_collection_item_params::DeleteUserDbCollectionItemParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GetUserDbCollectionItem(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::get_user_db_collection_item_params::GetUserDbCollectionItemParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListUserDbCollectionItemsTool(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::list_user_db_collection_items_tool_params::ListUserDbCollectionItemsToolParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::QueryUserDbCollectionItems(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::query_user_db_collection_items_params::QueryUserDbCollectionItemsParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::UpdateUserDbCollectionItem(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::update_user_db_collection_item_params::UpdateUserDbCollectionItemParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentCount(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::narrativ_document_count_params::NarrativDocumentCountParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentDelete(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::narrativ_document_delete_params::NarrativDocumentDeleteParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentFetchList(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::narrativ_document_fetch_list_params::NarrativDocumentFetchListParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentFindById(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::narrativ_document_find_by_id_params::NarrativDocumentFindByIdParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentInsert(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::narrativ_document_insert_params::NarrativDocumentInsertParams)
                )
            }
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::NarrativDocumentUpdate(..) => {
                serde_json::to_value(
                    schemars::schema_for!(crate::agent_tools::tool_params::narrativ_document_update_params::NarrativDocumentUpdateParams)
                )
            }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GenerateStyleFromUrl(..) => {
               serde_json::to_value(schemars::schema_for!(
                   crate::agent_tools::tool_params::generate_style_from_url_params::GenerateStyleFromUrlParams
               ))
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GenerateCreative(..) => {
               serde_json::to_value(schemars::schema_for!(
                   crate::agent_tools::tool_params::generate_creative_params::GenerateCreativeParams
               ))
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::GenerateCreativeFromBundle(..) => {
               serde_json::to_value(schemars::schema_for!(
                   crate::agent_tools::tool_params::generate_creative_from_bundle_params::GenerateCreativeFromBundleParams
               ))
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListAssets(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::list_assets_params::ListAssetsParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::SaveAsset(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::save_asset_params::SaveAssetParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListBundles(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::list_bundles_params::ListBundlesParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListStyles(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::list_styles_params::ListStylesParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListCollections(..) => {
               serde_json::to_value(schemars::schema_for!(
                   crate::agent_tools::tool_params::list_collections_params::ListCollectionsParams
               ))
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::CreateCollection(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::create_collection_params::CreateCollectionParams)
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::ListFormats(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::list_formats_params::ListFormatsParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::PropertyResearch(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::property_research_params::PropertyResearchParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::PropertyDescriptionToContents(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::property_description_to_contents_params::PropertyDescriptionToContentsParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::RetouchImages(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::retouch_images_params::RetouchImagesParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::VocalTour(..) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::vocal_tour_params::VocalTourParams),
               )
           }
           crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters::QuickEnhanceImage(_) => {
               serde_json::to_value(
                   schemars::schema_for!(crate::agent_tools::tool_params::quick_enhance_image_params::QuickEnhanceImageParams),
               )
           }
        }
        .map_err(|e| anyhow!("Failed to serialize schema to JSON for {}: {}", tool_name, e))?;

        declarations.push(
            llm::vendors::gemini::function_declaration::FunctionDeclaration {
                name: tool_name,
                description: tool_description,
                parameters: params_json_schema_value, // Use the serde_json::Value directly
            },
        );
    }
    // Wrap all function declarations into a single Tool object.
    // If no tools are defined, an empty Vec of declarations is fine.
    Ok(declarations)
}
