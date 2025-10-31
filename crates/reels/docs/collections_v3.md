# Technical Brief: Collections & File Objects Database Model (v3)

## 1. Overview

This document details the design for a flexible, user-centric database model (v3) to manage collections, structured data items, and unstructured file objects (blobs). It aims to provide:

*   **User-Specific Collections:** Logical grouping of items under a user account.
*   **Defined Structured Data:** Collections can define a JSON Schema for their structured items, which are stored separately.
*   **Flexible File Storage:** Support for unstructured file objects (documents, images, videos) linked to Google Cloud Storage (GCS).
*   **Unified Virtual Hierarchy:** A single table (`db_file_objects`) manages a user-friendly virtual folder structure containing folders, unstructured files, and references to structured items.
*   **Relationship Modeling:** Support for defining dependencies or relationships between items (DAG structure) using `parent_ids` within `db_file_objects`.
*   **Searchability:** Full-text search capabilities over item metadata and content.
*   **Access Control:** A basic mechanism (`is_private`) to control the visibility/usability of items.
*   **Scalability:** Design choices favoring performance, including targeted indexing and offloading large blobs to GCS.

This v3 design refines the previous model by introducing a dedicated table (`db_collection_items`) for structured data, clearly separating it from unstructured file/blob management. Collections (`db_collections`) now define a `schema` for their structured items. Both structured items (referenced via `db_file_objects`) and unstructured files/folders exist within the unified virtual hierarchy managed by `db_file_objects`.

## 2. Database Schema

### 2.1. `db_collections` Table

Represents a named logical group owned by a user. Acts as a container defining the schema for structured items (`db_collection_items`) and grouping file objects (`db_file_objects`) within a virtual hierarchy.

```sql
CREATE TABLE db_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE, -- Assuming a 'users' table exists
    name TEXT NOT NULL,
    description TEXT,
    schema JSONB NOT NULL,                     -- JSON schema definition for structured items in this collection
    metadata JSONB,                           -- General purpose metadata for the collection itself
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
*   `user_id`: Foreign key linking to the owner user.
*   `name`: Human-readable name of the collection (unique per user recommended via application logic or constraint).
*   `description`: Optional text describing the collection's purpose.
*   `schema`: **Mandatory** JSONB field storing the JSON Schema document that defines the structure of structured items stored in `db_collection_items` associated with this collection.
*   `metadata`: Optional JSONB field for storing arbitrary metadata about the collection itself.
*   `created_at`: Timestamp of creation.
*   `updated_at`: Timestamp of the last update.

**Indexing:**

*   Primary Key on `id`.
*   Index on `user_id`.
*   Optional unique constraint on `(user_id, name)`.
*   Optional GIN index on `metadata`.

### 2.2. `db_collection_items` Table

Stores the actual structured data conforming to the schema defined in the parent `db_collections` table. Each item also has a corresponding entry in `db_file_objects` for its place in the virtual hierarchy.

```sql
CREATE TABLE db_collection_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collection_id UUID NOT NULL REFERENCES db_collections(id) ON DELETE CASCADE,
    -- No user_id here, implicitly linked via collection_id or file_object_id
    data JSONB NOT NULL,                      -- The structured data itself, conforming to db_collections.schema
    -- file_object_id UUID NOT NULL UNIQUE REFERENCES db_file_objects(id) ON DELETE CASCADE, -- Link to hierarchy entry
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
    -- Optional: May add specific indexed fields extracted from 'data' if needed for performance
);

