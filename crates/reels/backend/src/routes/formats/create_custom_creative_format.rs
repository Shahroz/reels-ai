//! Handler for creating a new custom creative format.
//!
//! POST /api/custom-creative-formats

//! Allows authenticated users to define their own creative formats.
//! Requires JWT authentication. Input is validated against the request schema.

use crate::db::custom_creative_formats::CustomCreativeFormat;
use crate::routes::error_response::ErrorResponse;
use crate::routes::formats::create_custom_format_request::CreateCustomFormatRequest;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/formats/custom-creative-formats",
    tag = "Formats",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateCustomFormatRequest,
    responses(
        (status = 201, description = "Custom format created", body = CustomCreativeFormat),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal error", body = ErrorResponse)
    )
)]
#[actix_web::post("")]
#[instrument(skip(pool, payload, claims))]
pub async fn create_custom_creative_format(
    pool: actix_web::web::Data<sqlx::PgPool>,
    payload: actix_web::web::Json<CreateCustomFormatRequest>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let user_id: uuid::Uuid = claims.user_id;
    let format_data = payload.into_inner(); // Take ownership

    // Basic validation (could be expanded)
    // Dimensions are now optional, validate if present
    if format_data.name.is_empty()
        || format_data.width.is_some_and(|w| w <= 0)
        || format_data.height.is_some_and(|h| h <= 0)
    {
        return actix_web::HttpResponse::BadRequest().json(
            ErrorResponse {
                error: "Invalid format parameters: name required, dimensions (if provided) must be positive"
                    .to_string(),
            },
        );
    } // Assuming format_data.creative_type is an enum like crate::db::creative_type::CreativeType

    let result = crate::queries::custom_creative_formats::create::create_custom_creative_format(
        pool.get_ref(),
        user_id,
        &format_data,
    )
    .await;

    match result {
        Ok(item) => actix_web::HttpResponse::Created().json(item),
        Err(e) => {
            tracing::error!(
                "Database error inserting custom creative format for user {}: {:?}",
                user_id, e
            );
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to create custom format".to_string(),
            })
        }
    }
}

#[cfg(test)] // Changed from FALSE
mod tests {
    // Tests need database and auth setup.
    // #[sqlx::test]
    // async fn test_create_custom_format_success(pool: sqlx::PgPool) {
    //     // Setup user, generate token...
    //     // Prepare valid payload...
    //     // Make POST request with auth header and payload...
    //     // Assert 201 Created status and returned format matches payload...
    //     // Verify data in database...
    // }
    //
    // #[sqlx::test]
    // async fn test_create_custom_format_invalid_data(pool: sqlx::PgPool) {
    //     // Setup user, generate token...
    //     // Prepare invalid payload (e.g., empty name, zero width)...
    //     // Make POST request...
    //     // Assert 400 Bad Request status...
    // }
}
