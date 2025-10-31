 # Refactor: Separating Research Conversations from Documents (v2 - GCS State)

 **Date:** 2025-05-12
 **Status:** Proposed

 ## 1. Goal

 The current `research` table and associated backend/frontend components conflate the *process* of research (the conversation with an agent) and the final *output* (a research document). This refactor aims to separate these concepts to:

 *   Enable potential future features like resuming research conversations.
 *   Provide clearer data modeling.
 *   Align naming with the actual data stored (the final `research` record is more like a `document`).
 *   Leverage existing `agentloop` session state management for storing conversation history and state externally.

 ## 2. Proposed Changes

 ### 2.1. Database Schema

 #### 2.1.1. Rename `research` Table

 The existing `research` table will be renamed to `documents` to better reflect its purpose of storing the final output.

 #### 2.1.2. New `research_conversations` Table

 A new table will be created to store metadata about the agent interaction process, linking to the externally stored state.

 **Schema Definition:**

 ```sql
 CREATE TABLE IF NOT EXISTS research_conversations (
     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),              -- Unique conversation identifier (likely matches agentloop session_id)
     user_id UUID NOT NULL REFERENCES users(id),                 -- Owning user
     document_id UUID NULL REFERENCES documents(id),             -- Link to the final document produced (nullable, set when saved)
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     last_instruction TEXT NULL,                                 -- The initial or most recent user instruction driving the conversation
     conversation_state_gcs_uri TEXT NULL,                       -- URI/Path to the serialized conversation state in GCS
     status VARCHAR(50) NOT NULL DEFAULT 'Active'                -- e.g., Active, Completed, Failed, Archived, Paused
 );

 -- Indexes
 CREATE INDEX IF NOT EXISTS idx_research_conversations_user_id ON research_conversations(user_id);
 CREATE INDEX IF NOT EXISTS idx_research_conversations_document_id ON research_conversations(document_id);

 -- Trigger for updated_at
 CREATE TRIGGER update_research_conversations_updated_at
     BEFORE UPDATE ON research_conversations
     FOR EACH ROW
     EXECUTE FUNCTION update_updated_at_column();

 ```

 **Discussion: Conversation State Storage**

 Instead of storing the conversation state directly in the database (e.g., as JSONB), we will leverage Google Cloud Storage (GCS). The `agentloop` service is responsible for managing the lifecycle of a research session and its state, represented by the `SessionData` struct (see `crates/agentloop/src/types/session_data.rs`).

 When a session needs to be persisted (e.g., paused, completed, or for potential resumption), `agentloop` will:
 1.  Serialize its current `SessionData` (which includes history, context, status, config, etc.).
 2.  Upload this serialized state to a designated GCS bucket.
 3.  Communicate the resulting GCS URI back to the Narrativ backend (or the backend retrieves it).

 The Narrativ backend then stores this `conversation_state_gcs_uri` in the `research_conversations` table. To resume a conversation, the backend would provide this URI back to `agentloop`, which would download the state from GCS and deserialize it to restore the session.

 *Pros:* Scalable storage for potentially large states, decouples state format from the main database schema, leverages `agentloop`'s existing state definition (`SessionData`).
 *Cons:* Adds dependency on GCS, requires coordination between Narrativ backend and `agentloop` for URI management.

 #### 2.1.3. Update Foreign Keys

 Tables referencing the old `research` table need updating.

 *   **`creatives` table:** The `research_ids` column (`Option<Vec<Uuid>>`) currently links to `research.id`. This should be renamed to `document_ids` and point to the final `documents.id`.
 *   **`webflow_creatives` table:** (Assuming it has a similar reference) Needs similar updates.

 ### 2.2. Database Migrations

 A new migration file is required:

 1.  **Rename Table:**
     ```sql
     ALTER TABLE research RENAME TO documents;
     ```
 2.  **Create Table:**
     ```sql
     -- SQL from section 2.1.2 above
     CREATE TABLE IF NOT EXISTS research_conversations (...);
     CREATE INDEX ...;
     CREATE TRIGGER ...;
     ```
 3.  **Rename FK Column in `creatives`:**
     ```sql
     ALTER TABLE creatives RENAME COLUMN research_ids TO document_ids;
     -- Note: The constraint name might also need renaming if explicitly named.
     -- Example: ALTER TABLE creatives RENAME CONSTRAINT fk_creatives_research TO fk_creatives_documents;
     -- Need to check actual constraint name.
     ```
 4.  **Update FK Column in `webflow_creatives`:** (Assuming similar structure)
     ```sql
     ALTER TABLE webflow_creatives RENAME COLUMN research_ids TO document_ids;
     -- Potentially rename constraint as well.
     ```

 ### 2.3. Backend Changes (`crates/narrativ/backend`)

 1.  **DB Layer (`src/db/`):**
     *   Rename `src/db/research.rs` to `src/db/documents.rs`.
     *   Rename the `Research` struct within it to `Document`. Update `FromRow`, `Serialize`, `Deserialize`, `ToSchema` derives and field names if necessary (current fields seem appropriate for `Document`).
     *   Create `src/db/research_conversations.rs` defining a `ResearchConversation` struct mapping to the new table (including `conversation_state_gcs_uri`).
     *   Update `src/db/mod.rs` to reflect the renaming and new module.
     *   Update database query functions (CRUD operations) previously operating on `research` to operate on `documents`. Rename functions accordingly (e.g., `create_research` -> `create_document`).
     *   Implement new query functions for `research_conversations` (e.g., `create_conversation`, `update_conversation_uri`, `get_conversation_by_id`, `list_conversations_by_user`).

 2.  **API Layer (`src/routes/`):**
     *   Rename route modules: `src/routes/research/` -> `src/routes/documents/`.
     *   Update handlers within these modules (`create_research.rs` -> `create_document.rs`, etc.) to use the new `Document` struct and DB functions. Update paths in `utoipa::path` macros and route registration (`#[post("")]`, etc.).
     *   Update request/response structs (`CreateResearchRequest` -> `CreateDocumentRequest`, etc.).
     *   Update `src/routes/mod.rs` to configure the renamed `/api/documents` scope.
     *   Create new route modules/handlers for research conversations if needed (e.g., `src/routes/research_conversations/`). These might handle listing past conversations, retrieving their status and GCS URI, and potentially triggering archival or deletion. Initiating new conversations or resuming existing ones would likely still primarily go through `agentloop` endpoints, but the Narrativ backend needs to manage the `research_conversations` records.
     *   Update `src/routes/creatives/create_creative.rs` and `generate_creative.rs` (and potentially others) to use `document_ids` instead of `research_ids`.

 3.  **Update OpenAPI Documentation:**
     *   Regenerate or update `openapi.rs` to reflect the changed routes, schemas (`Document`, `CreateDocumentRequest`, `ResearchConversation`, etc.).

 ### 2.4. Frontend Changes (`crates/narrativ/frontend`)

 1.  **API Layer:**
     *   Regenerate API client models and services. This should rename `Research` -> `Document`, `CreateResearchRequest` -> `CreateDocumentRequest`, `ResearchService` -> `DocumentService`, etc.
     *   Create new models/services for `ResearchConversation` if API endpoints are added.

 2.  **Context (`src/features/research/context/`):**
     *   Rename `research-context.tsx` to `document-context.tsx`.
     *   Update `ResearchProvider` to `DocumentProvider`.
     *   Update state variables (`researchItems` -> `documents`).
     *   Update API calls (`ResearchService.*` -> `DocumentService.*`).
     *   Rename functions (`saveResearchFromChat` -> `saveDocumentFromChat`, `deleteResearch` -> `deleteDocument`, etc.).
     *   The `saveDocumentFromChat` function will now call `DocumentService.createDocument`. It might also need to coordinate with the backend to link the new `Document.id` to the corresponding `ResearchConversation.id`.

 3.  **Components (`src/features/research/components/`):**
     *   Rename components where appropriate (`ResearchListPage` -> `DocumentListPage`, `ResearchTable` -> `DocumentTable`, `research-columns.tsx` -> `document-columns.tsx`, etc.).
     *   Update components to use the renamed context (`useResearch` -> `useDocument`).
     *   Update data display to reflect `Document` properties.
     *   Modify `AgentLoopChat.tsx` (or the code handling the `FinalAnswerEvent`): When saving, it should call `saveDocumentFromChat`. The process of associating the `Document` with the `ResearchConversation` needs clarification (Does `agentloop` provide the conversation ID? Does the backend handle this association?).
     *   Update `src/routes/_authenticated/research/index.tsx` and `new.tsx` to use the renamed/updated components and context. Rename the routes themselves (`/research` -> `/documents`).

 4.  **UI for Conversations:**
     *   Design UI for managing/viewing past research conversations (listing, showing status, potentially linking to the generated document).
     *   If conversations are resumable, UI elements will be needed to trigger resumption (likely by calling an `agentloop` endpoint, possibly proxied via the Narrativ backend, passing the `conversation_state_gcs_uri`).

 ### 2.5. Agentloop Service Changes (`crates/agentloop`)

 *   **State Management:** `agentloop` already defines the session state via `SessionData` and has endpoints like `get_session_state` which implies serialization capability.
 *   **GCS Integration:**
     *   Implement logic within `agentloop` to serialize `SessionData` to a suitable format (e.g., JSON, MessagePack).
     *   Implement logic to upload the serialized data to GCS upon session pause, completion, or explicit save request.
     *   Implement logic to download state data from a given GCS URI and deserialize it back into `SessionData` to resume a session.
 *   **API Adjustments:**
     *   `/start_research` (or equivalent endpoint): Should return the `session_id` which will be used as the `id` for the `research_conversations` record in the Narrativ backend.
     *   Session Lifecycle: When a session ends (completed, failed, timeout, interrupted), `agentloop` should trigger the GCS state upload.
     *   New Endpoint for Resumption: A `POST /session/{session_id}/resume` endpoint (or similar) might be needed in `agentloop`. This endpoint would accept the `conversation_state_gcs_uri` (or the Narrativ backend provides it based on the `session_id`), download the state, deserialize it, and restart the agent loop.
     *   State Retrieval: The existing `GET /session/{session_id}/state` endpoint can continue to provide the *current* in-memory state. An endpoint might be needed to explicitly request the *persisted* state URI from GCS if the session is not active.

 ## 3. Open Questions

 1.  **GCS Bucket/Permissions:** Define the GCS bucket structure and IAM permissions needed for `agentloop` to read/write state files.
 2.  **State Serialization Format:** Choose a robust serialization format (JSON is simple, MessagePack might be more efficient). Ensure `SessionData` (and all its nested types like `SessionStatus`, `ConversationEntry`, etc.) correctly implement `serde::Serialize` and `serde::Deserialize`.
 3.  **Coordination Protocol:** How does the Narrativ backend get notified of the `conversation_state_gcs_uri` when `agentloop` saves the state? (e.g., Does `agentloop` call a webhook? Does the backend poll `agentloop`? Does the frontend orchestrate?).
 4.  **`creatives.document_ids` Target:** Confirm that `creatives.document_ids` should point to the `documents` table. Is there a need to link a creative back to the `research_conversations` record?
 5.  **UI for Conversations:** Finalize the UI design for listing, viewing, and potentially resuming conversations.
 6.  **Error Handling:** Define how errors during GCS upload/download or state deserialization in `agentloop` are handled and communicated back to the user/Narrativ backend. Update `research_conversations.status` accordingly.
 7.  **Data Migration for Existing Research:** How should existing `research` records be handled? Create a `research_conversations` record for each, leaving `conversation_state_gcs_uri` null initially?
 8.  **`webflow_creatives` Structure:** Verify the exact structure and foreign key usage in the `webflow_creatives` table.
 9.  **Session ID Mapping:** Ensure the `agentloop` `session_id` (likely a `uuid::Uuid`) is used consistently as the `id` in the `research_conversations` table.