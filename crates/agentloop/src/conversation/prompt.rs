//! Builds the prompt for the LLM based on conversation history and session data.
//!
//! This module defines the `build_llm_prompt` function, responsible for constructing
//! the message list or prompt string sent to the language model, considering the
//! full conversation flow, system messages, and the current research goal.
//! Adheres to project guidelines (FQN, one item per file).

use crate::state::app_state::AppState;
// Use fully qualified paths
use crate::types::session_data::SessionData;
use crate::types::message::Message; // Assuming Message has user/assistant/system methods
// For dynamic tool listing

/// Builds the list of messages for the LLM prompt.
///
/// Constructs the prompt ensuring the full conversation history, system message,
/// and the latest user request (potentially highlighting the transition from the
/// last agent answer) are included appropriately.
///
/// # Arguments
///
/// * `session_data` - The current session data containing history, goal, etc.
///
/// # Returns
///
/// * `Ok(Vec<Message>)` - A vector of messages ready for the LLM.
/// * `Err(String)` - If prompt construction fails (e.g., no history).
pub fn build_llm_prompt(
    session_data: &SessionData,
    app_state: &AppState
) -> Result<Vec<Message>, String> {
    let mut messages: Vec<Message> = Vec::new();

    // 1. Add System Message (if present)
    if let Some(ref sys_msg) = session_data.system_message {
        messages.push(Message::system(sys_msg.clone()));
    } else {
        // Default system message if none provided in session - now with dynamic tool listing
        let mut default_system_prompt_content = std::string::String::from(
            "You are a helpful research assistant operating in two modes: Conversation and Deep Research.\n\
            **Conversation Mode:** Engage in standard dialogue and answer directly.\n\
            **Deep Research Mode:** You have access to the following tools (built-in and external):\n"
        );

        // Dynamically add tool definitions from session_data.tool_definitions
        // This assumes session_data.tool_definitions: &std::vec::Vec<crate::types::tool_definition::ToolDefinition> exists
        if let Some(tool_schemas) = &*app_state.tool_schemas {
            let tool_schemas_string = format!("<TOOL_JSON_SCHEMA>{}</TOOL_JSON_SCHEMA>\n", tool_schemas.schema.to_string());
            default_system_prompt_content.push_str(&tool_schemas_string);
        }

        default_system_prompt_content.push_str(
            "To call a tool, return exactly JSON: { \"tool\": \"tool_name\", \"parameters\": { ... } }.\n\
            After calling a tool, await its results before proceeding.\n\
            You can choose multiple tools in a single answer and often that's the preferredn way to increase the speed of the research.\n\
            Only set `is_final: true` for the final answer when you are confident no further tools are needed.\n\
            Provide reasoning for all tool choices and responses."
        );
        messages.push(Message::system(default_system_prompt_content));
    }

    // 2. Add Research Goal (if present) - Frame it as context or initial instruction
    if let Some(ref goal) = session_data.research_goal {
         // Add goal as a system message for context, or adapt based on LLM preference
         messages.push(Message::system(format!("Current Research Goal: {}", goal)));
    }

    // 3. Add Conversation History
    if session_data.history.is_empty() {
        return Err(String::from("Cannot build prompt: Conversation history is empty."));
    }

    for entry in &session_data.history {
        let message = match entry.sender {
            crate::types::sender::Sender::User => Message::user(entry.message.clone()),
            // Combine agent answer and reasoning/tool calls for the prompt
            crate::types::sender::Sender::Agent => {
                let mut agent_content = entry.message.clone();
                // Optionally append simplified tool usage info if needed for context
                if !entry.tools.is_empty() {
                     agent_content.push_str("\\n\\n*Agent proposed actions:*");
                     for tool in &entry.tools {
                         // Basic representation; could be more detailed
                         // Format tool parameters more clearly if possible, or just name
                         agent_content.push_str(&format!("\\n- tool execution {}", serde_json::to_string(&tool.parameters).unwrap()));
                     }
                }
                Message::assistant(agent_content)
            },
            // Include Tool results as context for the agent
            crate::types::sender::Sender::Tool => {
                 // Prefix tool results clearly
                 Message::system(format!("*Tool Result:*\\n{}", entry.message.clone()))
            },
            // System messages within history might represent summaries or context
            crate::types::sender::Sender::System => Message::system(entry.message.clone()),
            // Map Assistant role if used interchangeably with Agent
            crate::types::sender::Sender::Assistant => Message::assistant(entry.message.clone()),
        };
        messages.push(message);
    }

    // 4. (Optional) Add explicit instruction if needed
    // Depending on the LLM, explicitly asking it to address the *last* user message
    // might be beneficial after presenting the history.
    // messages.push(Message::system("Address the latest user message based on the provided history and context.".to_string()));


    Ok(messages)
}

