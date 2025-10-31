//! Defines a struct for deserializing SQL count results.
//!
//! This module provides a simple struct `TotalCount` which is used
//! to map the result of SQL queries like `SELECT COUNT(*) AS count ...`.
//! This helps in providing a more structured way to handle count results
//! compared to raw scalar types.
//! Adheres to one item per file guideline.

/// Represents the result of a SQL COUNT query.
///
/// The `count` field holds the value from the SQL COUNT aggregate.
#[derive(sqlx::FromRow, serde::Deserialize, Debug)]
pub struct TotalCount {
    pub count: Option<i64>,
}