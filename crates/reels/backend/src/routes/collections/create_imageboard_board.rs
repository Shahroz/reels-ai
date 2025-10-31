//! Create an Imageboard board for a collection (listing) and inject its image assets.
//!
//! POST /api/collections/{collection_id}/imageboard
//! - Validates access to the collection for the current user
//! - Collects all image assets attached to the collection
//! - Calls the external Imageboard service to create a board with those assets
//! - Returns edit/view tokens and URLs

use actix_web::{post, web, HttpResponse, Responder};
use tracing::instrument;

use crate::auth::tokens::Claims;
use crate::queries::organizations::organization_exists;
use crate::routes::collections::create_imageboard_board_request::CreateImageboardBoardRequest;

#[derive(serde::Serialize, utoipa::ToSchema, Debug)]
pub struct CreateImageboardBoardResponse {
    pub board_id: String,
    pub edit_access_token: String,
    pub view_access_token: String,
    pub edit_url: String,
    pub view_url: String,
}

#[utoipa::path(
    post,
    path = "/api/collections/{collection_id}/imageboard",
    tag = "Collections",
    params(("collection_id" = uuid::Uuid, Path, description = "Collection (listing) ID")),
    request_body = CreateImageboardBoardRequest,
    responses(
        (status = 201, description = "Board created", body = CreateImageboardBoardResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden: No access to collection"),
        (status = 404, description = "Collection not found"),
        (status = 500, description = "Failed to create board")
    )
)]
#[post("/{collection_id}/imageboard")]
#[instrument(
    name = "create_imageboard_board_for_collection",
    skip(pool, img_client, claims)
)]
pub async fn create_imageboard_board(
    pool: web::Data<sqlx::PgPool>,
    img_client: web::Data<crate::services::imageboard_client::ImageboardClient>,
    path: web::Path<uuid::Uuid>,
    request_body: web::Json<CreateImageboardBoardRequest>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let collection_id = path.into_inner();
    let user_id = claims.user_id;
    let organization_id = request_body.organization_id;

    // Check if organization exists by ID
    match organization_exists(&**pool, organization_id).await {
        Ok(true) => {
            // Organization exists, continue with board creation
        }
        Ok(false) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Organization not found"
            }));
        }
        Err(e) => {
            tracing::error!(
                "Failed to check organization existence {}: {:?}",
                organization_id,
                e
            );
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to validate organization"
            }));
        }
    }

    // Update collection's organization_id if not set
    if let Err(e) = crate::queries::collections::update_collection_organization_id::update_collection_organization_id(
        pool.get_ref(),
        collection_id,
        organization_id,
    ).await {
        // If update fails, it might be because organization_id is already set, which is fine
        // Or it might be a DB error - log but don't fail the request
        tracing::warn!("Failed to update collection organization_id (collection_id={}, organization_id={}): {:?}", 
            collection_id, organization_id, e);
    }

    // Fetch collection with all assets (using a generous limit). Validates access.
    let limit: i64 = 10_000;
    let offset: i64 = 0;
    let search_pattern = "%".to_string();
    let sort_by = "created_at";
    let sort_order = "desc";

    let res = crate::queries::collections::get_collection_with_assets::get_collection_with_assets(
        &pool,
        collection_id,
        user_id,
        &search_pattern,
        sort_by,
        sort_order,
        limit,
        offset,
    )
    .await;

    let coll = match res {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Collection not found or you don't have permission to access it"
            }));
        }
        Err(e) => {
            tracing::error!("DB error fetching collection with assets: {e:?}");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch collection"
            }));
        }
    };

    // Build assets list (images only)
    let assets: Vec<crate::services::imageboard_client::AssetInput> = coll
        .assets
        .into_iter()
        .filter(|a| a.asset.r#type.starts_with("image/") && !a.asset.url.is_empty())
        .map(|a| crate::services::imageboard_client::AssetInput {
            url: a.asset.url.clone(),
            title: Some(a.asset.name.clone()),
            width: None,
            height: None,
        })
        .collect();

    // Prepare request
    // Use the listing (collection) UUID as the board_id for persistence
    let assets_count = assets.len();
    let body = crate::services::imageboard_client::CreateBoardRequest {
        owner_user_id: user_id.to_string(),
        board_id: Some(collection_id.to_string()),
        name: Some(coll.collection.name.clone()),
        check_balance_url: None,
        transactions_callback_url: None,
        // legacy callback - keep None
        callback_url: None,
        assets: if assets.is_empty() {
            None
        } else {
            Some(assets)
        },
        allowed_referrer_url: None,
        description: None,
        is_forkable: None,
        entity_id: Some(organization_id.to_string()),
        email: Some(claims.email.clone()),
        email_verified: Some(claims.email_verified), // Use actual email verification status from JWT claims
    };

    // Log the request for debugging
    tracing::info!(
        "Creating imageboard board for collection {} with {} assets",
        collection_id,
        assets_count
    );

    // Attempt public create (supports assets injection via API key).
    let attempt_admin = img_client.create_board_with_urls(body.clone()).await;
    let result = match attempt_admin {
        Ok(r) => Ok(r),
        Err(e) => {
            // If board already exists with this ID, fetch tokens via admin endpoint and build URLs
            if let crate::services::imageboard_client::ClientError::Http(code, _msg) = &e {
                if *code == 409 || *code == 400 {
                    // treat conflict/bad-request as possible already-exists
                    let bid = collection_id.to_string();
                    match img_client.get_access_tokens_admin(&bid).await {
                        Ok(tokens) => {
                            let edit_url =
                                img_client.build_board_url(&bid, &tokens.edit_access_token);
                            let view_url =
                                img_client.build_board_url(&bid, &tokens.view_access_token);
                            return HttpResponse::Created().json(CreateImageboardBoardResponse {
                                board_id: bid,
                                edit_access_token: tokens.edit_access_token,
                                view_access_token: tokens.view_access_token,
                                edit_url,
                                view_url,
                            });
                        }
                        Err(err2) => {
                            tracing::warn!("Failed to get existing board tokens after admin create error: {:?}", err2);
                        }
                    }
                }
            }
            tracing::warn!(
                "Admin board creation failed and no existing board tokens could be retrieved: {:?}",
                e
            );
            Err(e)
        }
    };

    match result {
        Ok(r) => {
            let resp = CreateImageboardBoardResponse {
                board_id: r.board_id,
                edit_access_token: r.edit_access_token,
                view_access_token: r.view_access_token,
                edit_url: r.edit_url,
                view_url: r.view_url,
            };
            HttpResponse::Created().json(resp)
        }
        Err(e) => {
            tracing::error!("Failed to create imageboard board: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create imageboard board"
            }))
        }
    }
}
