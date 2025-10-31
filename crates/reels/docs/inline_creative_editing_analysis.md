# Analysis: Enhancing Inline Creative Editing Experience

**Date:** 2025-05-21
**Author:** AI Assistant
**Status:** Initial Analysis

## 1. Introduction

This document analyzes potential enhancements to the creative editing functionality, specifically focusing on enabling a more interactive "inline" editing experience within the existing iframe-based editor. The goal is to explore features like direct HTML editing, a "highlight-to-context" mechanism for LLM instructions, and the technical prerequisites for such features, primarily addressing the browser's same-origin policy.

## 2. Current Creative Editing Workflow

Currently, the creative editing process (`EditCreativeDialog.tsx` and `edit_creative.rs` backend) involves:
1.  Displaying the creative's HTML (sourced from a GCS URL, either `draft_url` or `html_url`) within an iframe.
2.  The user provides textual instructions (e.g., "make the heading blue") in a separate input field.
3.  These instructions, along with the creative's current HTML content (fetched again by the backend from GCS), are sent to an LLM.
4.  The LLM processes the request and returns a modified HTML string.
5.  The backend validates this HTML, saves it to GCS as a new draft, and updates the database.
6.  The iframe in the frontend is then updated to show the new draft URL.

The key limitation is that the iframe content is typically cross-origin (GCS domain vs. application domain), preventing direct JavaScript interaction between the parent page and the iframe's content.

## 3. Proposed Enhancements & Analysis

### A. Direct Inline HTML Editing (WYSIWYG/Code Editor)

*   **Concept:** Allow users to directly manipulate the HTML content within the iframe, either through a WYSIWYG editor (like TinyMCE) or a code editor (like CodeMirror) embedded within/acting upon the iframe content.
*   **Feasibility:**
    *   **Same-Origin Prerequisite:** This is strictly required. The parent page (hosting the editing dialog) cannot access or modify the DOM of a cross-origin iframe. See Section 3.C for achieving same-origin.
    *   **Implementation Complexity:** If same-origin is achieved, integrating a rich text editor or code editor to work seamlessly with arbitrary HTML content can be complex. Care must be taken to preserve the creative's structure and not introduce malformed HTML. The `contentEditable` HTML attribute is a simpler alternative for basic text edits.
*   **Pros:**
    *   Provides immediate visual feedback for simple changes.
    *   Potentially faster for minor text corrections or style tweaks than writing LLM instructions.
*   **Cons:**
    *   Risk of users breaking the HTML structure if not carefully implemented.
    *   WYSIWYG editors can sometimes produce non-optimal HTML.
    *   Direct HTML code editing might be too technical for some users.
    *   Synchronization: How would these direct edits interact with the LLM instruction flow? Would they be distinct modes?
*   **Recommendation:** Explore as a secondary feature after achieving same-origin. Start with simple `contentEditable` for text elements before considering full WYSIWYG or code editors.

### B. Highlight-to-Context for LLM Instructions

*   **Concept:** Allow users to click or highlight an element within the iframe. Information about this selected element (e.g., its HTML snippet, a CSS selector, or XPath) is then captured and automatically appended/prepended to the textual instruction sent to the LLM. For example, user highlights a button, types "make this red," and the system sends the instruction along with context about the specific button.
*   **Feasibility:**
    *   **Same-Origin Prerequisite:** Required for the parent page to detect events (clicks, selection changes) within the iframe and inspect the DOM elements.
    *   **Implementation:**
        1.  Parent page attaches event listeners (e.g., `click`, `mouseup`) to the iframe's document.
        2.  On event, identify the target element.
        3.  Extract relevant information: `outerHTML`, a unique CSS selector, or XPath.
        4.  Provide a UI mechanism for the user to "select" an element and then type their instruction.
        5.  Modify the LLM prompt to include this contextual element information. E.g., "CONTEXT_ELEMENT: `<button>Old Text</button>` INSTRUCTION: 'Change text to New Text'".
