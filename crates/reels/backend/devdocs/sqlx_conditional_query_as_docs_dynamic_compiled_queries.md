Due to our disallowed use of noncompiled query_as a problem arised
that every query has to be a string literal to be compiled.
The problem is that dynamic queries are very awkard to rewrite to non-dynamic versions
because it explodes the number of queries one has to write as a string literal.

Luckily there is a crate conditional-query-as which allows us to generate those queries dynamically
and still compile them!

```rust
let rows = conditional_query_as!(
    IPData,
    r#"
        SELECT * 
        FROM ips
        {#filter}
        ORDER BY {#sort}
        LIMIT {#limit}
    "#,
    #filter = match  country.clone() {
        Some(_) =>  "WHERE cf_country = {country}",
        None => "",
    },
    #sort = match sort  {
        SortBy::Ping => "ping_avg ASC",
        SortBy::Speed => "total_avg DESC",
        SortBy::Random => "RANDOM()",
    },
    #limit = match limit {
        Some(_) =>  "{limit}" ,
        None => "10",
    },
)
.fetch_all(&self.pool)
.await;
```

There is some bug in sqlx

```rust
43: unexpected null; try decoding as an Option
```

Even though the struct is an option what help is adding the aliases with question mark

            c.creative_format_id as "creative_format_id?",
            c.style_id as "style_id?",
            c.document_ids as "document_ids?",
            c.asset_ids as "asset_ids?",
            c.html_url as "html_url?",
            c.draft_url as "draft_url?",
            c.screenshot_url as "screenshot_url",
            c.is_published as "is_published",
            c.publish_url as "publish_url?",
            c.created_at as "created_at?",
            c.updated_at as "updated_at?",


---

# Best Practices for `sqlx-conditional-queries` with Optional Parameters

The `sqlx-conditional-queries` crate can be a powerful tool for dynamically building SQL queries at compile time, ensuring type safety. However, when dealing with optional filter parameters (e.g., `Option<Uuid>`, `Option<String>`), care must be taken to avoid both excessive query variant generation (leading to long compile times) and runtime type inference errors with PostgreSQL.

Through experimentation, we've identified the following best practices:

## 1. The Challenge: Optional Parameters and Query Variants

*   **Naive Conditional Fragments (Works, but High Variant Count):**
    A common initial approach is to use conditional SQL fragments based on `Option` values:
    ```rust
    // Rust
    let style_id_param: Option<Uuid> = params.style_id;

    // conditional_query_as!
    // SQL String:
    // r#"... WHERE ... {#style_filter} ..."#,
    // Bindings:
    // #style_filter = match &style_id_param { Some(_) => "AND c.style_id = {style_id_param}", None => "" },
    // #style_id_param = match &style_id_param { _ => "{style_id_param}" },
    ```
    This works reliably because if `style_id_param` is `None`, the placeholder `{style_id_param}` is entirely absent from that SQL query variant, avoiding type issues for `NULL`. However, each such optional filter doubles the number of query variants, leading to exponential growth (e.g., 5 optional filters = 2<sup>5</sup> = 32 variants for the filter part alone).

*   **Failed Optimization Attempts (Type Inference Errors):**
    Attempts to reduce variants by keeping the SQL structure static and using SQL logic like `({param} IS NULL OR column = {param}::type)` or `CASE WHEN {param} IS NOT NULL THEN ... ELSE TRUE END` (while directly binding the `Option<T>` Rust variable) often failed. When the `Option<T>` was `None`, `sqlx` would bind a `NULL` to the placeholder. If PostgreSQL couldn't clearly infer the type of this `NULL` from its usage *at the statement preparation phase* (before conditional SQL logic fully resolves), it would result in "could not determine data type of parameter $N" errors.

## 2. The Solution: Static SQL Structure with Pre-Processed, Typed Sentinel Values

The most effective strategy to reduce query variants while maintaining correctness with optional parameters is:

