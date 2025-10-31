//! Resolves HTML content from style creation/update requests.
//!
//! This function handles the complex logic of determining HTML content from either
//! direct html_content or source_url fields. Validates mutual exclusivity,
//! URL format, and integrates with Zyte API for fetching content from URLs.
//! Returns validated HTML content or appropriate HTTP error responses.

/// Input parameters for HTML content resolution
pub struct HtmlContentRequest {
    pub html_content: std::option::Option<std::string::String>,
    pub source_url: std::option::Option<std::string::String>,
    pub context_name: std::string::String, // For logging context (e.g., "style creation")
}

/// Resolves HTML content from request parameters with comprehensive validation
/// 
/// Handles mutual exclusivity of html_content and source_url, validates URLs,
/// and fetches content from external sources using Zyte API when needed.
/// Returns validated HTML content or HTTP error response for immediate API use.
pub async fn fetch_html_from_request(
    request: HtmlContentRequest,
) -> std::result::Result<std::string::String, actix_web::HttpResponse> {
    let html_result = match (&request.html_content, &request.source_url) {
        (Some(_), Some(_)) => {
            log::warn!("Both html_content and source_url provided for {}", request.context_name);
            return std::result::Result::Err(actix_web::HttpResponse::BadRequest().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Provide either html_content or source_url, not both."),
                }
            ));
        }
        (None, Some(url)) => {
            // Validate URL format
            if url.parse::<url::Url>().is_err() {
                log::warn!("Invalid URL format provided for {}: {}", request.context_name, url);
                return std::result::Result::Err(actix_web::HttpResponse::BadRequest().json(
                    crate::routes::error_response::ErrorResponse {
                        error: std::string::String::from("Invalid URL format provided."),
                    }
                ));
            }

            log::info!("Fetching HTML content for {} from URL: {}", request.context_name, url);
            
            // Get Zyte API key from environment
            let api_key = match std::env::var("ZYTE_API_KEY") {
                std::result::Result::Ok(key) => key,
                std::result::Result::Err(_) => {
                    log::error!("ZYTE_API_KEY environment variable not set.");
                    return std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::string::String::from("Server configuration error: Missing API key."),
                        }
                    ));
                }
            };
            
            // Fetch HTML using Zyte client
            let zyte_client = crate::zyte::zyte::ZyteClient::new(api_key);
            match zyte_client.extract_styles_with_fallback(url).await {
                std::result::Result::Ok(fetched_html) => {
                    log::info!("Successfully fetched HTML from {} for {}", url, request.context_name);
                    std::result::Result::Ok(fetched_html)
                }
                std::result::Result::Err(e) => {
                    log::error!("Failed to fetch HTML from URL {} for {}: {}", url, request.context_name, e);
                    std::result::Result::Err(actix_web::HttpResponse::InternalServerError().json(
                        crate::routes::error_response::ErrorResponse {
                            error: std::format!("Failed to fetch style from source URL: {e}"),
                        }
                    ))
                }
            }
        }
        (Some(html), _) => {
            log::info!("Using provided HTML content for {}", request.context_name);
            std::result::Result::Ok(html.clone())
        }
        (None, None) => {
            log::warn!("Neither html_content nor source_url provided for {}", request.context_name);
            return std::result::Result::Err(actix_web::HttpResponse::BadRequest().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Either source_url or html_content must be provided."),
                }
            ));
        }
    };

    let html_content = match html_result {
        std::result::Result::Ok(content) => content,
        std::result::Result::Err(response) => return std::result::Result::Err(response),
    };

    // Validate HTML content is not empty
    if html_content.trim().is_empty() {
        log::warn!("Empty HTML content after fetch for {}", request.context_name);
        return std::result::Result::Err(actix_web::HttpResponse::BadRequest().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("HTML content is empty or invalid."),
            }
        ));
    }

    std::result::Result::Ok(html_content)
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_both_content_and_url_provided() {
        // Test mutual exclusivity validation
        let request = super::HtmlContentRequest {
            html_content: Some(std::string::String::from("<style>test</style>")),
            source_url: Some(std::string::String::from("https://example.com")),
            context_name: std::string::String::from("test"),
        };
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::fetch_html_from_request(request));
        assert!(result.is_err());
    }

    #[test]
    fn test_neither_content_nor_url_provided() {
        // Test missing input validation
        let request = super::HtmlContentRequest {
            html_content: None,
            source_url: None,
            context_name: std::string::String::from("test"),
        };
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::fetch_html_from_request(request));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_url_format() {
        // Test URL format validation
        let request = super::HtmlContentRequest {
            html_content: None,
            source_url: Some(std::string::String::from("not-a-valid-url")),
            context_name: std::string::String::from("test"),
        };
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::fetch_html_from_request(request));
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_html_content() {
        // Test direct HTML content provision
        let html = std::string::String::from("<style>body { color: red; }</style>");
        let request = super::HtmlContentRequest {
            html_content: Some(html.clone()),
            source_url: None,
            context_name: std::string::String::from("test"),
        };
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::fetch_html_from_request(request));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), html);
    }

    #[test]
    fn test_empty_html_content() {
        // Test empty HTML content rejection
        let request = super::HtmlContentRequest {
            html_content: Some(std::string::String::from("   ")),
            source_url: None,
            context_name: std::string::String::from("test"),
        };
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(super::fetch_html_from_request(request));
        assert!(result.is_err());
    }
} 