*   **Pros:**
    *   Significantly improves the precision of LLM instructions.
    *   More intuitive for users to specify the target of their changes.
    *   Reduces ambiguity for the LLM.
*   **Cons:**
    *   Generating robust and unique CSS selectors or XPaths can be challenging for complex DOMs. `outerHTML` might be simpler but could be large.
    *   Requires careful LLM prompt engineering to make effective use of the context.
*   **Recommendation:** High priority. This offers a substantial UX improvement and leverages the LLM's strengths more effectively. This should be a primary goal if same-origin can be achieved.

### C. Achieving Same-Origin Iframe Content (Proxy Endpoint)

*   **Problem:** Iframe content loaded directly from GCS URLs is cross-origin, blocking JavaScript interaction from the parent Narrativ application.
*   **Proposed Solution:** Implement a backend proxy endpoint (e.g., `/api/creatives/{creative_id}/content-proxy`).
    1.  The frontend `EditCreativeDialog` sets the `iframeUrl` to this backend endpoint.
    2.  The backend endpoint:
        *   Authenticates the request (verifies user session/claims).
        *   Authorizes access (ensures the user owns/can access the specified `creative_id`).
        *   Fetches the actual HTML content from its GCS URL (draft or published).
        *   **Crucially, modifies the HTML to handle relative asset paths (see below).**
        *   Serves the (modified) HTML content with `Content-Type: text/html`.
    Since the iframe is now served from the same domain as the parent application, JavaScript interaction becomes possible.
*   **Handling Relative Asset Paths in Proxied HTML:**
    HTML content often contains relative paths for images (`<img>` src), stylesheets (`<link>` href), scripts (`<script>` src), etc. If the HTML is served from `/api/creatives/.../content-proxy`, the browser will resolve these relative paths against this new base URL, leading to broken assets.
    *   **Solution 1: Inject `<base>` tag:** Before serving the HTML, the proxy injects a `<base href="BASE_GCS_URL_FOR_CREATIVE/">` tag into the `<head>` of the HTML document. `BASE_GCS_URL_FOR_CREATIVE` would be the original GCS "directory" where the creative's assets are stored. This tells the browser to resolve all relative paths against this GCS base URL. This is generally the most straightforward and robust solution if assets on GCS are publicly accessible.
    *   **Solution 2: Rewrite relative paths:** The proxy parses the HTML and prepends the GCS base URL to all relative `src` and `href` attributes, making them absolute. This is more complex due to parsing and rewriting HTML.
    *   **Solution 3: Proxy assets too:** The backend would also need to proxy requests for assets. This is the most complex and resource-intensive.
    **Recommendation for Asset Paths:** Prioritize Solution 1 (inject `<base>` tag) due to its relative simplicity and effectiveness, assuming GCS assets are public.
*   **Pros:**
    *   Enables same-origin access, unlocking inline editing and highlight-to-context features.
    *   Centralizes access control to creative content through the backend.
*   **Cons:**
    *   Adds load to the backend server for serving creative content.
    *   Requires careful implementation to handle asset paths correctly.
    *   Potential for increased latency if the proxy itself is slow.
*   **Recommendation:** Essential prerequisite for advanced inline editing features. The benefits likely outweigh the cons if implemented carefully, especially with the `<base>` tag solution for assets.

### D. Capturing User's Direct DOM Manipulations for LLM Refinement

*   **Concept:** If basic direct editing (e.g., via `contentEditable`) is enabled, the system could observe these DOM changes. Instead of saving them directly (which might result in messy HTML), these changes could be described or sent as "diffs" to the LLM, asking it to "cleanly integrate these user modifications into the original HTML structure."
*   **Feasibility (if same-origin achieved):**
    *   Use a `MutationObserver` attached to the iframe's document body (or specific editable regions) to detect DOM changes made by the user.
    *   Upon "committing" direct edits, the system could:
        *   Capture the `outerHTML` of the modified element(s) and its original state.
        *   Send this pair (original, modified) to the LLM with a specialized prompt. E.g., "USER_MODIFICATION: Original: `<p>Old text</p>`, New: `<p><b>New</b> text</p>`. Please apply this change to the relevant part of the full document..."
