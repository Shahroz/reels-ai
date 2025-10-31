Got it! Here's a concise `README.md` that **only describes the ideal structure**, omitting references to the existing (less ideal) state:

---

```markdown
# Rust Backend Architecture

This backend is organized into three clear layers to separate concerns and promote reuse across different contexts such as APIs, agents, and background tasks.

## Project Structure

```

src/
├── db/         # Database schema and model definitions
├── queries/    # Query layer for interacting with the database
└── routes/     # HTTP API routes and request handlers

```

### `db/` — Schema and Models

This layer defines the core database types and schema mappings. It includes:

- Rust structs representing tables or views
- Mappings for SQL types and relationships
- Schema metadata (e.g., migrations, joins)

This layer is pure and contains no business logic.

---

### `queries/` — Database Access Logic

This layer acts as a boundary between data models and application logic. It includes:

- Query functions for creating, retrieving, updating, and deleting data
- Abstracts over SQL or ORM details
- Composable operations for reuse across different contexts

This layer allows logic to be reused outside of the API, such as in internal agents or CLI tools.

---

### `routes/` — API Endpoints

This layer is responsible for:

- Handling HTTP requests and responses
- Input validation and authentication
- Calling query functions to perform actual work

By delegating all database interactions to the `queries/` layer, the routes stay clean and focused on HTTP concerns.

---

## Why This Structure?

- **Separation of concerns**: Each layer has a clear responsibility
- **Testability**: Logic in `queries/` can be unit tested without API overhead
- **Flexibility**: The same query logic can be reused by agents, background tasks, or other internal tools

This design leads to a clean, maintainable, and scalable Rust backend.
```

---

Let me know if you'd like an example of how a function might be split between `queries/` and `routes/`.