*   **Pre-process Optional Parameters in Rust:**
    Convert your `Option<T>` parameters into a definite type (e.g., `String` or `Vec<String>`) *before* binding. For the "None" or "no filter" case, use a specific sentinel value (e.g., an empty string `""`, or an empty `Vec`).

    ```rust
    // For Option<Uuid>
    let style_id_for_sql_str: String = params.style_id.map_or("".to_string(), |id| id.to_string());

    // For Option<Vec<Uuid>> (e.g., document_ids)
    let doc_ids_vec_str: Vec<String> = params.document_ids
        .as_ref()
        .filter(|v| !v.is_empty()) // Only map if Some and not empty
        .map_or_else(Vec::new, |v_uuid| v_uuid.iter().map(Uuid::to_string).collect());
    // Use slice for binding arrays to sqlx
    let doc_ids_slice_str: &[String] = doc_ids_vec_str.as_slice(); 

    // For Option<String> (e.g., search term)
    let search_for_sql_str: String = params.search
        .as_ref()
        .filter(|s| !s.trim().is_empty()) // Only map if Some and not empty/whitespace
        .map_or("".to_string(), |s| format!("%{}%", s.to_lowercase()));
    ```

*   **Use Static SQL with `CASE WHEN` and Sentinel Value Checks:**
    Embed the conditional logic directly into the SQL string using `CASE WHEN`, checking against your sentinel value.

    ```sql
    -- For single value parameters (now strings)
    AND CASE WHEN {style_id_for_sql_str} != '' THEN c.style_id = {style_id_for_sql_str}::uuid ELSE TRUE END

    -- For array parameters (now Vec<String>, bound as slice)
    AND CASE WHEN array_length({doc_ids_slice_str}::TEXT[], 1) > 0 THEN c.document_ids && ({doc_ids_slice_str}::uuid[]) ELSE TRUE END
    -- Note: {doc_ids_slice_str} is bound as &[String] from Rust.
    -- The ::TEXT[] cast is for array_length, ::uuid[] is for the actual operation.

    -- For string search parameters
    AND CASE WHEN {search_for_sql_str} != '' THEN (LOWER(col.name) LIKE {search_for_sql_str} OR LOWER(c.html_url) LIKE {search_for_sql_str}) ELSE TRUE END
    ```

*   **Bind the Pre-processed Values in `conditional_query_as!`:**
    Remove the conditional fragment bindings (`#style_filter = match ...`). Bind the new, pre-processed variables directly. The placeholder name in the SQL string (e.g., `{doc_ids_slice_str}`) should match the variable name used on the left of the `match` in the Rust binding.

    ```rust
    // conditional_query_as!
    /*
    r#"
    ...
    WHERE base_condition
        AND CASE WHEN {style_id_for_sql_str} != '' THEN c.style_id = {style_id_for_sql_str}::uuid ELSE TRUE END
        AND CASE WHEN array_length({doc_ids_slice_str}::TEXT[], 1) > 0 THEN c.document_ids && ({doc_ids_slice_str}::uuid[]) ELSE TRUE END
        AND CASE WHEN {search_for_sql_str} != '' THEN (LOWER(col.name) LIKE {search_for_sql_str} OR LOWER(c.html_url) LIKE {search_for_sql_str}) ELSE TRUE END
    ...
    "#,
    // ... other fixed bindings ...
    #style_id_for_sql_str = match &style_id_for_sql_str { _ => "{style_id_for_sql_str}" },
    #doc_ids_slice_str = match &doc_ids_slice_str { _ => "{doc_ids_slice_str}" },
    #search_for_sql_str = match &search_for_sql_str { _ => "{search_for_sql_str}" },
    // ...
    */
    ```

## 3. Benefits of This Approach

*   **Reduced Query Variants:** Each optional filter now contributes only 1 structural variant to the query string, drastically reducing the total number of variants the macro needs to generate and `sqlx` needs to check. This leads to significantly faster compile times.
*   **Type Safety:** By binding a concrete type (like `String` or `&[String]`) whose "None" state is represented by a specific value (e.g., `""`), `sqlx` can clearly communicate the parameter type to PostgreSQL, avoiding runtime "could not determine data type" errors.
*   **Clear SQL Logic:** The `CASE WHEN` statements in SQL explicitly define how optional filters are applied.

