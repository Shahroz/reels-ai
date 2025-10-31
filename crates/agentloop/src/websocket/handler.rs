//! Handles WebSocket connections for the AgentLoop service.
//!
//! Defines the WsSession actor for managing WebSocket connections.
//!
//! This actor handles the lifecycle of a single WebSocket connection, including initial setup,
//! heartbeat management for connection liveness, and message forwarding.
//! It registers each connection with the application's shared state (`AppState`)
//! to enable broadcasting of `WebsocketEvent`s to the appropriate client.
//! Incoming text messages from the client are logged, and the connection is
//! Adheres to the project's Rust coding guidelines, including FQN usage.
//! properly cleaned up upon termination or timeout.

//! Revision History
//! - 2025-05-14T19:11:38Z @AI: Refactor WsSession actor per new requirements. Simplify text message handling, inline registration/unregistration, update struct definition.
//! - 2025-04-24T14:54:10Z @AI: Fix nested combinator syntax errors in started() and stopping().
//! - 2025-04-24T13:58:24Z @AI: Apply FQN refactoring per rust_guidelines.
//! - 2025-04-24T13:41:46Z @AI: Refactor for async AppState and direct state manipulation.
//! - 2025-04-24T12:48:48Z @AI: Initial implementation based on instruction.

use actix::{ActorContext, AsyncContext};
// --- Standard Library Imports --- (Removed - Use FQN)
// --- External Crate Imports --- (Removed - Use FQN)
// --- Internal Crate Imports --- (Removed - Use FQN)
// Extension traits for actor futures
use actix::fut::{WrapFuture, ActorFutureExt};
use log; // Added for logging
use crate::types::ws_request::WebsocketRequest; // Added for deserializing client messages

/// How often heartbeat pings are sent.
const HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);
/// How long before lack of client response causes a timeout.
const CLIENT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Actor representing a WebSocket session.
pub struct WsSession {
    /// Unique session ID this WebSocket is associated with.
    session_id: crate::types::session_id::SessionId,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// we check every 5 seconds (HEARTBEAT_INTERVAL).
    hb: std::time::Instant,
    /// Shared application state, accessed via Data for Clone-ability.
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    /// Stores its own recipient for easier removal from AppState.
    recipient: Option<actix::Recipient<crate::types::ws_event::WebsocketEvent>>,
}

impl WsSession {
    /// Creates a new WebSocket session actor instance.
    pub fn new(
        session_id: crate::types::session_id::SessionId,
        app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    ) -> Self {
        Self {
            session_id,
            hb: std::time::Instant::now(),
            app_state,
            recipient: None,
        }
    }

    /// Helper function to send heartbeat pings to the client.
    /// Runs periodically within the actor's context.
    fn hb(&self, ctx: &mut actix_web_actors::ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check client heartbeats
            if std::time::Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // Heartbeat timed out
                std::println!("WebSocket Client heartbeat failed, disconnecting SessionId: {}", act.session_id);

                // Stop actor - unregistration happens in stopping()
                ctx.stop();
                return; // Don't try to send a ping
            }
            // Send ping
            ctx.ping(b"");
       });
   }

}

impl actix::Actor for WsSession {
    type Context = actix_web_actors::ws::WebsocketContext<Self>;

    /// Called when the actor is first started.
    /// Spawns async task for registration and starts heartbeating.
    fn started(&mut self, ctx: &mut Self::Context) {
        std::println!("WebSocket session starting for SessionId: {}", self.session_id);
        self.hb(ctx);

        self.recipient = Some(ctx.address().recipient()); // Store recipient

        // Spawn the async registration task using AppState directly
        let app_state_clone = self.app_state.clone();
        let session_id_clone = self.session_id.clone();

        if let Some(recipient_to_register) = &self.recipient {
            let recipient_clone = recipient_to_register.clone();
            ctx.spawn(
                async move {
                    let mut ws_connections = app_state_clone.ws_connections.lock().await;
                    ws_connections
                        .entry(session_id_clone)
                        .or_default()
                        .push(recipient_clone.clone()); // Clone recipient for registration
                    std::println!("Registered WebSocket recipient for SessionId: {}", session_id_clone);
                }
                .into_actor(self) // Wrap future to run in actor context
                .map(|_res, act, _ctx| { // Use map from ActorFutureExt trait
                     std::println!("Async registration completed for SessionId: {}.", act.session_id);
                })
            );
        } else {
            std::eprintln!("Error: Recipient not set in started() for SessionId: {}. Stopping actor.", self.session_id);
            ctx.stop();
        }
    }

