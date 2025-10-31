//! Handler for updating an existing custom creative format.
//!
//! PUT /api/custom-creative-formats/{id}

//! Allows authenticated users to modify their own custom formats.
//! Requires JWT authentication and ownership check (user_id matches).
//! Admin users can update any format regardless of ownership.
//! The format ID is taken from the URL path.

use crate::db::custom_creative_formats::CustomCreativeFormat;
use crate::queries::custom_creative_formats::{exists, update};
use crate::routes::error_response::ErrorResponse;
use crate::routes::formats::create_custom_format_request::CreateCustomFormatRequest;
use tracing::instrument;

#[utoipa::path(
    put,
    path = "/api/formats/custom-creative-formats/{id}",
    tag = "Formats",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = String, Path, description = "ID of the custom format to update")
    ),
    request_body = CreateCustomFormatRequest,
    responses(
        (status = 200, description = "Custom format updated", body = CustomCreativeFormat),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not owner (admin users can update any format)"),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::put("/{id}")]
#[instrument(skip(pool, payload, claims))]
pub async fn update_custom_creative_format(
    pool: actix_web::web::Data<sqlx::PgPool>,
    id: actix_web::web::Path<uuid::Uuid>,
    // Use the imported type directly
    payload: actix_web::web::Json<CreateCustomFormatRequest>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id: uuid::Uuid = claims.user_id;
    let is_admin: bool = claims.is_admin;
    let format_id = *id;
    let format_data = payload.into_inner();

    // Basic validation
    // Dimensions are now optional, validate if present
    if format_data.name.is_empty()
        || format_data.width.is_some_and(|w| w <= 0)
        || format_data.height.is_some_and(|h| h <= 0)
    {
        return actix_web::HttpResponse::BadRequest().json(
            ErrorResponse { // Use the imported type
                error: "Invalid format parameters: name required, dimensions (if provided) must be positive"
                    .to_string(),
            },
        );
    } // Assuming format_data.creative_type is an enum like crate::db::creative_type::CreativeType

    let result = update::update(pool.get_ref(), format_id, user_id, is_admin, &format_data).await;

    match result {
        Ok(item) => actix_web::HttpResponse::Ok().json(item),
        Err(sqlx::Error::RowNotFound) => {
            // Could be not found OR forbidden, check if exists at all for better error message
            let format_exists = exists::exists(pool.get_ref(), format_id)
                .await
                .unwrap_or(false); // Treat db error as "doesn't exist" for this check

            if format_exists && !is_admin {
                actix_web::HttpResponse::Forbidden().json(
                    ErrorResponse { // Use the imported type
                        error: "Forbidden: You do not own this format".to_string(),
                    },
                )
            } else {
                actix_web::HttpResponse::NotFound().json(
                    ErrorResponse { // Use the imported type
                        error: "Custom format not found".to_string(),
                    },
                )
            }
        }
        Err(e) => {
            tracing::error!(
                "Database error updating custom creative format {}: {:?}",
                format_id,
                e
            );
            actix_web::HttpResponse::InternalServerError().json(
                ErrorResponse { // Use the imported type
                    error: "Failed to update custom format".to_string(),
                },
            )
        }
    }
}

#[cfg(test)] // Changed from FALSE
mod tests {
    // Tests need database and auth setup.
    // #[sqlx::test]
    // async fn test_update_custom_format_success(pool: sqlx::PgPool) {
    //     // Setup user, token, create a format owned by user...
    //     // Prepare valid update payload...
    //     // Make PUT request with auth, path ID, payload...
    //     // Assert 200 OK and returned format has updated values...
    //     // Verify database state...
    // }
    //
    // #[sqlx::test]
    // async fn test_update_custom_format_not_owner(pool: sqlx::PgPool) {
    //     // Setup user1, token1, user2, create format owned by user2...
    //     // Prepare valid update payload...
    //     // Make PUT request AS USER1 to user2's format ID...
    //     // Assert 403 Forbidden...
    // }
    //
    // #[sqlx::test]
    // async fn test_update_custom_format_not_found(pool: sqlx::PgPool) {
    //     // Setup user, token...
    //     // Prepare valid update payload...
    //     // Make PUT request with auth to a non-existent UUID...
    //     // Assert 404 Not Found...
    // }
}
