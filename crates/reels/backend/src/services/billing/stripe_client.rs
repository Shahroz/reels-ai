use anyhow::Result;
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, instrument};

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

// Helper function to safely encode email for Stripe API
fn encode_email_for_stripe(email: &str) -> String {
    // URL encode the email to handle special characters like +, @, etc.
    // This ensures Stripe API receives the email properly
    urlencoding::encode(email).to_string()
}

// Helper function to decode email from Stripe API response
fn decode_email_from_stripe(encoded_email: &str) -> String {
    // URL decode the email from Stripe API response
    urlencoding::decode(encoded_email)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| encoded_email.to_string())
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripeRecurring {
    pub interval: String,
    pub interval_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripePriceTier {
    pub flat_amount: Option<i64>,
    pub flat_amount_decimal: Option<String>,
    pub unit_amount: Option<i64>,
    pub unit_amount_decimal: Option<String>,
    pub up_to: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripeTransformQuantity {
    pub divide_by: i64,
    pub round: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripeCustomUnitAmount {
    pub maximum: Option<i64>,
    pub minimum: Option<i64>,
    pub preset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripeMarketingFeature {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripePackageDimensions {
    pub height: Option<f64>,
    pub length: Option<f64>,
    pub weight: Option<f64>,
    pub width: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum StripeCancellationFeedback {
    CustomerService,
    LowQuality,
    MissingFeatures,
    Other,
    SwitchedService,
    TooComplex,
    TooExpensive,
    Unused
}

impl StripeCancellationFeedback {

    pub fn as_str(&self) -> &'static str {
        match self {
            StripeCancellationFeedback::CustomerService => "customerservice",
            StripeCancellationFeedback::LowQuality => "lowquality",
            StripeCancellationFeedback::MissingFeatures => "missingfeatures",
            StripeCancellationFeedback::Other => "other",
            StripeCancellationFeedback::SwitchedService => "switchedservice",
            StripeCancellationFeedback::TooComplex => "toocomplex",
            StripeCancellationFeedback::TooExpensive => "tooexpensive",
            StripeCancellationFeedback::Unused => "unused",
        }
    }
}

impl std::fmt::Display for StripeCancellationFeedback {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripeCancellationDetails {
    pub comment: Option<String>,
    pub feedback: Option<StripeCancellationFeedback>,
}

// Stripe API response structs (what Stripe actually returns)
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripeProduct {
    pub id: String,
    pub object: String,
    pub active: Option<bool>,
    pub attributes: Option<Vec<String>>,
    pub caption: Option<String>,
    pub created: Option<i64>,
    pub default_price: Option<String>,
    pub description: Option<String>,
    pub images: Option<Vec<String>>,
    pub livemode: Option<bool>,
    pub marketing_features: Option<Vec<StripeMarketingFeature>>,
    pub metadata: serde_json::Value,
    pub name: String,
    pub package_dimensions: Option<StripePackageDimensions>,
    pub shippable: Option<bool>,
    pub statement_descriptor: Option<String>,
    pub tax_code: Option<String>,
    pub unit_label: Option<String>,
    pub updated: Option<i64>,
    pub url: Option<String>,
    pub prices: Option<Vec<StripePrice>>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripePrice {
    pub id: String,
    pub object: String,
    pub active: bool,
    pub billing_scheme: Option<String>,
    pub created: Option<i64>,
    pub currency: String,
    pub currency_options: Option<serde_json::Value>,
    pub custom_unit_amount: Option<StripeCustomUnitAmount>,
    pub livemode: Option<bool>,
    pub lookup_key: Option<String>,
    pub metadata: serde_json::Value,
    pub nickname: Option<String>,
    pub product: String,
    pub recurring: Option<StripeRecurring>,
    pub tax_behavior: Option<String>,
    pub tiers_mode: Option<String>,
    pub tiers: Option<Vec<StripePriceTier>>,
    pub transform_quantity: Option<StripeTransformQuantity>,
    pub unit_amount: Option<i64>,
    pub unit_amount_decimal: Option<String>,
    #[serde(rename = "type")]
    pub price_type: Option<String>,
}

/// Processed Stripe product with prices included
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripeProductWithPrices {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub default_price: Option<String>,
    pub active: Option<bool>,
    pub created: Option<i64>,
    pub updated: Option<i64>,
    pub prices: Vec<StripePrice>,
    pub object: String,
    pub marketing_features: Option<Vec<StripeMarketingFeature>>,
    pub images: Option<Vec<String>>,
    pub package_dimensions: Option<StripePackageDimensions>,
    pub shippable: Option<bool>,
    pub statement_descriptor: Option<String>,
    pub tax_code: Option<String>,
    pub unit_label: Option<String>,
    pub url: Option<String>,
}

/// Supported billing schema types for Stripe
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Copy, Clone)]
pub enum StripeBillingSchema {
    #[serde(rename = "per_unit")]
    PerUnit,
    #[serde(rename = "tiered")]
    Tiered,
}

impl StripeBillingSchema {
    /// Convert the enum to its string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            StripeBillingSchema::PerUnit => "per_unit",
            StripeBillingSchema::Tiered => "tiered",
        }
    }
}

impl std::fmt::Display for StripeBillingSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Stripe plan object as per official API
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripePlan {
    pub id: String,
    pub object: String,
    pub active: bool,
    pub amount: Option<i64>,
    pub amount_decimal: Option<String>,
    pub billing_scheme: Option<String>,
    pub created: Option<i64>,
    pub currency: String,
    pub interval: String,
    pub interval_count: i64,
    pub livemode: Option<bool>,
    pub metadata: serde_json::Value,
    pub nickname: Option<String>,
    pub product: String,
    pub tiers_mode: Option<String>,
    pub transform_usage: Option<serde_json::Value>,
    pub trial_period_days: Option<i64>,
    pub usage_type: String,
}

/// Stripe plan with expanded product details
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripePlanWithProduct {
    pub plan: StripePlan,
    pub product_details: StripeProduct,
}

// Remove duplicate structs - now imported from billing_service.rs
// StripePrice, StripeRecurring are now imported above

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripeCheckoutSession {
    pub id: String,
    pub url: Option<String>,
    pub customer: Option<String>,
    pub metadata: serde_json::Value,
    pub status: String,
    pub created: i64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct StripeCustomer {
    pub id: String,
    pub object: String,
    pub address: Option<serde_json::Value>,
    pub balance: i64,
    pub created: i64,
    pub currency: Option<String>,
    pub default_source: Option<String>,
    pub delinquent: bool,
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_email")]
    pub email: Option<String>,
    pub invoice_credit_balance: Option<serde_json::Value>,
    pub invoice_prefix: Option<String>,
    pub invoice_settings: Option<serde_json::Value>,
    pub livemode: bool,
    pub metadata: serde_json::Value,
    pub name: Option<String>,
    pub next_invoice_sequence: Option<i64>,
    pub phone: Option<String>,
    pub preferred_locales: Option<Vec<String>>,
    pub shipping: Option<serde_json::Value>,
    pub sources: Option<serde_json::Value>,
    pub subscriptions: Option<serde_json::Value>,
    pub tax_exempt: Option<String>,
    pub tax_ids: Option<serde_json::Value>,
    pub test_clock: Option<String>,
}

// Custom deserializer for email to handle URL decoding
fn deserialize_email<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let email_opt: Option<String> = serde::Deserialize::deserialize(deserializer)?;
    match email_opt {
        Some(email) => {
            let decoded_email = decode_email_from_stripe(&email);
            Ok(Some(decoded_email))
        }
        None => Ok(None),
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct StripePortalSession {
    pub id: String,
    pub url: String,
    pub customer: String,
    pub created: i64,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripeListResponse<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    pub total_count: Option<i64>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripeSubscription {
    pub id: String,
    pub object: String,
    pub customer: String,
    pub status: String,
    pub current_period_start: i64,
    pub current_period_end: i64,
    pub created: i64,
    pub canceled_at: Option<i64>,
    pub cancellation_details: Option<StripeCancellationDetails>,
    pub metadata: serde_json::Value,
    pub items: StripeSubscriptionItems,
    pub plan: Option<StripePlan>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripeSubscriptionItems {
    pub object: String,
    pub data: Vec<StripeSubscriptionItem>,
    pub has_more: bool,
    pub total_count: i64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripeSubscriptionItem {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub current_period_start: i64,
    pub current_period_end: i64,
    pub metadata: serde_json::Value,
    pub plan: StripePlan,
    pub price: StripePrice,
    pub quantity: i64,
    pub subscription: String,
    pub tax_rates: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StripeSubscriptionList {
    pub object: String,
    pub data: Vec<StripeSubscription>,
    pub has_more: bool,
    pub url: String,
}

pub struct StripeClient {
    client: ReqwestClient,
    secret_key: String,
}

impl StripeClient {
    /// Create a new Stripe client with explicit configuration
    pub fn new_with_config(config: &crate::services::billing::billing_config::BillingConfig) -> Result<Self> {
        let secret_key = config.get_secret_key()
            .map_err(|e| anyhow::anyhow!("Failed to get Stripe secret key: {e}"))?;
        
        if config.is_test_environment {
            log::info!("Using test environment with dummy Stripe key");
        } else {
            log::info!("Using production environment with Stripe key: {}...", &secret_key[..8.min(secret_key.len())]);
        }
        
        Self::new_with_key(secret_key)
    }
    
    /// Create a new Stripe client with explicit secret key
    pub fn new_with_key(secret_key: std::string::String) -> Result<Self> {
        let client = ReqwestClient::new();
        
        Ok(StripeClient {
            client,
            secret_key,
        })
    }
    
    /// Legacy method: Create a new Stripe client using environment variables
    /// 
    /// This method reads STRIPE_SECRET_KEY from environment variables.
    /// Prefer using new_with_config() for better testability and dependency injection.
    pub fn new() -> Result<Self> {
        let secret_key = std::env::var("STRIPE_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("STRIPE_SECRET_KEY must be set"))?;
        
        Self::new_with_key(secret_key)
    }

    /// Make an authenticated request to Stripe API
    async fn make_request<T>(&self, method: &str, endpoint: &str, body: Option<HashMap<String, String>>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{STRIPE_API_BASE}{endpoint}");
        
        log::debug!("Making Stripe API request: {method} {url}");
        log::info!("Making Stripe API request to: {url}");
        
        let mut request = self.client
            .request(method.parse()?, &url)
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .header("Content-Type", "application/x-www-form-urlencoded");
        
        // Log the full Authorization header for debugging
        log::info!("Authorization header: Bearer {}...", &self.secret_key[..8]);
        log::debug!("Full Authorization header: Bearer {}", self.secret_key);

        if let Some(body_data) = body {
            let body_string = body_data
                .into_iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            log::debug!("Request body: {body_string}");
            request = request.body(body_string);
        }

        log::debug!("Sending request to Stripe...");
        let response = request.send().await?;
        
        log::debug!("Received response from Stripe: HTTP {}", response.status());
        log::info!("Received Stripe API response: HTTP {}", response.status());
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("Stripe API error (HTTP {}): {}", status, error_text);
            
            // Try to parse the error response for more details
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(error_obj) = error_json.get("error") {
                    if let Some(message) = error_obj.get("message") {
                        error!("Stripe API error message: {}", message);
                    }
                    if let Some(code) = error_obj.get("code") {
                        error!("Stripe API error code: {}", code);
                    }
                }
            }
            
            return Err(anyhow::anyhow!("Stripe API error (HTTP {}): {}", status, error_text));
        }

        let response_text = response.text().await?;
        log::debug!("Stripe API response: {response_text}");
        
        let data: T = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("Failed to deserialize Stripe response: {e}");
                log::error!("Response text: {response_text}");
                
                // Try to provide more helpful error information
                if response_text.contains("error") {
                    if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        if let Some(error_obj) = error_json.get("error") {
                            if let Some(message) = error_obj.get("message") {
                                log::error!("Stripe API error message: {message}");
                            }
                            if let Some(code) = error_obj.get("code") {
                                log::error!("Stripe API error code: {code}");
                            }
                        }
                    }
                }
                
                // Log the first 500 characters of the response for debugging
                let preview = if response_text.len() > 500 {
                    format!("{}...", &response_text[..500])
                } else {
                    response_text.clone()
                };
                log::error!("Response preview: {preview}");
                
                anyhow::anyhow!("Failed to deserialize Stripe response: {}", e)
            })?;
        Ok(data)
    }

    /// List products from Stripe
    #[instrument(skip(self))]
    pub async fn list_products(&self, active: Option<bool>, billing_schema: Option<&str>) -> Result<StripeListResponse<StripeProduct>> {
        let mut query_params = Vec::new();
        
        if let Some(active_status) = active {
            query_params.push(format!("active={}", active_status));
        }
        
        if let Some(schema) = billing_schema {
            query_params.push(format!("billing_schema={}", schema));
        }
        
        let endpoint = if query_params.is_empty() {
            "/products".to_string()
        } else {
            format!("/products?{}", query_params.join("&"))
        };
        
        self.make_request("GET", &endpoint, None).await
    }

    /// Get a specific product
    #[instrument(skip(self))]
    pub async fn get_product(&self, product_id: &str) -> Result<StripeProduct> {
        let endpoint = format!("/products/{product_id}");
        self.make_request("GET", &endpoint, None).await
    }

    /// List prices for a product
    #[instrument(skip(self))]
    pub async fn list_prices(
        &self, 
        product_id: Option<&str>, 
        active: Option<bool>, 
    ) -> Result<StripeListResponse<StripePrice>> {
        let mut query_params = Vec::new();
        
        if let Some(pid) = product_id {
            query_params.push(format!("product={}", pid));
        }
        
        if let Some(active_status) = active {
            query_params.push(format!("active={}", active_status));
        } else {
            // Default to active=true if not specified
            query_params.push("active=true".to_string());
        }
        
        let endpoint = format!("/prices?{}", query_params.join("&"));
        self.make_request("GET", &endpoint, None).await
    }

    /// Get a specific price
    #[instrument(skip(self))]
    pub async fn get_price(&self, price_id: &str) -> Result<StripePrice> {
        let endpoint = format!("/prices/{price_id}");
        self.make_request("GET", &endpoint, None).await
    }

    /// List plans from Stripe with optional filtering
    #[instrument(skip(self))]
    pub async fn list_plans(
        &self,
        active: Option<bool>,
        product: Option<&str>,
    ) -> Result<StripeListResponse<StripePlan>> {
        let mut query_params = Vec::new();

        if let Some(active_status) = active {
            query_params.push(format!("active={}", active_status));
        }

        if let Some(product_id) = product {
            query_params.push(format!("product={}", product_id));
        }

        let endpoint = if query_params.is_empty() {
            "/plans".to_string()
        } else {
            format!("/plans?{}", query_params.join("&"))
        };
        
        self.make_request("GET", &endpoint, None).await
    }

    /// Create a subscription in Stripe
    #[instrument(skip(self))]
    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<StripeSubscription> {
        let mut body = HashMap::new();
        body.insert("customer".to_string(), customer_id.to_string());
        body.insert("items[0][price]".to_string(), price_id.to_string());
        
        // Add metadata if provided
        if let Some(meta) = metadata {
            for (key, value) in meta {
                body.insert(format!("metadata[{}]", key), value);
            }
        }

        self.make_request("POST", "/subscriptions", Some(body)).await
    }

    /// Create a checkout session
    #[instrument(skip(self))]
    pub async fn create_checkout_session(
        &self,
        success_url: &str,
        cancel_url: &str,
        customer_id: Option<&str>,
        line_items: Vec<HashMap<String, String>>,
        mode: &str,
        metadata: HashMap<String, String>,
    ) -> Result<StripeCheckoutSession> {
        let mut body = HashMap::new();
        body.insert("success_url".to_string(), success_url.to_string());
        body.insert("cancel_url".to_string(), cancel_url.to_string());
        body.insert("mode".to_string(), mode.to_string());
        body.insert("allow_promotion_codes".to_string(), "true".to_string());

        if let Some(customer) = customer_id {
            body.insert("customer".to_string(), customer.to_string());
        }

        // Add line items
        for (i, item) in line_items.into_iter().enumerate() {
            for (key, value) in item {
                body.insert(format!("line_items[{i}][{key}]"), value);
            }
        }

        // Add metadata
        for (key, value) in metadata {
            body.insert(format!("metadata[{key}]"), value);
        }

        self.make_request("POST", "/checkout/sessions", Some(body)).await
    }

    /// Cancel a subscription in Stripe
    /// Based on Stripe API: https://docs.stripe.com/api/subscriptions/cancel
    #[instrument(skip(self))]
    pub async fn cancel_subscription(
        &self,
        subscription_id: &str,
        cancellation_details: Option<HashMap<String, String>>,
        invoice_now: Option<bool>,
        prorate: Option<bool>,
    ) -> Result<StripeSubscription> {
        let mut body = HashMap::new();
        
        // Add cancellation details if provided
        if let Some(details) = cancellation_details {
            for (key, value) in details {
                body.insert(format!("cancellation_details[{}]", key), value);
            }
        }
        
        // Add invoice_now parameter if provided
        if let Some(invoice_now_val) = invoice_now {
            body.insert("invoice_now".to_string(), invoice_now_val.to_string());
        }
        
        // Add prorate parameter if provided
        if let Some(prorate_val) = prorate {
            body.insert("prorate".to_string(), prorate_val.to_string());
        }

        let endpoint = format!("/subscriptions/{subscription_id}");
        self.make_request("DELETE", &endpoint, Some(body)).await
    }

    /// Create a customer
    #[instrument(skip(self))]
    pub async fn create_customer(&self, email: &str, metadata: HashMap<String, String>) -> Result<StripeCustomer> {
        let mut body = HashMap::new();
        // Encode email to handle special characters like + properly
        let encoded_email = encode_email_for_stripe(email);
        log::info!("Creating Stripe customer with email: {} (encoded: {})", email, encoded_email);
        body.insert("email".to_string(), encoded_email);

        for (key, value) in metadata {
            body.insert(format!("metadata[{key}]"), value);
        }

        self.make_request("POST", "/customers", Some(body)).await
    }

    /// List customers by email
    #[instrument(skip(self))]
    pub async fn list_customers_by_email(&self, email: &str) -> Result<StripeListResponse<StripeCustomer>> {
        // Encode email for URL query parameter to handle special characters
        let encoded_email = encode_email_for_stripe(email);
        log::info!("Searching Stripe customers with email: {} (encoded: {})", email, encoded_email);
        let endpoint = format!("/customers?email={encoded_email}");
        self.make_request("GET", &endpoint, None).await
    }

    /// Create a customer portal session
    #[instrument(skip(self))]
    pub async fn create_customer_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<StripePortalSession> {
        let mut body = HashMap::new();
        
        // Only include supported parameters for billing portal sessions API
        // The source parameter is not supported by Stripe's billing portal sessions endpoint
        body.insert("customer".to_string(), customer_id.to_string());
        body.insert("return_url".to_string(), return_url.to_string());
        
        self.make_request("POST", "/billing_portal/sessions", Some(body)).await
    }

    /// Get a customer by ID
    /// Based on Stripe API: https://docs.stripe.com/api/customers/retrieve
    #[instrument(skip(self))]
    pub async fn get_customer(&self, customer_id: &str) -> Result<StripeCustomer> {
        let endpoint = format!("/customers/{}", customer_id);
        self.make_request("GET", &endpoint, None).await
    }

    /// List subscriptions by customer ID
    /// Based on Stripe API: https://docs.stripe.com/api/subscriptions/list
    #[instrument(skip(self))]
    pub async fn get_subscriptions_by_customer(&self, customer_id: &str) -> Result<StripeSubscriptionList> {
        let endpoint = format!("/subscriptions?customer={}&limit=100", customer_id);
        self.make_request("GET", &endpoint, None).await
    }

    /// Update a customer's email address
    /// Based on Stripe API: https://docs.stripe.com/api/customers/update
    /// 
    /// Note: Updating the email does NOT change the customer ID.
    /// This is useful for organization ownership transfers.
    #[instrument(skip(self))]
    pub async fn update_customer_email(&self, customer_id: &str, new_email: &str) -> Result<StripeCustomer> {
        let mut body = HashMap::new();
        let encoded_email = encode_email_for_stripe(new_email);
        log::info!("Updating Stripe customer {} email to: {} (encoded: {})", customer_id, new_email, encoded_email);
        body.insert("email".to_string(), encoded_email);

        let endpoint = format!("/customers/{}", customer_id);
        self.make_request("POST", &endpoint, Some(body)).await
    }
} 