//! Handles the HTTP POST request to copy an existing custom creative format.
//!
//! This operation allows an authenticated user to duplicate a custom creative format.
//! The original format can be one they own or a public one.
//! Admin users can copy any format regardless of ownership or public status.
//! The newly created format will belong to the user and be private by default.
//! It generates a new ID and updates timestamps for the copy.

use tracing_attributes::instrument;
use crate::db::custom_creative_formats::CustomCreativeFormat;
use crate::routes::error_response::ErrorResponse;

/// Handler for `POST /api/formats/custom-creative-formats/{id}/copy`.
///
/// Copies an existing `CustomCreativeFormat`. The original can be a public format
/// or a private format owned by the requesting user. Admin users can copy any format.
/// The copy will be owned by the requesting user and marked as private.
#[utoipa::path(
    post,
    path = "/api/formats/custom-creative-formats/{id}/copy",
    tag = "Formats",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "ID of the custom creative format to copy")
    ),
    responses(
        (status = 201, description = "Custom creative format copied successfully", body = CustomCreativeFormat),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Custom creative format not found or not accessible", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    )
)]
#[actix_web::post("/{id}/copy")]
#[instrument(skip(pool, claims))]
pub async fn copy_custom_creative_format(
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    id: actix_web::web::Path<uuid::Uuid>,
    pool: actix_web::web::Data<sqlx::PgPool>,
) -> actix_web::HttpResponse {
    let original_format_id = id.into_inner();
    let user_id = claims.user_id;
    let is_admin = claims.is_admin;

    // Fetch the original CustomCreativeFormat by id, checking visibility
    let original_format = match crate::queries::custom_creative_formats::find_one_for_copy::find_one_for_copy(
        pool.get_ref(),
        original_format_id,
        user_id,
        is_admin,
    )
    .await
    {
        Ok(format) => format,
        Err(sqlx::Error::RowNotFound) => {
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Custom creative format not found or not accessible"),
                }
            );
        }
        Err(e) => {
            std::eprintln!("Failed to fetch custom creative format: {e}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to retrieve custom creative format"),
                }
            );
        }
    };

    let new_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    // Name and public status for the copy are now determined by server-side defaults.
    let new_name = std::format!("{} Copy", original_format.name);
    let new_is_public = false; // Copies are private by default

   let new_custom_format = crate::db::custom_creative_formats::CustomCreativeFormat {
       id: new_id,
       user_id: Some(user_id), // New format belongs to the current user
       name: new_name,
       description: original_format.description.clone(),
       width: original_format.width,
        height: original_format.height,
        creative_type: original_format.creative_type.clone(),
        json_schema: original_format.json_schema.clone(),
        is_public: new_is_public,
        metadata: original_format.metadata.clone(),
        created_at: now,
        updated_at: now,
    };

    // Insert the new CustomCreativeFormat into the database
    match crate::queries::custom_creative_formats::insert_copy::insert_copy(
        pool.get_ref(),
        &new_custom_format,
    )
    .await
    {
        Ok(created_format) => actix_web::HttpResponse::Created().json(created_format),
        Err(e) => {
            std::eprintln!("Failed to insert new custom creative format: {e}");
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to copy custom creative format"),
                }
            )
        }
    }
}
