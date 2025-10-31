# Studio Journey Sharing: Implementation Plan

This document outlines the implementation plan for a feature that allows users to share their Studio Journeys via a public, read-only link.

## 1. Feature Overview

The goal is to enable users to generate a secret link for one of their Studio Journeys. Anyone with this link can view the journey's visual history in a simplified, full-screen, read-only interface without needing to log in. The journey owner will have controls to create and revoke this link.

## 2. Backend Implementation

### 2.1. Database Schema

A new table, `studio_journey_shares`, will be created to manage sharing state. This approach is preferred over modifying the `studio_journeys` table to keep concerns separate and allow for future enhancements (e.g., different access levels, expiration dates).

**Migration File:** `crates/narrativ/backend/migrations/YYYYMMDDHHMMSS_create_studio_journey_shares.sql`

```sql
-- Purpose: Create the studio_journey_shares table for public journey sharing.

CREATE TABLE IF NOT EXISTS public.studio_journey_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journey_id UUID NOT NULL REFERENCES public.studio_journeys(id) ON DELETE CASCADE,
    share_token UUID NOT NULL DEFAULT gen_random_uuid(),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_journey_id UNIQUE (journey_id), -- A journey can only have one share link at a time
    CONSTRAINT uq_share_token UNIQUE (share_token)
);

CREATE INDEX IF NOT EXISTS idx_studio_journey_shares_journey_id ON public.studio_journey_shares(journey_id);
CREATE INDEX IF NOT EXISTS idx_studio_journey_shares_share_token ON public.studio_journey_shares(share_token);

-- Trigger to update 'updated_at' timestamp
CREATE TRIGGER trigger_studio_journey_shares_updated_at
BEFORE UPDATE ON public.studio_journey_shares
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE public.studio_journey_shares IS 'Manages public, read-only share links for Studio Journeys.';
COMMENT ON COLUMN public.studio_journey_shares.share_token IS 'The secret token used in the public URL to access the journey.';
```

### 2.2. API Endpoints

#### 2.2.1. Authenticated Endpoints (for Sharer)

These endpoints will be grouped under a new scope, e.g., `/api/studio/journeys/{journey_id}/share`. They will be protected by `JwtMiddleware` and will perform ownership checks to ensure the requesting user owns the journey.

*   **`POST /api/studio/journeys/{journey_id}/share`**
    *   **Description:** Creates or re-activates a share link for a journey. This will be an upsert operation. If a share record exists for the journey, it will set `is_active = true`. If not, it will create one.
    *   **Authorization:** User must be the owner of the `journey_id`.
    *   **Response (200 OK or 201 Created):**
        ```json
        {
          "share_token": "a1b2c3d4-e5f6-7890-1234-567890abcdef"
        }
        ```

*   **`GET /api/studio/journeys/{journey_id}/share`**
    *   **Description:** Retrieves the current sharing status and token for a journey.
    *   **Authorization:** User must be the owner of the `journey_id`.
    *   **Response (200 OK):**
        ```json
        {
          "share_token": "a1b2c3d4-e5f6-7890-1234-567890abcdef",
          "is_active": true
        }
        ```
    *   **Response (404 Not Found):** If no share link has ever been created.

*   **`DELETE /api/studio/journeys/{journey_id}/share`**
    *   **Description:** Deactivates the share link by setting `is_active = false`. We prefer soft deletion to allow reactivation with the same link.
    *   **Authorization:** User must be the owner of the `journey_id`.
    *   **Response:** `204 No Content`.

#### 2.2.2. Public Endpoint (for Viewer)

This endpoint will be unauthenticated and will provide the necessary data for the read-only view.

*   **`GET /api/public/journeys/view/{share_token}`**
    *   **Description:** Fetches the data for a publicly shared journey.
    *   **Authorization:** None.
    *   **Logic:** The handler will find the `studio_journey_shares` record by `share_token`. If found and `is_active` is true, it will fetch the corresponding journey and its nodes.
    *   **Response (200 OK):** The response payload will be a simplified version of the journey, excluding any sensitive user or account information.
        ```json
        {
          "name": "My Awesome Journey",
          "nodes": [
            {
              "id": "node-uuid-1",
              "name": "Initial Image",
              "url": "https://path/to/image1.png",
              "parentId": null
            },
            {
              "id": "node-uuid-2",
              "name": "Edit 1: Brighter",
              "url": "https://path/to/image2.png",
              "parentId": "node-uuid-1"
            }
          ]
        }
        ```
    *   **Response (404 Not Found):** If the token is invalid or inactive.

