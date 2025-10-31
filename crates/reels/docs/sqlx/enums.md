Hi Paweł,

To ensure that your PostgreSQL enums are correctly represented in Rust using SQLx and are exported as enums (not strings) in your OpenAPI-generated TypeScript models, you'll need to follow a structured approach. Here's how you can achieve this:

---

### 1. Define the PostgreSQL Enum Type

First, create the enum type in your PostgreSQL database:

```sql
CREATE TYPE user_role AS ENUM ('admin', 'user');
```

Then, use this enum in your table definition:

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    role user_role NOT NULL
);
```

---

### 2. Define the Corresponding Rust Enum

In Rust, define an enum that matches the PostgreSQL enum. Use the `sqlx::Type` derive macro to enable SQLx to map between Rust and PostgreSQL types:

```rust
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}
```

* `#[sqlx(type_name = "user_role")]` tells SQLx the name of the PostgreSQL enum type.
* `rename_all = "lowercase"` ensures that the Rust enum variants match the lowercase strings in PostgreSQL.

---

### 3. Use the Enum in Your Rust Structs

When defining your data models, use the `UserRole` enum:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}
```

---

### 4. Querying with SQLx

When writing queries with SQLx, especially using macros like `query_as!`, you need to explicitly specify the type for enum fields:

```rust
let user = sqlx::query_as!(
    User,
    r#"
    SELECT
        id,
        username,
        role as "role: UserRole",
        created_at
    FROM users
    WHERE username = $1
    "#,
    username
)
.fetch_one(&pool)
.await?;
```

The `as "role: UserRole"` part tells SQLx to map the `role` column to the `UserRole` enum.

---

### 5. Exporting Enums to TypeScript via OpenAPI

To have your enums represented as TypeScript enums in the OpenAPI schema, ensure that:

* Your Rust enums derive `Serialize` and `Deserialize` from `serde`.
* You're using a tool like `utoipa` or `paperclip` that can generate OpenAPI schemas from Rust code.

For example, with `utoipa`, you can derive `ToSchema`:

```rust
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}
```

This will generate an OpenAPI schema where `UserRole` is an enum, and tools like `openapi-typescript` can then generate corresponding TypeScript enums.

---

### 6. Generating TypeScript Types

After generating the OpenAPI schema (e.g., `openapi.json`), use `openapi-typescript` to generate TypeScript types:

```bash
npx openapi-typescript openapi.json -o types.ts
```

This will create a `types.ts` file with TypeScript interfaces and enums matching your Rust models.

---

### Summary

By aligning your PostgreSQL enums with Rust enums using `sqlx::Type` and ensuring proper serialization and OpenAPI schema generation, you can have consistent enum representations across your database, backend, and frontend.

If you're using a specific OpenAPI generation tool or have additional requirements, feel free to provide more details, and I can offer more tailored guidance.
