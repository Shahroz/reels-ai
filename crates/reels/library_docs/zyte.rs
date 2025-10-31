use gennodes_css_to_tailwind::css_to_tailwind::inline_css_to_tailwind;
use gennodes_css_to_tailwind::css_to_tailwind::get_result_code;
use anyhow::Result;
use base64::encode;
use reqwest::blocking::{Client as SyncClient, RequestBuilder};
use reqwest::{header, Client, Method};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use base64::decode;
use schemars::JsonSchema;
use serde::de::{self, Deserializer, Visitor};
use serde_json::Value;
use std::fmt;
use std::hash::{Hash, Hasher};
use kuchikiki::parse_html;
use kuchikiki::traits::TendrilSink;
use tokio::time::sleep;
use tracing::instrument;

// NEW: Import the scraper crate for HTML parsing
use scraper::{Html, Selector};

/// Represents an action to be executed by the Zyte API.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZyteAction {
    pub action: String,
    pub source: String,
}

/// Modified ZyteRequestData to support additional actions.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde_with::skip_serializing_none]
#[serde(rename_all = "camelCase")]
pub struct ZyteRequestData {
    pub url: String,
    pub browser_html: Option<bool>,
    pub http_response_body: Option<bool>,
    pub screenshot: Option<bool>,
    pub screenshot_options: Option<ScreenshotOptions>,
    pub article: Option<bool>,
    pub article_options: Option<ArticleOptions>,
    pub javascript: Option<bool>,
    pub actions: Option<Vec<ZyteAction>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ArticleOptions {
    pub extract_from: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotOptions {
    full_page: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq)]
#[serde_with::skip_serializing_none]
#[serde(rename_all = "camelCase")]
pub struct ZyteResponseData {
    pub url: Option<String>,
    pub status_code: Option<u16>,

    #[serde(default, deserialize_with = "deserialize_base64")]
    pub http_response_body: Option<String>,
    pub http_response_headers: Option<Value>,
    pub browser_html: Option<String>,
    pub session: Option<Session>,
    pub screenshot: Option<String>,
    pub article: Option<Value>,
    pub article_list: Option<Value>,
    pub article_navigation: Option<Value>,
    pub job_posting: Option<Value>,
    pub product: Option<Value>,
    pub product_list: Option<Value>,
    pub product_navigation: Option<Value>,
    pub echo_data: Option<Value>,
    pub job_id: Option<String>,
    pub actions: Option<Value>,
    pub response_cookies: Option<Value>,
    pub network_capture: Option<Value>,
}

impl Hash for ZyteResponseData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(ref url) = self.url {
            url.hash(state);
        }
        if let Some(status_code) = self.status_code {
            status_code.hash(state);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Eq)]
#[serde(rename_all = "camelCase")]
struct Session {
    id: Option<String>,
}

fn deserialize_base64<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Base64Visitor;

    impl<'de> Visitor<'de> for Base64Visitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a base64 encoded string or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value: String = Deserialize::deserialize(deserializer)?;
            match decode(&value) {
                Ok(bytes) => Ok(Some(
                    String::from_utf8(bytes).unwrap_or_else(|_| value.to_string()),
                )),
                Err(_) => Ok(Some(value)), // Return the original string on error
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match decode(value) {
                Ok(bytes) => Ok(Some(
                    String::from_utf8(bytes).unwrap_or_else(|_| value.to_string()),
                )),
                Err(_) => Ok(Some(value.to_string())), // Return the original string on error
            }
        }
    }

    deserializer.deserialize_option(Base64Visitor)
}

#[derive(Clone, Debug)]
pub struct ZyteClient {
    client: Client,
    auth_header: String,
    api_key: String,
    base_url: String,
}

impl ZyteClient {
    pub fn new(api_key: String) -> anyhow::Result<Self> {
        let auth_value = format!("{}:", api_key);
        let encoded_auth = encode(auth_value);
        let auth_header = format!("Basic {}", encoded_auth);

        Ok(ZyteClient {
            client: Client::builder().build()?,
            api_key: api_key,
            auth_header,
            base_url: "https://api.zyte.com/v1/extract".to_string(),
        })
    }

