use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{PgPool};
use tracing::{error, info};

use bytes::Bytes;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::db::billing::create_webhook_event;
use crate::services::stripe_webhook_handler::StripeWebhookEventsHandlerService;

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct StripeEvent {
    pub id: String,
    pub object: String,
    pub api_version: String,
    pub created: i64,
    pub data: serde_json::Value,
    pub livemode: bool,
    pub pending_webhooks: i64,
    pub request: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[utoipa::path(
    post,
    path = "/stripe/webhook",
    tag = "Stripe",
    request_body = String,
    responses(
        (status = 200, description = "Webhook processed successfully"),
        (status = 400, description = "Invalid payload or signature"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn stripe_webhook_handler(
    pool: web::Data<PgPool>,
    body: Bytes,
    req: HttpRequest,
) -> impl Responder {
    let start_time = Instant::now();

    // Log the incoming request details
    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let payload = String::from_utf8_lossy(&body);
    let payload_preview = if payload.len() > 200 {
        format!("{}...", &payload[..200])
    } else {
        payload.to_string()
    };

    log::info!(
        "[STRIPE WEBHOOK] Received from IP: {}, User-Agent: {}, payload: {} bytes, preview: '{}'",
        client_ip,
        user_agent,
        body.len(),
        payload_preview
    );

    // Extract and verify webhook signature
    let signature = req
        .headers()
        .get("Stripe-Signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if signature.is_empty() {
        log::error!("[STRIPE WEBHOOK] Missing Stripe-Signature header");
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Missing Stripe-Signature header"
        }));
    }

    log::info!("[STRIPE WEBHOOK] Signature header: present");

    let webhook_secret = match std::env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            log::error!("[STRIPE WEBHOOK] STRIPE_WEBHOOK_SECRET not set");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Webhook secret not configured"
            }));
        }
    };

    let (signature_valid, verification_time) =
        verify_webhook_signature_with_timing(&payload, signature, &webhook_secret);

    if !signature_valid {
        log::error!(
            "[STRIPE WEBHOOK] Invalid signature after {:?} (verification took {:?})",
            start_time.elapsed(),
            verification_time
        );
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid signature"
        }));
    }

    log::info!(
        "[STRIPE WEBHOOK] Signature verified successfully in {:?}",
        start_time.elapsed()
    );

    // Parse the webhook event
    let event = match serde_json::from_str::<StripeEvent>(&payload) {
        Ok(event) => {
            log::info!(
                "[STRIPE WEBHOOK] Successfully parsed webhook event: {}",
                event.event_type
            );
            event
        }
        Err(e) => {
            log::error!("[STRIPE WEBHOOK] Failed to parse webhook event: {e}");
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid JSON payload"
            }));
        }
    };

    // Log the event details for debugging
    log::info!(
        "[STRIPE WEBHOOK] Event ID: {}, Type: {}, Created: {}, Livemode: {}",
        event.id,
        event.event_type,
        event.created,
        event.livemode
    );

    // Track the webhook event
    let is_new = if event.event_type == "checkout.session.completed"
        || event.event_type == "invoice.paid"
        || event.event_type == "customer.subscription.created"
    {
        // For events that might have checkout session or subscription info, extract metadata
        let (checkout_session_id, promo_code) = extract_checkout_session_metadata(&event.data);

        log::info!(
            "[STRIPE WEBHOOK] Extracted metadata for {}: checkout_session_id={:?}, promo_code={:?}",
            event.event_type,
            checkout_session_id,
            promo_code
        );

        // Track the event with metadata
        match create_webhook_event(
            &pool,
            &event.id,
            &event.event_type,
            checkout_session_id.as_deref(),
            promo_code.as_deref(),
        )
        .await
        {
            Ok((_event, is_new)) => is_new,
            Err(e) => {
                log::error!("[STRIPE WEBHOOK] Failed to track webhook event with metadata: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to track event"
                }));
            }
        }
    } else {
        // For other events, use the standard tracking
        log::info!(
            "[STRIPE WEBHOOK] Processing event: {} (no metadata extraction)",
            event.event_type
        );

        match create_webhook_event(&pool, &event.id, &event.event_type, None, None).await {
            Ok((_event, is_new)) => is_new,
            Err(e) => {
                log::error!("[STRIPE WEBHOOK] Failed to track webhook event: {e}");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to track event"
                }));
            }
        }
    };

    if is_new {
        log::info!("[STRIPE WEBHOOK] Event {} tracked as new", event.id);
    } else {
        log::info!(
            "[STRIPE WEBHOOK] Event {} already tracked, skipping processing",
            event.id
        );
        return HttpResponse::Ok().json(serde_json::json!({
            "status": "already_processed"
        }));
    }

    // Process the webhook event using the service
    let webhook_event_handler_service = match StripeWebhookEventsHandlerService::new() {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to create webhook service: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Service initialization failed"
            }));
        }
    };

    match process_webhook_event_with_service(&webhook_event_handler_service, &pool, &event).await {
        Ok(_) => {
            info!("Successfully processed webhook: {}", event.event_type);
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Webhook processed successfully"
            }))
        }
        Err(e) => {
            error!("Failed to process webhook: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to process webhook"
            }))
        }
    }
}