## 4. Sorting and Other True Variants

*   For parts of the query that genuinely require different SQL syntax (like `ORDER BY column_name ASC/DESC` based on different sort parameters), the `match` arms producing different SQL literals in `conditional_query_as!` are still appropriate and necessary. The optimization above primarily targets optional `WHERE` clause conditions.

**In summary, when using `sqlx-conditional-queries` with optional filters, prefer pre-processing your `Option<T>` values in Rust to a concrete type with a sentinel for the `None` case, and then use static SQL `CASE WHEN` clauses that check this sentinel value. This balances dynamic query generation with compile-time performance and runtime type safety.**

---


# Debugging `sqlx` and `sqlx-conditional-queries`: A Troubleshooting Guide

Working with `sqlx` and its extension `sqlx-conditional-queries` provides compile-time checked SQL, which is a huge benefit. However, certain patterns, especially with dynamic query parts or custom types, can lead to errors. This guide summarizes common problems encountered and their solutions.

## 1. Problem: `syntax error at or near "$1"` with `QueryBuilder`

*   **Symptom:** Database error indicating a syntax error, often near the first placeholder (`$1`).
*   **Cause:** Incorrectly using `sqlx::QueryBuilder` by initializing it with a SQL string that *already contains* numbered placeholders (e.g., `QueryBuilder::new("SELECT * FROM t WHERE col = $1")`) and then calling `push_bind()`. `QueryBuilder` is designed to manage its own placeholders; mixing them with pre-existing numbered placeholders corrupts the query.
*   **Solution:**
    *   **Adhere to project rules:** If your project (like ours) restricts `QueryBuilder` and mandates macros, this is the primary driver for a different approach.
    *   **For fixed SQL with dynamic values:** Use `sqlx::query!`, `sqlx::query_as!`, or `sqlx::query_scalar!`. These macros correctly handle binding Rust variables to numbered placeholders.
    *   **For SQL with dynamic structural parts (e.g., `ORDER BY`, optional `WHERE` clauses):** Use `sqlx-conditional-queries::conditional_query_as!`. This macro is designed to generate different query variants at compile time.
    *   **If `QueryBuilder` is absolutely necessary (and allowed):** Ensure the initial SQL string passed to `QueryBuilder::new()` does *not* contain placeholders. Add all parts of the query, including placeholders, using `QueryBuilder`'s methods (`push()`, `push_bind()`).

## 2. Problem: `unsupported type <custom_enum_type> for param #N` with `conditional_query_as!`

