//! Gemini File API client for uploading and managing files.
//!
//! This client handles the resumable upload protocol for the Gemini File API,
//! allowing large files (up to 2GB) to be uploaded for processing.
//! Files are automatically deleted after 48 hours.
//! Uses fully qualified paths for dependencies.

use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

// Import the structs from their separate modules
use crate::vendors::gemini::FileInfo;
use crate::vendors::gemini::FileUploadResponse;

/// Gemini File API client.
pub struct FileApiClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl FileApiClient {
    /// Creates a new File API client.
    pub fn new(api_key: String) -> Result<Self, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for uploads
            .build()?;
        
        Ok(Self {
            client,
            api_key,
            base_url: "https://generativelanguage.googleapis.com".to_string(),
        })
    }

    /// Redacts API key from URLs for safe logging.
    fn redact_api_key(&self, url: &str) -> String {
        if url.contains("key=") {
            url.split("key=")
                .next()
                .unwrap_or(url)
                .to_string() + "key=[REDACTED]"
        } else {
            url.to_string()
        }
    }

    /// Uploads a file to the Gemini File API using resumable upload.
    pub async fn upload_file(
        &self,
        file_bytes: &[u8],
        mime_type: &str,
        display_name: &str,
    ) -> Result<FileInfo, Box<dyn Error>> {
        // Step 1: Initiate resumable upload
        let upload_url = self.initiate_upload(file_bytes.len(), mime_type, display_name).await?;
        
        // Step 2: Upload the file data
        let file_info = self.upload_file_data(&upload_url, file_bytes).await?;
        
        // Step 3: Wait for processing to complete
        self.wait_for_processing(&file_info.uri).await
    }

    /// Initiates a resumable upload and returns the upload URL.
    async fn initiate_upload(
        &self,
        file_size: usize,
        mime_type: &str,
        display_name: &str,
    ) -> Result<String, Box<dyn Error>> {
        let request_body = serde_json::json!({
            "file": {
                "display_name": display_name
            }
        });

        let upload_url = format!("{}/upload/v1beta/files?key={}", self.base_url, self.api_key);
        let response = self
            .client
            .post(&upload_url)
            .header("X-Goog-Upload-Protocol", "resumable")
            .header("X-Goog-Upload-Command", "start")
            .header("X-Goog-Upload-Header-Content-Length", file_size.to_string())
            .header("X-Goog-Upload-Header-Content-Type", mime_type)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let redacted_url = self.redact_api_key(&upload_url);
            return Err(format!("Failed to initiate upload to {}: {}", redacted_url, response.status()).into());
        }

        let upload_url = response
            .headers()
            .get("x-goog-upload-url")
            .ok_or("Missing upload URL in response")?
            .to_str()?
            .to_string();

        Ok(upload_url)
    }

    /// Uploads the file data to the provided upload URL.
    async fn upload_file_data(
        &self,
        upload_url: &str,
        file_bytes: &[u8],
    ) -> Result<FileInfo, Box<dyn Error>> {
        let response = self
            .client
            .post(upload_url)
            .header("Content-Length", file_bytes.len().to_string())
            .header("X-Goog-Upload-Offset", "0")
            .header("X-Goog-Upload-Command", "upload, finalize")
            .body(file_bytes.to_vec())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to upload file data: {}", response.status()).into());
        }

        // Get response text first for debugging
        let response_text = response.text().await?;
        log::debug!("Gemini API upload response: {}", response_text);
        
        // Try to parse the response
        let upload_response: FileUploadResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse upload response: {}. Response: {}", e, response_text))?;
        Ok(upload_response.file)
    }

    /// Waits for file processing to complete.
    async fn wait_for_processing(&self, file_uri: &str) -> Result<FileInfo, Box<dyn Error>> {
        let max_attempts = 60; // 5 minutes with 5-second intervals
        
        for attempt in 0..max_attempts {
            let file_info = self.get_file_info(file_uri).await?;
            
            match file_info.state.as_str() {
                "ACTIVE" => return Ok(file_info),
                "PROCESSING" => {
                    log::info!("File still processing, attempt {}/{}", attempt + 1, max_attempts);
                    sleep(Duration::from_secs(5)).await;
                }
                "FAILED" => {
                    return Err(format!("File processing failed for {}", file_uri).into());
                }
                state => {
                    log::warn!("Unknown file state: {}", state);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
        
        Err("File processing timed out".into())
    }

    /// Gets file information from the File API.
    pub async fn get_file_info(&self, file_uri: &str) -> Result<FileInfo, Box<dyn Error>> {
        let request_url = format!("{}?key={}", file_uri, self.api_key);
        let response = self
            .client
            .get(&request_url)
            .send()
            .await?;

        if !response.status().is_success() {
            let redacted_url = self.redact_api_key(&request_url);
            return Err(format!("Failed to get file info from {}: {}", redacted_url, response.status()).into());
        }

        // Get response text first for debugging
        let response_text = response.text().await?;
        log::debug!("Gemini API file info response: {}", response_text);
        
        // Try to parse the response
        let file_info: FileInfo = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse file info response: {}. Response: {}", e, response_text))?;
        Ok(file_info)
    }

    /// Lists all files associated with the API key.
    pub async fn list_files(&self) -> Result<Vec<FileInfo>, Box<dyn Error>> {
        let request_url = format!("{}/v1beta/files?key={}", self.base_url, self.api_key);
        let response = self
            .client
            .get(&request_url)
            .send()
            .await?;

        if !response.status().is_success() {
            let redacted_url = self.redact_api_key(&request_url);
            return Err(format!("Failed to list files from {}: {}", redacted_url, response.status()).into());
        }

        #[derive(serde::Deserialize)]
        struct ListFilesResponse {
            files: Option<Vec<FileInfo>>,
        }

        let list_response: ListFilesResponse = response.json().await?;
        Ok(list_response.files.unwrap_or_default())
    }

    /// Deletes a file from the File API.
    pub async fn delete_file(&self, file_name: &str) -> Result<(), Box<dyn Error>> {
        let request_url = format!("{}/v1beta/{}?key={}", self.base_url, file_name, self.api_key);
        let response = self
            .client
            .delete(&request_url)
            .send()
            .await?;

        if !response.status().is_success() {
            let redacted_url = self.redact_api_key(&request_url);
            return Err(format!("Failed to delete file from {}: {}", redacted_url, response.status()).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_redact_api_key() {
        let client = FileApiClient::new("test_key_123".to_string()).unwrap();
        
        // Test URL with API key
        let url_with_key = "https://api.example.com/files?key=secret_api_key_123";
        let redacted = client.redact_api_key(url_with_key);
        assert_eq!(redacted, "https://api.example.com/files?key=[REDACTED]");
        
        // Test URL without API key
        let url_without_key = "https://api.example.com/files";
        let redacted = client.redact_api_key(url_without_key);
        assert_eq!(redacted, "https://api.example.com/files");
    }
    

} 