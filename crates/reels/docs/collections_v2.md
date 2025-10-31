# Technical Brief: Collections & File Objects Database Model (v2)

## 1. Overview

This document details the design for a flexible, user-centric database model to manage collections and associated file objects (both structured data and unstructured blobs). It aims to provide:

*   **User-Specific Collections:** Logical grouping of items under a user account.
*   **Flexible Item Storage:** Support for both arbitrary JSONB data (structured items) and references to blobs stored in Google Cloud Storage (GCS) (unstructured items like documents, images, videos).
*   **Virtual Folder Hierarchy:** A user-friendly way to organize items using path-based conventions, mimicking a file system without relying on recursive database structures.
*   **Relationship Modeling:** Support for defining dependencies or relationships between items (DAG structure) using `parent_ids`.
*   **Searchability:** Full-text search capabilities over item metadata and content.
*   **Access Control:** A basic mechanism (`is_private`) to control the visibility/usability of items in public contexts or generative processes.
*   **Scalability:** Design choices favoring performance, including targeted indexing and offloading large blobs to GCS.

This design synthesizes the discussion from `collections_raw.md`, opting for a two-table approach to clearly separate collection metadata (`db_collections`) from the items themselves (`db_file_objects`), where items can represent structured data, unstructured files/blobs, or virtual folders.

## 2. Database Schema

### 2.1. `db_collections` Table

Represents a named logical group of items owned by a user. Acts as a namespace or container.

```sql
CREATE TABLE db_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE, -- Assuming a 'users' table exists
    name TEXT NOT NULL, -- Renamed from 'title' for consistency
    description TEXT,
    -- schema_id UUID, -- Optional: Link to a potential future db_schemas table for JSON schema validation
    metadata JSONB, -- General purpose metadata for the collection itself
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Optional: Auto-update 'updated_at' timestamp
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_collections_set_updated_at
BEFORE UPDATE ON db_collections
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

```

**Columns:**

*   `id`: Unique identifier for the collection.
*   `user_id`: Foreign key linking to the owner user. Essential for multi-tenancy.
*   `name`: Human-readable name of the collection (unique per user recommended via application logic or constraint).
*   `description`: Optional text describing the collection's purpose.
*   `metadata`: Optional JSONB field for storing arbitrary metadata about the collection itself (e.g., display preferences, configuration).
*   `created_at`: Timestamp of creation.
*   `updated_at`: Timestamp of the last update (automatically managed by trigger).

**Indexing:**

*   Primary Key on `id`.
*   Index on `user_id` for efficient lookup of user's collections.
*   Optional unique constraint on `(user_id, name)` if names must be unique per user.
*   Optional GIN index on `metadata` if querying collection metadata is required.

### 2.2. `db_file_objects` Table

Stores metadata for all items within collections, representing folders, structured data entries, or unstructured files/blobs linked to GCS.

```sql
CREATE TABLE db_file_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id UUID NOT NULL REFERENCES db_collections(id) ON DELETE CASCADE,

    -- Virtual structure & Identification
    path TEXT NOT NULL,               -- Virtual path, e.g., "projects/report.pdf" or "data/customer_1/"
    is_folder BOOLEAN DEFAULT false,  -- True if this entry represents a virtual folder
    item_type TEXT,                   -- Discriminator, e.g., 'folder', 'image', 'pdf', 'json_data', 'video'

    -- Blob Storage (for is_blob = true)
    is_blob BOOLEAN DEFAULT false,    -- True if this entry represents a blob stored externally (e.g., GCS)
    gcs_path TEXT,                    -- Path/key within the GCS bucket
    content_type TEXT,                -- MIME type (e.g., 'image/png', 'application/json')
    size_bytes BIGINT,                -- Size of the blob in bytes

    -- Access Control
    is_private BOOLEAN DEFAULT false, -- If true, excluded from public listings or generative use cases

    -- Descriptive & Content Metadata
    description TEXT,                 -- User-provided description for the folder or file
    metadata JSONB,                   -- System-extracted metadata (e.g., image dimensions, video duration, JSON schema reference)
    data JSONB,                       -- User-defined structured data or additional metadata (e.g., tags, custom fields)

    -- Optional Relationships (Primarily for structured data or dependencies)
    parent_ids UUID[] DEFAULT '{}',   -- IDs of other db_file_objects this item relates to (DAG)

    -- Full-Text Search Vector (Optional, depends on search needs)
    -- data_tsv tsvector GENERATED ALWAYS AS (to_tsvector('simple', coalesce(description, '') || ' ' || coalesce(data::text, ''))) STORED,

    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Auto-update 'updated_at' timestamp trigger (can reuse the function from db_collections)
CREATE TRIGGER trg_file_objects_set_updated_at
BEFORE UPDATE ON db_file_objects
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Enable trigram extension for path searching
CREATE EXTENSION IF NOT EXISTS pg_trgm;
```

