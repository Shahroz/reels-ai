//! Continuous New Relic validation script for real-time observability testing.
//!
//! This script continuously sends telemetry data to New Relic until canceled (Ctrl+C).
//! It generates a variety of spans, errors, and metrics to help verify that data is
//! being received and appears correctly in the New Relic UI. Useful for debugging
//! connectivity issues and real-time validation of observability enhancements.
//! Run this while monitoring your New Relic dashboard to see data flow in real-time.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    // Load environment variables from .env file if available
    dotenvy::dotenv().ok();
    
    // Set up Ctrl+C handler
    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            std::result::Result::Ok(()) => {
                std::println!("\n🛑 Received Ctrl+C, shutting down gracefully...");
                r.store(false, Ordering::SeqCst);
            }
            std::result::Result::Err(err) => {
                std::eprintln!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    std::println!("🔄 Continuous New Relic Validation");
    std::println!("This script will continuously send telemetry data until Ctrl+C");
    std::println!("Monitor your New Relic dashboard while this runs!");
    std::println!();

    // Show environment configuration
    let service_name = std::env::var("OTEL_SERVICE_NAME")
        .or_else(|_| std::env::var("NEW_RELIC_APP_NAME"))
        .unwrap_or_else(|_| "unknown-service".to_string());
    let endpoint = std::env::var("NEW_RELIC_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "https://otlp.nr-data.net".to_string());
    
    std::println!("📋 Configuration:");
    std::println!("  Service Name: {}", service_name);
    std::println!("  OTLP Endpoint: {}", endpoint);
    std::println!("  License Key: {}", if std::env::var("NEW_RELIC_LICENSE_KEY").is_ok() { "✅ Set" } else { "❌ Missing" });
    std::println!();
    
    if std::env::var("NEW_RELIC_LICENSE_KEY").is_err() {
        std::println!("❌ NEW_RELIC_LICENSE_KEY is not set!");
        std::println!("Please set it in your .env file or environment variables.");
        return std::result::Result::Ok(());
    }

    // Initialize tracing
    observability::init_tracer().await?;
    std::println!("✅ OpenTelemetry tracer initialized");
    std::println!();
    
    std::println!("🚀 Starting continuous data generation...");
    std::println!("Press Ctrl+C to stop");
    std::println!();
    
    // Run continuous validation
    let mut iteration = 0u64;
    while running.load(Ordering::SeqCst) {
        iteration += 1;
        
        std::println!("📊 Iteration {} - Sending telemetry batch...", iteration);
        
        // Generate different types of spans each iteration
        let batch_id = uuid::Uuid::new_v4().to_string();
        
        // Normal operation span
        send_success_span(iteration, &batch_id).await?;
        
        // Error spans (25% chance)
        if iteration % 4 == 0 {
            send_error_span(iteration, &batch_id).await?;
        }
        
        // Database operation span (50% chance)
        if iteration % 2 == 0 {
            send_database_span(iteration, &batch_id).await?;
        }
        
        // User operation span
        send_user_operation_span(iteration, &batch_id).await?;
        
        // Business metric span
        send_business_metric_span(iteration, &batch_id).await?;
        
        COUNTER.store(iteration, Ordering::SeqCst);
        
        std::println!("  ✅ Batch {} sent (Total spans: {})", batch_id, iteration * 4);
        
        // Wait before next iteration
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    
    let total_spans = COUNTER.load(Ordering::SeqCst) * 4;
    std::println!("\n🏁 Shutting down after sending {} spans across {} iterations", total_spans, iteration);
    std::println!("💡 Check New Relic for service: '{}'", service_name);
    std::println!("🔗 Direct links:");
    std::println!("   APM: https://one.newrelic.com/apm");
    std::println!("   Distributed Tracing: https://one.newrelic.com/distributed-tracing");
    std::println!("   Errors: https://one.newrelic.com/errors-inbox");
    
    // Give final export time
    std::println!("\n⏳ Waiting 10 seconds for final export...");
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    
    std::result::Result::Ok(())
}

async fn send_success_span(iteration: u64, batch_id: &str) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    let span = tracing::info_span!(
        "http_request_success",
        operation.type = "http_request",
        batch.id = batch_id,
        iteration = iteration,
        http.method = "GET",
        http.route = "/api/health",
        http.status_code = 200,
        success = true,
        user.id = format!("user-{}", iteration % 10).as_str(),
        request.duration_ms = tracing::field::Empty,
    );
    
    let _guard = span.enter();
    
    // Simulate processing time
    let start = std::time::Instant::now();
    tokio::time::sleep(std::time::Duration::from_millis(50 + (iteration % 100))).await;
    let duration = start.elapsed();
    
    span.record("request.duration_ms", duration.as_millis() as u64);
    
    tracing::info!(
        "Successful request processed: endpoint=/api/health, response_size=1024, cache_hit={}",
        iteration % 3 == 0,
    );
    
    std::result::Result::Ok(())
}

async fn send_error_span(iteration: u64, batch_id: &str) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    let error_type = match iteration % 3 {
        0 => "client_error",
        1 => "server_error", 
        _ => "timeout_error",
    };
    
    let (status_code, error_message) = match error_type {
        "client_error" => (404, "Resource not found"),
        "server_error" => (500, "Internal server error"),
        _ => (408, "Request timeout"),
    };
    
    let span = tracing::error_span!(
        "http_request_error",
        operation.type = "http_request",
        batch.id = batch_id,
        iteration = iteration,
        http.method = "POST",
        http.route = "/api/clone",
        http.status_code = status_code,
        error = true,
        error.type = error_type,
        error.severity = if status_code >= 500 { "high" } else { "medium" },
        user.id = format!("user-{}", iteration % 10).as_str(),
    );
    
    let _guard = span.enter();
    
    // Set OpenTelemetry span status to ERROR so New Relic recognizes it as an error
    span.record("otel.status_code", "ERROR");
    span.record("otel.status_description", &error_message);
    
    tracing::error!(
        "Request failed with error: {}, retry_count={}, correlation_id=corr-{}",
        error_message,
        iteration % 3,
        iteration,
    );
    
    std::result::Result::Ok(())
}

