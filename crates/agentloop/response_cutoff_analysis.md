# Analysis of LLM Response Cutoff Issue

**Reported Issue:** Users are experiencing instances where LLM responses are cut off, particularly after a colon. The content following the colon is often missing or severely truncated.

**Date of Analysis:** 2025-05-23

## Investigation Path and Key Code Areas:

The investigation focuses on the flow of information from prompt construction to response delivery, identifying potential points where truncation could occur.

1.  **Prompt Construction (`crates/agentloop/src/conversation/prompt.rs`)**
    *   The `build_llm_prompt` function constructs the messages sent to the LLM.
    *   The default system message is quite detailed and instructs the agent on its behavior, including providing reasoning and proposing actions.
    *   The way conversation history and tool results are formatted (e.g., `*Agent proposed actions:*`, `*Tool Result:*`) might also influence the LLM's output style.
    *   **Potential Issue:** If the LLM tries to mimic a structured output with headings or key-value pairs (e.g., "Reasoning: <text>", "Answer: <text>") and hits an internal generation limit for one of these "values" after printing the "key:", truncation could occur.

2.  **LLM Configuration (`crates/agentloop/src/config/llm_config.rs`)**
    *   The `LlmConfig` struct defines `max_tokens` (default `None`). If this is set to a low value elsewhere or if the underlying `llm_typed` library or the model itself has a restrictive default when `None` is passed, it could cause truncation.
    *   **Potential Issue:** A low `max_tokens` limit would typically truncate the entire response (or the end of the JSON structure). However, if the LLM prioritizes completing the JSON structure over content length within fields, it might truncate string values to fit.

3.  **LLM Interaction & Response Parsing (`crates/agentloop/src/conversation/stream.rs`)**
    *   The `conversation_event_stream` function calls `llm::llm_typed_unified::llm_typed` expecting a JSON response parsable into `crate::types::llm_agent_response::LlmAgentResponse`.
    *   `OutputFormat::Json` is specified, meaning the LLM is instructed to return JSON.
    *   The full prompt string is logged before the call, which is good for debugging the input to the LLM.
    *   The received `LlmAgentResponse` is logged (first 100 chars of `user_answer` and `agent_reasoning`).
    *   **Potential Issue (Most Likely):** The LLM might generate a syntactically valid JSON object, but the string value for `user_answer` or `agent_reasoning` could be truncated *by the LLM itself* during its generation process. For instance, if the LLM generates:
        ```json
        {
          "agent_reasoning": "The user asked for details. I will provide them: First point is about...",
          "user_answer": "Here are the details: The first key aspect is", // <-- Truncated here by LLM
          "is_final": false,
          "actions": []
        }
        ```
        This JSON is valid, and `serde_json` would parse it successfully, resulting in `user_answer` being the truncated string. This aligns well with the "cutoff after a colon" symptom, as the LLM might be starting a new thought/section after a colon and then failing to complete it.

4.  **LLM Response Structure (`crates/agentloop/src/types/llm_agent_response.rs`)**
    *   `LlmAgentResponse` defines fields like `agent_reasoning`, `user_answer`, `title`, `is_final`, and `actions`.
    *   The `FewShotsOutput` examples guide the LLM. None of the current examples explicitly demonstrate a pattern that would encourage truncation after a colon. However, the LLM might infer such patterns from the broader interaction or the system prompt.
    *   **Potential Issue:** If the LLM struggles to fit a detailed explanation into the `agent_reasoning` or `user_answer` fields while also adhering to other constraints (like including actions or trying to be concise), it might truncate these string fields.

5.  **Response Processing (`crates/agentloop/src/evaluator/research_loop/process_llm_turn.rs`)**
    *   This function receives the `LlmAgentResponse` from `conversation_event_stream`.
    *   It logs the response again.
    *   It adds the `llm_response.user_answer` to history.
    *   It broadcasts `agent_reasoning` and `user_answer` via WebSockets.
    *   There's logic to clear `actions` if `is_final` is true, but this doesn't affect the content of `user_answer` or `agent_reasoning`.
    *   **Potential Issue:** Unlikely to be the source of truncation itself, as it primarily deals with the already (de)serialized response. However, logs here are crucial for confirming if the truncation is present when `process_llm_turn` receives the response.

