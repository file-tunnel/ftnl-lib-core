//! Shared schema, generation, and persistence planning for File Tunnel.
//!
//! This crate is intentionally runtime-neutral. It does not open databases,
//! install a logger, retain credentials, or apply migrations. Applications own
//! those effects and may opt into the bounded [`dpm`] process adapter.

pub mod codegen;
pub mod dpm;
pub mod orm;
pub mod schema;
pub mod sql;

pub use codegen::{generate_dart, generate_rust, generate_typescript, GeneratedCode};
pub use orm::{EntityRecord, InsertPlan, QueryPlan};
pub use schema::{CanonicalSchema, Field, FieldKind, SchemaError};
pub use sql::{generate_create_table, SqlPlan};
