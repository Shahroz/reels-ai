// backend/src/routes/mod.rs

use crate::middleware::auth::JwtMiddleware;
use crate::middleware::trial_guard::{TrialGuard, BillingAccessGuard};
use crate::middleware::admin_guard::AdminGuard;
use crate::routes::admin::configure_admin_routes::configure_admin_routes;
use crate::routes::api_keys::configure_api_key_routes::configure_api_key_routes;
use crate::routes::bundles::configure_bundle_routes::configure_bundle_routes; // Added
use crate::routes::assets::configure_assets_routes::configure_assets_routes;
use crate::routes::auth::configure_auth_routes::configure_auth_routes;
use crate::routes::clone::clone_style::clone_style;
use crate::routes::collections::configure_collections_routes::configure_collections_routes;
use crate::routes::creatives::configure_creatives_routes::configure_creatives_routes;
use crate::routes::dashboard::configure_dashboard_routes::configure_dashboard_routes;
use crate::routes::research_conversations::configure_research_conversation_routes::configure_research_conversation_routes;
use crate::routes::documents::configure_documents_routes::configure_documents_routes;
use crate::routes::files::configure_files_routes;
use crate::routes::infinite_researches::configure_infinite_researches_routes::configure_infinite_researches_routes;
use crate::routes::internal::configure_internal_routes::configure_internal_routes;
use crate::routes::organizations::configure_organization_routes::configure_organization_routes;
use crate::routes::invitations::configure_invitation_routes;
use crate::routes::one_time_researches::configure_one_time_researches_routes::configure_one_time_researches_routes;
use crate::routes::formats::configure_formats_routes::configure_formats_routes;
use crate::routes::requests::configure_requests_routes;
use crate::routes::research_chat::configure_chat_routes;
use crate::routes::styles::configure_styles_routes::configure_styles_routes;
use crate::routes::users::configure_user_routes::configure_user_routes;
use agentloop::config::routes::{configure_routes as configure_loupe_routes};
use crate::routes::user_db_collections::configure_user_db_collections_routes::configure_user_db_collections_routes;
use actix_web::web;
use crate::routes::stripe::configure_stripe_routes;
use crate::routes::imageboard::configure_imageboard_routes::configure_imageboard_routes;
use crate::routes::shares::configure_shares_routes::configure_shares_routes;
use crate::routes::user_favorites::configure_user_favorites_routes::configure_user_favorites_routes;
use crate::routes::predefined_collections::configure_predefined_collections_routes::configure_predefined_collections_routes;
use crate::routes::pdf_conversion::configure_pdf_routes;
use crate::routes::billing::configure_billing_routes::configure_billing_routes;
use crate::routes::vocal_tour::configure_vocal_tour_routes::configure_vocal_tour_routes;
use crate::routes::property_contents::configure_property_contents_routes::configure_property_contents_routes;
use crate::routes::logo_collections::configure_logo_collections_routes::configure_logo_collections_routes;
use crate::routes::watermarking::configure_watermarking_routes::configure_watermarking_routes;
use crate::routes::studio_journey_shares::configure_studio_journey_shares_routes::configure_studio_journey_shares_routes;
use crate::routes::studio::configure_studio_routes::configure_studio_routes;
use crate::routes::public::configure_public_routes::configure_public_routes;
use crate::routes::feed::configure_feed_routes::configure_feed_routes;
use crate::routes::credit_rewards::configure_credit_rewards_routes::configure_credit_rewards_routes;
use crate::routes::analytics::configure_analytics_routes;

