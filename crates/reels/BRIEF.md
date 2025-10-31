# Expanded Project Brief: Website Style Cloning App

**Version:** 1.2
**Date:** 2025-04-04

## 1. Overview

Develop a web application that allows users to clone the visual style of existing websites. The application will feature a backend API (built with Rust/Actix-web), a frontend playground (React/TypeScript), user management with robust payment integration using **Stripe**, and a public-facing homepage. Development will adhere strictly to the defined [CODING_GUIDELINES.md](./CODING_GUIDELINES.md), leveraging the patterns and libraries found in `library_docs` where applicable (e.g., unified LLM interface, Zyte client, visual finetuning logic).

## 2. Core Functionalities

*   **Playground:**
    *   Input fields for user-provided content (text, basic structure) and target website URL.
    *   Utilizes style extraction logic (similar to `library_docs/website_style/extract_style.rs`) involving Zyte (`library_docs/zyte.rs`) and LLMs (`library_docs/llm/*`) to analyze the target website's styles.
    *   Applies the cloned styles to the user's content.
    *   Renders the result in an iframe for preview.
*   **Visual Finetuning (Slow Mode):**
    *   Leverages techniques shown in `library_docs/website_style/visual_html_finetuning/*`.
    *   Captures a screenshot of the initially generated HTML (`screenshot_html.rs`).
    *   Uses a visual AI model (e.g., Gemini via `library_docs/llm/vendors/gemini/completion.rs` or GPT-Vision via `library_docs/website_style/visual_html_finetuning/html_improvement.rs`) to iteratively adjust the generated styles/template (`tailwind_template_visual_improvement.rs`).
    *   Compares against the target website's appearance, aiming for a predefined accuracy score (e.g., 90%) using visual scoring (`html_improvement.rs::score_html`).
    *   Provides visual feedback during the process.
*   **Simple API Access (Fast Mode):**
    *   Provides RESTful API endpoints (defined using Actix-web as in `library_docs/app.rs`) for style cloning without the iterative visual feedback loop.
    *   Requires secure API key authentication (e.g., Bearer Tokens, potentially linked to user accounts).
    *   **Stripe Payment Integration:** Implements a pay-as-you-go pricing model per API request. Each request must be authorized and associated with a Stripe customer account with a valid payment method. Billing will be handled via Stripe.
*   **User Management:**
    *   Secure user registration (email verification required).
    *   Login/Logout functionality (potentially using JWT as hinted in `library_docs/app.rs`).
    *   Password reset mechanisms.
    *   **Stripe Integration:** Each registered user corresponds to a Stripe Customer object. Securely manage the mapping between application users and Stripe Customer IDs.
*   **User Dashboard:**
    *   User profile and settings management.
    *   Generation, display (partially masked), and revocation of API keys.
    *   **Stripe Integration:**
        *   Securely manage payment methods using **Stripe Elements** or **Stripe Checkout** hosted pages.
        *   Display usage tracking related to API calls.
        *   Provide access to billing history and invoices managed by Stripe (e.g., via Stripe Customer Portal).
*   **Homepage:**
    *   Modern landing page showcasing the product's capabilities.
    *   Include an interactive example or demo section (potentially using a limited version of the playground).
    *   Link to a live demo.
    *   Clear calls to action (Sign Up, View Demo, API Docs).

## 3. Technology Stack & Architecture (Based on Guidelines)

*   **Architecture:** Monolith (Single Codebase: Rust Backend + React Frontend)
*   **Backend:** Rust with Actix-web (as shown in `library_docs/app.rs`)
*   **Database:** PostgreSQL (using SQLx for compile-time checked queries)
*   **Frontend:** TypeScript with React
*   **UI Delivery:** Frontend assets embedded in the Rust binary (using `rust-embed`)
*   **Deployment:** Single binary deployed to Google Cloud Run
*   **Storage:** Google Cloud Storage for blob/asset storage (e.g., screenshots, user assets)
*   **Background Tasks:** Google Cloud Tasks (for potentially long-running finetuning jobs)
*   **Payments:** **Stripe SDK/API** for creating customers, managing payment methods, processing payments per API call, and handling subscriptions/billing logic.
*   **Authentication:** Secure API key mechanism (e.g., Bearer Tokens stored securely, potentially JWT for user sessions).
*   **Type Safety:** Strict typing in both Rust and TypeScript. OpenAPI schemas generated via `utoipa` (as used in `library_docs/app.rs`) and TypeScript types via `openapi-typescript-codegen`.
*   **External APIs:** Zyte API (`library_docs/zyte.rs`), LLM APIs (OpenAI/Gemini via `library_docs/llm/*`).

## 4. Development Practices (Based on Guidelines)

*   **Code Standards:** Adhere to `rustfmt`, `clippy`, `eslint`, `prettier`. No `unsafe` Rust, no debug logs (`dbg!`, `console.log`). Short functions (<30 lines), clear variable names (consistent between Rust/TS), file organization by feature (~200 lines max). Use type aliases (`type UserId = i32;`) and functional patterns (stateless functions, explicit I/O separation).
*   **Testing:** Comprehensive unit, integration, and E2E tests (Vitest/Jest, React Testing Library, Playwright). **Crucially, test Stripe integration using Stripe's test environment, test card numbers, and simulate webhook events.**
*   **AI Assistance:** Utilize tools like Cursor/Aider for code generation, refactoring, and test creation.
*   **LLM Integration:** Use `llm_typed` (`library_docs/llm/unified.rs`) for structured LLM interactions, implementing `FewShotsOutput` (`library_docs/llm/traits.rs`) for response types.
*   **Database Migrations:** Managed via `sqlx-cli` or `refinery`, embedded in the binary, applied on startup (configurable).
*   **Version Control:** Git monorepo.
*   **CI/CD:** Automated builds, tests (including checks against Stripe's test API and webhooks), and deployment via GitHub Actions or Cloud Build.
*   **Security:** Securely manage all API keys (Stripe, Zyte, LLMs) server-side using environment variables or a secrets manager. Never expose secret keys to the frontend.

## 5. UI/UX Guidelines (Based on Guidelines)

*   **Interface:** Clean, intuitive design with ample whitespace and consistent typography.
*   **Microinteractions:** Subtle animations for feedback (hovers, loading states, transitions, payment success/failure notifications).
*   **Payment Forms:** Use **Stripe Elements** for embedded, secure, PCI-compliant payment forms within the dashboard.
*   **Accessibility:** Ensure keyboard navigation and WCAG-compliant color contrast.
*   **Responsiveness:** Adapt seamlessly across different screen sizes using responsive design techniques (Tailwind CSS recommended).

## 6. Guiding Principles (Based on Guidelines)

*   Monolithic Simplicity
*   Compiled & Type-Safe
*   Single Binary Deployment
*   Automation First
*   Cloud Native & Serverless
*   AI Integration by Design
*   **Secure and Reliable Payment Processing (Stripe)**
