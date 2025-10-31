# Extending AgentLoop with Custom Tools

## 1. Overview

This document outlines how to extend AgentLoop with custom tools when AgentLoop is embedded as a service within a larger application (e.g., an Actix Web application). AgentLoop now features a unified tool system where both internal and externally-provided tools are registered and managed consistently. This allows a host application to seamlessly inject its own tool definitions and handlers into AgentLoop.

The core idea is that the host application:
1.  Defines its custom tool functionalities as Rust functions.
2.  Provides metadata for these tools (name, description, parameter schema).
3.  Passes these definitions and handlers to AgentLoop during its initialization.

AgentLoop then incorporates these custom tools into its LLM prompts and dispatches calls to them as directed by the language model.

## 2. Core Components for Tool Extension

To extend AgentLoop with custom tools, you need to understand and use the following core types:

### a. `ToolDefinition`

(`crate::types::tool_definition::ToolDefinition`)

This struct defines the metadata for a tool that the agent can use.

```rust
// From crate::types::tool_definition::ToolDefinition
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, utoipa::ToSchema)]
pub struct ToolDefinition {
    pub name: String,                     // The unique name of the tool (e.g., "get_user_profile").
    pub description: String,              // A clear description of what the tool does, for the LLM.
    pub parameters_json_schema: Value,    // A serde_json::Value representing the JSON schema
                                          // for the parameters this tool accepts.
}
```

*   `name`: A unique identifier for your tool. The LLM will use this name to request the tool.
*   `description`: A detailed explanation of the tool's purpose, capabilities, and when it should be used. This is crucial for the LLM to understand how to use your tool effectively.
*   `parameters_json_schema`: A `serde_json::Value` that holds the [JSON Schema](https://json-schema.org/) for the parameters your tool expects. The LLM will use this schema to understand the structure, types, and constraints of the parameters it needs to provide when calling the tool. AgentLoop will include this schema (as a string) in the system prompt.

Example `parameters_json_schema`:
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "The ID of the user to fetch."
    },
    "include_details": {
      "type": "boolean",
      "description": "Whether to include extended profile details."
    }
  },
  "required": ["user_id"]
}
```
You would typically create this `serde_json::Value` using `serde_json::json!`:
```rust
use agentloop::types::tool_definition::ToolDefinition; // Adjust path as needed
let schema = serde_json::json!({
    "type": "object",
    "properties": {
        "user_id": { "type": "string", "description": "The ID of the user to fetch." },
        "include_details": { "type": "boolean", "description": "Whether to include extended profile details." }
    },
    "required": ["user_id"]
});
let my_tool_def = ToolDefinition {
    name: "get_user_profile".to_string(),
    description: "Fetches a user's profile information by their ID.".to_string(),
    parameters_json_schema: schema,
};
```

### b. `ToolHandler`

(`crate::tools::tool_handler::ToolHandler`)

This is a type alias for the functions that implement the logic of your custom tools.

```rust
// From crate::tools::tool_handler::ToolHandler
pub type ToolHandler = fn(
    params_json: serde_json::Value, // Raw JSON parameters from the LLM
    app_state: actix_web::web::Data<crate::state::app_state::AppState>,
    session_id: crate::types::session_id::SessionId,
) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
```

Key points:
*   It's an `async` function (returning a pinned, boxed `Future`).
*   `params_json: serde_json::Value`: This is the crucial part. The handler receives the parameters as a raw `serde_json::Value` exactly as provided by the LLM in its `ToolChoice`.
*   **Your handler is responsible for deserializing and validating this `params_json`** against the schema you defined in `ToolDefinition`. You can use `serde_json::from_value` for this.
*   It receives the shared `AppState` (wrapped in `actix_web::web::Data`) and the current `SessionId`.
*   It returns a `Result<String, String>`, representing the tool's output (often a stringified JSON or plain text for the LLM) or an error message.

## 3. Host Application: Integrating Custom Tools

The host application (your main Actix Web server, for example) needs to perform the following steps to integrate its custom tools:

### Step 1: Define Custom Tool Handler Functions

For each custom tool, write an `async` function that implements its logic. This function should then be wrapped to match the `ToolHandler` signature.

```rust
use std::pin::Pin;
use std::future::Future;
use actix_web::web::Data;
use serde::Deserialize; // For deserializing parameters

