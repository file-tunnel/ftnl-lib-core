//! Canonical File Tunnel persistence contracts.
//!
//! Product services consume these contracts instead of maintaining independent
//! human-authored persistence schemas. This module is data-only: it opens no
//! database connections and performs no migration side effects.

use crate::schema::{CanonicalSchema, SchemaError};

const TUNNEL_PERSISTENCE_SCHEMA: &str = include_str!("../schema/tunnel-persistence.schema.json");

/// Parse the canonical File Tunnel persistence schema owned by `ftnl-lib-core`.
pub fn tunnel_persistence_schema() -> Result<CanonicalSchema, SchemaError> {
    CanonicalSchema::from_json(TUNNEL_PERSISTENCE_SCHEMA)
}

#[cfg(test)]
mod tests {
    use crate::generate_create_table;

    use super::*;

    #[test]
    fn tunnel_schema_is_additive_and_excludes_raw_authority_material() {
        let schema = tunnel_persistence_schema().expect("canonical schema parses");
        let properties = schema.raw()["properties"]
            .as_object()
            .expect("properties object");

        for prohibited in [
            "desktopCapability",
            "phoneCapability",
            "pairingSecret",
            "eventTicket",
            "fileBytes",
        ] {
            assert!(
                !properties.contains_key(prohibited),
                "{prohibited} must not be persisted"
            );
        }

        let sql = generate_create_table(&schema).as_script();
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"file_tunnels\""));
        for destructive in ["DROP ", "TRUNCATE ", "DELETE ", "ALTER "] {
            assert!(
                !sql.contains(destructive),
                "unexpected destructive SQL: {destructive}"
            );
        }
    }
}
