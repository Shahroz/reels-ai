# Plan for One-Time Research Progress Tracking

## 1. Objective & Analysis

The goal of this initiative is to enhance the one-time research feature by providing users with real-time, granular progress updates. Currently, users can only see the final status (`completed` or `failed`) and the URL to the full output log upon completion.

### Analysis of Current State:
-   **ID on Creation**: The current `POST /api/one-time-researches` endpoint correctly returns the full `OneTimeResearch` object, including its `id`, upon successful creation. This meets the user's first requirement.
-   **Progress Visibility**: There is no mechanism to view progress while a task is in the `running` state.
-   **Architecture**: The `agentloop` module, which executes the research, is used as a Rust library within the `narrativ/backend` process, not as a separate microservice. Communication happens via direct function calls.

### Proposed Solution:
To implement progress tracking, we will use a callback mechanism. This is more efficient than the originally considered HTTP callback, as it avoids network latency and the complexity of securing another endpoint.

The solution involves three main parts:
1.  **Database Enhancement**: A new `progress_log` column of type `JSONB` will be added to the `one_time_researches` table. It will store an array of progress-update objects.
2.  **Library Modification (`agentloop`)**: The core `run_research_loop_sync` function will be updated to accept an optional, asynchronous callback function. This callback will be invoked at key stages of the research process.
3.  **Service Logic (`narrativ/backend`)**: The backend service will provide an implementation for the callback. This implementation will write the progress updates it receives from the `agentloop` into the new `progress_log` database column.

---

## 2. High-Level Flow

1.  A user calls `POST /api/one-time-researches` with a prompt.
2.  The `create_one_time_research` handler creates a DB record and queues a Google Cloud Task. The task points to the internal endpoint `/api/internal/run-one-time-research/{id}`.
3.  Google Cloud Tasks invokes the internal endpoint.
4.  The `run_one_time_research` handler begins execution. It prepares an asynchronous callback function that captures the database connection pool and the task's ID.
5.  The handler calls `agent_service::run_and_log_research`, which in turn calls `agentloop::run_research_loop_sync`, passing the newly created callback function.
6.  Inside the `agentloop` research loop, whenever a significant event occurs (e.g., an LLM response is generated, a tool is used), the `agentloop` invokes the callback with a `ProgressUpdate` payload.
7.  The callback function executes, writing the `ProgressUpdate` payload into the `progress_log` JSONB array in the `one_time_researches` table for the corresponding task ID.
8.  The user can poll the `GET /api/one-time-researches/{id}` endpoint to retrieve the task status and the populated `progress_log` field to see live updates.

---

## 3. Detailed Implementation Steps

### Step 3.1: Database Schema Changes

A new migration file is required to add the `progress_log` column.

-   **New Migration File**: `crates/narrativ/backend/migrations/YYYYMMDDHHMMSS_add_progress_log_to_one_time_researches.sql`
-   **Content**:
    ```sql
    -- Adds a column to store real-time progress updates for one-time research tasks.
    ALTER TABLE one_time_researches
    ADD COLUMN progress_log JSONB NOT NULL DEFAULT '[]'::jsonb;
    ```

### Step 3.2: `agentloop` Library Modifications

The `agentloop` library needs to be made aware of the callback mechanism.

1.  **Define Progress Update Types**:
    -   Create a new file: `crates/agentloop/src/types/progress_update.rs`.
    -   Define the `ProgressUpdate` struct and a callback type alias.
      ```rust
      //! Defines the structure for progress updates and the callback type.
      
      /// Represents a single progress update from the research loop.
      #[derive(Debug, Clone, serde::Serialize)]
      pub struct ProgressUpdate {
          pub sender: String, // e.g., "user", "agent", "tool"
          pub message: String,
          pub timestamp: chrono::DateTime<chrono::Utc>,
      }

      /// Type alias for the async progress callback function.
      pub type ProgressCallback = Box<dyn FnMut(ProgressUpdate) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;
      ```

2.  **Update Research Loop Signatures**:
    -   Modify `crates/agentloop/src/evaluator/run_research_loop_sync.rs` and `run_research_loop.rs`.
    -   Add `progress_callback: Option<ProgressCallback>` as a new parameter to the function signatures.
      ```rust
      // In run_research_loop_sync.rs
      pub async fn run_research_loop_sync(
          app_state: actix_web::web::Data<crate::state::app_state::AppState>,
          session_id: crate::types::session_id::SessionId,
          research_goal: std::string::String,
          mut progress_callback: Option<crate::types::progress_update::ProgressCallback>, // New parameter
      ) -> ...
      ```

