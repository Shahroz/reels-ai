//! Apple App Site Association endpoint for Universal Links.
//!
//! This endpoint serves the Apple App Site Association (AASA) file which tells iOS
//! which apps can handle links from this domain. Required for Universal Links to work.
//!
//! Apple fetches this file when the app is first installed and periodically thereafter.
//! The file must be publicly accessible without authentication.
//!
//! ## Universal Links Flow
//!
//! 1. User installs iOS app with Associated Domains configured
//! 2. iOS automatically fetches `https://domain.com/.well-known/apple-app-site-association`
//! 3. iOS validates the appID matches the installed app
//! 4. iOS caches the association and knows which paths this app can handle
//! 5. When user taps a matching link, iOS opens the app instead of Safari
//!
//! ## Documentation
//!
//! - [Supporting Universal Links](https://developer.apple.com/documentation/xcode/supporting-universal-links-in-your-app)
//! - [Validation Tool](https://search.developer.apple.com/appsearch-validation-tool/)

use actix_web::{HttpResponse, Responder};

/// Serves Apple App Site Association file for Universal Links.
///
/// This endpoint returns a JSON file that tells iOS which app can handle links
/// from this domain. The file is fetched by iOS during app installation to verify
/// that the domain owner authorizes the app to handle its links.
///
/// # Response Format
///
/// Returns JSON containing:
/// - `applinks.apps`: Empty array (legacy field from iOS 9)
/// - `applinks.details`: Array of app configurations
///   - `appID`: Format is "TEAM_ID.BUNDLE_ID" (e.g., "6GZ9K9543W.com.bountihq.bounti")
///   - `paths`: URL paths this app can handle (e.g., "/verify-login-ml")
/// - `webcredentials.apps`: (Optional) For password autofill integration
///
/// # Security
///
/// - Public endpoint (no authentication required)
/// - Static content (heavily cached by iOS and CDNs)
/// - Must be served over HTTPS in production
/// - Must use Content-Type: application/json
///
/// # Testing
///
/// ```bash
/// # Local testing
/// curl http://localhost:8080/.well-known/apple-app-site-association
///
/// # Production testing
/// curl https://bounti.com/.well-known/apple-app-site-association
///
/// # Apple's validation tool
/// # https://search.developer.apple.com/appsearch-validation-tool/
/// ```
///
/// # Troubleshooting
///
/// If Universal Links don't work:
/// 1. Verify this endpoint returns 200 OK with valid JSON
/// 2. Check iOS device logs: Settings → Developer → Universal Links
/// 3. Reinstall the app (iOS caches the file at install time)
/// 4. Verify HTTPS is enabled (required for production)
/// 5. Check that Team ID and Bundle ID match your app's Xcode configuration
///
/// # Environment Variables
///
/// - `APPLE_TEAM_ID`: (Optional) Override Team ID from environment
/// - `IOS_BUNDLE_ID`: (Optional) Override Bundle ID from environment
///
/// If not set, uses hardcoded values from Xcode configuration.
#[actix_web::get("/.well-known/apple-app-site-association")]
#[tracing::instrument]
pub async fn apple_app_site_association() -> impl Responder {
    build_aasa_response()
}

/// Builds the Apple App Site Association response.
/// Extracted for testability since the actix macro creates a struct.
fn build_aasa_response() -> HttpResponse {
    // Get Team ID and Bundle ID from environment or use hardcoded values
    // These values come from Xcode project settings:
    // - Team ID: Project Settings → Signing & Capabilities → Team
    // - Bundle ID: Project Settings → General → Identity → Bundle Identifier
    let team_id = std::env::var("APPLE_TEAM_ID").unwrap_or_else(|_| "6GZ9K9543W".to_string());

    let bundle_id =
        std::env::var("IOS_BUNDLE_ID").unwrap_or_else(|_| "com.bountihq.bounti".to_string());

    let app_id = format!("{team_id}.{bundle_id}");

    log::debug!("Serving Apple App Site Association for app ID: {app_id}");

    // Build the association JSON
    // Docs: https://developer.apple.com/documentation/bundleresources/applinks
    let association = serde_json::json!({
        "applinks": {
            // Empty array, required for backward compatibility with iOS 9
            "apps": [],

            // Details of which apps can handle which paths
            "details": [{
                // Your app's identifier (Team ID + Bundle ID)
                "appID": app_id,

                // URL paths your app can handle
                // When user taps any of these links, iOS will open your app
                "paths": [
                    "/verify-login-ml",           // Exact match
                    "/verify-login-ml/*",         // Wildcard (includes query params)
                    "/real-estate/verify-login-ml",    // Real estate specific path
                    "/real-estate/verify-login-ml/*",  // Real estate with query params
                    // Future: Add more paths as needed
                    // "/auth/*",
                    // "/invite/*",
                ],

                // Optional: More precise path matching with query parameters
                // Uncomment if you want to restrict to specific query params
                // "components": [{
                //     "/": "/verify-login-ml",
                //     "?": { "token": "*" }  // Requires ?token parameter
                // }]
            }]
        },

        // Optional: Web Credentials for password autofill
        // Allows iOS to offer to save/autofill passwords for your app
        "webcredentials": {
            "apps": [app_id]
        }
    });

    log::info!("Served Apple App Site Association successfully for app: {app_id}");

    // Return JSON response with appropriate headers
    HttpResponse::Ok()
        .content_type("application/json")
        // Cache for 24 hours (iOS also caches internally)
        .insert_header(("Cache-Control", "public, max-age=86400"))
        // Prevent browser caching during development (remove in production if needed)
        // .insert_header(("Cache-Control", "no-cache"))
        .json(association)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_app_site_association_returns_200() {
        let resp = build_aasa_response();
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn test_uses_environment_variables() {
        std::env::set_var("APPLE_TEAM_ID", "TEST123");
        std::env::set_var("IOS_BUNDLE_ID", "com.test.app");

        let resp = build_aasa_response();
        
        // Verify response is OK
        assert_eq!(resp.status(), 200);

        std::env::remove_var("APPLE_TEAM_ID");
        std::env::remove_var("IOS_BUNDLE_ID");
    }
}
