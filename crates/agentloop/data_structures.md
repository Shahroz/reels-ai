# AgentLoop Data Structures

Below is the consolidated list of Rust data structures needed to implement the AgentLoop API. Each item should live in its own `*.rs` file under `src/types/` or `src/models/` following one-item-per-file guidelines. All structures derive `Debug`, `Clone`, `Serialize`, and `Deserialize` where appropriate.

---

## Common Type Aliases
- `type SessionId = uuid::Uuid;`
- `type Timestamp = chrono::DateTime<Utc>;`

---

## 1. Session Management

File: `src/types/session.rs`

```rust
/// Current status of a research session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Pending,
    InProgress,
    Completed,
    TimedOut,
    Terminated,
}
```

---

## 2. API Request/Response

File: `src/types/research_request.rs`

```rust
/// Payload to start a new research session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRequest {
    pub instruction: String,
}
```

File: `src/types/research_response.rs`

```rust
/// Response after creating a research session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResponse {
    pub session_id: SessionId,
    pub status: SessionStatus,
}
```

File: `src/types/termination_request.rs`

```rust
/// Request to force-terminate a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationRequest {
    pub session_id: SessionId,
}
```

File: `src/types/status_response.rs`

```rust
/// Status query response, including optional remaining time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub session_id: SessionId,
    pub status: SessionStatus,
    pub time_remaining: Option<std::time::Duration>,
}
```

---

## 3. Configuration & Policies

File: `src/types/session_config.rs`

```rust
/// Configuration for a research session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub time_limit: std::time::Duration,
    pub token_threshold: usize,
    pub preserve_exchanges: usize,
}
```

File: `src/types/compaction_policy.rs`

```rust
/// How to compact conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionPolicy {
    pub keep_last: usize,
    pub summary_length: usize,
}
```

---

## 4. Conversation & Context

File: `src/types/sender.rs`

```rust
/// Who sent a message in the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Sender {
    User,
    Agent,
    Tool,
}
```

File: `src/types/conversation_entry.rs`

```rust
/// A single entry in the conversation stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub sender: Sender,
    pub message: String,
    pub timestamp: Timestamp,
    pub tools: Vec<ToolChoice>,
}
```

File: `src/types/context_entry.rs`

```rust
/// A piece of persisted context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub content: String,
    pub source: Option<String>,
    pub timestamp: Timestamp,
}
```

---

## 5. Tool Framework

File: `src/types/tool_choice.rs`

```rust
/// A tool selected by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoice {
    pub name: String,
    pub parameters: serde_json::Value,
}
```

File: `src/types/tool_result.rs`

```rust
/// Result returned by a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub output: serde_json::Value,
    pub success: bool,
    pub timestamp: Timestamp,
}
```

---

## 6. Background Evaluator Feedback

File: `crates/agentloop/src/types/context_evaluator_feedback.rs`

```rust
/// Feedback from the background context evaluator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEvaluatorFeedback {
    pub missing_information: Option<String>,
    pub next_steps: Vec<String>,
    pub is_sufficient: bool,
}
```

---

## 7. WebSocket / Event Streaming

File: `src/types/websocket_event.rs`

```rust
/// Events streamed to clients over WebSocket/SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketEvent {
    ReasoningUpdate(String),
    ToolExecution(ToolResult),
    ContextFeedback(ContextEvaluatorFeedback),
}
```

---

## 8. Error Handling

File: `src/types/agent_error.rs`

```rust
/// Common error type for AgentLoop operations.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),
    #[error("Timeout reached")]
    Timeout,
    #[error("Unexpected error: {0}")]
    Other(String),
}
```

---

## 9. WebSocket Request (Input) Messages

File: `src/types/ws_request.rs`

```rust
/// Messages sent by clients over WebSocket to the AgentLoop service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketRequest {
    /// Start a new research session.
    Research { instruction: String },
    /// Send a user or tool message within an existing session.
    Message { session_id: SessionId, content: String },
    /// Query session status.
    StatusRequest { session_id: SessionId },
    /// Request forced termination of a session.
    Terminate { session_id: SessionId },
}
```

## 10. WebSocket Event (Output) Messages

File: `src/types/ws_event.rs`

```rust
/// Messages sent by the AgentLoop service to clients over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebsocketEvent {
    /// Streamed reasoning or status updates.
    ReasoningUpdate(String),
    /// Result of a tool execution.
    ToolExecution(ToolResult),
    /// Feedback from the background evaluator.
    ContextFeedback(ContextEvaluatorFeedback),
    /// Current session status and optional time remaining.
    StatusUpdate(StatusResponse),
    /// Indicates session completion or timeout.
    SessionTerminated { session_id: SessionId, reason: Option<String> },
}
```


---

## 11. Session Data

File: `src/types/session_data.rs`

```rust
/// In-memory representation of a research session.
#[derive(Debug, Clone)]
pub struct SessionData {
    /// Current status of the session.
    pub status: SessionStatus,
    /// Configuration used for the session.
    pub config: SessionConfig,
    /// Ordered conversation entries between user, agent, and tools.
    pub history: Vec<ConversationEntry>,
    /// Persisted context entries collected during the session.
    pub context: Vec<ContextEntry>,
    /// Timestamp when the session was created.
    pub created_at: Timestamp,
}
```

---

## 12. Application State

File: `src/types/app_state.rs`

```rust
use std::collections::HashMap;
use crate::types::{SessionId, SessionData, WebsocketEvent};
use actix::Recipient;

/// In-memory application state holding active sessions and WebSocket connections.
#[derive(Debug, Default)]
pub struct AppState {
    /// Map of active session IDs to their session data.
    pub sessions: HashMap<SessionId, SessionData>,
    /// Map of session IDs to WebSocket event recipients.
    pub ws_connections: HashMap<SessionId, Vec<Recipient<WebsocketEvent>>>,
}
```
