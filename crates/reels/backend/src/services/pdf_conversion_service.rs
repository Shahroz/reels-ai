use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::error::Error;
use tracing::instrument;

/// Service for converting websites to PDF using external lighthouse service
pub struct PdfConversionService {
    api_base_url: String,
    api_key: String,
}



impl PdfConversionService {
    /// Create a new PDF conversion service instance
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let api_base_url = std::env::var("LIGHTHOUSE_API_BASE_URL")
            .unwrap_or_else(|_| "https://lighthouse-server-208984921666.us-central1.run.app".to_string());
        
        let api_key = std::env::var("LIGHTHOUSE_API_KEY")
            .map_err(|_| "LIGHTHOUSE_API_KEY environment variable not set")?;
        
        Ok(Self {
            api_base_url,
            api_key,
        })
    }





    /// Convert a website URL to PDF and return raw bytes directly (no file saving)
    #[instrument(skip(self))]
    pub async fn convert_url_to_pdf_direct(
        &self,
        url: &str,
        filename: &str,
        config: Option<String>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let client = reqwest::Client::new();
        
        // Build the request URL
        let request_url = format!("{}/convert_to_pdf", self.api_base_url);
        let mut params = vec![("url", url), ("filename", filename)];
        
        // Add optional config parameter
        if let Some(ref config) = config {
            params.push(("config", config.as_str()));
        }
        
        // Setup headers
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION, 
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?
        );
        
        // Log the exact curl command for debugging
        let curl_command = format!(
            "curl -H \"Authorization: Bearer {}\" \"{}?{}\"",
            self.api_key,
            request_url,
            params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&")
        );
        tracing::info!("Lighthouse API curl command: {}", curl_command);
        
        // Make the request
        tracing::info!("Converting URL to PDF (direct): {}", url);
        let response = client
            .get(&request_url)
            .query(&params)
            .headers(headers)
            .send()
            .await?;
        
        // Check if the response status is successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            tracing::error!("PDF conversion failed with status: {} - {}", status, error_text);
            return Err(format!("PDF conversion failed with status: {status} - {error_text}").into());
        }
        
        // Get the PDF bytes and return them directly
        let pdf_bytes = response.bytes().await?;
        let file_size = pdf_bytes.len();
        
        tracing::info!("PDF conversion completed successfully: {} bytes", file_size);
        
        Ok(pdf_bytes.to_vec())
    }


}

/// Configuration options for PDF conversion
#[derive(Debug, Default)]
pub struct PdfConversionOptions {
    pub filename: Option<String>,
    pub config: Option<String>, // "custom" or "default"
    pub paper_width: Option<f64>, // inches
    pub paper_height: Option<f64>, // inches
    pub landscape: Option<bool>,
    pub include_background: Option<bool>,
    pub margin_top: Option<f64>, // inches
    pub margin_bottom: Option<f64>, // inches
    pub margin_left: Option<f64>, // inches
    pub margin_right: Option<f64>, // inches
    pub scale: Option<f64>, // 0.1 to 2.0
    pub timeout_seconds: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_pdf_service_creation_with_api_key() {
        // Ensure LIGHTHOUSE_API_KEY is available in environment - fail loudly if not
        let api_key = env::var("LIGHTHOUSE_API_KEY")
            .expect("LIGHTHOUSE_API_KEY environment variable must be set for this test");
        
        let result = PdfConversionService::new();
        assert!(result.is_ok(), "PdfConversionService::new() should succeed when LIGHTHOUSE_API_KEY is set");
        
        let service = result.unwrap();
        assert_eq!(service.api_key, api_key);
        assert!(service.api_base_url.contains("lighthouse-server"));
    }

    #[test] 
    fn test_pdf_options_default() {
        let options = PdfConversionOptions::default();
        assert!(options.filename.is_none());
        assert!(options.config.is_none());
        assert!(options.paper_width.is_none());
    }
} 