6.  **WebSocket Handling (`crates/agentloop/src/websocket/handler.rs`)**
    *   Messages are serialized to JSON before being sent to the client.
    *   **Potential Issue:** Unlikely to cause content truncation within a string. Serialization errors typically result in the message not being sent or a malformed message, rather than a clean cutoff within a field's content.

## Hypotheses for the Cutoff Issue:

1.  **LLM-Generated Truncation within JSON String Values (Most Plausible):**
    *   The LLM generates a response that is structurally valid JSON, but the content of string fields (e.g., `user_answer`, `agent_reasoning`) is truncated *by the LLM* before it finishes generating that specific string. This could happen if the LLM hits an internal token limit for that part of the generation or if its attempt to produce structured output (e.g., "Heading: Content") leads to incomplete "Content".
    *   This would pass `serde_json` parsing without error, and the application would process the truncated string as if it were complete.

2.  **Overall LLM `max_tokens` Limit:**
    *   If the overall `max_tokens` for the LLM response is being hit, the LLM might abruptly stop generation. This could manifest as incomplete JSON (leading to a parsing error in `llm_typed`) or, if the LLM tries to complete the JSON structure, it might aggressively truncate string field values.

3.  **Prompt-Induced Behavior:**
    *   The detailed system prompt or the way history/tools are presented might be leading the LLM to adopt a specific output format that is prone to this kind of truncation when content is lengthy.

## Recommended Debugging Steps:

1.  **Inspect Logs:**
    *   Check the logs in `crates/agentloop/src/conversation/stream.rs` for the line:
        `println!("Typed LLM Response Received: user_answer='{}...', agent_reasoning='{}...'");`
        And more importantly, examine the full `response` object at this stage. Is the truncation already present in `response.user_answer` or `response.agent_reasoning` immediately after the `llm_typed` call returns?
    *   Check the logs in `crates/agentloop/src/evaluator/research_loop/process_llm_turn.rs` for the line:
        `log::info!("LLM response for session {}: User Answer: '{}...', ...Full answer: {:#?}", ...);`
        This will show the state of the `LlmAgentResponse` as it's being processed further.

2.  **Examine Raw LLM Output (If Possible):**
    *   If the `llm::llm_typed_unified::llm_typed` function or the underlying LLM client library allows access to the raw JSON string *before* deserialization, inspect this raw string when a cutoff is observed. This would definitively show whether the truncation is in the data received from the LLM.

3.  **Experiment with `LlmConfig.max_tokens`:**
    *   Explicitly set a higher `max_tokens` value in `LlmConfig` for `conversation_models` and observe if the issue persists or changes.

4.  **Simplify the Prompt:**
    *   Temporarily simplify the system prompt in `crates/agentloop/src/conversation/prompt.rs` to see if a less prescriptive prompt reduces the incidence of this specific truncation pattern. For example, remove parts of the prompt that heavily emphasize structure or multi-part responses.

5.  **Review `LlmAgentResponse` Few-Shot Examples:**
    *   While the current examples don't seem problematic, consider if adding an example of a longer, multi-sentence response (potentially with a colon) could guide the LLM better or expose if the LLM struggles with such examples.

6.  **Check Model-Specific Behavior:**
    *   The issue might be more prevalent with certain LLM models. If multiple models are configured in `conversation_models`, try isolating the issue to a specific model.

## Conclusion:

The most likely cause of the described cutoff issue is that the LLM itself is truncating the string content within the JSON fields (`user_answer` or `agent_reasoning`) during its generation process, particularly when trying to structure output after a colon. The overall JSON structure might remain valid and parsable, leading to the application processing the truncated content. Debugging should focus on capturing the LLM's output as early as possible to confirm where the truncation originates.