**Columns:**

*   `id`: Unique identifier for the file object.
*   `user_id`: Owning user.
*   `collection_id`: Parent collection.
*   `path`: The virtual path used to simulate folder structure. Should be unique within a `(user_id, collection_id)` scope. Examples: `/`, `/images/`, `/images/logo.png`, `/data/entry1.json`.
*   `is_folder`: Boolean flag indicating if this object represents a virtual folder. Folders typically don't have blob data (`is_blob=false`).
*   `item_type`: A string discriminator (e.g., 'folder', 'image/jpeg', 'application/pdf', 'text/markdown', 'application/json'). Useful for filtering and UI rendering. Could mirror `content_type` for blobs or be custom for structured data.
*   `is_blob`: Boolean flag indicating if the primary content is an external blob (referenced by `gcs_path`). If `false`, the content might be fully contained within the `data` JSONB field or it might be a folder.
*   `gcs_path`: The unique path/key for the object within the designated GCS bucket (e.g., `user_id/collection_id/object_id.ext` or a content-addressable hash). Only relevant if `is_blob` is true.
*   `content_type`: The MIME type of the blob. Important for serving files correctly.
*   `size_bytes`: The size of the blob in bytes. Useful for display, quotas, etc.
*   `is_private`: Controls visibility. Private items are typically hidden from public views or generative processes unless explicitly requested by the owner.
*   `description`: Optional user-provided text description for the file or folder.
*   `metadata`: JSONB field for system-extracted or non-user-editable metadata (e.g., image dimensions `{"width": 1024, "height": 768}`, video duration `{"duration_sec": 120.5}`, PDF page count `{"pages": 15}`).
*   `data`: JSONB field for user-defined content or metadata. For structured items (`is_blob=false`, `is_folder=false`), this holds the primary data. For blobs/folders, it can hold tags, labels, custom attributes (`{"tags": ["urgent", "review"], "priority": 1}`).
*   `parent_ids`: An array of UUIDs referencing other `db_file_objects.id`. Enables building relationships (e.g., dependencies, links) forming a Directed Acyclic Graph (DAG). Primarily useful for structured data or workflow modeling.
*   `created_at`: Timestamp of creation.
*   `updated_at`: Timestamp of last update.

**Indexing:**