// Assuming these types are accessible from your host application,
// potentially via re-exports from the agentloop crate or by adding agentloop as a dependency.
use agentloop::state::app_state::AppState; // Adjust path as needed
use agentloop::types::session_id::SessionId; // Adjust path as needed

// Define a struct for your tool's parameters
#[derive(Deserialize, Debug)]
struct MyEchoToolParams {
    message: String,
    prefix: Option<String>,
}

// Example: An "echo" tool handler
async fn handle_my_custom_echo_tool(
    params_json: serde_json::Value,
    app_state: Data<AppState>, // Access to shared AgentLoop state
    session_id: SessionId
) -> Result<String, String> {
    // Log access if needed
    // log::info!("Custom echo tool called for session: {}", session_id);
    // log::info!("AppState example field: {}", app_state.config.server_address);

    // Deserialize and validate parameters
    let params: MyEchoToolParams = match serde_json::from_value(params_json.clone()) {
        Ok(p) => p,
        Err(e) => {
            return Err(format!("Invalid parameters for echo tool: {}. Expected JSON: {}. Received: {}", 
                e, 
                r#"{"message": "string", "prefix": "optional_string"}"#, // Example schema for error message
                params_json
            ));
        }
    };

    let prefix_str = params.prefix.unwrap_or_default();
    Ok(format!("{}Echo from custom tool: {}", prefix_str, params.message))
}

// Wrapper to match the exact fn pointer type required by ToolHandler
// Ensure agentloop::tools::tool_handler::ToolHandler is accessible
use agentloop::tools::tool_handler::ToolHandler; // Adjust path as needed

fn my_custom_echo_tool_wrapper(
    params_json: serde_json::Value,
    state: Data<AppState>,
    session_id: SessionId,
) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> {
    Box::pin(handle_my_custom_echo_tool(params_json, state, session_id))
}
```

### Step 2: Create `ToolDefinition` Instances

For each custom tool, create a `ToolDefinition` struct.

```rust
// Assuming ToolDefinition is accessible (e.g., agentloop::types::tool_definition::ToolDefinition)
use agentloop::types::tool_definition::ToolDefinition; // Adjust path as needed

fn get_my_custom_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "custom_echo".to_string(),
            description: "A custom tool that echoes back a message, optionally with a prefix. Demonstrates external tool integration.".to_string(),
            parameters_json_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo."
                    },
                    "prefix": {
                        "type": "string",
                        "description": "An optional prefix to add to the echoed message."
                    }
                },
                "required": ["message"]
            }),
        },
        // ... other custom tool definitions
    ]
}
```

### Step 3: Prepare Collections for AgentLoop

Collect all your custom tool definitions into a `Vec<ToolDefinition>` and your tool handlers into a `HashMap<String, ToolHandler>`. The keys in the HashMap must match the `name` field in your `ToolDefinition`s.

```rust
use std::collections::HashMap;
// Assuming ToolHandler is accessible (e.g., agentloop::tools::tool_handler::ToolHandler)
use agentloop::tools::tool_handler::ToolHandler; // Adjust path as needed

fn get_my_custom_tool_handlers() -> HashMap<String, ToolHandler> {
    let mut handlers = HashMap::new();
    handlers.insert(
        "custom_echo".to_string(),
        my_custom_echo_tool_wrapper as ToolHandler
    );
    // ... add other custom tool handlers
    handlers
}
```

### Step 4: Integrate with AgentLoop via `setup_agentloop_core`

AgentLoop exposes a public setup function, `setup_agentloop_core` (typically found in `crates/agentloop/src/setup.rs` and re-exported from `crates/agentloop/src/lib.rs`). Your host application calls this function during its initialization phase to provide its configuration, custom tool definitions, and custom tool handlers.

```rust
// In your host application's main.rs or a setup module:

// Assume agentloop crate is a dependency and its items are accessible.
// For example:
// use agentloop;
// use agentloop::config::app_config::AppConfig;
// use agentloop::types::tool_definition::ToolDefinition;
// use agentloop::tools::tool_handler::ToolHandler;
// use std::collections::HashMap;

/*
async fn main() -> std::io::Result<()> {
    // 1. Load or define AgentLoop's configuration (AppConfig)
    //    This typically comes from a config file or environment variables.
    //    let agentloop_config: agentloop::config::app_config::AppConfig = load_my_agentloop_config_somehow();

    // 2. Get your custom tool definitions and handlers
    //    let my_custom_tool_definitions: Vec<agentloop::types::tool_definition::ToolDefinition> = get_my_custom_tool_definitions();
    //    let my_custom_tool_handlers: HashMap<String, agentloop::tools::tool_handler::ToolHandler> = get_my_custom_tool_handlers();

    // 3. Call AgentLoop's setup function
    //    let agentloop_app_data = match agentloop::setup::setup_agentloop_core( // Adjust path if not re-exported directly
    //        agentloop_config,
    //        my_custom_tool_definitions,
    //        my_custom_tool_handlers
    //    ).await {
    //        Ok(data) => data,
    //        Err(e) => {
    //            eprintln!("Failed to setup AgentLoop core: {}", e);
    //            // Handle error appropriately, e.g., exit
    //            return Err(std::io::Error::new(std::io::ErrorKind::Other, "AgentLoop setup failed"));
    //        }
    //    };

    // 4. Initialize Actix Web server (or your application framework)
    //    actix_web::HttpServer::new(move || {
    //        actix_web::App::new()
    //            // Share AgentLoop's AppState with Actix handlers
    //            .app_data(agentloop_app_data.clone())
    //            // Configure AgentLoop's routes (e.g., /api/agentloop/...)
    //            .configure(agentloop::config::routes::configure_routes) // Adjust path as needed
    //            // ... add your host application's other services, routes, and middleware
    //    })
    //    .bind(("127.0.0.1", 8080))?
    //    .run()
    //    .await
}
*/
```
*(Note: The above `main` function is a conceptual example. Adapt it to your application's structure, error handling, and specific paths to AgentLoop items.)*

The `setup_agentloop_core` function will initialize AgentLoop's `AppState`, merging your external tools with any built-in tools. The returned `actix_web::web::Data<agentloop::state::app_state::AppState>` can then be shared throughout your Actix Web application.

## 4. How AgentLoop Uses Custom Tools

Once integrated, AgentLoop handles your custom tools as follows:

### a. Prompt Generation

The `build_llm_prompt` function in AgentLoop (see `crates/agentloop/src/conversation/prompt.rs`) automatically gathers all registered tool definitions (both internal and your custom external ones) from `AppState`. It then includes their:
*   `name`
*   `description`
*   `parameters_json_schema` (stringified)

in the system prompt sent to the LLM. This enables the LLM to know about your custom tools, understand what they do, and how to call them with the correct parameter structure.

### b. Tool Dispatch

When the LLM decides to use a tool, it responds with a `ToolChoice` containing the tool's `name` and `parameters` (as a `serde_json::Value`).

AgentLoop's `dispatch_tools` function (see `crates/agentloop/src/tools/dispatch_tools.rs`):
1.  Looks up the tool `name` in its merged map of tool handlers (which includes your custom handlers stored in `AppState.merged_tool_handlers`).
2.  If found, it calls the corresponding `ToolHandler` function.
3.  Crucially, it passes the `parameters` field from `ToolChoice` (which is a `serde_json::Value`) directly to your handler function as the `params_json` argument.
4.  Your handler, as described earlier, is then responsible for parsing and using these JSON parameters.

This unified approach ensures that custom tools are first-class citizens within AgentLoop, discoverable by the LLM and callable through the same mechanism as built-in tools.