    /// Called when the actor is stopping.
    /// Spawns async task for unregistration.
    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        std::println!("WebSocket session stopping for SessionId: {}", self.session_id);

        if let Some(own_recipient) = self.recipient.take() { // Take the recipient
            let app_state_clone = self.app_state.clone();
            let session_id_clone = self.session_id.clone();

            // Use AsyncContext trait method wait for unregistration
            <Self::Context as actix::AsyncContext<Self>>::wait(ctx,
                async move {
                    let mut ws_connections = app_state_clone.ws_connections.lock().await;
                    if let Some(session_conns) = ws_connections.get_mut(&session_id_clone) {
                        session_conns.retain(|r| r != &own_recipient);
                        std::println!("Attempted to unregister recipient for SessionId: {}", session_id_clone);
                        if session_conns.is_empty() {
                            ws_connections.remove(&session_id_clone);
                            std::println!("Removed SessionId {} from ws_connections as it has no active recipients.", session_id_clone);
                        } else {
                            std::println!("SessionId {} still has {} active recipients after unregistration attempt.", session_id_clone, session_conns.len());
                        }
                    }
                }
                .into_actor(self) // Wrap future to run in actor context
                .map(|_res, act, _ctx| {
                    std::println!("Async unregistration logic completed for SessionId: {}.", act.session_id);
                })
            );
        } else {
            std::println!("No recipient to unregister for SessionId: {} (already taken or never set).", self.session_id);
        }
        actix::Running::Stop
    }
}

/// Handler for incoming WebSocket messages from the client.
impl actix::StreamHandler<Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(actix_web_actors::ws::Message::Ping(msg)) => {
                self.hb = std::time::Instant::now();
                ctx.pong(&msg);
            }
            Ok(actix_web_actors::ws::Message::Pong(_)) => {
                self.hb = std::time::Instant::now();
            }
            Ok(actix_web_actors::ws::Message::Text(text)) => {
                // As per new instruction, just log the received text.
                // std::println!("Received text from WebSocket for session {}: {}", self.session_id, text); // Replaced by log
                log::debug!("Session {}: Received text: {}", self.session_id, text);

                match serde_json::from_str::<WebsocketRequest>(&text) {
                    Ok(ws_request) => {
                        let app_state_clone = self.app_state.clone();
                        let session_id_clone = self.session_id.clone();

                        ctx.spawn(
                            async move {
                                match ws_request {
                                    WebsocketRequest::UserInput { instruction, attachments } => {
                                        log::info!("Session {}: Processing UserInput: '{}', attachments: {}", session_id_clone, instruction, attachments.len());
                                        
                                        // Extract data and determine if we need to spawn a research task - minimize lock duration
                                        let (should_spawn_research_task, app_state_for_task, session_id_for_task) = {
                                            let mut sessions_map = app_state_clone.sessions.lock().await;
                                            if let Some(session_data) = sessions_map.get_mut(&session_id_clone) {
                                                // Add to history
                                                let new_entry = crate::types::conversation_entry::ConversationEntry {
                                                    id: uuid::Uuid::new_v4(),
                                                    parent_id: session_data.history.last().map(|entry| entry.id),
                                                    depth: session_data.history.len() as u32,
                                                    sender: crate::types::sender::Sender::User,
                                                    message: instruction.clone(),
                                                    timestamp: chrono::Utc::now(),
                                                    attachments: attachments.clone(),
                                                    tools: std::vec::Vec::new(),
                                                    tool_choice: None,
                                                    tool_response: None,
                                                };
                                                session_data.history.push(new_entry);

                                                // Update status and goal if session is in a state to accept new work
                                                let should_spawn = match session_data.status {
                                                    crate::types::session_status::SessionStatus::AwaitingInput |
                                                    crate::types::session_status::SessionStatus::Completed |
                                                    crate::types::session_status::SessionStatus::Error |
                                                    crate::types::session_status::SessionStatus::Timeout |
                                                    crate::types::session_status::SessionStatus::Interrupted => {
                                                        session_data.status = crate::types::session_status::SessionStatus::Pending;
                                                        session_data.research_goal = Some(instruction);
                                                        log::info!("Session {}: Transitioned to Pending with new research goal.", session_id_clone);
                                                        true // Should spawn research task
                                                    }
                                                    crate::types::session_status::SessionStatus::Pending |
                                                    crate::types::session_status::SessionStatus::Running { .. } => {
                                                        log::info!("Session {}: UserInput added to history for active session (status: {:?}).", session_id_clone, session_data.status);
                                                        false // Don't spawn research task
                                                    }
                                                };
                                                
                                                (should_spawn, app_state_clone.clone(), session_id_clone.clone())
                                            } else {
                                                log::error!("Session {}: Not found when processing UserInput.", session_id_clone);
                                                (false, app_state_clone.clone(), session_id_clone.clone())
                                            }
                                        }; // Lock is released here!

                                        // Spawn research task AFTER releasing the lock
                                        if should_spawn_research_task {
                                            tokio::spawn(async move {
                                                log::info!("Spawning research task for session {}", session_id_for_task);
                                                match crate::evaluator::run_research_loop::run_research_loop(
                                                    app_state_for_task,
                                                    session_id_for_task,
                                                    None, // No progress callback for websocket handler
                                                )
                                                .await
                                                {
                                                    Ok(_) => log::info!("Research task for session {} completed successfully.", session_id_for_task),
                                                    Err(e) => log::error!("Research task for session {} failed: {}", session_id_for_task, e),
                                                }
                                            });
                                        }
                                    }
                                    WebsocketRequest::Interrupt => {
                                        log::info!("Session {}: Processing Interrupt request.", session_id_clone);
                                        if let Err(e) = crate::session::manager::update_status(
                                            app_state_clone,
                                            &session_id_clone,
                                            crate::types::session_status::SessionStatus::Interrupted,
                                        ).await {
                                            log::error!("Session {}: Failed to set status to Interrupted: {}", session_id_clone, e);
                                        }
                                    }
                                }
                            }
                            .into_actor(self)
                            .map(|_res, _act, _ctx| {
                                log::debug!("Session {}: Async WebSocketRequest processing complete.", _act.session_id);
                            })
                        );
                    }
                    Err(e) => {
                        log::error!("Session {}: Failed to deserialize WebSocketRequest: {}. Raw text: '{}'", self.session_id, e, text);
                        ctx.text(format!("{{\"error\":\"Failed to parse request: {}\"}}", e)); // Send error back to client
                    }
                }
            }
            Ok(actix_web_actors::ws::Message::Binary(bin)) => {
                std::println!("Received unexpected binary message for session {}: {:?}", self.session_id, bin);
            }
            Ok(actix_web_actors::ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(actix_web_actors::ws::Message::Continuation(_)) => {
                ctx.stop(); // Stop on unexpected continuation fragments
            }
            Ok(actix_web_actors::ws::Message::Nop) => (),
            Err(e) => {
                log::error!("WebSocket protocol error for session {}: {}", self.session_id, e);
                ctx.stop(); // Stop actor on protocol error
            }
        }
    }
}