*   Primary Key on `id`.
*   Index on `(user_id, collection_id)` for scoping queries.
*   Index on `(user_id, collection_id, path)` potentially with uniqueness constraint for path integrity within a collection.
*   GIN index on `path gin_trgm_ops` using `pg_trgm` extension for efficient prefix (`LIKE 'folder/%'`) and substring searches on paths (virtual folder navigation).
*   Index on `is_folder` and `item_type` for common filtering.
*   Index on `is_private` for filtering public/private content.
*   GIN index on `parent_ids` for querying relationships (`WHERE id = ANY(parent_ids)` or `WHERE parent_id = ANY(...)`).
*   Optional GIN index on `data` if querying arbitrary JSON fields is frequent (`data @> '{"key": "value"}'`).
*   Optional GIN index on `metadata` if querying system metadata is frequent.
*   Optional GIN index on a generated `tsvector` column (e.g., `data_tsv`) for full-text search across `description` and `data`. Definition example:
    ```sql
    -- Add the tsvector column
    ALTER TABLE db_file_objects
    ADD COLUMN data_tsv tsvector
    GENERATED ALWAYS AS (
        to_tsvector('simple', coalesce(description, '') || ' ' || coalesce(data::text, '')) -- Combine description and stringified data
    ) STORED;

    -- Index the tsvector column
    CREATE INDEX idx_file_objects_data_tsv ON db_file_objects USING GIN (data_tsv);
    ```

## 3. Key Concepts & Implementation Details

### 3.1. Virtual Folders

*   **Mechanism:** The `path` column simulates hierarchy. A path like `"projects/alpha/design.png"` implies a folder structure.
*   **Folder Representation:** Items with `is_folder = true` represent the folders themselves. Their `path` should typically end with a `/` (e.g., `"projects/alpha/"`). They can have their own `description`, `data` (e.g., tags), but usually `is_blob=false`.
*   **Querying:** Use `LIKE` with the `pg_trgm` index for efficient listing:
    *   List immediate children of `projects/alpha/`: `WHERE path LIKE 'projects/alpha/%' AND path NOT LIKE 'projects/alpha/%/%'`
    *   List all descendants: `WHERE path LIKE 'projects/alpha/%'`
*   **Path Management:** Application logic must ensure path consistency (e.g., managing uniqueness, handling renames/moves requires updating paths of descendants). Using UUIDs or stable identifiers in paths is generally discouraged; keep paths human-readable but ensure uniqueness within the scope.

### 3.2. GCS Integration

*   **Workflow:**
    1.  **Upload:** Client gets a signed URL from the backend to upload directly to GCS.
    2.  **Metadata Creation:** After successful GCS upload, the client (or backend) creates/updates the `db_file_objects` record with `is_blob=true`, `gcs_path`, `content_type`, `size_bytes`, and other relevant metadata.
    3.  **Download:** Backend generates a signed GCS URL for the `gcs_path` when a client requests access to the blob.
    4.  **Deletion:** Deleting a `db_file_objects` record with `is_blob=true` should trigger a background job or use a GCS lifecycle rule to delete the corresponding object from the GCS bucket.
*   **`gcs_path` Strategy:** Use a predictable but unique path, e.g., `users/<user_id>/collections/<collection_id>/<object_id>.<ext>` or `objects/<object_id_hash>`. Avoid storing the bucket name in the database; configure it in the application.

### 3.3. Privacy (`is_private`)

