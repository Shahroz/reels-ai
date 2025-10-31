# Get or Create User Collection by Predefined Collection ID

## Overview

This API endpoint allows users to fetch their collection based on a predefined collection ID. If the user doesn't have a collection for that predefined collection, it automatically creates one.

## Endpoint

```
GET /api/user-db-collections/by-predefined/{predefined_collection_id}
```

## Parameters

- `predefined_collection_id` (path parameter): UUID of the predefined collection

## Authentication

Requires Bearer token authentication.

## Response

### Success Response (200 OK)

```json
{
  "id": "uuid",
  "user_id": "uuid",
  "name": "string",
  "description": "string or null",
  "schema_definition": {},
  "source_predefined_collection_id": "uuid or null",
  "ui_component_definition": {},
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Error Responses

- `401 Unauthorized`: Invalid or missing authentication token
- `404 Not Found`: Predefined collection not found
- `500 Internal Server Error`: Database or server error

## Behavior

1. **Check Predefined Collection**: First validates that the predefined collection exists
2. **Find Existing User Collection**: Searches for an existing user collection based on the predefined collection ID
3. **Return or Create**: 
   - If found: Returns the existing user collection
   - If not found: Creates a new user collection based on the predefined collection template and returns it

## Example Usage

```bash
curl -X GET \
  "https://api.example.com/api/user-db-collections/by-predefined/123e4567-e89b-12d3-a456-426614174000" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Notes

- The endpoint ensures users always have a collection for any valid predefined collection
- New collections are created with the same schema and UI component definitions as the predefined collection 