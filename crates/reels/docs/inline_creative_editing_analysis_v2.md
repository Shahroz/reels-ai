# Analysis: Enhancing Inline Creative Editing Experience (v2)

**Date:** 2025-05-21
**Author:** AI Assistant
**Status:** Version 2 - Revised Analysis

## 1. Introduction

This document (v2) revisits and refines the analysis for enhancing the creative editing functionality. It focuses on enabling a more interactive "inline" editing experience, emphasizing direct text manipulation via `contentEditable`, a "highlight-to-context" mechanism for LLM instructions, and the necessary backend infrastructure, including a content proxy and draft management. The goal is to achieve a WYSIWYG-like feel for simple text edits while leveraging LLMs for robust HTML integration.

## 2. Current Creative Editing Workflow

The existing creative editing process (`EditCreativeDialog.tsx` and `edit_creative.rs` backend) involves:
1.  Displaying the creative's HTML (sourced from a GCS URL, either `draft_url` or `html_url`) within an iframe.
2.  The user provides textual instructions in a separate input field.
3.  These instructions, along with the creative's current HTML content (fetched again by the backend from GCS), are sent to an LLM.
4.  The LLM returns a modified HTML string.
5.  The backend validates this HTML, saves it to GCS as a new draft (updating `draft_url`), and updates the database.
6.  The iframe in the frontend is then updated to show the new draft URL.

The primary limitation is the cross-origin nature of the iframe content (GCS domain vs. application domain), preventing direct JavaScript interaction.

## 3. Proposed Enhancements & Analysis (v2 Revisions)

### A. Direct Inline HTML Editing (via `contentEditable`)

*   **Concept:** Allow users to directly edit textual content within the iframe by making relevant HTML elements (paragraphs, headings, spans, etc.) `contentEditable`. This provides a literal, inline WYSIWYG experience for text changes, without involving complex rich text editor libraries.
*   **Feasibility:**
    *   **Same-Origin Prerequisite:** Strictly required (see Section 3.C).
    *   **Implementation Complexity:** Relatively low for enabling `contentEditable` on elements. The main complexity lies in capturing these changes and integrating them cleanly (see Section 3.D).
*   **Pros:**
    *   Highly intuitive and immediate visual feedback for text modifications.
    *   Aligns with user expectations for "inline editing."
    *   Avoids the overhead and potential HTML issues of full WYSIWYG editors.
*   **Cons:**
    *   Users could still inadvertently affect HTML structure if not managed carefully (e.g., deleting block elements).
    *   Limited to primarily textual changes; structural or style changes would still rely on LLM instructions.
*   **Recommendation:** High priority after achieving same-origin. This is the preferred method for direct user manipulation of content. Changes should be captured and processed (ideally via LLM) rather than saved directly.

### B. Highlight-to-Context for LLM Instructions

*   **Concept:** Users click or highlight an element within the iframe. Information about this element (e.g., its HTML snippet, a CSS selector) is captured and appended to the textual instruction sent to the LLM. Example: user highlights a button, types "make this red," and the system sends the instruction along with context about the specific button.
*   **Feasibility:**
    *   **Same-Origin Prerequisite:** Required.
    *   **Implementation:** Parent page attaches event listeners to the iframe's document. On event, identify the target, extract information (e.g., `outerHTML`), and integrate it into the LLM prompt in `edit_creative.rs`.
*   **Pros:**
    *   Significantly improves LLM instruction precision and reduces ambiguity.
    *   More intuitive for users to specify the target of their changes.
*   **Cons:**
    *   Generating robust selectors can be challenging; `outerHTML` might be simpler but larger.
    *   Requires careful LLM prompt engineering.
*   **Recommendation:** High priority, dependent on same-origin. Offers substantial UX improvement.

### C. Achieving Same-Origin Iframe Content (Proxy Endpoint & Draft Management)

*   **Problem:** Iframe content from GCS is cross-origin.
*   **Proposed Solution:** Implement a backend proxy endpoint (e.g., `/api/creatives/{creative_id}/content`).
    1.  The frontend `EditCreativeDialog` sets the `iframeUrl` to this backend endpoint.
    2.  The backend endpoint:
        *   Authenticates the request (user session/claims).
        *   Authorizes access (user owns/can access `creative_id`).
        *   **Draft/Published Logic:** Fetches the creative record. If `draft_url` exists and is valid, it fetches HTML content from `draft_url`. Otherwise, it fetches from `html_url`.
        *   Serves the (modified) HTML with `Content-Type: text/html`.
*   **"Discard Draft" Functionality:**
    *   A new backend endpoint (e.g., `POST /api/creatives/{creative_id}/discard-draft`) will be required.
    *   This endpoint would:
        *   Authenticate and authorize the user.
        *   Set the `draft_url` field in the `creatives` table to `NULL` for the given creative.
        *   Optionally, delete the corresponding draft HTML file from GCS to save storage.
    *   The frontend would call this when the user wishes to revert to the last published version.
*   **Pros:**
    *   Enables same-origin access, unlocking all proposed inline editing features.
    *   Centralizes access control and content serving logic.
    *   Clear draft management workflow.