async fn send_database_span(iteration: u64, batch_id: &str) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    let operation = match iteration % 4 {
        0 => "SELECT",
        1 => "INSERT", 
        2 => "UPDATE",
        _ => "DELETE",
    };
    
    let table = match iteration % 3 {
        0 => "users",
        1 => "orders",
        _ => "products",
    };
    
    let span = tracing::info_span!(
        "database_operation",
        operation.type = "database",
        batch.id = batch_id,
        iteration = iteration,
        db.operation = operation,
        db.table = table,
        db.rows_affected = tracing::field::Empty,
        query.duration_ms = tracing::field::Empty,
    );
    
    let _guard = span.enter();
    
    // Simulate database query time
    let start = std::time::Instant::now();
    let query_time = 10 + (iteration % 200); // 10-210ms
    tokio::time::sleep(std::time::Duration::from_millis(query_time)).await;
    let duration = start.elapsed();
    
    let rows_affected = match operation {
        "SELECT" => iteration % 100,
        "INSERT" => 1,
        "UPDATE" => iteration % 10,
        "DELETE" => iteration % 5,
        _ => 0,
    };
    
    span.record("db.rows_affected", rows_affected);
    span.record("query.duration_ms", duration.as_millis() as u64);
    
    tracing::info!(
        "Database operation completed: {} FROM {}, pool_size=10, slow_query={}",
        operation,
        table,
        duration.as_millis() > 100,
    );
    
    std::result::Result::Ok(())
}

async fn send_user_operation_span(iteration: u64, batch_id: &str) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    let operations = ["login", "logout", "profile_update", "password_change", "data_export"];
    let operation = operations[iteration as usize % operations.len()];
    
    let span = tracing::info_span!(
        "user_operation", 
        operation.type = "user_action",
        batch.id = batch_id,
        iteration = iteration,
        user.operation = operation,
        user.id = format!("user-{}", iteration % 50).as_str(),
        auth.method = "jwt",
        session.id = format!("sess-{}", iteration).as_str(),
    );
    
    let _guard = span.enter();
    
    tracing::info!(
        "User operation executed: {}, ip=192.168.1.{}, agent=Mozilla/5.0, feature=feature_{}",
        operation,
        iteration % 255,
        iteration % 5,
    );
    
    std::result::Result::Ok(())
}

async fn send_business_metric_span(iteration: u64, batch_id: &str) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    let span = tracing::info_span!(
        "business_metric",
        operation.type = "business_event", 
        batch.id = batch_id,
        iteration = iteration,
        metric.name = "revenue_event",
        metric.value = (iteration * 10) % 1000,
        business.category = "e-commerce",
    );
    
    let _guard = span.enter();
    
    tracing::info!(
        "Business metric recorded: purchase_completed, revenue=${}, currency=USD, segment={}, step=checkout_complete",
        (iteration * 10) % 1000,
        if iteration % 2 == 0 { "premium" } else { "standard" },
    );
    
    std::result::Result::Ok(())
}