#[cfg(test)]
mod tests {
    use crate::types::{
        SessionData, SessionConfig, SessionStatus, ConversationEntry, Sender, Timestamp,
        ToolChoice, ToolParameters, Message, tool_definition::ToolDefinition
    }; // FQN needed here
    use std::time::Duration; // For SessionConfig duration
    use chrono::Utc; // For timestamps
    use serde_json::json; // For creating serde_json::Value instances with json! macro

    // Helper to create default SessionData for testing
    fn create_test_session_data(history: Vec<ConversationEntry>, goal: Option<String>, tool_defs: std::vec::Vec<ToolDefinition>) -> SessionData {
        // This assumes SessionData struct has been updated to include `tool_definitions`
        SessionData {
            session_id: "test-session".to_string(),
            research_goal: goal,
            status: SessionStatus::Running { progress: None },
            config: SessionConfig {
                time_limit: Duration::from_secs(300),
                token_threshold: 1000,
                preserve_exchanges: 5,
                initial_instruction: None,
                compaction_policy: Default::default(),
                evaluation_policy: Default::default(),
            },
            history,
            context: vec![],
            created_at: Utc::now(),
            last_activity_timestamp: Utc::now(),
            system_message: Some("Test System Prompt".to_string()),
            messages: vec![], // messages field is not directly used by build_llm_prompt currently
            tool_definitions: tool_defs, // Ensure field name matches struct
        }
    }

