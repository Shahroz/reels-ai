use actix_web::{post, web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, OpenApi};
use uuid::Uuid;
use crate::services::pdf_conversion_service::{PdfConversionService, PdfConversionOptions};
use crate::auth::tokens::Claims;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConvertToPdfRequest {
    /// The URL to convert to PDF
    pub url: String,
    /// Optional custom filename for the PDF
    pub filename: Option<String>,
    /// Configuration preset: "custom" for best results, "default" for basic
    pub config: Option<String>,
    /// Paper width in inches (default: 8.27 for A4)
    pub paper_width: Option<f64>,
    /// Paper height in inches (default: 11.7 for A4) 
    pub paper_height: Option<f64>,
    /// Set to true for landscape orientation
    pub landscape: Option<bool>,
    /// Set to false to exclude background graphics
    pub include_background: Option<bool>,
    /// Margins in inches
    pub margin_top: Option<f64>,
    pub margin_bottom: Option<f64>,
    pub margin_left: Option<f64>,
    pub margin_right: Option<f64>,
    /// Scale factor (0.1 to 2.0)
    pub scale: Option<f64>,
    /// Page load timeout in seconds
    pub timeout_seconds: Option<u32>,
}



#[derive(Debug, Serialize, ToSchema)]
pub struct PdfConversionError {
    /// Error status
    pub success: bool,
    /// Error message
    pub error: String,
}

/// Convert a website URL to PDF and return the PDF file directly
#[utoipa::path(
    post,
    path = "/api/pdf/convert",
    tag = "PDF Conversion",
    request_body = ConvertToPdfRequest,
    responses(
        (status = 200, description = "PDF file download", content_type = "application/pdf"),
        (status = 400, description = "Bad request", body = PdfConversionError),
        (status = 500, description = "Internal server error", body = PdfConversionError)
    )
)]
#[post("/convert")]
pub async fn convert_to_pdf(
    request: web::Json<ConvertToPdfRequest>,
) -> Result<impl Responder> {
    tracing::info!("Starting PDF conversion for URL: {}", request.url);
    
    // Create the PDF conversion service
    let pdf_service = match PdfConversionService::new() {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to create PDF service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(PdfConversionError {
                success: false,
                error: format!("Failed to initialize PDF service: {e}"),
            }));
        }
    };
    
    // Convert the request to options
    let options = PdfConversionOptions {
        filename: request.filename.clone(),
        config: request.config.clone(),
        paper_width: request.paper_width,
        paper_height: request.paper_height,
        landscape: request.landscape,
        include_background: request.include_background,
        margin_top: request.margin_top,
        margin_bottom: request.margin_bottom,
        margin_left: request.margin_left,
        margin_right: request.margin_right,
        scale: request.scale,
        timeout_seconds: request.timeout_seconds,
    };
    
    // Generate filename if not provided
    let filename = options.filename.unwrap_or_else(|| format!("{}.pdf", uuid::Uuid::new_v4()));
    let filename = if filename.ends_with(".pdf") {
        filename
    } else {
        format!("{filename}.pdf")
    };
    
    // Perform the PDF conversion using direct streaming
    match pdf_service.convert_url_to_pdf_direct(&request.url, &filename, options.config).await {
        Ok(pdf_bytes) => {
            let file_size = pdf_bytes.len() as u64;
            tracing::info!("PDF conversion successful: {} ({} bytes)", filename, file_size);
            
            // Return the PDF directly as binary response
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{filename}\"")))
                .body(pdf_bytes))
        }
        Err(e) => {
            tracing::error!("PDF conversion failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(PdfConversionError {
                success: false,
                error: format!("PDF conversion failed: {e}"),
            }))
        }
    }
}