    /// Sends a request to the Zyte API and returns a parsed `ZyteResponseData`.
    #[instrument(skip(self, data))]
    pub async fn send_request(&self, data: ZyteRequestData) -> Result<ZyteResponseData> {
        let mut attempts = 0;
        let delays = [5, 10, 30];

        loop {
            let response_result = self
                .client
                .request(Method::POST, &self.base_url)
                .header(header::AUTHORIZATION, &self.auth_header)
                .timeout(Duration::from_secs(45))
                .json(&data)
                .send()
                .await;

            match response_result {
                Ok(response) => {
                    let result: ZyteResponseData = response.json().await?;
                    return Ok(result);
                }
                Err(e) => {
                    log::warn!("Request failed for website {} error {:?}", data.url, e);

                    if attempts < 3 {
                        let delay_secs = delays[attempts];
                        attempts += 1;

                        log::info!("Retrying in {} seconds...", delay_secs);
                        sleep(Duration::from_secs(delay_secs)).await; // Wait before retrying
                        continue;
                    } else {
                        return Err(anyhow::Error::new(e));
                    }
                }
            }
        }
    }

    #[instrument(skip(self, data))]
    pub fn send_sync_request(&self, data: ZyteRequestData) -> Result<String> {
        let client = SyncClient::new();
        let mut attempts = 0;
        let delays = [5, 10, 30];

        loop {
            let response_result = client
                .request(Method::POST, &self.base_url)
                .header(header::AUTHORIZATION, &self.auth_header)
                .timeout(Duration::from_secs(45))
                .json(&data)
                .send();

            match response_result {
                Ok(response) => {
                    let text_result = response.text();
                    match text_result {
                        Ok(text) => return Ok(text),
                        Err(e) => {
                            log::warn!(
                                "Failed to read response text for website {} error {:?}",
                                data.url,
                                e
                            );
                            return Err(anyhow::Error::new(e));
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Request failed for website {} error {:?}", data.url, e);

                    if attempts < 3 {
                        let delay_secs = delays[attempts];
                        attempts += 1;

                        log::info!("Retrying in {} seconds...", delay_secs);
                        std::thread::sleep(Duration::from_secs(delay_secs)); // Wait before retrying
                        continue;
                    } else {
                        return Err(anyhow::Error::new(e));
                    }
                }
            }
        }
    }

    pub fn new_from_env() -> anyhow::Result<Self> {
        let api_key = env::var("ZYTE_API_KEY")
            .map_err(|e| anyhow::anyhow!("ZYTE_API_KEY must be set: {}", e))?;
        Self::new(api_key)?
    }

    /// Extracts the computed styles from the given website URL.
    ///
    /// This function sends a request to the Zyte API with JavaScript that creates a hidden
    /// div containing the JSON string of extracted CSS properties from selected elements.
    /// It then parses the returned `browserHtml` to retrieve and deserialize the JSON.
    #[instrument(skip(self))]
    pub async fn extract_styles(&self, url: &str) -> Result<serde_json::Value> {
        // JavaScript snippet to extract styles and store them in a hidden div.
        let js_snippet = include_str!("../backend/src/zyte/zyte_javascript/extract_styles_compressed.js");
        let request_data = ZyteRequestData {
            url: url.to_string(),
            browser_html: Some(true),
            http_response_body: None,
            screenshot: None,
            screenshot_options: None,
            article: None,
            article_options: None,
            javascript: Some(true),
            actions: Some(vec![ZyteAction {
                action: "evaluate".to_string(),
                source: js_snippet.to_string(),
            }]),
        };

        let response = self.send_request(request_data).await?;
        let browser_html = response.browser_html.ok_or_else(|| anyhow::anyhow!("No browserHtml in response"))?;

        // Parse the HTML and extract the content of the hidden div with id "extracted-styles"
        let document = Html::parse_document(&browser_html);
        let selector = Selector::parse("#extracted-styles").map_err(|_| anyhow::anyhow!("Invalid selector '#extracted-styles'"))?;
        let extracted_div = document
            .select(&selector)
            .next()
            .ok_or_else(|| anyhow::anyhow!("No element with id 'extracted-styles' found"))?;

        // Get the text content from the div
        let styles_json_text = extracted_div.text().collect::<Vec<_>>().join("").trim().to_string();
        let styles_json: serde_json::Value = serde_json::from_str(&styles_json_text)?;
        Ok(styles_json)
    }

    /// Extracts inline styles from a website
    /// Extracts inline styles from a website and optionally converts them to Tailwind classes
    #[instrument(skip(self))]
    pub async fn extract_inline_styles(&self, url: &str, translate_to_tailwind: bool) -> Result<String> {
        // JavaScript snippet to ensure we get the full rendered HTML
        let js_snippet = include_str!("../backend/src/zyte/zyte_javascript/extract_inline_styles.js");
        let request_data = ZyteRequestData {
            url: url.to_string(),
            browser_html: Some(true),
            http_response_body: None,
            screenshot: None,
            screenshot_options: None,
            article: None,
            article_options: None,
            javascript: Some(true),
            actions: Some(vec![ZyteAction {
                action: "evaluate".to_string(),
                source: js_snippet.to_string(),
            }]),
        };

        let response = self.send_request(request_data).await?;
        let browser_html = response.browser_html.ok_or_else(|| anyhow::anyhow!("No browserHtml in response"))?;

        if !translate_to_tailwind {
            return Ok(browser_html);
        }

        // Parse the HTML into a Kuchiki document
        let document = parse_html().one(browser_html);

        // Process all elements with a style attribute
        for element in document.select("[style]").map_err(|_| anyhow::anyhow!("Invalid selector"))? {
            let node = element.as_node();
            if let Some(element_data) = node.as_element() {
                // Get the current attributes
                let mut attributes = element_data.attributes.borrow_mut();

                // Extract and remove the style attribute
                if let Some(style) = attributes.remove("style") {
                    // Convert CSS to Tailwind classes
                    let tailwind_classes = inline_css_to_tailwind(&style.value).unwrap_or_default();

                    // Replace the class attribute with Tailwind classes
                    if !tailwind_classes.is_empty() {
                        attributes.insert("class", tailwind_classes);
                    } else {
                        attributes.remove("class");
                    }
                }
            }
        }

        // Serialize the modified document back to HTML
        let mut html = Vec::new();
        document.serialize(&mut html).map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))?;
        Ok(String::from_utf8(html)?)
    }

