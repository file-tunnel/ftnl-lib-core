use serde_json::Value;
use thiserror::Error;

use crate::sql::quote_identifier;
use crate::{CanonicalSchema, SchemaError};

#[derive(Debug, Clone, PartialEq)]
pub struct EntityRecord {
    pub values: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertPlan {
    pub sql: String,
    pub parameters: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPlan {
    pub sql: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OrmError {
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error("entity must be a JSON object")]
    NotObject,
    #[error("schema does not declare exactly one primary key")]
    PrimaryKey,
}

impl EntityRecord {
    pub fn validated(schema: &CanonicalSchema, values: Value) -> Result<Self, OrmError> {
        schema.validate_instance(&values)?;
        if !values.is_object() {
            return Err(OrmError::NotObject);
        }
        Ok(Self { values })
    }

    pub fn insert_plan(&self, schema: &CanonicalSchema) -> Result<InsertPlan, OrmError> {
        let object = self.values.as_object().ok_or(OrmError::NotObject)?;
        let present = schema
            .fields
            .values()
            .filter_map(|field| {
                object
                    .get(&field.json_name)
                    .map(|value| (field, value.clone()))
            })
            .collect::<Vec<_>>();
        let columns = present
            .iter()
            .map(|(field, _)| quote_identifier(&field.column_name))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = (1..=present.len())
            .map(|index| format!("${index}"))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(InsertPlan {
            sql: format!(
                "INSERT INTO {} ({columns}) VALUES ({placeholders})",
                quote_identifier(&schema.table_name)
            ),
            parameters: present.into_iter().map(|(_, value)| value).collect(),
        })
    }
}

impl QueryPlan {
    pub fn select_by_primary_key(schema: &CanonicalSchema) -> Result<Self, OrmError> {
        let primary_keys = schema
            .fields
            .values()
            .filter(|field| field.primary_key)
            .collect::<Vec<_>>();
        let [primary_key] = primary_keys.as_slice() else {
            return Err(OrmError::PrimaryKey);
        };
        Ok(Self {
            sql: format!(
                "SELECT * FROM {} WHERE {} = $1",
                quote_identifier(&schema.table_name),
                quote_identifier(&primary_key.column_name)
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn schema() -> CanonicalSchema {
        CanonicalSchema::from_json(
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","title":"Job","type":"object","x-ftnl-table":"jobs","required":["id","name"],"properties":{"name":{"type":"string"},"id":{"type":"string","format":"uuid","x-ftnl-primary-key":true}}}"#,
        )
        .unwrap()
    }

    #[test]
    fn values_never_enter_generated_sql() {
        let record = EntityRecord::validated(
            &schema(),
            json!({"id":"f5cc96e7-9a11-4b9f-97fb-d0f504494c4e","name":"Robert'); DROP TABLE jobs;--"}),
        )
        .unwrap();
        let plan = record.insert_plan(&schema()).unwrap();
        assert_eq!(
            plan.sql,
            "INSERT INTO \"jobs\" (\"id\", \"name\") VALUES ($1, $2)"
        );
        assert!(!plan.sql.contains("Robert"));
        assert_eq!(plan.parameters.len(), 2);
    }
}