pub mod api_keys;
pub mod apple_app_site_association; // Apple App Site Association for Universal Links
pub mod assets;
pub mod auth;
pub mod bundles; // Added
pub mod dashboard; // Added dashboard module
pub mod clone;
pub mod admin;
pub mod collections;
pub mod creatives;
pub mod error_response;
pub mod formats;
pub mod files;
pub mod internal;
pub mod infinite_researches;
pub mod health;
pub mod environment; // Added for environment variables module
pub mod objects_common;
pub mod pdf_conversion;
pub mod requests;
pub mod documents;
pub mod research_chat;
pub mod search;
pub mod shares;
pub mod stripe;
pub mod styles;
pub mod users;
pub mod invitations;
pub mod one_time_researches;
pub mod organizations;
pub mod research_conversations;
pub mod user_db_collections;
pub mod user_favorites; // Added for user favorites module
pub mod predefined_collections; // Added for predefined collections module
pub mod billing; // Added for billing routes
pub mod vocal_tour; // Added for vocal tour routes
pub mod property_contents; // Added for property contents routes
pub mod dub; // Added for Dub attribution tracking routes
pub mod studio; // Added for Studio tutorial events
// pub mod studio; // moved lineage graph under assets routes
pub mod lineage;
pub mod analytics; // Added for analytics routes
pub mod logo_collections; // Added for logo collections routes
pub mod watermarking; // Added for watermarking routes
pub mod studio_journey_shares;
pub mod content_studio;
pub mod public;
pub mod feed; // Feed posts for mobile app
pub mod credit_rewards;
// pub mod credits; // deprecated - handled via imageboard webhook
pub mod imageboard; // Imageboard webhook routes

