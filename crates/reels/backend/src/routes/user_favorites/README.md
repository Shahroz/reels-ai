# User Favorites API

This module provides CRUD operations for user favorites, allowing users to mark entities (styles, creatives, documents) as favorites.

## Database Schema

The favorites are stored in the `user_favorites` table with the following structure:

```sql
CREATE TABLE user_favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    entity_id UUID NOT NULL,
    entity_type favorite_entity_type NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_user_favorites_identity UNIQUE (user_id, entity_id, entity_type)
);
```

The `favorite_entity_type` enum supports:
- `style`
- `creative` 
- `document`

## API Endpoints

### POST /api/user-favorites
Create a new favorite for the authenticated user.

**Request Body:**
```json
{
  "entity_id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "entity_type": "creative"
}
```

**Response:**
```json
{
  "id": "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy",
  "user_id": "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz",
  "entity_id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "entity_type": "creative",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### GET /api/user-favorites
List favorites for the authenticated user with entity relation data and optional filtering.

**Query Parameters:**
- `page` (optional): Page number (default: 1)
- `limit` (optional): Items per page (default: 10, max: 100)
- `entity_type` (optional): Filter by entity type (style, creative, document)
- `sort_by` (optional): Field to sort by (created_at, updated_at)
- `sort_order` (optional): Sort order (asc, desc)

**Response:**
```json
{
  "items": [
    {
      "id": "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy",
      "user_id": "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz",
      "entity_id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "entity_type": "creative",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z",
      "entity_data": {
        "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
        "collection_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "creative_format_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "style_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "document_ids": ["doc1", "doc2"],
        "asset_ids": ["asset1", "asset2"],
        "html_url": "https://example.com/creative.html",
        "draft_url": null,
        "bundle_id": null,
        "screenshot_url": "https://example.com/screenshot.jpg",
        "is_published": true,
        "publish_url": "https://example.com/published.html",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
      }
    }
  ],
  "total_count": 1
}
```

**Entity Data Behavior:**
- If `entity_type` filter is provided, only entities of that type will have their data populated
- If no filter is provided, all entity types will have their data populated
- Entity data will be `null` if:
  - The entity doesn't exist
  - The entity type doesn't match the filter

**Entity Data Types:**

**Style Entity:**
```json
{
  "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "user_id": "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz",
  "name": "My Style",
  "html_url": "https://example.com/style.html",
  "screenshot_url": "https://example.com/screenshot.jpg",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

**Creative Entity:**
```json
{
  "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "collection_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
  "creative_format_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
  "style_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
  "document_ids": ["doc1", "doc2"],
  "asset_ids": ["asset1", "asset2"],
  "html_url": "https://example.com/creative.html",
  "draft_url": null,
  "bundle_id": null,
  "screenshot_url": "https://example.com/screenshot.jpg",
  "is_published": true,
  "publish_url": "https://example.com/published.html",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

**Document Entity:**
```json
{
  "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "user_id": "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz",
  "title": "My Document",
  "content": "Document content...",
  "sources": ["source1", "source2"],
  "status": "completed",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z",
  "is_public": true,
  "is_task": false,
  "include_research": "task_dependent"
}
```

### DELETE /api/user-favorites/{favorite_id}
Delete a specific favorite by ID.

**Response:** 204 No Content

### POST /api/user-favorites/toggle
Toggle a favorite (add if not exists, remove if exists).

**Request Body:**
```json
{
  "entity_id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "entity_type": "creative"
}
```

**Response:**
```json
{
  "is_favorite": true,
  "favorite": {
    "id": "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy",
    "user_id": "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz",
    "entity_id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "entity_type": "creative",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

## Integration with Creatives API

The creatives list and detail endpoints now include an `is_favorite` field:

### GET /api/creatives
Each creative item in the response includes:
```json
{
  "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  // ... other fields ...
  "is_favorite": true
}
```

### GET /api/creatives/{id}
The creative detail response includes:
```json
{
  "creative": {
    "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    // ... other fields ...
  },
  "is_favorite": true,
  // ... other related data ...
}
```

## Error Responses

All endpoints return standard error responses:

- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Missing or invalid authentication
- `403 Forbidden`: User cannot perform the action
- `404 Not Found`: Entity or favorite not found
- `409 Conflict`: Favorite already exists (for create endpoint)
- `500 Internal Server Error`: Server error

## Security

- All endpoints require JWT authentication
- Users can only manage their own favorites
- Entity existence is verified before creating favorites
- Access control is enforced for entity types
- Entity data is returned if the entity exists, regardless of sharing permissions 