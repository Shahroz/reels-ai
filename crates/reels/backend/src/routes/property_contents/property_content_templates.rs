//! HTML templates for different marketing content types.
//!
//! This module provides formatted HTML templates for each type of marketing content
//! generated from property descriptions. Each template includes styling and structure
//! appropriate for the specific platform and content type.

pub const MLS_REMARKS_TEMPLATE: &str = r#"
<div class="property-content mls-remarks">
    <div class="header">
        <h1>🏠 MLS Public Remarks</h1>
        <div class="platform-badge">MLS Listing Platform</div>
    </div>
    
    <div class="content-section">
        <div class="content-body">
            {content}
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> MLS Public Remarks</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #2563eb; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #1e40af; margin: 0; font-size: 24px; }
        .platform-badge { background: #dbeafe; color: #1e40af; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #f8fafc; border-left: 4px solid #2563eb; padding: 20px; margin: 20px 0; }
        .content-body { line-height: 1.6; color: #374151; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const PORTAL_FULL_DESCRIPTION_TEMPLATE: &str = r#"
<div class="property-content portal-description">
    <div class="header">
        <h1>🌐 Real Estate Portal Description</h1>
        <div class="platform-badge">Real Estate Portals</div>
    </div>
    
    <div class="content-section">
        <div class="content-body">
            {content}
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Portal Full Description</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #059669; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #047857; margin: 0; font-size: 24px; }
        .platform-badge { background: #d1fae5; color: #047857; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #f0fdf4; border-left: 4px solid #059669; padding: 20px; margin: 20px 0; }
        .content-body { line-height: 1.6; color: #374151; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const HEADLINE_BULLETS_TEMPLATE: &str = r#"
<div class="property-content headline-bullets">
    <div class="header">
        <h1>📝 Property Headlines & Bullets</h1>
        <div class="platform-badge">Marketing Materials</div>
    </div>
    
    <div class="content-section">
        <div class="content-body">
            {content}
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Headlines & Bullet Points</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #7c3aed; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #6d28d9; margin: 0; font-size: 24px; }
        .platform-badge { background: #ede9fe; color: #6d28d9; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #faf5ff; border-left: 4px solid #7c3aed; padding: 20px; margin: 20px 0; }
        .content-body { line-height: 1.6; color: #374151; white-space: pre-line; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const INSTAGRAM_CAPTION_TEMPLATE: &str = r#"
<div class="property-content instagram-caption">
    <div class="header">
        <h1>📸 Instagram Feed Caption</h1>
        <div class="platform-badge">Instagram</div>
    </div>
    
    <div class="content-section">
        <div class="instagram-preview">
            <div class="content-body">
                {content}
            </div>
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Instagram Feed Caption</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #e91e63; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #c2185b; margin: 0; font-size: 24px; }
        .platform-badge { background: #fce4ec; color: #c2185b; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #fef7ff; border-left: 4px solid #e91e63; padding: 20px; margin: 20px 0; }
        .instagram-preview { background: white; border: 1px solid #e5e7eb; border-radius: 12px; padding: 20px; }
        .content-body { line-height: 1.5; color: #374151; white-space: pre-line; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const REELS_TIKTOK_CAPTION_TEMPLATE: &str = r#"
<div class="property-content reels-tiktok-caption">
    <div class="header">
        <h1>🎬 Reels/TikTok Caption</h1>
        <div class="platform-badge">Reels/TikTok</div>
    </div>
    
    <div class="content-section">
        <div class="video-preview">
            <div class="content-body">
                {content}
            </div>
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Video Caption</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #ff6b6b; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #e03131; margin: 0; font-size: 24px; }
        .platform-badge { background: #ffe0e0; color: #e03131; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #fff5f5; border-left: 4px solid #ff6b6b; padding: 20px; margin: 20px 0; }
        .video-preview { background: #000; color: white; border-radius: 12px; padding: 20px; text-align: center; }
        .content-body { line-height: 1.4; white-space: pre-line; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const FACEBOOK_POST_TEMPLATE: &str = r#"
<div class="property-content facebook-post">
    <div class="header">
        <h1>📘 Facebook Post</h1>
        <div class="platform-badge">Facebook</div>
    </div>
    
    <div class="content-section">
        <div class="facebook-preview">
            <div class="content-body">
                {content}
            </div>
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Social Media Post</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #1877f2; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #1565c0; margin: 0; font-size: 24px; }
        .platform-badge { background: #e3f2fd; color: #1565c0; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #f3f8ff; border-left: 4px solid #1877f2; padding: 20px; margin: 20px 0; }
        .facebook-preview { background: white; border: 1px solid #e4e6ea; border-radius: 8px; padding: 20px; }
        .content-body { line-height: 1.5; color: #1c1e21; white-space: pre-line; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const GOOGLE_BUSINESS_PROFILE_TEMPLATE: &str = r#"
<div class="property-content google-business-profile">
    <div class="header">
        <h1>🏢 Google Business Profile Post</h1>
        <div class="platform-badge">Google Business Profile</div>
    </div>
    
    <div class="content-section">
        <div class="google-preview">
            <div class="content-body">
                {content}
            </div>
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Business Profile Post</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #34a853; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #2e7d32; margin: 0; font-size: 24px; }
        .platform-badge { background: #e8f5e8; color: #2e7d32; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #f1f8e9; border-left: 4px solid #34a853; padding: 20px; margin: 20px 0; }
        .google-preview { background: white; border: 1px solid #dadce0; border-radius: 8px; padding: 20px; }
        .content-body { line-height: 1.5; color: #3c4043; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const EMAIL_NEWSLETTER_TEMPLATE: &str = r#"
<div class="property-content email-newsletter">
    <div class="header">
        <h1>📧 Email Newsletter Listing</h1>
        <div class="platform-badge">Email Newsletter</div>
    </div>
    
    <div class="content-section">
        <div class="email-preview">
            <div class="content-body">
                {content}
            </div>
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Newsletter Listing</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #f59e0b; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #d97706; margin: 0; font-size: 24px; }
        .platform-badge { background: #fef3c7; color: #d97706; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #fffbeb; border-left: 4px solid #f59e0b; padding: 20px; margin: 20px 0; }
        .email-preview { background: white; border: 1px solid #e5e7eb; border-radius: 8px; padding: 20px; font-family: Georgia, serif; }
        .content-body { line-height: 1.6; color: #374151; white-space: pre-line; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

pub const SMS_WHATSAPP_TEMPLATE: &str = r#"
<div class="property-content sms-whatsapp">
    <div class="header">
        <h1>💬 SMS/WhatsApp Message</h1>
        <div class="platform-badge">SMS/WhatsApp</div>
    </div>
    
    <div class="content-section">
        <div class="message-preview">
            <div class="content-body">
                {content}
            </div>
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> Text Message</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #25d366; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #128c7e; margin: 0; font-size: 24px; }
        .platform-badge { background: #d4f7dc; color: #128c7e; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #f0fff4; border-left: 4px solid #25d366; padding: 20px; margin: 20px 0; }
        .message-preview { background: #dcf8c6; border-radius: 18px; padding: 15px 20px; max-width: 400px; margin: 0 auto; position: relative; }
        .message-preview::after { content: ''; position: absolute; bottom: 0; right: -8px; width: 0; height: 0; border: 8px solid transparent; border-top-color: #dcf8c6; border-bottom: 0; margin-left: -8px; margin-bottom: -8px; }
        .content-body { line-height: 1.4; color: #303030; font-size: 16px; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#;

/// Formats marketing content using the appropriate template.
///
/// Takes a MarketingContent item and applies the corresponding HTML template
/// based on the content type. Replaces template placeholders with actual content
/// and metadata values.
///
/// # Arguments
///
/// * `content` - The MarketingContent item to format
///
/// # Returns
///
/// A formatted HTML string ready for document storage.
pub fn format_marketing_content(
    content: &crate::routes::property_contents::parse_property_marketing_response::MarketingContent,
) -> std::string::String {
    let template = match content.content_type.as_str() {
        "mls_public_remarks" => MLS_REMARKS_TEMPLATE,
        "portal_full_description" => PORTAL_FULL_DESCRIPTION_TEMPLATE,
        "headline_bullets" => HEADLINE_BULLETS_TEMPLATE,
        "instagram_feed_caption" => INSTAGRAM_CAPTION_TEMPLATE,
        "reels_tiktok_caption" => REELS_TIKTOK_CAPTION_TEMPLATE,
        "facebook_post" => FACEBOOK_POST_TEMPLATE,
        "google_business_profile_post" => GOOGLE_BUSINESS_PROFILE_TEMPLATE,
        "email_newsletter_listing" => EMAIL_NEWSLETTER_TEMPLATE,
        "sms_whatsapp_message" => SMS_WHATSAPP_TEMPLATE,
        _ => {
            // Fallback generic template
            r#"
<div class="property-content generic">
    <div class="header">
        <h1>📄 {title}</h1>
        <div class="platform-badge">Marketing Content</div>
    </div>
    
    <div class="content-section">
        <div class="content-body">
            {content}
        </div>
    </div>
    
    <div class="metadata">
        <p><strong>Content Type:</strong> {content_type}</p>
        <p><strong>Platform:</strong> {platform}</p>
        <p><strong>Category:</strong> {content_category}</p>
        <p><strong>Generated from:</strong> Property Description</p>
    </div>
    
    <style>
        .property-content { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
        .header { border-bottom: 3px solid #6b7280; padding-bottom: 15px; margin-bottom: 20px; }
        .header h1 { color: #4b5563; margin: 0; font-size: 24px; }
        .platform-badge { background: #f3f4f6; color: #4b5563; padding: 5px 12px; border-radius: 15px; font-size: 12px; margin-top: 8px; display: inline-block; }
        .content-section { background: #f9fafb; border-left: 4px solid #6b7280; padding: 20px; margin: 20px 0; }
        .content-body { line-height: 1.6; color: #374151; }
        .metadata { background: #f1f5f9; padding: 15px; border-radius: 8px; margin-top: 25px; }
        .metadata p { margin: 5px 0; color: #64748b; font-size: 14px; }
    </style>
</div>
"#
        }
    };

    // Replace placeholders with actual content
    let formatted = template
        .replace("{content}", &content.content)
        .replace("{title}", &content.title)
        .replace("{content_type}", &content.content_type)
        .replace("{platform}", &content.metadata.get("platform").unwrap_or(&"Unknown".to_string()))
        .replace("{content_category}", &content.metadata.get("content_category").unwrap_or(&"general".to_string()));

    formatted
} 