*   **Symptom:** Compile-time error from `sqlx` (often originating in `expand_query` macro) stating that your custom PostgreSQL enum type (e.g., `object_share_entity_type`) is unsupported for a given parameter.
*   **Cause:** `sqlx-conditional-queries` (and `sqlx`'s type inference for it) can struggle to correctly bind Rust enum values directly to SQL placeholders when the intent is for them to be treated as a custom database enum type. The `as _` hint used with built-in `sqlx` macros (like `query_scalar!($1 as _)` for an enum) doesn't have a direct equivalent for the `{placeholder}` syntax in `conditional_query_as!`.
    *   Attempting to cast the string parameter in SQL like `({my_string_param}::my_db_enum_type)` can still lead to this error because `sqlx` might incorrectly infer that `my_string_param` *itself* should be of `my_db_enum_type` at the binding stage.
*   **Solution: Compare column as text with string parameter.**
    1.  **In Rust:** Ensure the variable you are binding is a simple string (e.g., `&str`) representing the enum variant (e.g., `"user"`, `"organization"`).
        ```rust
        let entity_type_str: &str = "user";
        ```
    2.  **In SQL (within `conditional_query_as!`)**: Cast the database *column* (which is of the enum type) to `text` and compare it against your bound string parameter.
        ```sql
        -- ... WHERE my_table.my_enum_column::text = {entity_type_str_placeholder} ...
        ```
    3.  **Binding:**
        ```rust
        // conditional_query_as! macro
        // ...
        #entity_type_str_placeholder = match &entity_type_str { _ => "{entity_type_str_placeholder}" },
        // ...
        ```
    *   **Why this works:** `sqlx` sees the parameter `{entity_type_str_placeholder}` being compared to `my_enum_column::text` (which is a `text` expression). It correctly infers that the parameter should be a text-compatible type from Rust (like `&str`), which it is. This avoids the problematic inference related to the custom enum type during parameter binding.

## 3. Problem: Trait bound not satisfied (`String: From<Option<String>>` or vice-versa)

*   **Symptom:** Compile-time error like `the trait bound std::string::String: std::convert::From<std::option::Option<std::string::String>> is not satisfied`.
*   **Cause:** Mismatch between the nullability of a field in your Rust struct and how `sqlx` interprets the nullability of the corresponding column from the SQL query result. This is often due to the `?` suffix in SQL aliases.
    *   In `sqlx` macros, an alias like `my_column AS "my_field?"` tells `sqlx` to treat `my_field` as an `Option<T>`.
    *   If your Rust struct has `my_field: T` (e.g., `String`), but the SQL alias is `"my_field?"`, you get an error because `sqlx` tries to map an `Option<String>` to a `String`.
    *   Conversely, if the struct has `my_field: Option<T>` but the alias is `"my_field"` (no `?`), and the column *can* be NULL, you might get a runtime error when a NULL is encountered.
*   **Solution: Ensure consistency.**
    *   **If the database column can be `NULL` OR if a `LEFT JOIN` could make it `NULL` in the result set:**
        *   The Rust struct field **must** be `Option<T>` (e.g., `Option<String>`).
        *   The SQL alias **should** use the `?` suffix: `my_column AS "my_field?"`.
    *   **If the database column is `NOT NULL` AND your query logic guarantees it will never be `NULL` in the result set:**
        *   The Rust struct field can be `T` (e.g., `String`).
        *   The SQL alias **must NOT** use the `?` suffix: `my_column AS "my_field"`.
    *   **Rule of thumb:** The `?` in the SQL alias must match the `Option<>` in the Rust struct field type for nullable columns.

## 4. Problem: `cannot find value <placeholder_name> in this scope` with `conditional_query_as!`

*   **Symptom:** Compile-time error indicating that a placeholder name used within the SQL string (e.g., `{my_param_bind}`) cannot be found.
*   **Cause:** The names used for placeholders *inside the SQL string* (e.g., `{name_in_sql}`) must exactly match the names used on the *left-hand side* of the binding assignments in the `conditional_query_as!` macro (e.g., `#name_in_rust_binding = ...`). The string on the right-hand side of the `match` in the binding (e.g., `"{name_for_sqlx_parsing}"`) is what `sqlx` uses for its internal parsing and must also match `name_in_sql`.
*   **Solution: Ensure all three names are consistent for each parameter.**
    1.  **Name in SQL string:** e.g., `WHERE column = {my_var}`
    2.  **Name on the left of the binding in `conditional_query_as!`:** e.g., `#my_var = ...` (this refers to the Rust variable `my_var` in the current scope).
    3.  **Name inside the `{}` on the right of the `match` in the binding:** e.g., `match &my_var { _ => "{my_var}" }` (this string literal must match what's in the SQL string).

    **Example:**
    ```rust
    // Rust variable
    let user_search_term = "%test%";

    // conditional_query_as!
    /*
    query_as!(
        MyStruct,
        r#"SELECT * FROM my_table WHERE name ILIKE {user_search_term}"#, // 1. Name in SQL
        // ...
        #user_search_term = match &user_search_term { // 2. Left side of binding (Rust var name)
            _ => "{user_search_term}"                 // 3. String literal for sqlx (matches #1)
        },
        // ...
    )
    */
    ```
    It's often simplest to use the exact Rust variable name as the placeholder in all three locations.

By understanding these common pitfalls and their solutions, you can more effectively use `sqlx` and `sqlx-conditional-queries` to build robust, compile-time checked database interactions.