# Feature Plan: Tasks as Documents and Research Usage Control

**Date:** 2025-06-04
**Status:** Proposed

## 1. Goal

This feature aims to:
1.  Introduce a concept of "Tasks" which are a special type of `Document` used to describe research workflows.
2.  Add a mechanism to control how research documents (or other documents) are utilized within these tasks, specified by an `include_research` flag.
3.  Update the backend and frontend to manage and display these tasks and their properties.

## 2. Backend Changes

### 2.1. Database Schema & Migrations (`crates/narrativ/backend/db_schema.sql`)

A new SQL migration file will be created (e.g., `YYYYMMDDHHMMSS_add_task_and_research_usage_to_documents.sql`) with the following changes to the `documents` table:

*   **Add `is_task` column:**
    *   Type: `BOOLEAN`
    *   Constraints: `NOT NULL`
    *   Default: `FALSE`
    *   Comment: `'True if the document represents a research workflow task, false otherwise.'`

*   **Add `include_research` column:**
   * it should be tied to a new enum in Rust but dont use postgres enum contraints - the validation logic is in the backend
      *   Type: `TEXT`
      *   Default: `NULL` (or 'TaskDependent' if that makes more sense as a default for tasks)
      *   Comment: `'Controls how associated research documents are used: Never, TaskDependent (e.g., based on task context), Always.'`

**Migration SQL (Example):**
```sql
-- In new migration file: YYYYMMDDHHMMSS_add_task_and_research_usage_to_documents.sql
ALTER TABLE public.documents
    ADD COLUMN IF NOT EXISTS is_task BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS include_research TEXT NULL
```

### 2.2. Backend Models (`crates/narrativ/backend/src/db/documents.rs`)

*   **Update `Document` struct:**
    *   Add `pub is_task: bool,`
    *   Add `pub include_research: Option<DocumentResearchUsage>,`
    *   Define a Rust enum `DocumentResearchUsage` (e.g., in `src/db/documents.rs` or a common types module) if strong typing is desired in Rust, and handle conversion from/to `Option<String>`.
        ```rust
        // Potentially in src/db/types.rs or directly in documents.rs
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, sqlx::Type, utoipa::ToSchema)]
        #[sqlx(type_name = "TEXT")] // To ensure sqlx maps it correctly if used directly in queries for TEXT
        pub enum DocumentResearchUsage {
            Never,
            TaskDependent,
            Always,
        }

        // In Document struct:
        // pub include_research: Option<DocumentResearchUsage>, // if using sqlx::Type and direct mapping
        // Or keep as Option<String> and parse manually.
        // For simplicity with CHECK constraint, Option<String> is easier for sqlx::FromRow.
        ```
    *   Update `utoipa::ToSchema` annotations for the new fields.

*   **SQL Queries:** All `SELECT` queries in document routes for `documents` must be updated to include `is_task` and `include_research`. `INSERT` and `UPDATE` queries must also be adapted.

### 2.3. Backend Routes

#### 2.3.1. List Documents (`crates/narrativ/backend/src/routes/documents/list_documents.rs`)

*   **Update `ListDocumentsParams` struct:**
    *   Add `pub is_task: Option<bool>,`
    *   Add `utoipa::ToSchema` annotation for the new parameter.
*   **Update `list_documents` handler:**
    *   Modify the SQL query to conditionally add `AND is_task = $N` to the `WHERE` clause if `params.is_task` is provided.
    *   Ensure the `COUNT(*)` query also respects the `is_task` filter.

#### 2.3.2. Create/Update Documents
*   `crates/narrativ/backend/src/routes/documents/create_document_request.rs`:
    *   Add `pub is_task: Option<bool>,` (defaulting to `false` if not provided, or require it).
    *   Add `pub include_research: Option<String>,` (validate against 'Never', 'TaskDependent', 'Always' if provided).
    *   Update `utoipa::ToSchema` and `validator` annotations.
*   `crates/narrativ/backend/src/routes/documents/create_document.rs`:
    *   Handle new fields from `CreateDocumentRequest` when inserting into the database.
*   `crates/narrativ/backend/src/routes/documents/update_document_request.rs`:
    *   Add `pub is_task: Option<bool>,`
    *   Add `pub include_research: Option<String>,`
    *   Update `utoipa::ToSchema`.
*   `crates/narrativ/backend/src/routes/documents/update_document.rs`:
    *   Handle new fields when updating the document. Ensure `COALESCE` or similar logic is used for optional fields in the `UPDATE` statement.

#### 2.3.3. New "Start Research From Task" Action (Conceptual)
*   This is a new piece of functionality. For now, the plan is to acknowledge it.
*   A new route might be needed, e.g., `POST /api/documents/{id}/start_research_workflow`.
*   This route would:
    *   Fetch the task document by `id`.
    *   Verify `is_task` is true.
    *   Potentially initiate a `research_conversation` or interact with `agentloop` using the task's content and `include_research` setting.
    *   This will be detailed in a separate plan/task.

