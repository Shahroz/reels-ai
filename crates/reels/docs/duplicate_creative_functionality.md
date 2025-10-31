# Creative Duplication Functionality Plan

This document outlines the plan for implementing a "Duplicate Creative" feature. This will allow users to create a personal, editable copy of any creative they have access to.

## 1. Overview

The goal is to provide a one-click action for users to duplicate a creative. This involves creating a new, independent creative entity in the database with copies of all its associated data and assets. The new creative will be owned by the user who performs the duplication.

## 2. Backend Implementation

The backend will expose a new API endpoint to handle the duplication logic.

### 2.1. New API Endpoint

-   **Endpoint:** `POST /api/creatives/{id}/duplicate`
-   **Method:** `POST`
-   **Authentication:** Required (via `JwtMiddleware`).
-   **Description:** Creates a new creative as a copy of the specified creative.

### 2.2. Handler Logic

The handler for this endpoint will perform the following steps:

1.  **Authorization:**
    -   Accept a `creative_id` from the path and `Claims` from the JWT.
    -   Verify that the requesting user has at least `viewer` access to the original creative. Access can be through ownership, a direct share, an organization share, or because the original is public. A user does not need to be an editor to make their own copy.
2.  **Fetch Original Creative:**
    -   Retrieve the full record of the original creative, including its `style_id`, `asset_ids`, `document_ids`, and `creative_format_id`.
3.  **Asset Duplication (GCS):**
    -   Generate a new `UUID` for the duplicated creative.
    -   Copy the HTML content from the original creative's `html_url` to a new GCS path corresponding to the new creative's ID (e.g., `/creatives/{new_id}/creative.html`).
    -   Generate a new screenshot for the new HTML and upload it to a new GCS path (e.g., `/creatives/{new_id}/screenshot.png`).
4.  **Database Insertion:**
   -   Create a new record in the `creatives` table with the following attributes:
       -   `id`: The new `UUID`.
       -   `name`: `{original_name} (COPY)`.
       -   `html_url` and `screenshot_url`: The new GCS URLs.
       -   `is_published`: `false`. The new copy is a draft.
       -   `draft_url`: Should be set to the new `html_url`.
       -   `style_id`, `asset_ids`, `document_ids`, `creative_format_id`, `collection_id`: Copied directly from the original creative.
5.  **Response:**
    -   Return the full `CreativeResponse` object for the newly created creative with a `201 Created` status.

### 2.3. New Database Query Files

To maintain consistency with existing patterns (e.g., `copy_document`), the following new query files should be created in `crates/narrativ/backend/src/queries/creatives/`:

-   `fetch_creative_for_duplication.rs`: A query to fetch the original creative's data and verify the user's access rights.
-   `insert_creative_duplicate.rs`: A query to insert the new creative record into the database.

## 3. Frontend Implementation

The frontend will be updated to expose this new functionality on the `CreativeCard`.

### 3.1. API Service Update

-   **File:** `crates/narrativ/frontend/src/api/services/CreativesService.ts`
-   **Action:** Add a new static method `duplicateCreative(creativeId: string)` that calls the `POST /api/creatives/{id}/duplicate` endpoint.

### 3.2. CreativeCard Component Update

-   **File:** `crates/narrativ/frontend/src/features/creatives/components/CreativeCard.tsx`
-   **Action:** Add a new action button to the card's UI.

#### 3.2.1. Icon Selection

To avoid confusion with existing "copy" actions, a new, distinct icon will be used.

-   **Current Icons:**
    -   `IconCopy` (Copy URL)
    -   `CopyPlusIcon` (Regenerate)
-   **Proposed Icon:**
    -   **`IconBoxMultiple`** from `@tabler/icons-react`. This icon visually represents creating multiple items from a single source, which is a good metaphor for duplication.

#### 3.2.2. New Action Button

-   A new `<Button>` component will be added inside the action button group in `CreativeCard.tsx`.
-   It will use the `IconBoxMultiple` icon.
-   The `title` and `aria-label` will be "Duplicate".
-   The `onClick` handler will call a new prop function, e.g., `onDuplicate(creative.id)`.
-   **Permissions:** The button will be visible to any user who can view the creative (i.e., if the card is visible to them).

### 3.3. State Management / Context

-   The parent component (`CreativesFeature`) will implement the `onDuplicate` handler.
-   This handler will call `CreativesService.duplicateCreative`, and on success, it should either show a toast notification ("Creative duplicated successfully!") and/or refresh the list of creatives to show the new copy.

## 4. User Flow

1.  A user sees a creative in their list that they want to copy.
2.  They hover over the creative card to reveal the action icons.
3.  They click the "Duplicate" button (`IconBoxMultiple`).
4.  The system calls the backend, creates a new creative, and copies the assets.
5.  A success toast appears.
6.  The list of creatives refreshes, and the user now sees "(COPY) {original_name}" at the top of their list (when sorted by creation date), which they can now edit independently of the original.