    #[test]
    fn test_build_prompt_with_history_and_goal() {
        let history = vec![
            ConversationEntry {
                sender: Sender::User,
                message: "What is Bounti.ai?".to_string(),
                timestamp: Utc::now(),
                tools: vec![],
            },
            ConversationEntry {
                sender: Sender::Agent,
                message: "Bounti.ai helps with GTM strategy.".to_string(),
                timestamp: Utc::now(),
                tools: vec![],
            },
            ConversationEntry {
                sender: Sender::User,
                message: "Who is the CEO?".to_string(),
                timestamp: Utc::now(),
                tools: vec![],
            },
        ];
        let session_data = create_test_session_data(history, Some("Find Bounti.ai CEO".to_string()), vec![]);

        let result = super::build_llm_prompt(&session_data);
        assert!(result.is_ok());
        let messages = result.unwrap();

        assert_eq!(messages.len(), 3 + 2); // System + Goal + History(3)
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "Test System Prompt");
        assert_eq!(messages[1].role, "system");
        assert!(messages[1].content.contains("Find Bounti.ai CEO"));
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[2].content, "What is Bounti.ai?");
        assert_eq!(messages[3].role, "assistant");
        assert_eq!(messages[3].content, "Bounti.ai helps with GTM strategy.");
        assert_eq!(messages[4].role, "user");
        assert_eq!(messages[4].content, "Who is the CEO?");
    }

    #[test]
    fn test_build_prompt_empty_history_fails() {
        let session_data = create_test_session_data(vec![], None, vec![]);
        let result = super::build_llm_prompt(&session_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot build prompt: Conversation history is empty.");
    }

    #[test]
    fn test_build_prompt_includes_tool_results() {
         let history = vec![
             ConversationEntry { sender: Sender::User, message: "Search for Bounti".to_string(), timestamp: Utc::now(), tools: vec![] },
             ConversationEntry {
                 sender: Sender::Agent,
                 message: "Searching...".to_string(),
                 timestamp: Utc::now(),
                 tools: vec![ToolChoice {
                     name: "search".to_string(),
                     parameters: ToolParameters::Search { query: "Bounti".to_string() }
                 }]
             },
             ConversationEntry {
                 sender: Sender::Tool,
                 message: "Search Result: Bounti is mentioned on website X.".to_string(),
                 timestamp: Utc::now(),
                 tools: vec![] },
             ConversationEntry {
                 sender: Sender::User,
                 message: "Thanks".to_string(),
                 timestamp: Utc::now(),
                 tools: vec![] },
         ];
         let session_data = create_test_session_data(history, None, vec![]);
         let result = super::build_llm_prompt(&session_data);
         assert!(result.is_ok());
         let messages = result.unwrap();

         assert_eq!(messages.len(), 4 + 1); // System + History(4)
         assert_eq!(messages[3].role, "system"); // Tool result shown as system message
         assert!(messages[3].content.contains("Tool Result:"));
         assert!(messages[3].content.contains("Bounti is mentioned"));
         assert_eq!(messages[4].role, "user");
    }

    #[test]
    fn test_default_system_prompt_dynamic_tools() {
        // Create session data without a specific system message to test the default
        let mut session_data = create_test_session_data(
            vec![ConversationEntry { sender: Sender::User, message: "Hi".to_string(), timestamp: Utc::now(), tools: vec![] }],
            None,
            vec![
                ToolDefinition {
                    name: "test_tool_1".to_string(),
                    description: "Description for test tool 1".to_string(),
                    parameters_json_schema: json!({
                        "type": "object",
                        "properties": {
                            "param1": { "type": "string" }
                        }
                    }),
                },
                ToolDefinition {
                    name: "test_tool_2".to_string(),
                    description: "Description for test tool 2".to_string(),
                    parameters_json_schema: json!({
                        "type": "object",
                        "properties": {
                            "paramA": { "type": "integer" },
                            "paramB": { "type": "boolean" }
                        }
                    }),
                },
            ]
        );
        session_data.system_message = None; // Ensure default is used

        let result = super::build_llm_prompt(&session_data);
        assert!(result.is_ok());
        let messages = result.unwrap();
        println!("System prompt: {}", messages[0].content);

        assert!(!messages.is_empty());
        assert_eq!(messages[0].role, "system");
        // Check if keywords related to the modes are present in the default prompt
        assert!(messages[0].content.contains("Conversation Mode"));
        assert!(messages[0].content.contains("Deep Research Mode"));
        
        // Check for dynamic tool descriptions
        assert!(messages[0].content.contains("- test_tool_1: Description for test tool 1 Parameters: {\"type\":\"object\",\"properties\":{\"param1\":{\"type\":\"string\"}}}"));
        assert!(messages[0].content.contains("- test_tool_2: Description for test tool 2 Parameters: {\"type\":\"object\",\"properties\":{\"paramA\":{\"type\":\"integer\"},\"paramB\":{\"type\":\"boolean\"}}}"));
        
        // Ensure old hardcoded tool descriptions are NOT present
        assert!(!messages[0].content.contains("- search: { \"tool_name\": \"Search\""));
        assert!(!messages[0].content.contains("- browse: { \"tool_name\": \"Browse\""));
        assert!(!messages[0].content.contains("- save_context: { \"tool_name\": \"SaveContext\""));
        assert!(!messages[0].content.contains("- <external tools>:"));

        // Check for tool usage instructions
        assert!(messages[0].content.contains("To call a tool, return exactly JSON:"));
    }

    #[test]
    fn test_default_system_prompt_no_tools() {
        // Create session data with no tool definitions
        let mut session_data = create_test_session_data(
            vec![ConversationEntry { sender: Sender::User, message: "Hi".to_string(), timestamp: Utc::now(), tools: vec![] }],
            None,
            vec![] // Empty tool definitions
        );
        session_data.system_message = None; // Ensure default is used

        let result = super::build_llm_prompt(&session_data);
        assert!(result.is_ok());
        let messages = result.unwrap();

        assert!(!messages.is_empty());
        assert_eq!(messages[0].role, "system");
        assert!(messages[0].content.contains("Conversation Mode"));
        assert!(messages[0].content.contains("Deep Research Mode"));
        assert!(messages[0].content.contains("(No tools are currently configured for your use in this session.)"));
        assert!(messages[0].content.contains("To call a tool, return exactly JSON:"));
    }
}
