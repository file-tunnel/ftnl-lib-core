use crate::{CanonicalSchema, FieldKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlPlan {
    pub statements: Vec<String>,
}

impl SqlPlan {
    pub fn as_script(&self) -> String {
        let mut script = self.statements.join("\n");
        script.push('\n');
        script
    }
}

/// Generates only additive bootstrap DDL. Drift and migration SQL belong to
/// the reviewed `dpm diff`/`dpm verify` boundary in [`crate::dpm`].
pub fn generate_create_table(schema: &CanonicalSchema) -> SqlPlan {
    let columns = schema
        .fields
        .values()
        .map(|field| {
            let mut column = format!(
                "  {} {}",
                quote_identifier(&field.column_name),
                postgres_type(&field.kind)
            );
            if field.required {
                column.push_str(" NOT NULL");
            }
            if field.primary_key {
                column.push_str(" PRIMARY KEY");
            }
            column
        })
        .collect::<Vec<_>>()
        .join(",\n");
    SqlPlan {
        statements: vec![format!(
            "CREATE TABLE IF NOT EXISTS {} (\n{}\n);",
            quote_identifier(&schema.table_name),
            columns
        )],
    }
}

pub(crate) fn quote_identifier(value: &str) -> String {
    format!("\"{value}\"")
}

fn postgres_type(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::String => "TEXT",
        FieldKind::Uuid => "UUID",
        FieldKind::Timestamp => "TIMESTAMPTZ",
        FieldKind::Integer => "BIGINT",
        FieldKind::Number => "DOUBLE PRECISION",
        FieldKind::Boolean => "BOOLEAN",
        FieldKind::Json => "JSONB",
    }
}

#[cfg(test)]
mod tests {
    use crate::CanonicalSchema;

    use super::*;

    #[test]
    fn bootstrap_sql_is_additive_and_deterministic() {
        let schema = CanonicalSchema::from_json(
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","title":"Job","type":"object","x-ftnl-table":"jobs","required":["id"],"properties":{"name":{"type":"string"},"id":{"type":"string","format":"uuid","x-ftnl-primary-key":true}}}"#,
        )
        .unwrap();
        let script = generate_create_table(&schema).as_script();
        assert_eq!(script, "CREATE TABLE IF NOT EXISTS \"jobs\" (\n  \"id\" UUID NOT NULL PRIMARY KEY,\n  \"name\" TEXT\n);\n");
        for destructive in ["DROP ", "TRUNCATE ", "DELETE ", "ALTER "] {
            assert!(!script.contains(destructive));
        }
    }
}