3.  **Invoke Callback in Research Loop**:
    -   Inside the `loop` in `run_research_loop_sync.rs`, call the callback at key points.
    -   Example invocation after an LLM turn:
      ```rust
      // After `process_llm_turn` succeeds
      Ok(llm_response) => {
          if let Some(cb) = &mut progress_callback {
              // Assuming llm_response has a message field
              let update = crate::types::progress_update::ProgressUpdate {
                  sender: "agent".to_string(),
                  message: llm_response.message.clone(), // Or a trimmed version
                  timestamp: chrono::Utc::now(),
              };
              cb(update).await;
          }
          // ... rest of the logic
      }
      ```
    -   Callbacks should also be added after injecting the initial user goal and after tool calls are handled.

### Step 3.3: `narrativ/backend` Modifications

1.  **Add New Database Query**:
    -   Create file: `crates/narrativ/backend/src/queries/one_time_research/update_one_time_research_progress.rs`.
    -   Implement a function to append a new entry to the `progress_log`.
      ```rust
      //! Appends a progress entry to a one-time research task's progress log.
      
      #[tracing::instrument(skip(pool, progress_entry))]
      pub async fn update_one_time_research_progress(
          pool: &sqlx::PgPool,
          id: uuid::Uuid,
          progress_entry: &serde_json::Value,
      ) -> Result<(), sqlx::Error> {
          sqlx::query!(
              r#"
              UPDATE one_time_researches
              SET progress_log = progress_log || $1::jsonb
              WHERE id = $2
              "#,
              progress_entry,
              id
          )
          .execute(pool)
          .await?;
          Ok(())
      }
      ```

2.  **Update DB Model**:
    -   In `crates/narrativ/backend/src/db/one_time_research.rs`, add the new field to the `OneTimeResearch` struct.
      ```rust
      #[derive(sqlx::FromRow, serde::Serialize, Debug, Clone, utoipa::ToSchema)]
      pub struct OneTimeResearch {
          // ... existing fields
          #[schema(value_type = Option<Object>)] // For utoipa
          pub progress_log: serde_json::Value, // Or a more specific type
      }
      ```

3.  **Update Service Layer**:
    -   Modify `crates/narrativ/backend/src/services/agent_service.rs`.
    -   Change `run_and_log_research` to accept `pool: &sqlx::PgPool`.
    -   Inside `run_and_log_research`, construct the callback closure and pass it to `run_research_loop_sync`.
      ```rust
      // In agent_service.rs
      pub async fn run_and_log_research(
          request: agentloop::types::research_request::ResearchRequest,
          gcs_client: &crate::services::gcs::gcs_client::GCSClient,
          pool: &sqlx::PgPool, // New parameter
          execution_id: uuid::Uuid,
      ) -> ... {
          let agentloop_state = create_agentloop_state().await?;
          // ... create session ...
      
          let captured_pool = pool.clone();
          let progress_callback = Box::new(move |update: agentloop::types::progress_update::ProgressUpdate| {
              let pool = captured_pool.clone();
              Box::pin(async move {
                  let progress_json = serde_json::json!(update);
                  if let Err(e) = crate::queries::one_time_research::update_one_time_research_progress(&pool, execution_id, &progress_json).await {
                      log::error!("Failed to log progress for task {}: {}", execution_id, e);
                  }
              })
          });

          let conversation_history = run_research_loop_sync(
              agentloop_state,
              session_id,
              request.instruction,
              Some(progress_callback), // Pass the callback
          ).await.map_err(...);

          // ... rest of the function
      }
      ```

4.  **Update Internal Route**:
    -   In `crates/narrativ/backend/src/routes/internal/run_one_time_research.rs`, pass the `pool` to the service call.
      ```rust
      // In run_one_time_research.rs
      let result = agent_service::run_and_log_research(
          research_request,
          gcs_client.get_ref(),
          pool.get_ref(), // Pass the pool
          task_id,
      ).await;
      ```

### Step 3.4: API & Frontend Impact

-   **API Response**: The `OneTimeResearch` object returned from `GET /api/one-time-researches/{id}` will now include the `progress_log` field, which will be an array of progress objects.
-   **Frontend Model**: The TypeScript model at `crates/narrativ/frontend/src/api/models/OneTimeResearch.ts` must be updated to include the new field.
  ```typescript
  // In OneTimeResearch.ts
  export type OneTimeResearch = {
      // ... existing fields
      progress_log: Array<any>; // Or a more specific type like Array<{ sender: string; message: string; timestamp: string; }>
  };
  ```
-   **Frontend UI**: The UI can now poll the research task endpoint and render the `progress_log` array to show live updates to the user.