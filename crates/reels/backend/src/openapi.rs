use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health_check,
        // AgentLoop API endpoints
        agentloop::handlers::start_research::start_research,
        agentloop::handlers::get_status::get_status,
        agentloop::handlers::post_message::post_message,
        agentloop::handlers::terminate_session::terminate_session,
        agentloop::handlers::conversation_stream::conversation_stream,
        agentloop::handlers::get_session_state::get_session_state,
        agentloop::handlers::load_session_state::load_session_state,
   ),
   components(
       schemas(
            // AgentLoop schemas
            agentloop::types::research_request::ResearchRequest,
            agentloop::types::status_response::StatusResponse,
            agentloop::types::message::Message,
            agentloop::types::ws_request::WebsocketRequest,
            agentloop::types::ws_event::WebsocketEvent,
            agentloop::types::session_status::SessionStatus,
            agentloop::types::sender::Sender,
            agentloop::types::context_evaluator_feedback::ContextEvaluatorFeedback,
            agentloop::types::session_data::SessionData,
            agentloop::types::load_session_request::LoadSessionRequest,
            agentloop::types::conversation_entry::ConversationEntry,
      )
   ),
   tags(
        (name = "Health", description = "Application health status"),
        (name = "Loupe", description = "AgentLoop Session Management API"),
        (name = "Research", description = "Endpoints for initiating research tasks"),
        (name = "Session", description = "Endpoints for managing session lifecycle and interaction"),
        (name = "Session Management", description = "Session state management endpoints"),
    ),
    info(
        title = "Reels API",
        version = "1.0.0",
        description = "API documentation for Reels API.",
        contact(
            name = "Support",
            url = "https://reels.ai",
            email = "support@reels.ai",
        ),
        license(
            name = "MIT",
            identifier = "MIT"
        )
    ),
    servers(
        (url = "/", description = "API Root")
    )
)]
pub struct ApiDoc;