    #[instrument(skip(self))]
    pub async fn screenshot_website(&self, url: &str, full_page: bool) -> Result<String> {
        let request_data = ZyteRequestData {
            url: url.to_string(),
            browser_html: Some(true),
            http_response_body: None,
            screenshot: Some(true),
            screenshot_options: if full_page {Some(ScreenshotOptions{ full_page: true })} else {None},
            article: None,
            article_options: None,
            javascript: Some(true),
            actions: None,
        };

        let response = self.send_request(request_data).await?;
        let screenshot = response.screenshot.ok_or_else(|| anyhow::anyhow!("No screenshot"))?;
        Ok(screenshot)
    }
}

#[cfg(FALSE)]
mod test {
    use super::*;
    use std::env;
    use dotenvy::dotenv;

    #[test]
    pub fn test_deserialization_zyte_response() {
        let raw_response = include_str!("zyte_response.json");
        let response: ZyteResponseData = serde_json::from_str(&raw_response).unwrap();
        log::debug!("{:#?}", response);
    }

    #[test]
    pub fn test_deserialization_zyte_response_2() {
        let raw_response = include_str!("zyte_response_2.json");
        let response: ZyteResponseData = serde_json::from_str(&raw_response).unwrap();
    }

    // NEW: An asynchronous test that extracts styles from instawork.com and prints them.
    #[tokio::test]
    async fn test_extract_styles_instawork() -> Result<()> {
        dotenv().ok();
        // Ensure the ZYTE_API_KEY environment variable is set.
        let _ = env_logger::builder().is_test(true).try_init();
        let api_key = env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set for tests");
        let client = ZyteClient::new(api_key);

        let url = "https://www.instawork.com/";
        let extracted_styles = client.extract_styles(url).await?;

        log::debug!("Extracted Styles JSON from {}:\n{}", url, serde_json::to_string_pretty(&extracted_styles)?);
        Ok(())
    }

    #[tokio::test]
    async fn test_screenshot() {
        dotenv().ok();
        // Ensure the ZYTE_API_KEY environment variable is set.
        let _ = env_logger::builder().is_test(true).try_init();
        let api_key = env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set for tests");
        let client = ZyteClient::new(api_key);

        let url = "https://instawork.com/";
        let screenshot= client.screenshot_website(url, true).await.unwrap();
        println!("{}", screenshot);
        log::debug!("Screenshot JSON from {}:\n{}", url, screenshot);
    }

    #[tokio::test]
    async fn test_to_tailwind_classes() {
        dotenv().ok();
        // Ensure the ZYTE_API_KEY environment variable is set.
        let _ = env_logger::builder().is_test(true).try_init();
        let api_key = env::var("ZYTE_API_KEY").expect("ZYTE_API_KEY must be set for tests");
        let client = ZyteClient::new(api_key);

        let url = "https://instawork.com/";
        let html= client.extract_inline_styles(url, true).await.unwrap();
        println!("End");
        // println!("{}", html);
    }
}
