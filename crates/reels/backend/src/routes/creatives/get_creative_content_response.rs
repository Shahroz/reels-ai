//! Defines the response structure for the get_creative_content endpoint.
//!
//! This structure includes the HTML content of a creative and a flag indicating
//! whether the content represents a draft version or the published version.
//! It is designed to provide clients with both the creative's markup and its status.

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct GetCreativeContentResponse {
   #[schema(example = "<html><body><h1>Creative Content</h1></body></html>")]
   pub html_content: std::string::String,
   #[schema(example = true)]
   pub is_draft: bool,
   #[schema(example = json!(std::vec!["#ff0000".to_string(), "rgba(0,255,0,0.5)".to_string()]), value_type = Option<Vec<String>>)]
   pub extracted_colors: Option<std::collections::HashSet<std::string::String>>,
}
