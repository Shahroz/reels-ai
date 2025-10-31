//! Parses GenNodes PropertyMarketingContentCollection response into document entries.
//!
//! This module handles the complex GenNodes response structure that contains multiple
//! marketing content types (MLS remarks, Instagram captions, etc.) and converts them
//! into structured MarketingContent items for document creation. Each item inherits
//! the collection_id from the source document to maintain listing association.

/// Represents a single marketing content item extracted from GenNodes response.
#[derive(std::fmt::Debug, std::clone::Clone)]
pub struct MarketingContent {
    pub title: std::string::String,
    pub content: std::string::String,
    pub content_type: std::string::String,
    pub metadata: std::collections::HashMap<std::string::String, std::string::String>,
    pub collection_id: std::option::Option<uuid::Uuid>,
}

/// Parses the GenNodes PropertyMarketingContentCollection response into marketing content items.
///
/// Takes the raw JSON response from GenNodes and extracts each content type into separate
/// MarketingContent items. Each item gets a descriptive title, formatted content, semantic
/// content type, and inherits the collection_id from the source document.
///
/// # Arguments
///
/// * `response` - The JSON response from GenNodes PropertyMarketingContentCollection workflow
/// * `source_collection_id` - Collection ID inherited from the source document
///
/// # Returns
///
/// A vector of MarketingContent items, one for each content type in the response.
/// Returns error if the response structure is invalid or missing required fields.
pub fn parse_property_marketing_response(
    response: &serde_json::Value,
    source_collection_id: std::option::Option<uuid::Uuid>,
) -> std::result::Result<std::vec::Vec<MarketingContent>, std::string::String> {
    let mut marketing_contents = std::vec::Vec::new();

    // Navigate to the PropertyMarketingContentCollection data
    let data = response
        .get("data")
        .and_then(|d| d.get("PropertyMarketingContentCollection"))
        .ok_or_else(|| "Missing PropertyMarketingContentCollection in response".to_string())?;

    // Extract MLS Public Remarks
    if let std::option::Option::Some(mls_remarks) = data.get("mlsPublicRemarks") {
        if let std::option::Option::Some(content) = mls_remarks.get("content").and_then(|c| c.as_str()) {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "MLS".to_string());
            metadata.insert("content_category".to_string(), "public_remarks".to_string());
            
            marketing_contents.push(MarketingContent {
                title: "MLS Public Remarks".to_string(),
                content: content.to_string(),
                content_type: "mls_public_remarks".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    // Extract Portal Full Description
    if let std::option::Option::Some(portal_desc) = data.get("portalFullDescription") {
        if let std::option::Option::Some(content) = portal_desc.get("content").and_then(|c| c.as_str()) {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "Real Estate Portal".to_string());
            metadata.insert("content_category".to_string(), "full_description".to_string());
            
            marketing_contents.push(MarketingContent {
                title: "Real Estate Portal Description".to_string(),
                content: content.to_string(),
                content_type: "portal_full_description".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    // Extract Headline Bullets
    if let std::option::Option::Some(headline_bullets) = data.get("headlineBullets") {
        let headline = headline_bullets.get("headline").and_then(|h| h.as_str()).unwrap_or("Property Highlights");
        let bullets = headline_bullets.get("bullets").and_then(|b| b.as_array());
        
        if let std::option::Option::Some(bullets_array) = bullets {
            let bullet_list: std::vec::Vec<std::string::String> = bullets_array
                .iter()
                .filter_map(|b| b.as_str().map(|s| s.to_string()))
                .collect();
            
            let content = format!("{}\n\n{}", headline, bullet_list.join("\n• "));
            
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "Marketing Materials".to_string());
            metadata.insert("content_category".to_string(), "headline_bullets".to_string());
            metadata.insert("headline".to_string(), headline.to_string());
            
            marketing_contents.push(MarketingContent {
                title: "Property Headlines & Bullets".to_string(),
                content,
                content_type: "headline_bullets".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    // Extract Instagram Feed Caption
    if let std::option::Option::Some(instagram_caption) = data.get("instagramFeedCaption") {
        if let std::option::Option::Some(content) = instagram_caption.get("content").and_then(|c| c.as_str()) {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "Instagram".to_string());
            metadata.insert("content_category".to_string(), "feed_caption".to_string());
            
            marketing_contents.push(MarketingContent {
                title: "Instagram Feed Caption".to_string(),
                content: content.to_string(),
                content_type: "instagram_feed_caption".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    // Extract Reels/TikTok Caption
    if let std::option::Option::Some(reels_caption) = data.get("reelsTiktokCaption") {
        if let std::option::Option::Some(content) = reels_caption.get("content").and_then(|c| c.as_str()) {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "Reels/TikTok".to_string());
            metadata.insert("content_category".to_string(), "video_caption".to_string());
            
            marketing_contents.push(MarketingContent {
                title: "Reels/TikTok Caption".to_string(),
                content: content.to_string(),
                content_type: "reels_tiktok_caption".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    // Extract Facebook Post
    if let std::option::Option::Some(facebook_post) = data.get("facebookPost") {
        if let std::option::Option::Some(content) = facebook_post.get("content").and_then(|c| c.as_str()) {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "Facebook".to_string());
            metadata.insert("content_category".to_string(), "social_post".to_string());
            
            marketing_contents.push(MarketingContent {
                title: "Facebook Post".to_string(),
                content: content.to_string(),
                content_type: "facebook_post".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    // Extract Google Business Profile Post
    if let std::option::Option::Some(google_post) = data.get("googleBusinessProfilePost") {
        if let std::option::Option::Some(content) = google_post.get("content").and_then(|c| c.as_str()) {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "Google Business Profile".to_string());
            metadata.insert("content_category".to_string(), "business_post".to_string());
            
            marketing_contents.push(MarketingContent {
                title: "Google Business Profile Post".to_string(),
                content: content.to_string(),
                content_type: "google_business_profile_post".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    // Extract Email Newsletter Listing
    if let std::option::Option::Some(email_newsletter) = data.get("emailNewsletterListing") {
        let subject = email_newsletter.get("subject").and_then(|s| s.as_str()).unwrap_or("Property Listing");
        let body = email_newsletter.get("body").and_then(|b| b.as_str()).unwrap_or("");
        
        let content = format!("Subject: {}\n\n{}", subject, body);
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("platform".to_string(), "Email Newsletter".to_string());
        metadata.insert("content_category".to_string(), "newsletter_listing".to_string());
        metadata.insert("subject".to_string(), subject.to_string());
        
        marketing_contents.push(MarketingContent {
            title: "Email Newsletter Listing".to_string(),
            content,
            content_type: "email_newsletter_listing".to_string(),
            metadata,
            collection_id: source_collection_id,
        });
    }

    // Extract SMS/WhatsApp Message
    if let std::option::Option::Some(sms_whatsapp) = data.get("smsWhatsapp") {
        if let std::option::Option::Some(content) = sms_whatsapp.get("content").and_then(|c| c.as_str()) {
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("platform".to_string(), "SMS/WhatsApp".to_string());
            metadata.insert("content_category".to_string(), "messaging".to_string());
            
            marketing_contents.push(MarketingContent {
                title: "SMS/WhatsApp Message".to_string(),
                content: content.to_string(),
                content_type: "sms_whatsapp_message".to_string(),
                metadata,
                collection_id: source_collection_id,
            });
        }
    }

    if marketing_contents.is_empty() {
        return std::result::Result::Err("No marketing content found in response".to_string());
    }

    std::result::Result::Ok(marketing_contents)
} 