*   **Purpose:** Control access in specific contexts. Default to `false` (publicly usable within the application's context).
*   **Enforcement:** Application logic must filter based on `is_private` where necessary:
    *   Public galleries/listings: `WHERE is_private = false`
    *   Generative AI inputs: `WHERE is_private = false`
    *   User's private workspace: Show all items (`is_private = true` or `false`).
    *   APIs: May require specific permissions or flags to access private items.

### 3.4. Relationships (`parent_ids`)

*   **Use Cases:** Linking related documents, defining dependencies in workflows, associating data points.
*   **Querying:** Use array operators and the GIN index:
    *   Find items that have `item_x` as a parent: `WHERE $item_x_id = ANY(parent_ids)`
    *   Find parents of `item_y`: Requires fetching `item_y` first to get its `parent_ids`, then querying `WHERE id = ANY($item_y_parent_ids)`.
*   **Graph Traversal:** Use Recursive Common Table Expressions (CTEs) in SQL for complex graph queries if needed.

### 3.5 Full-Text Search

*   **Mechanism:** Use a generated `tsvector` column indexed with GIN. Combine relevant text fields (`description`, stringified `data`) into the vector.
*   **Querying:** Use `@@ to_tsquery(...)`. Example: `WHERE data_tsv @@ to_tsquery('simple', 'report & quarterly')`.
*   **Language:** Choose the appropriate text search configuration (e.g., `'simple'` for basic tokenization, `'english'` for stemming).

## 4. Rust Implementation Considerations

*   **ORM/Query Builder:** Use `sqlx` for type-safe SQL interaction.
*   **Struct Definitions:** Define Rust structs corresponding to the tables, using `sqlx::FromRow`, `serde::{Serialize, Deserialize}`, `uuid::Uuid`, `chrono::{DateTime, Utc}`, `serde_json::Value`. Add `utoipa::ToSchema` and necessary annotations if exposing via OpenAPI.

    ```rust
    // Example for db_collections
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct Collection {
        #[schema(value_type = String, format = "uuid")]
        pub id: uuid::Uuid,
        #[schema(value_type = String, format = "uuid")]
        pub user_id: uuid::Uuid,
        pub name: String,
        pub description: Option<String>,
        pub metadata: Option<serde_json::Value>,
        #[schema(value_type = String, format = "date-time")]
        pub created_at: chrono::DateTime<chrono::Utc>,
        #[schema(value_type = String, format = "date-time")]
        pub updated_at: chrono::DateTime<chrono::Utc>,
    }

    // Example for db_file_objects
    #[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub struct FileObject {
        #[schema(value_type = String, format = "uuid")]
        pub id: uuid::Uuid,
        #[schema(value_type = String, format = "uuid")]
        pub user_id: uuid::Uuid,
        #[schema(value_type = String, format = "uuid")]
        pub collection_id: uuid::Uuid,
        pub path: String,
        pub is_folder: bool,
        pub item_type: Option<String>,
        pub is_blob: bool,
        pub gcs_path: Option<String>,
        pub content_type: Option<String>,
        pub size_bytes: Option<i64>, // Use i64 for potentially large files
        pub is_private: bool,
        pub description: Option<String>,
        pub metadata: Option<serde_json::Value>,
        pub data: Option<serde_json::Value>,
        #[schema(value_type = Vec<String>, format = "uuid")]
        pub parent_ids: Vec<uuid::Uuid>, // sqlx maps UUID[] to Vec<Uuid>
        // data_tsv: Option<String>, // tsvector might map differently or not be directly mapped
        #[schema(value_type = String, format = "date-time")]
        pub created_at: chrono::DateTime<chrono::Utc>,
        #[schema(value_type = String, format = "date-time")]
        pub updated_at: chrono::DateTime<chrono::Utc>,
    }
    ```
*   **GCS Client:** Use a crate like `google-cloud-storage` to interact with GCS for generating signed URLs and potentially managing uploads/deletions.
*   **Path Handling:** Implement robust path validation and manipulation logic, potentially in a dedicated utility module. Ensure canonical forms (e.g., leading/trailing slashes).
*   **API Design:** Design API endpoints carefully to handle creation, listing (with path prefix filtering), updating, and deletion of collections and file objects. Consider separate endpoints or parameters for blob uploads. Use `ReqData<Claims>` to get `user_id` for scoping queries.

## 5. Future Considerations

*   **JSON Schema Validation:** Introduce a `db_schemas` table and link `db_collections` or `db_file_objects` (for structured `data`) to enforce specific JSON structures.
*   **Versioning:** Add versioning to `db_file_objects` if tracking changes over time is required.
*   **Advanced Permissions:** Implement more granular Role-Based Access Control (RBAC) beyond the simple `is_private` flag.
*   **Materialized Paths/Nested Sets:** For very deep or frequently traversed hierarchies, consider alternative tree representations if virtual paths become a bottleneck (though unlikely with trigram indexing for moderate depths).
