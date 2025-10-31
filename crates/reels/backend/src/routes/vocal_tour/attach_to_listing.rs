//! Handler for attaching vocal tour assets to a collection/listing.
//!
//! Defines the `attach_vocal_tour_to_listing` HTTP handler under `/api/vocal-tour/{vocal_tour_id}/attach-to-listing`.
//! This handler retrieves all assets from a vocal tour (if any) and attaches them to the specified collection.
//! It also attaches the vocal tour document and related marketing documents to the collection.
//! Includes proper authorization to ensure users can only attach their own vocal tours to their own collections.
//! Note: Vocal tours without assets are still valid and will have their documents attached.

use crate::routes::vocal_tour::attach_to_listing_request::AttachVocalTourToListingRequest;
use crate::routes::vocal_tour::attach_to_listing_response::AttachVocalTourToListingResponse;


#[utoipa::path(
    post,
    path = "/api/vocal-tour/{vocal_tour_id}/attach-to-listing",
    tag = "Vocal Tour",
    params(
        ("vocal_tour_id" = uuid::Uuid, Path, description = "Vocal tour ID to attach assets from")
    ),
    request_body = AttachVocalTourToListingRequest,
    responses(
        (status = 200, description = "Vocal tour content successfully attached to listing (assets and/or documents)", body = AttachVocalTourToListingResponse),
        (status = 400, description = "Bad Request - Invalid input"),
        (status = 403, description = "User does not own the vocal tour or collection"),
        (status = 404, description = "Vocal tour or collection not found"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("user_auth" = []))
)]
#[actix_web::post("/{vocal_tour_id}/attach-to-listing")]
#[tracing::instrument(skip(pool, claims))]
pub async fn attach_vocal_tour_to_listing(
    pool: actix_web::web::Data<sqlx::PgPool>,
    path: actix_web::web::Path<uuid::Uuid>,
    req: actix_web::web::Json<AttachVocalTourToListingRequest>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
) -> impl actix_web::Responder {
    let vocal_tour_id = path.into_inner();
    let user_id = claims.user_id;
    let collection_id = req.collection_id;

    // Start transaction for atomicity
    let mut tx = match pool.begin().await {
        std::result::Result::Ok(tx) => tx,
        std::result::Result::Err(e) => {
            log::error!("Failed to begin transaction for vocal tour attachment: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Failed to process attachment request".into(),
                }
            );
        }
    };

    // 1. Verify user owns the vocal tour and get asset IDs
    let vocal_tour = match crate::queries::vocal_tours::get_vocal_tour_by_id::get_vocal_tour_by_id(
        &pool,
        vocal_tour_id,
    ).await {
        std::result::Result::Ok(std::option::Option::Some(vt)) if vt.user_id == user_id => vt,
        std::result::Result::Ok(std::option::Option::Some(_)) => {
            let _ = tx.rollback().await;
            return actix_web::HttpResponse::Forbidden().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Vocal tour not found or access denied".into(),
                }
            );
        }
        std::result::Result::Ok(std::option::Option::None) => {
            let _ = tx.rollback().await;
            return actix_web::HttpResponse::NotFound().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Vocal tour not found".into(),
                }
            );
        }
        std::result::Result::Err(e) => {
            let _ = tx.rollback().await;
            log::error!("Database error fetching vocal tour {vocal_tour_id}: {e}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Failed to fetch vocal tour".into(),
                }
            );
        }
    };

    // 2. Verify user owns the collection
    let _collection = match crate::routes::assets::validation::validate_collection_ownership::validate_collection_ownership(&mut *tx, collection_id, user_id).await {
        std::result::Result::Ok(collection) => collection,
        std::result::Result::Err(response) => {
            let _ = tx.rollback().await;
            return response;
        }
    };

    // 3. Attach all assets from vocal tour to collection (if any exist)
    let updated_count = if vocal_tour.asset_ids.is_empty() {
        log::info!("Vocal tour has no assets to attach, proceeding with document attachment only");
        0
    } else {
        match sqlx::query!(
            "UPDATE assets SET collection_id = $1, updated_at = NOW() WHERE id = ANY($2) AND user_id = $3",
            collection_id,
            &vocal_tour.asset_ids,
            user_id
        )
        .execute(&mut *tx)
        .await {
            std::result::Result::Ok(result) => result.rows_affected(),
            std::result::Result::Err(e) => {
                let _ = tx.rollback().await;
                log::error!("Database error during asset attachment: {e:?}");
                return actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: "Failed to attach assets to collection".into(),
                    }
                );
            }
        }
    };

    // 4. Attach vocal tour document and related marketing documents to collection
    log::info!("Attaching vocal tour document and related marketing documents to collection {}", collection_id);
    
    let document_update_count = match sqlx::query!(
        "UPDATE documents SET collection_id = $1, updated_at = NOW() 
         WHERE (id = $2 OR $2::text = ANY(sources)) AND user_id = $3",
        collection_id,
        vocal_tour.document_id,
        user_id
    )
    .execute(&mut *tx)
    .await {
        std::result::Result::Ok(result) => result.rows_affected(),
        std::result::Result::Err(e) => {
            let _ = tx.rollback().await;
            log::error!("Database error during document attachment: {e:?}");
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::assets::error_response::ErrorResponse {
                    error: "Failed to attach documents to collection".into(),
                }
            );
        }
    };

    if updated_count > 0 {
        log::info!("Attached {} assets and {} documents to collection {}", updated_count, document_update_count, collection_id);
    } else {
        log::info!("Attached {} documents to collection {} (no assets to attach)", document_update_count, collection_id);
    }

    // 5. Commit transaction
    if let std::result::Result::Err(e) = tx.commit().await {
        log::error!("Failed to commit attachment transaction: {e:?}");
        return actix_web::HttpResponse::InternalServerError().json(
            crate::routes::assets::error_response::ErrorResponse {
                error: "Failed to complete attachment operation".into(),
            }
        );
    }

    // 6. Return success response
    let response = AttachVocalTourToListingResponse {
        attached_asset_count: updated_count as usize,
        attached_document_count: document_update_count as usize,
        collection_id,
    };

    actix_web::HttpResponse::Ok().json(response)
} 