/// Extract customer context for logging
fn extract_customer_context(event: &StripeEvent) -> String {
    if let Some(data) = event.data.get("object") {
        if let Some(customer) = data.get("customer") {
            if let Some(customer_id) = customer.as_str() {
                return customer_id.to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Verify webhook signature with timing information
fn verify_webhook_signature_with_timing(
    payload: &str,
    signature: &str,
    secret: &str,
) -> (bool, Option<std::time::Duration>) {
    let start_time = Instant::now();

    if signature.is_empty() {
        log::error!("[STRIPE WEBHOOK] Empty signature");
        return (false, Some(start_time.elapsed()));
    }

    // Extract timestamp and all v1 signatures
    let mut timestamp: Option<&str> = None;
    let mut signatures: Vec<&str> = vec![];

    for part in signature.split(',') {
        if part.starts_with("t=") {
            timestamp = Some(&part[2..]);
        } else if part.starts_with("v1=") {
            signatures.push(&part[3..]);
        }
    }

    let timestamp = match timestamp {
        Some(t) => t,
        None => {
            log::error!("[STRIPE WEBHOOK] Missing timestamp in signature header");
            return (false, Some(start_time.elapsed()));
        }
    };

    // Check timestamp validity (within 5 minutes)
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let event_time = timestamp.parse::<u64>().unwrap_or(0);

    if current_time > event_time + 300 {
        log::error!(
            "[STRIPE WEBHOOK] Signature timestamp too old: {event_time} (current: {current_time})"
        );
        return (false, Some(start_time.elapsed()));
    }

    // Compute expected signature
    let signed_payload = format!("{timestamp}.{payload}");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signed_payload.as_bytes());
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    let is_valid = signatures.iter().any(|s| *s == expected_signature);
    let elapsed = start_time.elapsed();

    if !is_valid {
        log::error!(
            "[STRIPE WEBHOOK] Signature verification failed. Expected: {}, Received ({}): {:?}",
            expected_signature,
            signatures.len(),
            signatures
        );
    }

    (is_valid, Some(elapsed))
}

/// Process webhook event using the service
async fn process_webhook_event_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    event: &StripeEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    match event.event_type.as_str() {
        "checkout.session.completed" => {
            process_checkout_session_completed_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "customer.subscription.created" => {
            process_subscription_created_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "customer.subscription.updated" => {
            process_subscription_updated_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "customer.subscription.deleted" => {
            process_subscription_deleted_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "invoice.payment_succeeded" => {
            process_invoice_payment_succeeded_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "invoice.payment_failed" => {
            process_invoice_payment_failed_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "invoice.created" => {
            process_invoice_created_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "invoice.finalized" => {
            process_invoice_finalized_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "invoice.paid" => {
            process_invoice_paid_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        "product.updated" => {
            process_product_updated_with_service(webhook_event_handler_service, pool, &event.data).await?;
        }
        _ => {
            info!("Unhandled event type: {}", event.event_type);
            info!("Event data: {}", event.data);
        }
    }

    Ok(())
}

/// Process checkout.session.completed event using service
async fn process_checkout_session_completed_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_checkout_session_completed(pool, data).await?;
    Ok(())
}

/// Process customer.subscription.created event using service
async fn process_subscription_created_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_subscription_created(pool, data).await?;
    Ok(())
}

/// Process customer.subscription.updated event using service
async fn process_subscription_updated_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_subscription_updated(pool, data).await?;
    Ok(())
}

/// Process customer.subscription.deleted event using service
async fn process_subscription_deleted_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_subscription_deleted(pool, data).await?;
    Ok(())
}

/// Process invoice.payment_succeeded event using service
async fn process_invoice_payment_succeeded_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_invoice_payment_succeeded(pool, data).await?;
    Ok(())
}

/// Process invoice.payment_failed event using service
async fn process_invoice_payment_failed_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_invoice_payment_failed(pool, data).await?;
    Ok(())
}

/// Process product.updated event using service
async fn process_product_updated_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_product_updated(pool, data).await?;
    Ok(())
}

/// Process invoice.created event using service
async fn process_invoice_created_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_invoice_created(pool, data).await?;
    Ok(())
}

/// Process invoice.finalized event using service
async fn process_invoice_finalized_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_invoice_finalized(pool, data).await?;
    Ok(())
}

/// Process invoice.paid event using service
async fn process_invoice_paid_with_service(
    webhook_event_handler_service: &StripeWebhookEventsHandlerService,
    pool: &PgPool,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    webhook_event_handler_service.handle_invoice_paid(pool, data).await?;
    Ok(())
}

/// Extract promo code from event data
fn extract_promo_code_from_event(data: &serde_json::Value) -> Option<String> {
    // Try to get promo code from various possible locations
    data["object"]["promo_code"]["code"].as_str()
        .or_else(|| data["object"]["discount"]["promo_code"]["code"].as_str())
        .or_else(|| data["object"]["metadata"]["promo_code"].as_str())
        .map(|s| s.to_string())
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/webhook").route(web::post().to(stripe_webhook_handler)));
}

// Helper function to store promo code in subscription metadata
async fn store_promo_code_in_subscription_metadata(
    subscription_id: &str,
    promo_code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("[STRIPE WEBHOOK] Attempting to store promo code '{promo_code}' in subscription {subscription_id} metadata");

    let stripe_secret =
        std::env::var("STRIPE_SECRET_KEY").map_err(|_| "STRIPE_SECRET_KEY not set")?;

    log::info!(
        "[STRIPE WEBHOOK] Stripe secret loaded, length: {} characters",
        stripe_secret.len()
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("promotion_code", promo_code);

    log::info!("[STRIPE WEBHOOK] Sending metadata update to Stripe: {metadata:?}");

    let url = format!("https://api.stripe.com/v1/subscriptions/{subscription_id}");
    log::info!("[STRIPE WEBHOOK] Making POST request to: {url}");

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {stripe_secret}"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&metadata)
        .send()
        .await?;

    log::info!(
        "[STRIPE WEBHOOK] Stripe API response status: {}",
        response.status()
    );

    if !response.status().is_success() {
        let error_text = response.text().await?;
        log::error!("[STRIPE WEBHOOK] Failed to update subscription metadata: {error_text}");
        return Err(format!("Stripe API error: {error_text}").into());
    }

    log::info!("[STRIPE WEBHOOK] Successfully updated subscription {subscription_id} metadata with promo code: {promo_code}");
    Ok(())
}

// Helper function to retrieve promo code from subscription metadata
async fn get_promo_code_from_subscription_metadata(
    subscription_id: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    log::info!("[STRIPE WEBHOOK] Attempting to retrieve promo code from subscription {subscription_id} metadata");

    let stripe_secret =
        std::env::var("STRIPE_SECRET_KEY").map_err(|_| "STRIPE_SECRET_KEY not set")?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let response = client
        .get(format!(
            "https://api.stripe.com/v1/subscriptions/{subscription_id}"
        ))
        .header("Authorization", format!("Bearer {stripe_secret}"))
        .send()
        .await?;

    log::info!(
        "[STRIPE WEBHOOK] Stripe API response status: {}",
        response.status()
    );

    if !response.status().is_success() {
        let error_text = response.text().await?;
        log::error!("[STRIPE WEBHOOK] Failed to retrieve subscription: {error_text}");
        return Err(format!("Stripe API error: {error_text}").into());
    }

    let subscription: serde_json::Value = response.json().await?;
    log::info!(
        "[STRIPE WEBHOOK] Retrieved subscription data: {:?}",
        subscription.get("metadata")
    );

    let promo_code = subscription
        .get("metadata")
        .and_then(|metadata| metadata.get("promotion_code"))
        .and_then(|code| code.as_str())
        .map(|s| s.to_string());

    log::info!("[STRIPE WEBHOOK] Extracted promo code from metadata: {promo_code:?}");
    Ok(promo_code)
}

// Helper function to extract checkout session ID and promo code from event data
fn extract_checkout_session_metadata(data: &serde_json::Value) -> (Option<String>, Option<String>) {
    log::debug!("[STRIPE WEBHOOK] Extracting metadata from event data: {data:?}");

    let object = data.get("object");
    if object.is_none() {
        log::debug!("[STRIPE WEBHOOK] No 'object' found in event data");
        return (None, None);
    }

    let obj = object.unwrap();

    // Extract checkout session ID from the object
    let checkout_session_id = obj
        .get("id")
        .and_then(|id| id.as_str())
        .map(|s| s.to_string());

    log::debug!("[STRIPE WEBHOOK] Extracted checkout_session_id: {checkout_session_id:?}");

    // Extract promo code from multiple possible locations
    let promo_code = {
        // Try to get promo code from discounts array first (most common location)
        if let Some(discounts) = obj.get("discounts") {
            if let Some(discounts_arr) = discounts.as_array() {
                for discount in discounts_arr {
                    if let Some(promo) = discount.get("promotion_code") {
                        if let Some(code) = promo.as_str() {
                            log::debug!("[STRIPE WEBHOOK] Found promo code in discounts: {code}");
                            return (checkout_session_id, Some(code.to_string()));
                        }
                    }
                }
            }
        }

        // Try to get promo code from metadata
        if let Some(metadata) = obj.get("metadata") {
            if let Some(promo) = metadata.get("promotion_code") {
                if let Some(code) = promo.as_str() {
                    log::debug!("[STRIPE WEBHOOK] Found promo code in metadata: {code}");
                    return (checkout_session_id, Some(code.to_string()));
                }
            }
        }

        // Try to get promo code from subscription metadata if this is a subscription event
        if let Some(subscription) = obj.get("subscription") {
            if let Some(sub_id) = subscription.as_str() {
                log::debug!("[STRIPE WEBHOOK] Found subscription ID: {sub_id}, would need API call to get metadata");
            }
        }

        // Try to get promo code from invoice metadata
        if let Some(invoice) = obj.get("invoice") {
            if let Some(invoice_id) = invoice.as_str() {
                log::debug!("[STRIPE WEBHOOK] Found invoice ID: {invoice_id}, would need API call to get metadata");
            }
        }

        None
    };

    log::debug!("[STRIPE WEBHOOK] Final extracted metadata: checkout_session_id={checkout_session_id:?}, promo_code={promo_code:?}");

    (checkout_session_id, promo_code)
}
