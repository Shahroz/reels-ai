# Plan for One-Time Background Research Tasks

## 1. Objective

To implement a system for running research tasks as a one-time, asynchronous background job. This is distinct from the existing "Infinite Research" feature, which handles recurring tasks via cron schedules. This new feature will leverage Google Cloud Tasks for queueing and execution with automatic retries.

The user will submit a research prompt via an API endpoint, and the system will queue it for execution. The user can later check the status and retrieve the results of the task via another endpoint.

## 2. Comparison with Infinite Research

This feature will be architecturally similar to the existing `infinite_research` implementation, but with key differences:

| Feature             | Infinite Research (Existing)                               | One-Time Research (New)                                        |
| ------------------- | ---------------------------------------------------------- | -------------------------------------------------------------- |
| **Triggering**      | Recurring, based on a CRON schedule.                       | Single execution, triggered by an API call.                    |
| **Google Service**  | Google Cloud Scheduler                                     | Google Cloud Tasks                                             |
| **DB Table**        | `infinite_researches` (stores config) and `infinite_research_executions` (stores run history). | `one_time_researches` (stores config and execution result).    |
| **Core API**        | CRUD operations for managing recurring tasks.              | Create a task, get its status/result.                          |

## 3. Database Schema Changes

A new table, `one_time_researches`, will be created to store the task details and its execution state. A separate `executions` table is not planned for the initial version to maintain simplicity, as Cloud Tasks handles retries internally.

### New Migration File

A new migration file will be created at `crates/narrativ/backend/migrations/YYYYMMDDHHMMSS_create_one_time_researches_table.sql`.

```sql
-- Migration for creating the one-time research tasks table.

CREATE TABLE one_time_researches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    prompt TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, queued, running, completed, failed
    cloud_task_name TEXT, -- To store the full Google Cloud Tasks task name
    output_log_url TEXT, -- URL to the GCS object containing the output log
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

-- Add a trigger to automatically update the updated_at timestamp
CREATE TRIGGER trigger_one_time_researches_updated_at
BEFORE UPDATE ON one_time_researches
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Add indexes
CREATE INDEX idx_one_time_researches_user_id ON one_time_researches(user_id);
CREATE INDEX idx_one_time_researches_status ON one_time_researches(status);
CREATE INDEX idx_one_time_researches_created_at ON one_time_researches(created_at);
```

## 4. Backend Implementation Plan

### a. GCP Integration: Cloud Tasks Client

A new module will be created to interact with the Google Cloud Tasks API.

-   **File:** `crates/narrativ/backend/src/gcp/cloud_tasks.rs`
-   **Dependencies:** Add `google-cloud-tasks-v2` to `Cargo.toml`.
-   **Struct:** `CloudTasksClient` similar to `SchedulerClient`.
-   **Methods:**
    -   `new()`: Initializes the client and reads config from env vars (`GCP_PROJECT_ID`, `GCP_LOCATION`, `GCP_TASKS_QUEUE_ID`, `BACKEND_URL`).
    -   `create_http_task()`: A method to create a new task with an HTTP target. It will accept the target URL, a JWT, and a payload. It should configure retry policies.

### b. Authentication: Task Token

A new module will handle JWT generation for authenticating calls from Cloud Tasks to our internal endpoint.

-   **File:** `crates/narrativ/backend/src/auth/task_token.rs`
-   **Claims Struct:** `OneTimeResearchClaims` containing `user_id` and `one_time_research_id`.
-   **Function:** `generate_task_jwt()` that creates a short-lived JWT for a specific task.

### c. DB Models and Queries

-   **Model:** `crates/narrativ/backend/src/db/one_time_research.rs` will define the `OneTimeResearch` struct mapping to the new table.
-   **Queries:** `crates/narrativ/backend/src/queries/one_time_research/` will contain functions for:
    -   `create_one_time_research()`
    -   `get_one_time_research_by_id()`
    -   `update_one_time_research_status()`
    -   `update_one_time_research_on_start()`
    -   `update_one_time_research_on_finish()`

### d. API Routes

#### New Route Module

-   **Directory:** `crates/narrativ/backend/src/routes/one_time_researches/`
-   This module will contain handlers and request/response structs.