-- Auto-update 'updated_at' timestamp trigger
CREATE TRIGGER trg_collection_items_set_updated_at
BEFORE UPDATE ON db_collection_items
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
```

**Columns:**

*   `id`: Unique identifier for the structured data item.
*   `collection_id`: Foreign key linking to the parent collection (and its schema).
*   `data`: **Mandatory** JSONB field holding the structured data instance. Must validate against `db_collections.schema`.
*   `created_at`: Timestamp of creation.
*   `updated_at`: Timestamp of last update.

**Indexing:**

*   Primary Key on `id`.
*   Index on `collection_id`.
*   GIN index on `data` for querying within the JSON structure.
*   Consider creating expression indexes on specific paths within `data` if frequently queried/filtered (e.g., `CREATE INDEX idx_items_data_field ON db_collection_items ((data->>'fieldName'))`).

### 2.3. `db_file_objects` Table

Stores metadata for *all* entries within a collection's virtual hierarchy: folders, unstructured files (blobs), and references to structured items (stored in `db_collection_items`).

```sql
CREATE TABLE db_file_objects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id UUID NOT NULL REFERENCES db_collections(id) ON DELETE CASCADE,

    -- Virtual structure & Identification
    path TEXT NOT NULL,               -- Virtual path, e.g., "projects/report.pdf" or "data/customer_1/" or "structured/item_abc"
    item_type TEXT NOT NULL,          -- Discriminator: 'folder', 'blob', 'structured_item'
    collection_item_id UUID NULL UNIQUE REFERENCES db_collection_items(id) ON DELETE SET NULL, -- Link to structured data IF item_type='structured_item'

    -- Blob Storage (for item_type = 'blob')
    gcs_path TEXT,                    -- Path/key within the GCS bucket
    content_type TEXT,                -- MIME type (e.g., 'image/png', 'application/pdf')
    size_bytes BIGINT,                -- Size of the blob in bytes

    -- Access Control
    is_private BOOLEAN DEFAULT false, -- If true, excluded from public listings or generative use cases

    -- Descriptive & Content Metadata
    name TEXT NOT NULL,               -- Display name for the object (extracted from path or user-defined)
    description TEXT,                 -- User-provided description
    metadata JSONB,                   -- System-extracted (e.g., image dimensions) or user-defined metadata (e.g., tags)

    -- Relationships (Applies to all item types)
    parent_ids UUID[] DEFAULT '{}',   -- IDs of other db_file_objects this item relates to (DAG)

    -- Full-Text Search Vector (Optional)
    -- data_tsv tsvector GENERATED ALWAYS AS (...) STORED,

    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now(),

    -- Constraint to ensure structured items link correctly
    CONSTRAINT chk_structured_item_link CHECK (
        (item_type = 'structured_item' AND collection_item_id IS NOT NULL) OR
        (item_type != 'structured_item' AND collection_item_id IS NULL)
    ),
    -- Constraint to ensure blobs have GCS info
    CONSTRAINT chk_blob_info CHECK (
        (item_type = 'blob' AND gcs_path IS NOT NULL AND content_type IS NOT NULL AND size_bytes IS NOT NULL) OR
        (item_type != 'blob') -- Allow NULLs for non-blobs
    )
);

