# Function Signatures and Definitions

## crate::agentloop::types::websocket_request::WebsocketRequest

```rust
//! WebsocketRequest definition
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketRequest {
    pub id: String,
    pub payload: String,
    pub timestamp: u64,
}
```

## crate::agentloop::types::websocket_event::WebsocketEvent

```rust
//! WebsocketEvent definition
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum WebsocketEvent {
    Connected { session_id: String },
    Disconnected { session_id: String, reason: String },
    Message { session_id: String, data: String },
}
```

## crate::agentloop::session::manager::get_session_mut

```rust
pub fn get_session_mut<'a>(
    &mut self,
    session_id: &str,
) -> Option<&'a mut Session>
```

## crate::agentloop::session::manager::add_conversation_entry

```rust
pub fn add_conversation_entry(
    &mut self,
    session_id: &str,
    entry: ConversationEntry,
) -> Result<(), SessionError>
```

## crate::agentloop::websocket::manager::register_ws_recipient

```rust
pub fn register_ws_recipient(
    &mut self,
    session_id: &str,
    recipient: Recipient<WsMessage>,
)
```

## crate::agentloop::websocket::manager::broadcast_event

```rust
pub fn broadcast_event<E: Into<WsMessage> + Clone>(
    &self,
    event: E,
)
```
