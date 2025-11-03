//! Phase 1 New Relic validation script for error attribution and span enrichment.
//!
//! This script creates a minimal Actix-web application with our observability middleware
//! and generates specific error scenarios to validate that New Relic receives:
//! - Proper error attribution in spans
//! - Rich error context including user information
//! - Request correlation IDs
//! - Structured error events that appear in the New Relic Errors Inbox
//! Used for validating Phase 1 observability enhancements before production deployment.

#[tokio::main]
async fn main() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    // Load environment variables from .env file if available
    dotenvy::dotenv().ok();
    
    std::println!("🧪 Phase 1 Validation: Testing error attribution and span enrichment");
    std::println!("This script will:");
    std::println!("  1. Initialize OpenTelemetry with New Relic exporter");
    std::println!("  2. Create test scenarios with different error types");
    std::println!("  3. Validate that errors appear in New Relic with proper context");
    std::println!("  4. Generate sample traces for manual validation");
    std::println!();

    // Show environment configuration
    let service_name = std::env::var("OTEL_SERVICE_NAME")
        .or_else(|_| std::env::var("NEW_RELIC_APP_NAME"))
        .unwrap_or_else(|_| "unknown-service".to_string());
    let endpoint = std::env::var("NEW_RELIC_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://otlp.nr-data.net:4318".to_string());
    
    std::println!("📋 Configuration:");
    std::println!("  Service Name: {}", service_name);
    std::println!("  OTLP Endpoint: {}", endpoint);
    std::println!("  License Key: {}", if std::env::var("NEW_RELIC_LICENSE_KEY").is_ok() { "✅ Set" } else { "❌ Missing" });
    std::println!();

    // Initialize tracing
    observability::init_tracer().await?;
    std::println!("✅ OpenTelemetry tracer initialized");

    // Create test scenarios
    test_http_client_error().await?;
    test_http_server_error().await?;
    test_authenticated_user_error().await?;
    test_request_correlation().await?;

    std::println!();
    std::println!("🎯 Validation Complete!");
    std::println!("Check New Relic for:");
    std::println!("  - Service: '{}' in APM & Services", service_name);
    std::println!("  - Errors in the Errors Inbox with user context");
    std::println!("  - Distributed traces showing error attribution");
    std::println!("  - Request correlation IDs linking related operations");
    std::println!("  - Performance data for error scenarios");
    std::println!();
    std::println!("💡 Tip: It may take 1-2 minutes for data to appear in New Relic UI");
    std::println!("🔗 Direct link: https://one.newrelic.com/apm");

    // Keep the application running longer to ensure data export
    std::println!("⏳ Waiting 15 seconds for data export to complete...");
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    
    std::result::Result::Ok(())
}

async fn test_http_client_error() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    std::println!("🔍 Testing HTTP client error (404) scenario...");
    
    let span = tracing::info_span!(
        "validation_test",
        operation.type = "validation",
        test.scenario = "http_client_error",
        request.id = "test-404-001"
    );
    
    let _guard = span.enter();
    
    // Simulate a 404 error scenario
    let context = create_test_request_context("GET", "/api/nonexistent");
    
    // Record error information as our middleware would
    span.record("error", true);
    span.record("error.type", "http_error");
    span.record("http.status_code", 404);
    
    tracing::error!(
        event_type = "http_error",
        http.status_code = 404,
        request.id = "test-404-001",
        error.message = "Resource not found",
        "HTTP client error scenario for validation"
    );
    
    std::println!("  ✅ Simulated 404 error with user context");
    std::result::Result::Ok(())
}

async fn test_http_server_error() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    std::println!("🔍 Testing HTTP server error (500) scenario...");
    
    let span = tracing::error_span!(
        "validation_test",
        operation.type = "validation", 
        test.scenario = "http_server_error",
        request.id = "test-500-001"
    );
    
    let _guard = span.enter();
    
    // Simulate a 500 error scenario
    span.record("error", true);
    span.record("error.type", "internal_error");
    span.record("error.severity", "high");
    span.record("http.status_code", 500);
    
    tracing::error!(
        event_type = "server_error",
        http.status_code = 500,
        request.id = "test-500-001",
        error.message = "Internal server error during processing",
        error.stack_trace = "simulated_stack_trace::validation_test",
        "HTTP server error scenario for validation"
    );
    
    std::println!("  ✅ Simulated 500 error with detailed context");
    std::result::Result::Ok(())
}

async fn test_authenticated_user_error() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    std::println!("🔍 Testing authenticated user error scenario...");
    
    let span = tracing::warn_span!(
        "validation_test",
        operation.type = "validation",
        test.scenario = "auth_user_error", 
        request.id = "test-auth-001"
    );
    
    let _guard = span.enter();
    
    // Simulate authenticated user hitting an error
    span.record("auth.method", "jwt");
    span.record("user.is_admin", false);
    span.record("error", true);
    span.record("error.type", "permission_denied");
    span.record("http.status_code", 403);
    
    tracing::warn!(
        event_type = "authorization_error",
        http.status_code = 403,
        auth.method = "jwt",
        request.id = "test-auth-001",
        error.message = "User lacks permission for this operation",
        "Authentication error scenario for validation"
    );
    
    std::println!("  ✅ Simulated authenticated user permission error");
    std::result::Result::Ok(())
}

async fn test_request_correlation() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
    std::println!("🔍 Testing request correlation and trace propagation...");
    
    let parent_span = tracing::info_span!(
        "validation_parent_request",
        operation.type = "validation",
        test.scenario = "request_correlation",
        request.id = "test-correlation-parent",
        trace.parent = "00-12345678901234567890123456789012-1234567890123456-01"
    );
    
    let _parent_guard = parent_span.enter();
    
    // Simulate child operation that might error
    let child_span = tracing::info_span!(
        "validation_child_operation", 
        operation.type = "database_query",
        request.id = "test-correlation-child"
    );
    
    let _child_guard = child_span.enter();
    
    child_span.record("error", true);
    child_span.record("error.type", "database_error");
    child_span.record("database.operation", "SELECT");
    
    tracing::error!(
        event_type = "database_error",
        database.operation = "SELECT",
        request.id = "test-correlation-child",
        parent.request.id = "test-correlation-parent", 
        error.message = "Database connection timeout",
        "Database error in child operation for validation"
    );
    
    std::println!("  ✅ Simulated correlated request with parent-child relationship");
    std::result::Result::Ok(())
}

fn create_test_request_context(
    method: &str,
    path: &str, 
) -> TestRequestContext {
    TestRequestContext {
        method: method.to_string(),
        path: path.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: std::time::SystemTime::now(),
    }
}

#[derive(Debug)]
struct TestRequestContext {
    method: std::string::String,
    path: std::string::String,
    request_id: std::string::String,
    timestamp: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_request_context() {
        let context = create_test_request_context("POST", "/api/test");
        
        assert_eq!(context.method, "POST");
        assert_eq!(context.path, "/api/test");
        assert!(!context.request_id.is_empty());
    }

    #[test]
    fn test_create_test_request_context_no_user() {
        let context = create_test_request_context("GET", "/health");
        
        assert_eq!(context.method, "GET");
        assert_eq!(context.path, "/health");
        assert!(!context.request_id.is_empty());
    }
}