#### Public-Facing Endpoints

1.  **Create One-Time Research Task**
    -   **Endpoint:** `POST /api/one-time-researches`
    -   **Handler:** `crates/narrativ/backend/src/routes/one_time_researches/create_one_time_research.rs`
    -   **Request Body:** `CreateOneTimeResearchRequest { prompt: String }`
    -   **Auth:** Standard User JWT (`Claims`).
    -   **Logic:**
        1.  Validate request.
        2.  Call `queries::create_one_time_research` to create a DB record with `status: 'pending'`.
        3.  Generate a task-specific JWT using `auth::task_token`.
        4.  Call `gcp::cloud_tasks::create_http_task` to queue the job. The target URL will be `/api/internal/run-one-time-research/{id}`.
        5.  Update the DB record with the `cloud_task_name` and set `status: 'queued'`.
        6.  Return `202 Accepted` with the `OneTimeResearch` object.

2.  **Get One-Time Research Status & Result**
    -   **Endpoint:** `GET /api/one-time-researches/{id}`
    -   **Handler:** `crates/narrativ/backend/src/routes/one_time_researches/get_one_time_research.rs`
    -   **Auth:** Standard User JWT (`Claims`).
    -   **Logic:**
        1.  Fetch the `OneTimeResearch` record from the DB by `id`, ensuring the `user_id` matches the authenticated user.
        2.  Return the record.

#### Internal Endpoint

1.  **Run One-Time Research Task**
    -   **Endpoint:** `POST /api/internal/run-one-time-research/{id}`
    -   **Handler:** `crates/narrativ/backend/src/routes/internal/run_one_time_research.rs`
    -   **Auth:** Custom Task JWT (`OneTimeResearchClaims`).
    -   **Logic:**
        1.  Verify the task JWT. Ensure the `one_time_research_id` in the claims matches the path `{id}`.
        2.  Fetch the task from the DB.
        3.  Call `queries::update_one_time_research_on_start` to set `status: 'running'`.
        4.  Execute the agent using `services::agent_service::run_and_log_research`.
        5.  Call `queries::update_one_time_research_on_finish` with the result (success or failure), saving the `output_log_url` or `error_message`.
        6.  Return `200 OK` on success, or an appropriate error code to signal Cloud Tasks (e.g., `500` to trigger a retry, `2xx` to confirm completion).

### e. New File Structure

```
crates/narrativ/backend/
├── migrations/
│   └── YYYYMMDDHHMMSS_create_one_time_researches_table.sql  (new)
└── src/
    ├── auth/
    │   └── task_token.rs                                    (new)
    ├── db/
    │   └── one_time_research.rs                             (new)
    ├── gcp/
    │   └── cloud_tasks.rs                                   (new)
    ├── queries/
    │   └── one_time_research/                               (new)
    │       ├── mod.rs
    │       ├── create_one_time_research.rs
    │       └── ... (other query files)
    └── routes/
        ├── internal/
        │   └── run_one_time_research.rs                     (new)
        └── one_time_researches/                             (new)
            ├── mod.rs
            ├── configure_one_time_researches_routes.rs
            ├── create_one_time_research.rs
            ├── get_one_time_research.rs
            └── ... (request/response struct files)
```

## 5. High-Level Development Steps

1.  **Setup:** Add `google-cloud-tasks-v2` to `Cargo.toml`.
2.  **Database:** Create and run the new migration file.
3.  **Core Logic:**
    -   Implement `db::one_time_research` model.
    -   Implement `queries::one_time_research` functions.
    -   Implement `gcp::cloud_tasks::CloudTasksClient`.
    -   Implement `auth::task_token` generation.
4.  **Endpoints:**
    -   Implement the internal `run_one_time_research` handler.
    -   Implement the public `create_one_time_research` handler.
    -   Implement the public `get_one_time_research` handler.
5.  **Integration:**
    -   Add the new routes to the Actix-web app configuration (`routes/mod.rs`).
    -   Add new handlers and schemas to `openapi.rs`.
6.  **Testing:** Add unit and integration tests for the new logic and endpoints.