-- Auto-update 'updated_at' timestamp trigger (reuse function)
CREATE TRIGGER trg_file_objects_set_updated_at
BEFORE UPDATE ON db_file_objects
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Enable trigram extension for path searching
CREATE EXTENSION IF NOT EXISTS pg_trgm;
```

**Columns:**

*   `id`: Unique identifier for the file object entry (represents a node in the hierarchy).
*   `user_id`: Owning user.
*   `collection_id`: Parent collection.
*   `path`: The virtual path used to simulate folder structure. Unique within `(user_id, collection_id)`.
*   `item_type`: **Mandatory** discriminator: 'folder', 'blob', or 'structured_item'.
*   `collection_item_id`: Foreign key to `db_collection_items.id`. **Only** set if `item_type` is 'structured_item'. Links this hierarchy node to the actual structured data. `UNIQUE` constraint ensures one hierarchy node per structured item. `ON DELETE SET NULL` means if the structured data is deleted, this node might become an orphan (or could be `CASCADE` deleted).
*   `gcs_path`, `content_type`, `size_bytes`: Blob storage details, relevant only when `item_type = 'blob'`.
*   `is_private`: Controls visibility/usability.
*   `name`: The display name of the object in the hierarchy (e.g., filename, folder name, structured item name). Often derived from the last part of `path`.
*   `description`: Optional user-provided text description.
*   `metadata`: JSONB field for system-extracted or user-defined metadata (tags, labels, image dimensions, etc.). For `structured_item` types, this might store summary info or metadata *about* the item, not the item data itself.
*   `parent_ids`: Array of `db_file_objects.id` for relationship modeling (DAG).
*   `created_at`, `updated_at`: Timestamps.

**Indexing:**

*   Primary Key on `id`.
*   Index on `(user_id, collection_id)`.
*   Unique index on `(user_id, collection_id, path)`.
*   GIN index on `path gin_trgm_ops` using `pg_trgm` for path prefix/substring searches.
*   Index on `item_type`.
*   Index on `collection_item_id` (especially useful due to `UNIQUE` constraint).
*   Index on `is_private`.
*   GIN index on `parent_ids`.
*   Optional GIN index on `metadata`.
*   Optional FTS index (consider indexing `name`, `description`, potentially `metadata`).

## 3. Key Concepts & Implementation Details

### 3.1. Unified Virtual Hierarchy

*   **Mechanism:** The `db_file_objects` table represents *every* node in the user's virtual filesystem for a collection, regardless of type. The `path` column and `item_type` define the structure and nature of each node.
*   **Folders:** Represented by `item_type = 'folder'`. Path typically ends in `/`.
*   **Blobs:** Represented by `item_type = 'blob'`. Linked to GCS via `gcs_path`.
*   **Structured Items:** Represented by `item_type = 'structured_item'`. Linked to the actual data in `db_collection_items` via `collection_item_id`. The `db_file_objects` entry stores metadata *about* the structured item (name, description, path, relationships) but not the primary data itself.
*   **Querying Hierarchy:** Use `LIKE` with `pg_trgm` on `path` as before. Filter by `item_type` as needed.

### 3.2. Structured Data Flow

1.  **Collection Creation:** User creates a `db_collections` entry, providing a `name` and the defining `schema` (JSON Schema).
2.  **Item Creation:**
    *   User provides the structured `data` (JSON) and desired virtual `path` and `name`.
    *   Backend validates the provided `data` against the `db_collections.schema`.
    *   If valid:
        *   Create a record in `db_collection_items` with the `data` and `collection_id`.
        *   Create a record in `db_file_objects` with `item_type='structured_item'`, the provided `path` and `name`, the `collection_id`, `user_id`, and the `collection_item_id` referencing the new `db_collection_items` record.
3.  **Listing/Browsing:** Queries against `db_file_objects` show all items in the hierarchy. When a 'structured_item' is selected, the application uses its `collection_item_id` to fetch the full data from `db_collection_items`.

### 3.3. GCS Integration, Privacy, Relationships, FTS

*   These concepts remain largely the same as described in v2, but apply within the context of the `db_file_objects` table structure.
*   GCS integration applies only to `db_file_objects` entries with `item_type = 'blob'`.
*   `is_private` applies to the hierarchy node in `db_file_objects`, controlling visibility of folders, blobs, and structured item references. Access to the actual structured data in `db_collection_items` might require separate checks or rely solely on the visibility of its corresponding `db_file_objects` entry.
*   `parent_ids` in `db_file_objects` can link any item types together.
*   FTS can be configured on `db_file_objects` (indexing `name`, `description`, `metadata`) and potentially separately on `db_collection_items` (indexing `data`).

## 4. Rust Implementation Considerations

*   Update Rust structs to reflect the three tables (`Collection`, `CollectionItem`, `FileObject`).
*   Ensure `FileObject` struct handles the conditional fields based on `item_type` (e.g., `collection_item_id` is `Option<Uuid>`, GCS fields are `Option<String>`, etc.).
*   Implement validation logic for `data` in `CollectionItem` against the `schema` in `Collection` upon creation/update.
*   Adjust API endpoints:
    *   Collection creation requires `schema`.
    *   Item creation needs logic to handle the three `item_type` cases (folder, blob, structured_item), potentially involving interactions with both `db_file_objects` and `db_collection_items` for structured items.
    *   Listing endpoints query `db_file_objects`.
    *   Fetching item details might involve joining `db_file_objects` with `db_collection_items` or performing separate lookups based on `item_type` and `collection_item_id`.

```rust
// Example Rust Structs (Illustrative)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;
use sqlx::FromRow;
use utoipa::ToSchema;

// Matches db_collections
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Collection {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema: Value, // JSONB maps to serde_json::Value
    pub metadata: Option<Value>,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

// Matches db_collection_items
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct CollectionItem {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub collection_id: Uuid,
    pub data: Value, // The actual structured data
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

// Matches db_file_objects
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct FileObject {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub user_id: Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub collection_id: Uuid,
    pub path: String,
    pub item_type: String, // Consider an enum: Folder, Blob, StructuredItem
    #[schema(value_type = Option<String>, format = "uuid")]
    pub collection_item_id: Option<Uuid>, // Link to CollectionItem if item_type='structured_item'
    pub gcs_path: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub is_private: bool,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
    #[schema(value_type = Vec<String>, format = "uuid")]
    pub parent_ids: Vec<Uuid>, // sqlx maps UUID[] to Vec<Uuid>
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

```

## 5. Future Considerations

*   **Schema Evolution:** How to handle changes to `db_collections.schema` when `db_collection_items` already exist. Might require versioning schemas or data migration strategies.
*   **Advanced Validation:** Integrate more robust JSON Schema validation libraries in the backend.
*   **Performance Tuning:** Monitor query performance, especially joins between `db_file_objects` and `db_collection_items` and queries involving JSONB `data`. Add specific expression indexes as needed.
*   **Cross-Collection References:** Current design scopes `parent_ids` within `db_file_objects`. Referencing items across collections would require adjustments.