### 2.4. OpenAPI Documentation (`crates/narrativ/backend/src/openapi.rs`)

*   Update `paths` for `list_documents` to include the new `is_task` query parameter.
*   Update `components.schemas` for `CreateDocumentRequest`, `UpdateDocumentRequest`, and `Document` to reflect the new fields.

## 3. Frontend Changes

### 3.1. API Client Regeneration

*   Run `make generate-api-client` (or equivalent command) to update frontend API service and models based on the new OpenAPI spec.
    *   This should update `Document.ts`, `CreateDocumentRequest.ts`, `UpdateDocumentRequest.ts` models.
    *   The `DocumentsService.listDocuments` method signature should now include `isTask`.

### 3.2. State Management (`crates/narrativ/frontend/src/features/document/context/research-context.tsx`)
*(This context is currently named `research-context.tsx` but should ideally be `document-context.tsx` as per previous refactoring plans. Assuming it's `document-context.tsx` for this plan)*

*   **Add new state:**
    *   `showTasks: boolean` (default `false` or `true` depending on desired initial view).
    *   `setShowTasks: (show: boolean) => void`.
*   **Update `fetchData` (or the `useEffect` hook responsible for fetching documents):**
    *   When calling `DocumentsService.listDocuments`, pass the `isTask: showTasks` parameter.
    *   Consider if `showTasks` should fetch *only* tasks, or if it's a filter on top of `showPublic`. The backend `list_documents` currently implies `is_task` is an independent filter. The UI might need separate toggles or a combined filter logic. For simplicity, assume `showTasks = true` means `is_task = true` and `showTasks = false` means `is_task = false`. If both user documents and tasks are to be shown together, the context and API might need adjustment.
    *   Add `showTasks` to the dependency array of the `useEffect` hook.

### 3.3. UI Components

#### 3.3.1. Data Table Toolbar (`crates/narrativ/frontend/src/features/document/components/data-table-toolbar.tsx`)

*   Add a new `Switch` component similar to "Show Public Documents".
    *   Label: "Show Tasks" or "Show Only Tasks".
    *   Controlled by `showTasks` state from `useResearch` (useDocument) context.
    *   `onCheckedChange` should call `setShowTasks`.
    *   `data-testid`: e.g., `data-table-toolbar-show-tasks-switch`.

#### 3.3.2. Document Columns (`crates/narrativ/frontend/src/features/document/components/research-columns.tsx`)

*   **Optional: Add "Type" or "Is Task" column:**
    *   Display "Task" or "Document" based on the `is_task` field.
    *   Or a simple checkmark if it's a task.
*   **Update Actions Column (`DataTableRowActions` in `data-table-row-actions.tsx`):**
    *   Conditionally render a new "Start Research" button if `row.original.is_task` is true.
        *   Icon: e.g., `PlayIcon` or a specific "workflow" icon.
        *   `onClick` handler: This will initially be a placeholder or log to console. Implementation will depend on the backend route from 2.3.3.
        *   `data-testid`: e.g., `document-table-row-actions-start-research-button`.

#### 3.3.3. Document List Page (`crates/narrativ/frontend/src/features/document/components/ResearchListPage.tsx`)

*   Review page title and subtitle ("Documents") if `showTasks` extensively changes the view's purpose. It might be fine as "Documents" can encompass tasks.
*   Ensure "New Document" button correctly sets `is_task: false` (or provides an option if tasks can also be manually created this way).

#### 3.3.4. Create/Edit Document Forms/Dialogs
*   If tasks are created/edited through the same UI as regular documents:
    *   The create/edit forms (e.g., potentially within `ResearchDialogs.tsx` or a new `DocumentForm.tsx`) need to include fields for:
        *   `is_task` (e.g., a checkbox).
        *   `include_research` (e.g., a dropdown select with "Never", "TaskDependent", "Always").
    *   These fields should be passed to the `createDocument` / `updateDocument` service calls.

## 4. Testing

*   **Backend:**
    *   Unit tests for new/modified DB functions.
    *   Integration tests for API endpoints (`/api/documents`) ensuring correct filtering with `is_task` and handling of new fields in create/update.
*   **Frontend:**
    *   Component tests for the new "Show Tasks" switch and "Start Research" button.
    *   Integration tests for the document list page verifying filtering by tasks.

## 5. Follow-up Considerations

*   Detailed implementation of the "Start Research From Task" backend endpoint and corresponding frontend interaction.
*   Refining the UI/UX for how tasks and regular documents are presented (e.g., separate tabs, combined list with clear visual distinction).
*   Defining the `DocumentResearchUsage` enum values and their exact behavior within research workflows.