/// Handler for server-sent `WebsocketEvent` messages.
/// This remains synchronous as it just sends data to the client.
impl actix::Handler<crate::types::ws_event::WebsocketEvent> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: crate::types::ws_event::WebsocketEvent, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg) {
            Ok(json_msg) => {
                ctx.text(json_msg);
            }
            Err(e) => {
                std::eprintln!("Failed to serialize WebsocketEvent for session {}: {:?}", self.session_id, e);
            }
        }
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    // Note: Meaningful tests require significant mocking of Actix context,
    // AppState (with async Mutex), and potentially actor interactions.
    // These tests primarily check compilation and basic logic.

    #[test]
    fn test_placeholder_compilation() {
        // Basic assertion to ensure the test suite runs.
        // Using FQN for assert_eq! per guidelines.
        std::assert_eq!(1, 1);
    }

    // Example structure for a more involved test (requires test harnesses like `actix_rt::test`):
    // #[actix_rt::test]
    // async fn test_handle_user_input_async() {
    //     // 1. Setup mock AppState with async Mutex and initial SessionData
    //     //    (e.g., using std::sync::Arc<tokio::sync::Mutex<...>>)
    //     // 2. Create a mock WsSession actor environment (e.g., using TestActor)
    //     // 3. Create a UserInput request
    //     // 4. Call process_websocket_request_async directly or simulate message send using actor Addr
    //     // 5. Lock AppState after await and assert changes (e.g., history updated, status changed)
    // }
}

    // TODO: Add specific integration tests for process_websocket_request_async,
    // particularly covering the state transition from SessionStatus::AwaitingInput
    // to SessionStatus::Pending upon receiving a WebsocketRequest::UserInput.
    // This requires significant mocking of Actix context and AppState with async Mutex.