*   **Pros:**
    *   Allows users the immediacy of direct manipulation for small tweaks.
    *   Leverages the LLM to ensure changes are well-integrated and maintain HTML quality.
*   **Cons:**
    *   `MutationObserver` can generate many events; logic is needed to consolidate changes.
    *   Designing effective prompts for the LLM to "understand" and apply user-made DOM changes as "intent" is complex.
    *   Might be an overly complex workflow if the LLM is already good at interpreting direct instructions.
*   **Recommendation:** Lower priority compared to highlight-to-context. Could be considered a future enhancement if users find direct LLM instructions insufficient for fine-grained control after other improvements are made. The JavaScript files under `crates/narrativ/backend/src/zyte/zyte_javascript/` show that sophisticated client-side DOM manipulation and style extraction is already being done in other parts of the system, suggesting technical capability for such observation.

## 4. Overall Recommended Strategy & Phased Approach

1.  **Phase 1: Implement Same-Origin Iframe Proxy (High Priority)**
    *   Create the backend proxy endpoint (`/api/creatives/{id}/content-proxy`).
    *   Implement robust authentication and authorization.
    *   Critically, solve the relative asset path issue, likely by injecting a `<base>` tag pointing to the GCS asset location.
    *   Update `EditCreativeDialog.tsx` to use this proxied URL for the iframe.
    *   **Goal:** Achieve a same-origin iframe environment.

2.  **Phase 2: Implement Highlight-to-Context (High Priority, depends on Phase 1)**
    *   Develop the JavaScript logic in `EditCreativeDialog.tsx` to:
        *   Detect user clicks/selections within the (now same-origin) iframe.
        *   Extract element information (e.g., `outerHTML` or a selector).
        *   Integrate this context into the instruction sent to the `handleEditCreativeInstruction` function.
    *   Update the backend LLM prompt generation in `edit_creative.rs` to incorporate this new contextual element information.
    *   **Goal:** Allow users to pinpoint elements for more precise LLM instructions.

3.  **Phase 3: Explore Basic Direct Editing (Medium Priority, depends on Phase 1)**
    *   Experiment with making specific text elements within the iframe `contentEditable`.
    *   Decide on a workflow:
        *   Option A: Direct edits are temporary; user still uses LLM to "apply" them (by describing or having them captured as per Section 3.D).
        *   Option B: A "save direct changes" button that attempts to update the draft directly (potentially risky for HTML quality) or uses an LLM to sanitize/integrate.
    *   **Goal:** Evaluate the utility and complexity of simple direct edits.

4.  **Phase 4: Advanced Inline Editing & LLM Refinement (Lower Priority / Future)**
    *   If basic direct editing proves valuable but messy, implement the `MutationObserver` approach (Section 3.D) to capture changes and have the LLM refine them.
    *   Consider integrating a lightweight WYSIWYG editor if there's strong user demand.
    *   **Goal:** Provide a rich, interactive editing experience with LLM oversight for quality.

## 5. Open Questions & Future Considerations

*   **Performance:** What is the performance impact of proxying creative content through the backend? Caching strategies for the proxy might be needed if GCS fetches are slow.
*   **Security:** Ensure the proxy endpoint is robust against unauthorized access and correctly handles user permissions.
*   **Asset Path Edge Cases:** Thoroughly test the `<base>` tag or path rewriting solution with various creative structures and asset types. What if creatives use absolute paths to other domains? (These should remain untouched).
*   **LLM Prompt Robustness:** Continuously refine LLM prompts as new contextual information (like highlighted elements) is added.
*   **User Experience for Mode Switching:** If multiple editing modes (LLM instruction, highlight-to-context, direct edit) are implemented, how will users switch between them? The UI needs to be clear.

This analysis suggests a path towards a significantly more interactive and powerful creative editing experience. Prioritizing same-origin access and highlight-to-context features appears to offer the best immediate value.