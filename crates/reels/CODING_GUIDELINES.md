### ALWAYS INCLUDE ME IN THE CONTEXT!!! ###

# Proposed Technology Stack and Development Practices

**Version:** 1.0  
**Date:** 2025-04-03

---

## 1. Introduction

This document defines the technology stack and development practices for building robust, performant, and maintainable AI-integrated applications. The focus is on operational simplicity, deep AI integration, and automation-first workflows.

---

## 2. Guiding Principles

- **Monolithic Simplicity:** Unified codebase (frontend + backend) to streamline development and deployment.
- **Compiled & Type-Safe:** Leverage Rust and TypeScript for compile-time safety and reliability.
- **Single Binary Deployment:** Package backend logic, frontend assets, and migrations into one executable.
- **Automation First:** Use AI tools to accelerate code generation, refactoring, and testing.
- **Cloud Native & Serverless:** Offload infrastructure to managed services (e.g., Cloud Run, Google Cloud Storage).
- **AI Integration by Design:** Build UIs optimized for conversational and AI-driven features.

---

## 3. Technology Stack

| **Component**        | **Technology**                     | **Rationale**                     |
|----------------------|------------------------------------|------------------------------------|
| Architecture         | Monolith (Single Codebase)         | Simplifies deployment & local dev |
| Backend Language     | Rust                               | Safety, speed, async support      |
| Backend Web Framework| Actix-web                          | High performance, async-first     |
| DB Access            | SQLx                               | Compile-time SQL validation       |
| Frontend Language    | TypeScript                         | Static typing, refactoring safety |
| Frontend Framework   | React                              | Modern UIs, reusable components   |
| UI Delivery          | Embedded in Rust Binary (rust-embed) | Single binary delivery          |
| Database             | PostgreSQL                         | Relational + JSONB support        |
| Blob Storage         | Google Cloud Storage               | Scalable object storage           |
| Deployment           | Google Cloud Run                   | Serverless scaling, zero infra    |
| Background Tasks     | Google Cloud Tasks                 | Reliable async task handling      |

---

## 4. Development Practices

### Code Standards

- **Strict Typing:** Enforce `noImplicitAny` in TypeScript and exhaustive matching in Rust.
- **Types for everything** Request responses must be typed in APIs
- **No Unsafe Rust:** Disallow `unsafe` unless explicitly audited.
- **Linters & Formatters:** Use `rustfmt`, `clippy`, `eslint`, and `prettier`.
- **No Debug Outputs:** Ban `console.log` and `dbg!` in committed code.
- **Declarative Style:** Prefer declarative over imperative code where possible.
- **Short Functions:** Keep functions concise (ideally <20-30 lines), focused on a single responsibility.
- **File Organization:** Limit file length to ~200 lines; organize by feature/domain (e.g., group components, hooks, and utilities), not type.
- **Clear Variable Names:** Use descriptive, purpose-driven names. For concepts shared between Rust and TypeScript (e.g., `userId`), maintain consistent naming to bridge the languages.
- **Composition Over Inheritance:** Build modular, reusable code.
- **Type Aliases:** Use type aliases in Rust (`type UserId = i32`) and TypeScript (`type UserId = number`) for complex types, enhancing readability.
- **Functional Programming:** Prefer stateless functions; clearly separate I/O (e.g., database calls, HTTP requests) from pure logic for testability and clarity.
- **Comprehensive Testing:** Test all public functions, reducers, hooks, and components.

### Testing Strategy

#### Backend (Rust)
- **Unit Tests:** Use `#[cfg(FALSE)]` modules for core logic.
- **Integration Tests:** Leverage SQLx testing utils with `testcontainers` or `docker-compose`.
- **API Tests:** Test endpoints with Rust HTTP clients in test mode.