/// Configures all API routes for the application, including AgentLoop integration.
pub fn config(cfg: &mut web::ServiceConfig) {
   cfg.service(web::scope("/api/public").configure(configure_public_routes));
   cfg.service(
       web::scope("/auth") // Group auth related routes
           .configure(configure_auth_routes),
    );
    // Scope for internal service-to-service calls, like from Cloud Scheduler.
    // This scope is NOT protected by the standard user JWT middleware.
    cfg.service(
        web::scope("/api/internal").configure(configure_internal_routes),
    );
    cfg.service(
        web::scope("/api") // Group v1 API routes
            .service(
                web::scope("/clone") // Clone endpoint under /api
                    .wrap(JwtMiddleware) // Apply JWT Auth to all /api routes
                    .service(clone_style), // No longer needs specific wrap
            )
            .service(
                web::scope("/lineage")
                    .wrap(JwtMiddleware)
                    .configure(crate::routes::lineage::configure_lineage_routes),
            )
            .service(
                web::scope("/keys") // API Key management under /api
                    .wrap(JwtMiddleware) // Apply JWT Auth to all /api routes
                    .configure(configure_api_key_routes),
            )
            .service(
                web::scope("/requests") // Added Requests scope under /api
                    .wrap(JwtMiddleware) // Apply JWT Auth to all /api routes
                    .configure(configure_requests_routes),
            )
            .service(
                web::scope("/styles") // Added Styles scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_styles_routes),
            )
            .service(
                web::scope("/formats") // Added Formats scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_formats_routes),
            )
            .service(
                web::scope("/assets") // Added Assets scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_assets_routes),
            )
            .service(
                web::scope("/workflows") // Added Workflows scope under /api
                    .wrap(JwtMiddleware) // Apply JWT Auth to all /api routes
                    .configure(configure_documents_routes), // Point /workflows to documents routes
            )
            .service(
                web::scope("/documents") // Added Documents scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_documents_routes), // Point /documents to documents routes
            )
            .service(
                web::scope("/research-chat") // Added Research Chat scope under /api
                    .wrap(JwtMiddleware) // Apply JWT Auth to all /api routes
                    .configure(configure_chat_routes),
            )
            .service(
                web::scope("/research/conversations") // Added Research Conversations scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_research_conversation_routes),
            )
            .service(
                web::scope("/creatives") // Added Creatives scope under /api
                    .wrap(JwtMiddleware)
                    .wrap(TrialGuard)
                    .configure(configure_creatives_routes),
            )
            .service(
                web::scope("/files") // Added Files scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_files_routes),
            )
            .service(
                web::scope("/collections") // Added Collections scope under /api
                    .wrap(JwtMiddleware)
                    .wrap(TrialGuard)
                    .configure(configure_collections_routes),
            )
            .service(
                web::scope("/bundles") // Added Bundles scope under /api
                    .wrap(JwtMiddleware)
                    .wrap(TrialGuard)
                    .configure(configure_bundle_routes),
            )
            .service(
                web::scope("/organizations") // Added organizations scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_organization_routes),
            )
            .service(
                web::scope("/invitations") // Added invitations scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_invitation_routes),
            )
            .service(
                web::scope("/users") // Added users scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_user_routes),
            )
            .service(
                web::scope("/dashboard") // Added dashboard scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_dashboard_routes),
            )
            .service(
                web::scope("/user-db-collections") // Added user db collections scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_user_db_collections_routes),
            )
            .service(
                web::scope("/shares") // Added shares scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_shares_routes),
            )
            .service(
                web::scope("/user-favorites") // Added user favorites scope under /api
                   .wrap(JwtMiddleware)
                   .configure(configure_user_favorites_routes),
           )
                       .service(
                web::scope("/infinite-researches") // Added infinite researches scope under /api
                    .wrap(JwtMiddleware)
                    .wrap(TrialGuard)
                    .configure(configure_infinite_researches_routes),
            )
           .service( // Added admin service scope
               web::scope("/admin") // Added admin service scope under /api
                   .wrap(AdminGuard)
                   .wrap(JwtMiddleware)
                    .configure(configure_admin_routes),
            )
            .service(
                web::scope("/predefined-collections") // Added predefined collections scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_predefined_collections_routes),
            )
            .service(
                web::scope("/pdf") // Added PDF conversion scope under /api
                    .wrap(JwtMiddleware)
                    .configure(configure_pdf_routes),
            )
            .service(
                web::scope("/one-time-researches")
                    .wrap(JwtMiddleware)
                    .wrap(TrialGuard)
                    .configure(configure_one_time_researches_routes),
            )
            .service(
                web::scope("/environment") // Added environment scope under /api
                    .wrap(JwtMiddleware)
                    .service(crate::routes::environment::get_environment_variables),
            )
            .service(
                web::scope("/billing") // Added billing scope under /api
                    .wrap(JwtMiddleware)
                    .wrap(BillingAccessGuard)
                    .configure(configure_billing_routes),
            )
            .configure(configure_vocal_tour_routes) // Add vocal tour routes directly under /api
            .service(
                web::scope("/property-contents") // Added property contents scope under /api
                    .wrap(JwtMiddleware)
                    .wrap(TrialGuard)
                    .configure(configure_property_contents_routes),
            )
            .service(
                web::scope("/dub")
                    .service(
                        web::scope("/track")
                            .service(crate::routes::dub::track_lead::track_lead)
                    ) // Add Dub scope under /api with auth for user-specific operations
                    .service(
                        web::scope("/referral-token")
                        .wrap(JwtMiddleware)
                        .service(crate::routes::dub::generate_referral_token::generate_referral_token)
                    )
            )
            .service(
                web::scope("/logo-collections") // Added logo collections scope under /api
                    .wrap(JwtMiddleware)
                    .wrap(TrialGuard)
                    .configure(configure_logo_collections_routes),
            )
            .service(
                web::scope("/watermark") // Added watermarking scope under /api
                    .wrap(JwtMiddleware)
                   .wrap(TrialGuard)
                   .configure(configure_watermarking_routes),
           )
           .service(
               web::scope("/studio/journeys")
                   .wrap(JwtMiddleware)
                   .configure(configure_studio_journey_shares_routes),
           )
           .service(
               web::scope("/studio")
                   .wrap(JwtMiddleware)
                   .configure(configure_studio_routes),
           )
           .service(
               web::scope("/content-studio")  // Content studio routes under /api
                   .wrap(JwtMiddleware)
                   .wrap(TrialGuard)
                   .configure(content_studio::configure_content_studio_routes::configure_content_studio_routes),
           )
           .service(
               web::scope("/credit-rewards") // Added credit rewards scope under /api
                   .wrap(JwtMiddleware)
                   .configure(configure_credit_rewards_routes),
           )
           .service(
               web::scope("/feed") // Feed routes mounted at /api/feed
                   .wrap(JwtMiddleware)
                   .configure(configure_feed_routes),
           )
           // lineage graph exposed under /api/assets/lineage/graph via assets routes
   );
   
   // Configure analytics routes
   // Note: configure_analytics_routes already creates /analytics scope with /api prefix
   cfg.configure(configure_analytics_routes);
   
   // Mount AgentLoop routes under /loupe, passing in the pre-initialized AppState
    cfg.service(
        web::scope("/loupe") // Add scope for agentloop service
            .configure(configure_loupe_routes),
    );
    // Configure Stripe routes (both API routes and webhooks)
    cfg.configure(configure_stripe_routes);
    // Configure Imageboard webhook routes
    cfg.configure(configure_imageboard_routes);
}