*   **Cons:**
    *   Adds load to the backend.
    *   Requires careful implementation of asset path handling and draft logic.
*   **Recommendation:** Essential prerequisite. The benefits are critical for the desired UX.

### D. Capturing and Integrating `contentEditable` DOM Manipulations via LLM

*   **Concept:** When `contentEditable` is used, the system observes these DOM changes. Instead of attempting to save the (potentially messy) resulting HTML directly, these changes are described or sent as "diffs" to the LLM, asking it to "cleanly integrate these user modifications into the original HTML structure."
*   **Feasibility (if same-origin achieved):**
    *   Use a `MutationObserver` attached to the iframe's document body (or specific editable regions) to detect DOM changes.
    *   When the user indicates they are done with direct edits (e.g., by clicking "Apply changes with AI" or similar, or by focusing out of the edited area after a timeout):
        1.  Consolidate observed mutations.
        2.  Capture the `outerHTML` of the modified element(s) and its original state (or a relevant snippet of the changed content).
        3.  Send this information (original snippet, modified snippet, or a description of the change) to the LLM. The prompt would be specialized, e.g., "USER_MODIFICATION: In the paragraph starting with 'Old text...', the user changed it to '<b>New</b> text...'. Please apply this change to the relevant part of the full document, ensuring HTML validity and structural integrity."
*   **Pros:**
    *   Combines the immediacy of direct manipulation with the robustness of LLM-driven HTML generation.
    *   Leverages the LLM to ensure changes are well-integrated and maintain HTML quality.
    *   Users don't need to be HTML experts.
*   **Cons:**
    *   `MutationObserver` can be chatty; logic is needed to batch/consolidate changes.
    *   Designing effective prompts for the LLM to understand and apply user-made DOM changes as "intent" is complex and iterative.
    *   The workflow needs to be clear to the user (e.g., "Your direct edits will be refined by AI").
*   **Recommendation:** High priority, as this is key to making `contentEditable` a robust feature rather than a risky one. This leverages the AI to bridge the gap between simple user input and clean HTML.

## 4. Overall Recommended Strategy & Phased Approach (v2)

1.  **Phase 1: Implement Same-Origin Iframe Proxy & Draft Management (Critical Priority)**
    *   Create the backend proxy endpoint (`/api/creatives/{id}/content`) with auth, draft/published logic, and `<base>` tag injection for asset paths.
    *   Create the backend "discard draft" endpoint (`/api/creatives/{id}/discard-draft`).
    *   Update `EditCreativeDialog.tsx` to use the proxied URL and provide a "discard draft" UI option if a draft exists.
    *   **Goal:** Achieve a same-origin iframe environment with proper draft handling.

2.  **Phase 2: Implement Highlight-to-Context (High Priority, depends on Phase 1)**
    *   Develop JavaScript in `EditCreativeDialog.tsx` to detect selections, extract element info, and integrate it into the LLM instruction payload.
    *   Update backend LLM prompt generation in `edit_creative.rs` to use this context.
    *   **Goal:** More precise LLM instructions.

3.  **Phase 3: Implement `contentEditable` with LLM Integration (High Priority, depends on Phase 1 & LLM work)**
    *   In `EditCreativeDialog.tsx`, enable `contentEditable` on text-based elements within the proxied iframe content.
    *   Implement `MutationObserver` to capture changes.
    *   Design a UI flow for users to "submit" their direct edits for LLM processing (this might be a new button or integrated into the existing "Send Instruction" flow if the instruction input is left blank).
    *   Develop specialized LLM prompts and backend logic in `edit_creative.rs` to receive these observed changes and integrate them into the full HTML.
    *   **Goal:** Allow direct text editing, refined by AI for quality.

4.  **Phase 4: UX Refinements & Advanced Scenarios (Medium Priority / Future)**
    *   Refine the UI/UX for switching between or combining editing modes (highlight-to-context, direct `contentEditable` + LLM, pure LLM instruction).
    *   Explore more sophisticated change tracking if `MutationObserver` proves insufficient for complex edits.
    *   **Goal:** A polished and versatile inline editing experience.

## 5. Open Questions & Future Considerations (v2)

*   **Performance:** Monitor backend load from the proxy. Implement caching if GCS fetches are slow or content is frequently re-requested.
*   **Security:** Rigorously test proxy endpoint security (auth, authorization, input sanitization if any).
*   **Asset Path Edge Cases:** Test `<base>` tag solution with diverse creative structures.
*   **LLM Prompt Engineering for `contentEditable`:** This will require significant iteration to make the LLM reliably interpret and apply user-made DOM changes. How to handle deletions, additions, style changes within `contentEditable`?
*   **User Experience for `contentEditable` "Commit":** How does the user signal they are done with direct edits and want the AI to process them? A dedicated button? Auto-detection?
*   **"Discard Draft" Implications:** Ensure deleting the GCS draft file (if implemented) is handled gracefully and doesn't affect any published versions. What if the draft was based on a previous draft? (Current model is simpler: one draft at a time).

This revised analysis (v2) provides a clearer path towards a more powerful and intuitive inline creative editing experience, strongly favoring `contentEditable` for direct manipulation, backed by LLM intelligence for robust integration.