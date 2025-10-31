# Repository Guidelines

## Project Structure & Module Organization
- Backend (Rust): `backend/` with `src/`, `tests/`, and `Cargo.toml`. Key entry points: `backend/src/main.rs`, `backend/src/app.rs`, OpenAPI in `backend/src/openapi.rs`.
- Data & Docs: `pgdata/` (local DB volume), `test-data/`, `docs/`, and environment files (`.env`, `.env-example`).

## Build, Test, and Development Commands
- Local dev (backend): `make dev` (use `EMULATOR=true` to enable fake GCS).
- Backend run only: `make app-run`.
- Tests (fast path): `make test-fast` (backend), or full: `make test`.
- Backend tests: `cd backend && DATABASE_URL=... APP_ENV=test cargo test`.
- DB: `make db-start`, `make db-connect`, schema dump via `make dump-db-schema`.

## Coding Style & Naming Conventions
- Rust: follow `rust_coding_guidelines.md`. Format with `cargo fmt --all`. Lint with `cargo clippy -- -D warnings` (Makefile enforces additional disallowed methods).
- Naming: modules and files are `snake_case` in Rust.

## Testing Guidelines
- Rust: unit tests inline (`mod tests`) and integration tests in `backend/tests/*.rs`. Use `serial_test` where required for DB coupling.

## Commit & Pull Request Guidelines
- Commits: concise, imperative subject; optional scope (e.g., "backend: add auth middleware"). Group related changes.
- PRs: include summary, linked issues, and steps to validate. Add screenshots for UI changes and note API/DB migrations. Ensure `make test-fast` passes and code is formatted/linted.

## Security & Configuration Tips
- Copy `.env-example` to `.env` and set `DATABASE_URL`. Use `EMULATOR=true` for local storage via the fake GCS server. Never commit secrets.

## Backend Architecture (for Agents)
- db module (`backend/src/db/`)
  - Purpose: 1–1 data structs that map directly to DB tables. No business logic.
  - Conventions: one table per file; derive `Serialize`, `Deserialize`, `sqlx::FromRow`, and `utoipa::ToSchema` on API-facing structs.
  - Examples: `backend/src/db/assets.rs`, `backend/src/db/users.rs`.

- queries module (`backend/src/queries/`)
  - Purpose: separation of HTTP/API concerns from DB access. Pure SQLx queries and small helpers.
  - Conventions: one query or tightly related set per file; accept a `&PgPool` or `&mut Transaction<'_, Postgres>`; return typed models from `db/` or purpose-specific DTOs; no Actix types here.
  - Examples: `backend/src/queries/collections/create_collection.rs`, `backend/src/queries/requests.rs`.

- routes module (`backend/src/routes/`)
  - Purpose: API routes and HTTP transport. Small handlers that validate/authorize, then delegate to `queries/` and `services/`.
  - Conventions: one handler per file; group by domain in subfolders; annotate every handler with `#[utoipa::path(...)]`; register in the domain `configure_*` and in `routes::config`.
  - Examples: `backend/src/routes/collections/create_collection.rs`, `backend/src/routes/users/...`, aggregator: `backend/src/routes/mod.rs`.

- services module (`backend/src/services/`)
  - Purpose: everything else (non-DB domain logic and persistent clients), e.g., GCS, screenshot, Cloud Tasks, Dub, Imageboard, watermarking.
  - Conventions: encapsulate external integrations; inject via `web::Data` in `main.rs`; keep IO boundaries here; support testability by traits where appropriate.
  - Examples: `backend/src/services/gcs/*`, `backend/src/services/screenshot/*`, `backend/src/services/imageboard_client.rs`.

- OpenAPI (`backend/src/openapi.rs`)
  - Purpose: aggregate all documented routes and schemas and export a single spec.
  - Conventions: add each new handler to the `paths(...)` list; ensure all response/request structs derive `ToSchema` so they appear in `components(schemas(...))`.
  - Client export: used by `make generate-api-client` (TypeScript) and `make generate-swift-sdk` (iOS).

- App entry points
  - `backend/src/main.rs`: wiring (pools/clients/services), middleware, Swagger UI, and route configuration via `routes::config`.
  - `backend/src/app.rs`: app scaffolding types; keep this minimal.


## One-File-Per-Item Pattern
- Back end
  - One DB table → one file in `db/` with its struct(s).
  - One operation (query) → one file in `queries/` (keep it small and focused).
  - One HTTP handler (endpoint) → one file in `routes/<domain>/` with `#[utoipa::path]`.
  - One service/integration → one file or tight folder in `services/`.

## Typical Flow For A New Endpoint
1. Model: if a new entity or shape is needed, add/extend a struct in `backend/src/db/<entity>.rs` with `ToSchema` and `FromRow`.
2. Query: add a file in `backend/src/queries/<domain>/<action>.rs` (or extend an existing small module). Use `sqlx` and return typed models.
3. Route: add a file in `backend/src/routes/<domain>/<action>.rs` with an Actix handler and `#[utoipa::path(...)]` covering params, request, and responses.
4. Wire-up: register the handler in the domain `configure_*` and ensure the scope is mounted in `backend/src/routes/mod.rs::config`.
5. OpenAPI: add the handler to `paths(...)` in `backend/src/openapi.rs`; ensure models are in `components(schemas(...))` (via `ToSchema`).
6. Tests: add backend integration tests under `backend/tests/<domain>/` as needed.

## Practical Tips For Agents
- Keep route handlers thin; move IO and business logic to `queries/` and `services/`.
- Use `web::Data` for shared clients (GCS, screenshot, Dub, Imageboard) and pass pools explicitly to queries.
- Prefer explicit return types using structs from `db/` or dedicated response DTOs that also derive `ToSchema`.
- Error handling: return consistent `ErrorResponse` from routes; surface typed errors in `queries/`/`services/` and map at the boundary.
- When adding fields to DB structs used in APIs, remember to:
  - update `ToSchema` annotations (examples, formats),
  - update queries and tests,
  - regenerate clients as needed.
