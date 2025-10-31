# AgentLoop API Brief v2

## 1. Introduction
The AgentLoop service provides a Rust (Actix Web) HTTP API that orchestrates iterative research sessions with large language models (LLMs). It manages tool selection, execution, conversation accumulation, and background context evaluation, terminating when objectives are met or a time limit is reached.

## 2. Core Capabilities

1. Accept Research Instructions  
   - Endpoint: POST `/research`  
   - Payload: User instruction describing the research goal.  
   - Response: Session ID and initial status.

2. Interactive LLM Conversation  
   - Orchestrates back-and-forth with configured LLMs (e.g., Gemini Flash 2.5, OpenAI).  
   - Presents user instruction and dynamically chosen tools at each step.  
   - Streams LLM responses via WebSocket or Server-Sent Events (SSE).

3. Tool Framework  
   - **Available Tools:**  
     - Context Saver: Persist valuable insights to final context.  
     - API Clients: Modules under `api_clients` for search (Serper), browse (Zyte), sitemap processing, etc.  
     - Custom Executors: Future tool integration (e.g., data analysis scripts).  
   - Agents select one or multiple tools per turn.  
   - Tools execute asynchronously; results feed back into the conversation.

4. Background Context Evaluator  
   - Monitors the accumulated context in real time.  
   - Identifies missing information or suggests next steps.  
   - Runs at configurable intervals (default: every 30 s).

5. Termination Criteria  
   - Explicit signal from the background process when context suffices.  
   - Automatic timeout after a configurable duration (default: 5 minutes).  
   - Endpoint: GET `/research/{session_id}/status` to query current state.

6. Conversation Management  
   - Retains full conversation history per session.  
   - If history exceeds token limit (100 000 tokens), compaction triggers:  
     - Keep last N exchanges (configurable, default: 10).  
     - Summarize earlier exchanges and discard detailed tool outputs.  
   - Summaries stored alongside history for transparency.

## 3. Configuration Options
- **Time Limit:** Adjustable per session via query parameter or environment default.  
- **Token Threshold:** Maximum conversation tokens before compaction.  
- **Compaction Policy:** Number of preserved exchanges and summary length.  
- **Model Preferences:** Priority order of LLMs and model-specific parameters.

## 4. API Endpoints Summary

| Method | Path                                | Description                              |
| ------ | ----------------------------------- | ---------------------------------------- |
| POST   | /research                           | Start a new research session.            |
| GET    | /research/{session_id}/status       | Get session status and progress.         |
| POST   | /research/{session_id}/message      | Send user or tool messages to the session. |
| GET    | /research/{session_id}/conversation | Stream conversation updates (SSE/WebSocket). |
| POST   | /research/{session_id}/terminate    | Force-terminate the session.             |

## 5. Clarification Questions
1. What data format and schema should the final context use, and how long must it be persisted? 
    Answer: for now lets say that the final answer is markdown format the intermediate results are not important
2. Which LLMs and model parameters (temperature, max tokens) should be configurable via the API?  
    Answer: use defaults
3. How are tools registered and exposed to the agent at runtime? Will there be a plugin mechanism?  
    Answer: the tools are in the api_clients crate: browse and search
4. What specific criteria or heuristics should the background evaluator use to declare “sufficient context”?
    Answer: It should analyze the context if it provides enought sources to answer the user in a very detailed fashion
5. Should the compaction summaries include metadata (timestamps, tool outputs) or just distilled text?  
    Answer: the compaction summaries should remove the tool outputs - everything valuable should be stored in the final context
6. Are there authentication/authorization requirements for API clients invoking research sessions?  
    Answer: Bearer token
7. Do we need endpoints to retrieve or delete stored context/history for auditing or GDPR compliance?  
    Answer: No everything is removed at the end
8. How should errors and tool execution failures be reported back to the user?  
    Answer: Every LLM answer should have 2 parts: {"user_answer": "I'm doing this and that. Let me search for X and visit Y", "actions": [a,b,c]} - where a,b,c are chosen actions by the agent
9. What logging and monitoring features are required for both the Web API and background evaluator?  
    Answer: standard loging
10. Is horizontal scaling (multiple worker instances) or distributed state management a requirement?  
    Answer: No this is rather simple function based endpoint


Also what's the most important we need to expose it via websockets:
- The websockets share a stream of agent reasoning, tool choices, and outputs
- The user can also interrupt the process or provide more details while it is executed - the additional instructions are added to the previous ones

UI:
- we need a simple UI where we expose a chat interface
- the research is like a nested app within the chat with its own progress
- not every user question starts the research the users can also comment on the results
- and only if it is evident that they want to do the research we spawn the research as a mini-app on the side

For now let's keep everything in memory no database

This tools should also be available as CLI chat

The UI should be written in dioxus and embedded in the server app