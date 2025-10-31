//! Defines an enum to represent all possible Reels tool parameters for direct deserialization.
//!
//! This enum uses `#[serde(untagged)]` to allow `serde_json::from_value` to attempt
//! deserialization of `tool_choice.parameters` into one of its variants based on the
//! distinct structure of the parameter types.
//! Adheres strictly to project Rust coding standards.

use schemars::JsonSchema;
use strum_macros::{AsRefStr, Display, EnumIter, EnumProperty};

/// Enum representing the parameters for various Reels tools.
///
/// Used for deserializing the `parameters` field from an `agentloop::types::tool_choice::ToolChoice`.
/// The `#[serde(untagged)]` attribute means Serde will try to deserialize into each variant
/// in order until one succeeds. This requires the parameter structs to be structurally distinct.
#[derive(
    serde::Deserialize, std::fmt::Debug, JsonSchema, EnumIter, Display, EnumProperty, AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum NarrativToolParameters {
   #[schemars(description = "Performs a search engine search")]
   #[strum(props(description = "Performs a search engine search"))]
   Search(crate::agent_tools::tool_params::search_params::SearchParams),
   #[schemars(description = "Fetches the raw HTML content of a web page given its URL. Use 'browse_with_query' for summarization or extraction.")]
   #[strum(props(description = "Fetches the raw HTML content of a web page given its URL. Use 'browse_with_query' for summarization or extraction."))]
   BrowseRaw(crate::agent_tools::tool_params::browse_raw_params::BrowseRawParams),
   #[schemars(description = "PREFERRED. Fetches and processes web page content based on a query. Use this to summarize, answer questions, or extract specific information from a URL.")]
   #[strum(props(description = "PREFERRED. Fetches and processes web page content based on a query. Use this to summarize, answer questions, or extract specific information from a URL."))]
   BrowseWithQuery(crate::agent_tools::tool_params::browse_with_query_params::BrowseWithQueryParams),
   #[schemars(description = "Searches Google for a query or browses a URL, then extracts specific information from the content. Use this for complex research tasks requiring both searching and information extraction. If other scrapers are blocked this is what should be tried next.")]
   #[strum(props(description = "Searches Google for a query or browses a URL, then extracts specific information from the content. Use this for complex research tasks requiring both searching and information extraction. If other scrapers are blocked this is what should be tried next."))]
   GoogleSearchBrowse(crate::agent_tools::tool_params::google_search_browse_params::GoogleSearchBrowseParams),
   #[schemars(description = "Saves a piece of text or data to the current context or a named scratchpad for later retrieval or reference in subsequent steps.")]
   #[strum(props(description = "Saves a piece of text or data to the current context or a named scratchpad for later retrieval or reference in subsequent steps."))]
   SaveContext(crate::agent_tools::tool_params::save_context_params::SaveContextParams),
   #[schemars(description = "Creates a new user-defined database collection. CRITICAL: Before using, ALWAYS check if a suitable collection already exists using 'list_user_db_collections' or 'get_user_db_collection' to avoid duplicates.")]
   #[strum(props(description = "Creates a new user-defined database collection. CRITICAL: Before using, ALWAYS check if a suitable collection already exists using 'list_user_db_collections' or 'get_user_db_collection' to avoid duplicates."))]
   CreateUserDbCollection(crate::agent_tools::tool_params::create_user_db_collection_params::CreateUserDbCollectionParams),
   #[schemars(description = "Deletes an existing user-defined database collection. Ensure the collection name is correct and that its deletion is intended and irreversible.")]
   #[strum(props(description = "Deletes an existing user-defined database collection. Ensure the collection name is correct and that its deletion is intended and irreversible."))]
   DeleteUserDbCollection(crate::agent_tools::tool_params::delete_user_db_collection_params::DeleteUserDbCollectionParams),
   #[schemars(description = "Retrieves details and schema of a specific user-defined database collection by its name or ID. Use this to understand a collection's structure or to verify its existence before attempting to create it.")]
   #[strum(props(description = "Retrieves details and schema of a specific user-defined database collection by its name or ID. Use this to understand a collection's structure or to verify its existence before attempting to create it."))]
   GetUserDbCollection(crate::agent_tools::tool_params::get_user_db_collection_params::GetUserDbCollectionParams),
   #[schemars(description = "Lists all available user-defined database collections. Use this to discover existing collections and their names/IDs before attempting to create a new one or interact with a specific collection.")]
   #[strum(props(description = "Lists all available user-defined database collections. Use this to discover existing collections and their names/IDs before attempting to create a new one or interact with a specific collection."))]
   ListUserDbCollections(crate::agent_tools::tool_params::list_user_db_collections_params::ListUserDbCollectionsParams),
   #[schemars(description = "Updates metadata of an existing user-defined database collection (e.g., description, tags). Does not update the schema; use 'update_user_db_collection_schema' for schema changes.")]
   #[strum(props(description = "Updates metadata of an existing user-defined database collection (e.g., description, tags). Does not update the schema; use 'update_user_db_collection_schema' for schema changes."))]
   UpdateUserDbCollection(crate::agent_tools::tool_params::update_user_db_collection_params::UpdateUserDbCollectionParams),
   #[schemars(description = "Updates the schema of an existing user-defined database collection. This is a significant operation; ensure you understand the implications for existing data and the new schema structure.")]
   #[strum(props(description = "Updates the schema of an existing user-defined database collection. This is a significant operation; ensure you understand the implications for existing data and the new schema structure."))]
   UpdateUserDbCollectionSchema(crate::agent_tools::tool_params::update_user_db_collection_schema_params::UpdateUserDbCollectionSchemaParams),
   #[schemars(description = "Creates a new item within a specified user-defined database collection. CRITICAL: Before creating, ALWAYS query for similar existing items using 'query_user_db_collection_items' or 'get_user_db_collection_item' (if ID is known) to prevent duplicates.")]
   #[strum(props(description = "Creates a new item within a specified user-defined database collection. CRITICAL: Before creating, ALWAYS query for similar existing items using 'query_user_db_collection_items' or 'get_user_db_collection_item' (if ID is known) to prevent duplicates."))]
   CreateUserDbCollectionItem(crate::agent_tools::tool_params::create_user_db_collection_item_params::CreateUserDbCollectionItemParams),
   #[schemars(description = "Deletes an item from a specified user-defined database collection by its ID. Ensure the item ID and collection name/ID are correct.")]
   #[strum(props(description = "Deletes an item from a specified user-defined database collection by its ID. Ensure the item ID and collection name/ID are correct."))]
   DeleteUserDbCollectionItem(crate::agent_tools::tool_params::delete_user_db_collection_item_params::DeleteUserDbCollectionItemParams),
   #[schemars(description = "Retrieves a specific item from a user-defined database collection by its ID. Use this to fetch details of a known item or to verify its existence.")]
   #[strum(props(description = "Retrieves a specific item from a user-defined database collection by its ID. Use this to fetch details of a known item or to verify its existence."))]
   GetUserDbCollectionItem(crate::agent_tools::tool_params::get_user_db_collection_item_params::GetUserDbCollectionItemParams),
   #[schemars(description = "Lists items within a specified user-defined database collection, typically with pagination. Use this for browsing items or as a preliminary step before more specific queries if item details are unknown.")]
   #[strum(props(description = "Lists items within a specified user-defined database collection, typically with pagination. Use this for browsing items or as a preliminary step before more specific queries if item details are unknown."))]
   ListUserDbCollectionItemsTool(crate::agent_tools::tool_params::list_user_db_collection_items_tool_params::ListUserDbCollectionItemsToolParams),
   #[schemars(description = "Queries items within a user-defined database collection based on specified criteria (e.g., filters, semantic query). Use this to find specific items, check for existence before creating new ones, or gather data.")]
   #[strum(props(description = "Queries items within a user-defined database collection based on specified criteria (e.g., filters, semantic query). Use this to find specific items, check for existence before creating new ones, or gather data."))]
   QueryUserDbCollectionItems(crate::agent_tools::tool_params::query_user_db_collection_items_params::QueryUserDbCollectionItemsParams),
   #[schemars(description = "Updates an existing item within a specified user-defined database collection. Requires the item ID and the new data for the item.")]
   #[strum(props(description = "Updates an existing item within a specified user-defined database collection. Requires the item ID and the new data for the item."))]
   UpdateUserDbCollectionItem(crate::agent_tools::tool_params::update_user_db_collection_item_params::UpdateUserDbCollectionItemParams),
   #[schemars(description = "Counts documents in the Narrativ general document store, optionally matching a filter. Useful for understanding data volume or query impact.")]
   #[strum(props(description = "Counts documents in the Narrativ general document store, optionally matching a filter. Useful for understanding data volume or query impact."))]
   NarrativDocumentCount(crate::agent_tools::tool_params::narrativ_document_count_params::NarrativDocumentCountParams),
   #[schemars(description = "Deletes documents from the Narrativ general document store based on IDs or a query. Use with extreme caution as this is irreversible.")]
   #[strum(props(description = "Deletes documents from the Narrativ general document store based on IDs or a query. Use with extreme caution as this is irreversible."))]
   NarrativDocumentDelete(crate::agent_tools::tool_params::narrativ_document_delete_params::NarrativDocumentDeleteParams),
   #[schemars(description = "Fetches a list of documents (or their summaries) from the Narrativ general document store, with options for pagination and filtering. Use for browsing or retrieving multiple documents.")]
   #[strum(props(description = "Fetches a list of documents (or their summaries) from the Narrativ general document store, with options for pagination and filtering. Use for browsing or retrieving multiple documents."))]
   NarrativDocumentFetchList(crate::agent_tools::tool_params::narrativ_document_fetch_list_params::NarrativDocumentFetchListParams),
   #[schemars(description = "Finds and retrieves a specific document from the Narrativ general document store by its unique ID.")]
   #[strum(props(description = "Finds and retrieves a specific document from the Narrativ general document store by its unique ID."))]
   NarrativDocumentFindById(crate::agent_tools::tool_params::narrativ_document_find_by_id_params::NarrativDocumentFindByIdParams),
   #[schemars(description = "Inserts a new document into the Narrativ general document store. CRITICAL: If applicable, search for existing similar documents first using 'narrativ_search' or 'narrativ_document_find_by_id' to avoid duplicates.")]
   #[strum(props(description = "Inserts a new document into the Narrativ general document store. CRITICAL: If applicable, search for existing similar documents first using 'narrativ_search' or 'narrativ_document_find_by_id' to avoid duplicates."))]
   NarrativDocumentInsert(crate::agent_tools::tool_params::narrativ_document_insert_params::NarrativDocumentInsertParams),
   #[schemars(description = "Updates an existing document in the Narrativ general document store, identified by its ID. Can be used to modify content or metadata.")]
   #[strum(props(description = "Updates an existing document in the Narrativ general document store, identified by its ID. Can be used to modify content or metadata."))]
   NarrativDocumentUpdate(crate::agent_tools::tool_params::narrativ_document_update_params::NarrativDocumentUpdateParams),
   #[schemars(description = "Generates a new style from a provided URL. The tool will fetch the URL's content, process it, and create a new style asset.")]
   #[strum(props(description = "Generates a new style from a provided URL. The tool will fetch the URL's content, process it, and create a new style asset."))]
   GenerateStyleFromUrl(crate::agent_tools::tool_params::generate_style_from_url_params::GenerateStyleFromUrlParams),
   #[schemars(description = "Generates a new creative based on specified styles, assets, documents, and formats. This is a comprehensive tool that uses an LLM to generate HTML content.")]
   #[strum(props(description = "Generates a new creative based on specified styles, assets, documents, and formats. This is a comprehensive tool that uses an LLM to generate HTML content."))]
   GenerateCreative(crate::agent_tools::tool_params::generate_creative_params::GenerateCreativeParams),
   #[schemars(description = "Generates a new creative from a predefined bundle. The bundle provides the necessary style, assets, and formats, simplifying the generation process.")]
   #[strum(props(description = "Generates a new creative from a predefined bundle. The bundle provides the necessary style, assets, and formats, simplifying the generation process."))]
   GenerateCreativeFromBundle(
      crate::agent_tools::tool_params::generate_creative_from_bundle_params::GenerateCreativeFromBundleParams,
   ),
   #[schemars(description = "Lists assets available to the user, with optional filters for pagination and searching.")]
   #[strum(props(description = "Lists assets available to the user, with optional filters for pagination and searching."))]
   ListAssets(crate::agent_tools::tool_params::list_assets_params::ListAssetsParams),
   #[schemars(description = "Saves multiple assets to the database with existing GCS URIs. Use this when you have GCS URLs from external tools and want to save them as assets. Accepts an array of assets for batch processing.")]
   #[strum(props(description = "Saves multiple assets to the database with existing GCS URIs. Use this when you have GCS URLs from external tools and want to save them as assets. Accepts an array of assets for batch processing."))]
   SaveAsset(crate::agent_tools::tool_params::save_asset_params::SaveAssetParams),
   #[schemars(description = "Lists all expanded creative bundles available to the user. Bundles group together styles, documents, assets, and formats.")]
   #[strum(props(description = "Lists all expanded creative bundles available to the user. Bundles group together styles, documents, assets, and formats."))]
   ListBundles(crate::agent_tools::tool_params::list_bundles_params::ListBundlesParams),
   #[schemars(description = "Lists styles available to the user, with optional filters for pagination and searching.")]
   #[strum(props(description = "Lists styles available to the user, with optional filters for pagination and searching."))]
   ListStyles(crate::agent_tools::tool_params::list_styles_params::ListStylesParams),
   #[schemars(description = "Lists collections available to the user, with optional filters for pagination and searching.")]
   #[strum(props(description = "Lists collections available to the user, with optional filters for pagination and searching."))]
   ListCollections(crate::agent_tools::tool_params::list_collections_params::ListCollectionsParams),
   #[schemars(description = "Creates a new collection with a name and optional metadata.")]
   #[strum(props(description = "Creates a new collection with a name and optional metadata."))]
   CreateCollection(crate::agent_tools::tool_params::create_collection_params::CreateCollectionParams),
   #[schemars(description = "Lists creative formats available to the user, with optional filters for pagination and searching.")]
   #[strum(props(description = "Lists creative formats available to the user, with optional filters for pagination and searching."))]
   ListFormats(crate::agent_tools::tool_params::list_formats_params::ListFormatsParams),
   #[schemars(description = "Researches a property using web search, AI analysis of provided files, and Perplexity to gather information and populate an MLS Entry form. Analyzes property identifier and optional file URIs.")]
   #[strum(props(description = "Researches a property using web search, AI analysis of provided files, and Perplexity to gather information and populate an MLS Entry form. Analyzes property identifier and optional file URIs."))]
   PropertyResearch(crate::agent_tools::tool_params::property_research_params::PropertyResearchParams),
   #[schemars(description = "Generates comprehensive marketing content collection from property description. Converts property descriptions into various types of marketing content suitable for real estate purposes.")]
   #[strum(props(description = "Generates comprehensive marketing content collection from property description. Converts property descriptions into various types of marketing content suitable for real estate purposes."))]
   PropertyDescriptionToContents(crate::agent_tools::tool_params::property_description_to_contents_params::PropertyDescriptionToContentsParams),
   #[schemars(description = "Retouches a list of images from GCS using an optional prompt. Uses DALL-E processing to enhance, modify, or improve the provided images based on the specified instructions.")]
   #[strum(props(description = "Retouches a list of images from GCS using an optional prompt. Uses DALL-E processing to enhance, modify, or improve the provided images based on the specified instructions."))]
   RetouchImages(crate::agent_tools::tool_params::retouch_images_params::RetouchImagesParams),
   #[schemars(description = "Generates a property description using AI analysis of provided files (videos, photos, documents). Analyzes the content to create detailed property descriptions for real estate purposes.")]
   #[strum(props(description = "Generates a property description using AI analysis of provided files (videos, photos, documents). Analyzes the content to create detailed property descriptions for real estate purposes."))]
   VocalTour(crate::agent_tools::tool_params::vocal_tour_params::VocalTourParams),
   #[schemars(description = "Quickly enhances images using various image enhancement models based on user prompts. Currently uses Gemini 2.5 Flash Image model but designed for flexibility. Returns enhanced image data directly without creating asset records.")]
   #[strum(props(description = "Quickly enhances images using various image enhancement models based on user prompts. Currently uses Gemini 2.5 Flash Image model but designed for flexibility. Returns enhanced image data directly without creating asset records."))]
   QuickEnhanceImage(crate::agent_tools::tool_params::quick_enhance_image_params::QuickEnhanceImageParams),
   #[schemars(description = "Generates a reel (short video) from a product/service URL or text description with a specified time duration. Fetches product information from URL if provided, then creates an engaging video montage using the video-to-montage service.")]
   #[strum(props(description = "Generates a reel (short video) from a product/service URL or text description with a specified time duration. Fetches product information from URL if provided, then creates an engaging video montage using the video-to-montage service."))]
   GenerateReel(crate::agent_tools::tool_params::generate_reel_params::GenerateReelParams),
}

#[cfg(test)]
mod tests {
    // Use super::* to access the item in the parent module (the file scope)
    use schemars::schema_for;
    use serde_json::json;
    use strum::EnumProperty;

    #[test]
    fn test_schema() {
        let schema = schema_for!(super::NarrativToolParameters);
        // Basic assertion to ensure schema generation runs
        assert!(serde_json::to_string_pretty(&schema).is_ok());
    }

    #[test]
    fn test_parsing() {
        let json = json! {{
           "ListUserDbCollections": {
              "user_id": "00000000-0000-0000-0000-000000000000",
              "limit": 20,
              "offset": 0,
              "sort_by_db_col_name": "name",
              "sort_order_db": "asc",
              "search_pattern_db": ""
           }
        }};

        let narrativ_tool_params_result: std::result::Result<
            crate::agent_tools::narrativ_tool_parameters::NarrativToolParameters,
            serde_json::Error,
        > = serde_json::from_value(json.clone());

        println!("{:?}", narrativ_tool_params_result);
    }
}