/// Convert a creative to PDF by creative ID and return the PDF file directly
#[utoipa::path(
    post,
    path = "/api/pdf/convert-creative/{creative_id}",
    tag = "PDF Conversion",
    params(
        ("creative_id" = String, Path, description = "Creative ID to convert to PDF")
    ),
    responses(
        (status = 200, description = "PDF file download", content_type = "application/pdf"),
        (status = 404, description = "Creative not found", body = PdfConversionError),
        (status = 400, description = "Bad request", body = PdfConversionError),
        (status = 500, description = "Internal server error", body = PdfConversionError)
    )
)]
#[post("/convert-creative/{creative_id}")]
pub async fn convert_creative_to_pdf(
    pool: web::Data<sqlx::PgPool>,
    auth: web::ReqData<Claims>,
    path: web::Path<Uuid>,
) -> Result<impl Responder> {
    let creative_id = path.into_inner();
    let user_id = auth.user_id;
    
    tracing::info!("Converting creative {} to PDF for user {}", creative_id, user_id);
    tracing::debug!("User ID: {}, Creative ID: {}", user_id, creative_id);
    
    // First, let's check if the creative exists at all
    let creative_exists = sqlx::query!(
        "SELECT id, name, html_url, is_published, collection_id FROM creatives WHERE id = $1",
        creative_id,
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    match &creative_exists {
        Ok(Some(creative)) => {
            tracing::info!("Creative found: id={}, name={:?}, html_url={}, collection_id={:?}", 
                creative.id, creative.name, creative.html_url, creative.collection_id);
        }
        Ok(None) => {
            tracing::warn!("Creative {} does not exist in database", creative_id);
            return Ok(HttpResponse::NotFound().json(PdfConversionError {
                success: false,
                error: format!("Creative {creative_id} not found"),
            }));
        }
        Err(e) => {
            tracing::error!("Database error checking creative existence: {}", e);
        }
    }

    // Now check permissions
    let creative_result = sqlx::query!(
        r#"
        SELECT id, name, html_url, is_published
        FROM creatives 
        WHERE id = $1 AND (
            -- User owns the creative (through collection ownership)
            id IN (
                SELECT c.id 
                FROM creatives c
                INNER JOIN collections col ON c.collection_id = col.id
                WHERE c.id = $1 AND col.user_id = $2
            )
            OR
            -- Creative is shared with the user through object_shares
            id IN (
                SELECT os.object_id
                FROM object_shares os
                WHERE os.object_type = 'creative' AND os.object_id = $1 AND os.entity_type = 'user' AND os.entity_id = $2
            )
        )
        "#,
        creative_id,
        user_id,
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    let creative = match creative_result {
        Ok(Some(c)) => c,
        Ok(None) => {
            tracing::warn!("Creative {} not accessible to user {} (permission denied)", creative_id, user_id);
            
            // Check if user owns any collections to help debug
            let user_collections = sqlx::query!(
                "SELECT id, name FROM collections WHERE user_id = $1",
                user_id
            )
            .fetch_all(pool.get_ref())
            .await;
            
            match user_collections {
                Ok(collections) => {
                    tracing::info!("User {} owns {} collections: {:?}", 
                        user_id, collections.len(), 
                        collections.iter().map(|c| (&c.id, &c.name)).collect::<Vec<_>>());
                }
                Err(e) => {
                    tracing::error!("Error fetching user collections: {}", e);
                }
            }
            
            return Ok(HttpResponse::NotFound().json(PdfConversionError {
                success: false,
                error: "Creative not found or you don't have access to it".to_string(),
            }));
        }
        Err(e) => {
            tracing::error!("Database error fetching creative {}: {}", creative_id, e);
            return Ok(HttpResponse::InternalServerError().json(PdfConversionError {
                success: false,
                error: "Failed to fetch creative from database".to_string(),
            }));
        }
    };
    
    // Check if the creative has an HTML URL
    let html_url = if creative.html_url.is_empty() {
        tracing::warn!("Creative {} has no HTML URL", creative_id);
        return Ok(HttpResponse::BadRequest().json(PdfConversionError {
            success: false,
            error: "Creative does not have a published HTML URL".to_string(),
        }));
    } else {
        creative.html_url.clone()
    };
    
    // Create the PDF conversion service
    let pdf_service = match PdfConversionService::new() {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("Failed to create PDF service: {}", e);
            return Ok(HttpResponse::InternalServerError().json(PdfConversionError {
                success: false,
                error: format!("Failed to initialize PDF service: {e}"),
            }));
        }
    };
    
    // Generate a meaningful filename with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let creative_name = if creative.name.is_empty() {
        "creative".to_string()
    } else {
        // Sanitize the creative name for filename use
        creative.name
            .trim()
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                ' ' => '_',
                _ => '_',
            })
            .collect::<String>()
            .chars()
            .take(50) // Limit length to avoid overly long filenames
            .collect::<String>()
    };
    let filename = format!("{creative_name}_{timestamp}.pdf");
    
    // Log the exact URL and parameters that will be sent to Lighthouse
    tracing::info!("=== PDF CONVERSION DEBUG ===");
    tracing::info!("Creative ID: {}", creative_id);
    tracing::info!("Creative name: {}", creative.name);
    tracing::info!("HTML URL to convert: {}", html_url);
    tracing::info!("Generated filename: {}", filename);
    
    tracing::info!("Calling PDF service with URL: {}", html_url);
    tracing::info!("=== END DEBUG ===");
    
    // Perform the PDF conversion and return PDF bytes directly
    match pdf_service.convert_url_to_pdf_direct(&html_url, &filename, Some("custom".to_string())).await {
        Ok(pdf_bytes) => {
            let file_size = pdf_bytes.len() as u64;
            tracing::info!("PDF conversion successful for creative {}: {} ({} bytes)", creative_id, filename, file_size);
            
            // Return the PDF directly as binary response
            Ok(HttpResponse::Ok()
                .content_type("application/pdf")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{filename}\"")))
                .body(pdf_bytes))
        }
        Err(e) => {
            tracing::error!("PDF conversion failed for creative {}: {}", creative_id, e);
            Ok(HttpResponse::InternalServerError().json(PdfConversionError {
                success: false,
                error: format!("PDF conversion failed: {e}"),
            }))
        }
    }
}



/// Configure PDF conversion routes
pub fn configure_pdf_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(convert_to_pdf)
       .service(convert_creative_to_pdf);
}

#[derive(OpenApi)]
#[openapi(
    paths(
        convert_to_pdf,
        convert_creative_to_pdf,
    ),
    components(
        schemas(ConvertToPdfRequest, PdfConversionError)
    ),
    tags(
        (name = "PDF Conversion", description = "Website to PDF conversion endpoints")
    )
)]
pub struct PdfApiDoc; 