#### Frontend (TypeScript)
- **Unit Tests:**
    - Test logic with [Vitest](https://vitest.dev/) for speed or [Jest](https://jestjs.io/).
    - Test UI behavior with [React Testing Library](https://testing-library.com/).
- **Integration Tests:** Test multiple wired components with rendering and DOM events.
- **E2E Tests:**
    - Use [Playwright](https://playwright.dev/) to simulate user flows (auth, forms, uploads) against the single binary.

### Type Sharing (Rust <-> TypeScript)

- Generate OpenAPI schemas with [`utoipa`](https://docs.rs/utoipa) on the backend.
- Create TypeScript client types using `openapi-typescript-codegen` or `orval`.
- Use type aliases (e.g., `type OrderStatus = 'pending' | 'shipped'`) to maintain consistency and readability across languages.

### AI-Assisted Development

- Employ tools like [Cursor](https://www.cursor.so/) or [Aider](https://github.com/paul-gauthier/aider) to automate:
    - Boilerplate generation
    - Cross-module refactors
    - Test creation
    - Feature implementation from specs
- Treat AI as a productivity tool, not a replacement for understanding code.

### LLM Integration

- When passing request to GPT use llm_typed prefereably with few shots implemented on data structures that you request
- Design UIs for chat-like interactions and smart autofill features.

### Database Migrations

- Manage with `sqlx-cli` or `refinery`.
- Embed migrations in the binary; optionally apply on startup (configurable).

### Version Control & Monorepo

- Use Git with a single repo for Rust (backend) and TypeScript (frontend).

### CI/CD

With GitHub Actions or Cloud Build:
1. Lint and format Rust + TypeScript.
2. Build frontend assets.
3. Compile Rust binary with embedded UI.
4. Run unit, integration, and E2E tests.
5. Build and deploy container to Cloud Run.

---

## 5. Architecture Overview

- **Deployment:** Cloud Run
- **Single Binary:** Rust delivers API and UI via Actix-web and rust-embed.
- **Frontend:** React app runs in-browser, consuming typed API clients.
- **Async Jobs:** Handled via Cloud Tasks with HTTP callbacks to Cloud Run.

---

## 6. Benefits

- **Safety & Performance:** Rust and TypeScript minimize runtime errors.
- **Developer Speed:** AI tools, typed APIs, and monorepo boost productivity.
- **Simplicity:** Single binary and serverless infra reduce ops overhead.
- **Scalability:** Cloud Run, PostgreSQL, and GCS scale effortlessly.
- **Maintainability:** Strong types, functional patterns, and tests ensure longevity.
- **Modern UX:** Built-in AI and clean, interactive interfaces.

---

## 7. UI Guidelines

- **Microinteractions:** Add subtle animations (e.g., hover effects, loading spinners, transitions) for responsive feedback.
- **Clean Interfaces:** Emphasize simplicity with ample whitespace, consistent typography, and intuitive navigation.
- **Accessibility:** Ensure keyboard navigation and high color contrast.
- **Responsive Design:** Support seamless adaptation across devices and screen sizes.

LLM Code guidelines:
- if a file is short rewrite it fully - don't use complex regexes 
- load useful documentation files from library_docs

Documentation format Rust
```rust
/// Creates a new user in the database.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `email` - The user's email address.
/// * `password_hash` - The user's hashed password.
///
/// # Returns
///
/// A `Result` containing the new user's UUID on success, or an `sqlx::Error` on failure.
/// Note: Assumes the 'id' column in the 'users' table is of type UUID.
/// If the database schema uses SERIAL/INT4 for id, the return type should be i32.
```

## Frequent contexts (update it when user asked specifically to connect some files in the context or you find some far relationships between project files especially across the frontend and backend code)
- Example: db/users.rs + routes/auth.rs + db_schema.sql

## Common problems (tricky repeating problems) - one line per problem and solution - prevent future problems or solve existing ones 
- Type mismatch between Rust code (e.g., `Uuid`) and database schema (e.g., `INTEGER`) for foreign keys. Solution: Ensure foreign key column types in migrations match the referenced primary key type (e.g., use `UUID` if referencing a `UUID` primary key).
- async_stripe crate is imported via `use stripe` instead of `use of undeclared crate or module `async_stripe`` it must be `use stripe` 

## Coding guidelines specific to this project
- For every task load coding guidelines as the context
- Create folders for everything especially if it mentions the folder for the first time
- For every new API endpoint remember to add utoipa annotations and update openapi.rs
- If a file to be changed is short rewrite it instead of doing search and replace.
- When rewriting the code also format it properly and delete all temporary comments
- Please add db_schema.sql to the context
- Run `make test` after each task if it fails rerun it
- Update the coding guidelines with sets of files that are frequently needed together in the context
- Do not update existing migrations - please look at db_schema.sql and create a new one if needed
- When creating migrations add --READONLY comment at the top
- Always use CODING_GUIDELINES.md as context and pass this information also to nested instructions
- Run `make dump-db-schema` before considering any changes in the DB schema and load it from db_schema.sql
- library_docs has references of previous projects don't use them directly - if you want to use them incorporate them in the current project by copying and transforming the files
- cargo commands can be run only in backend folder
- use anyhow::Result 
- when rewriting files don't leave anything out for brevity sake

RUN CARGO COMMANDS ONLY IN BACKEND: "cd backend && cargo ..."
USE README FILES FROM THE FOLDERS YOU ARE TRYING TO CREATE FILES IN