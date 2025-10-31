//! Handles the request to stream conversation events for a session via WebSocket.
//!
//! Establishes a WebSocket connection for a specific session, upgrading the HTTP request.
//! It uses `actix_web_actors::ws::start` to initiate the WebSocket session
//! with the `WsSession` actor defined in `crate::websocket::handler`.
//! Adheres to the one-item-per-file and FQN guidelines.

//! Revision History
//! - 2025-04-24T14:11:33Z @AI: Implemented WebSocket handshake using ws::start.
//! - 2025-04-24T12:45:12Z @AI: Initial stub implementation.


/// Handles GET requests to establish a WebSocket conversation stream.
///
/// Upgrades the connection to WebSocket and starts the `WsSession` actor.
/// Requires `SessionId` from the path and `AppState` from application data.
#[utoipa::path(
    get,
    path = "/loupe/session/{session_id}/stream",
    tag = "Session",
    params(
        ("session_id" = Uuid, Path, description = "ID of the session to stream events for")
    ),
    responses(
        (status = 101, description = "WebSocket connection successfully established"),
        (status = 400, description = "Bad request during WebSocket handshake"),
        (status = 500, description = "Internal server error during WebSocket handshake")
    ),
    tag = "Loupe"
)]
pub async fn conversation_stream(
    session_id_path: actix_web::web::Path<crate::types::session_id::SessionId>,
    req: actix_web::HttpRequest,
    stream: actix_web::web::Payload,
    app_state: actix_web::web::Data<crate::state::app_state::AppState>, // Access AppState via Data
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let session_id = session_id_path.into_inner();
    std::println!("Attempting WebSocket handshake for session: {}", session_id);

    // Create the WsSession actor instance
    // WsSession::new expects Data<AppState>, which we have directly
    let ws_session_actor = crate::websocket::handler::WsSession::new(
        session_id,
        app_state, // Pass the AppState Data wrapper directly
    );

    // Start the WebSocket actor using the provided request and stream payload
    let resp = actix_web_actors::ws::start(ws_session_actor, &req, stream);

    // Log success or failure of the handshake attempt
    match &resp {
        Ok(http_response) => {
            // Note: This logs the response prepared by ws::start, typically 101 Switching Protocols
            std::println!("WebSocket handshake successful for session: {}. Response status: {}", session_id, http_response.status());
        },
        Err(e) => {
             // Log if ws::start itself returns an error (e.g., invalid headers)
             std::eprintln!("WebSocket handshake failed for session: {}. Error: {}", session_id, e);
        }
    }

    resp // Return the Result<HttpResponse, Error> from ws::start
}

// No tests included here, as testing WebSocket handlers often requires
// more complex integration or E2E testing setups. Unit testing focuses on the actor (`WsSession`).
// Example test structure would involve mocking HttpRequest, Payload, and AppState.