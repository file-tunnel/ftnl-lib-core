# ftnl-lib-core

The shared schema and persistence-planning foundation for File Tunnel. It turns
reviewed JSON Schema Draft 2020-12 documents into one deterministic,
ORM-neutral model and can produce:

- validated runtime records;
- Rust, TypeScript, and Dart data types;
- additive PostgreSQL bootstrap DDL;
- parameterized insert and primary-key query plans; and
- bounded `dpm diff`, `dpm verify`, and `dpm bootstrap` process requests.

This repository is deliberately not a database service or migration runner. It
does not open a database, hold credentials, apply migrations, install a global
logger, or know anything about pairing capabilities and file contents.

## Canonical schema extensions

The input remains ordinary JSON Schema Draft 2020-12. File Tunnel recognizes
three optional, narrowly typed annotations:

| Annotation | Location | Purpose |
|---|---|---|
| `x-ftnl-table` | root | safe PostgreSQL table identifier |
| `x-ftnl-column` | property | safe PostgreSQL column identifier |
| `x-ftnl-primary-key` | property | declares a required primary-key field |

Identifiers are limited to lowercase ASCII letters, digits, and underscores;
arbitrary SQL types/defaults are intentionally not accepted. Object and array
fields map to `JSONB`; `uuid` and `date-time` formats map to PostgreSQL `UUID`
and `TIMESTAMPTZ`.

## Declarative migrations

The Zed package depends on
[`declarative-postgres-migrate.rs`](https://github.com/declarative-migrations/declarative-postgres-migrate.rs).
`DpmCli` invokes its `dpm` executable directly—never through a shell—with a
timeout, bounded stdout/stderr, null stdin, redacted database arguments, and no
representable `apply` operation. Set `FTNL_DPM_BIN` when Zed or another package
manager installs the binary outside `PATH`.

Applications should write `generate_create_table(...).as_script()` to a
reviewable file, then call `dpm diff` or `dpm verify` against their target. Any
eventual apply remains an explicit operator-owned DPM action with DPM's own
destructive-change consent gates.

## Usage

```rust
use ftnl_lib_core::{CanonicalSchema, EntityRecord, generate_create_table};

let schema = CanonicalSchema::from_json(include_str!("schema.json"))?;
let sql = generate_create_table(&schema).as_script();
let record = EntityRecord::validated(&schema, serde_json::json!({
    "id": "f5cc96e7-9a11-4b9f-97fb-d0f504494c4e"
}))?;
let insert = record.insert_plan(&schema)?;
assert!(insert.sql.contains("$1"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Validate

```bash
nix develop --command agent-check
```

MIT licensed.