## 3. Frontend Implementation

### 3.1. Sharer UI (Inside Studio)

*   **New API Service:** Create `frontend/src/api/services/StudioJourneySharesService.ts` to interact with the new authenticated endpoints ---- IMPORTANT THIS IS AUTOGENERATED AND SHOULD NOT BE EDITED OR ADDED DIRECTLY - FIRST THE BACKEND NEEDS TO BE IMPLEMENTED AND openapi.rs specs updated then I regenerated the SDK

*   **Share Button:** Add a "Share" button to `StudioHeader.tsx`. This button will be enabled only when a journey is active.

*   **Share Modal Component:**
    *   Create a new component, e.g., `frontend/src/features/studio/components/ShareJourneyModal.tsx`.
    *   This modal will be triggered by the "Share" button.
    *   **State:** It will fetch the journey's share status using the `GET` endpoint.
    *   **UI:**
        *   If no link exists, it shows a "Create public link" button.
        *   If a link exists, it displays the full public URL (`https://app.narrativ.com/journeys/view/{token}`) with a "Copy" button.
        *   It will include a toggle or button to activate/deactivate the link (`POST` and `DELETE` endpoints).
        *   A clear disclaimer will state that anyone with the link can view the journey.

### 3.2. Public Viewer UI

*   **New Public Route:**
    *   Create a new file-based route: `frontend/src/routes/journeys/view/$shareToken.tsx`.
    *   This route is outside the `/_authenticated` directory, so it will not use the standard application layout (sidebar, header) and will not trigger the authentication `beforeLoad` check.

*   **Public Journey Viewer Component:**
    *   Create a new feature component: `frontend/src/features/studio/PublicJourneyViewer.tsx`.
    *   **Data Fetching:** On load, it will use the `shareToken` from the URL to call the public API endpoint (`/api/public/journeys/view/{share_token}`). It should handle loading and error states (e.g., display "Journey not found").
    *   **Layout:** The component will render a minimal, full-screen layout.
    *   **Content:**
        *   It will display the journey name as a title.
        *   It will reuse the `HistoryPanel.tsx` component from the main Studio feature.
        *   All interactive elements on the `HistoryPanel` (clicking to select, etc.) will be disabled. The panel will be for viewing only.
        *   No other Studio controls (chat input, tools panel, header actions) will be rendered.

## 4. Security Considerations

1.  **Token Entropy:** The `share_token` will be a UUID, making it cryptographically unguessable.
2.  **Data Exposure:** The public API endpoint (`/api/public/journeys/view/{share_token}`) must be carefully designed to only return non-sensitive data required for rendering the journey. No user IDs, emails, or other PII should be included.
3.  **Ownership:** All authenticated endpoints must rigorously check that the `user_id` from the JWT claims is the owner of the `studio_journey`.

## 5. Task Breakdown

1.  **Backend:**
    *   [ ] Create database migration for `studio_journey_shares` table.
    *   [ ] Implement DB queries for managing shares.
    *   [ ] Implement authenticated routes (`POST`, `GET`, `DELETE`) for share management.
    *   [ ] Implement the public route (`GET`) for viewing a shared journey.
    *   [ ] Add new routes to `openapi.rs` and update UTOIPA schemas.

2.  **Frontend:**
    *   [ ] Generate the new API service client for Studio Journey Shares.
    *   [ ] Add a "Share" icon/button to `StudioHeader.tsx`.
    *   [ ] Implement the `ShareJourneyModal.tsx` component with logic for creating, viewing, and deactivating links.
    *   [ ] Create the new public route file (`/journeys/view/$shareToken.tsx`).
    *   [ ] Implement the `PublicJourneyViewer.tsx` component for the read-only view.
    *   [ ] Style the public view to be clean and full-screen.

3.  **Testing:**
    *   [ ] Add backend tests for API endpoints (auth checks, correct data retrieval).
    *   [ ] Add frontend tests for the Share Modal interaction.
    *   [ ] Perform E2E testing of the entire flow: create link -> open in incognito window -> view journey -> revoke link -> verify inaccessibility.