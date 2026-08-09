use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use thiserror::Error;

const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    String,
    Uuid,
    Timestamp,
    Integer,
    Number,
    Boolean,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub json_name: String,
    pub column_name: String,
    pub kind: FieldKind,
    pub required: bool,
    pub primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct CanonicalSchema {
    raw: Value,
    pub title: String,
    pub table_name: String,
    pub fields: BTreeMap<String, Field>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchemaError {
    #[error("schema must be a JSON object")]
    NotObject,
    #[error("schema must declare JSON Schema draft 2020-12")]
    UnsupportedDraft,
    #[error("schema root type must be object")]
    RootMustBeObject,
    #[error("schema requires a non-empty title")]
    MissingTitle,
    #[error("schema properties must be an object")]
    InvalidProperties,
    #[error("schema required must be an array of unique property names")]
    InvalidRequired,
    #[error("property {0:?} has an unsupported or ambiguous type")]
    UnsupportedProperty(String),
    #[error("identifier {0:?} is not a safe SQL identifier")]
    InvalidIdentifier(String),
    #[error("primary key field {0:?} must be required")]
    OptionalPrimaryKey(String),
    #[error("schema instance is invalid: {0}")]
    InvalidInstance(String),
    #[error("schema could not be compiled: {0}")]
    InvalidSchema(String),
}

impl CanonicalSchema {
    pub fn from_value(raw: Value) -> Result<Self, SchemaError> {
        let object = raw.as_object().ok_or(SchemaError::NotObject)?;
        if object.get("$schema").and_then(Value::as_str) != Some(DRAFT_2020_12) {
            return Err(SchemaError::UnsupportedDraft);
        }
        if object.get("type").and_then(Value::as_str) != Some("object") {
            return Err(SchemaError::RootMustBeObject);
        }
        let title = object
            .get("title")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or(SchemaError::MissingTitle)?
            .trim()
            .to_owned();
        let table_name = object
            .get("x-ftnl-table")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| to_snake_case(&title));
        validate_identifier(&table_name)?;

        let properties = object
            .get("properties")
            .and_then(Value::as_object)
            .ok_or(SchemaError::InvalidProperties)?;
        let required = parse_required(object.get("required"), properties)?;
        let mut fields = BTreeMap::new();
        for (json_name, definition) in properties {
            let definition = definition
                .as_object()
                .ok_or_else(|| SchemaError::UnsupportedProperty(json_name.clone()))?;
            let column_name = definition
                .get("x-ftnl-column")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| to_snake_case(json_name));
            validate_identifier(&column_name)?;
            let kind = field_kind(definition)
                .ok_or_else(|| SchemaError::UnsupportedProperty(json_name.clone()))?;
            let is_required = required.contains(json_name);
            let primary_key = definition
                .get("x-ftnl-primary-key")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if primary_key && !is_required {
                return Err(SchemaError::OptionalPrimaryKey(json_name.clone()));
            }
            fields.insert(
                json_name.clone(),
                Field {
                    json_name: json_name.clone(),
                    column_name,
                    kind,
                    required: is_required,
                    primary_key,
                },
            );
        }

        jsonschema::options()
            .should_validate_formats(true)
            .build(&raw)
            .map_err(|error| SchemaError::InvalidSchema(error.to_string()))?;
        Ok(Self {
            raw,
            title,
            table_name,
            fields,
        })
    }

    pub fn from_json(json: &str) -> Result<Self, SchemaError> {
        let value = serde_json::from_str(json)
            .map_err(|error| SchemaError::InvalidSchema(error.to_string()))?;
        Self::from_value(value)
    }

    pub fn validate_instance(&self, instance: &Value) -> Result<(), SchemaError> {
        jsonschema::options()
            .should_validate_formats(true)
            .build(&self.raw)
            .map_err(|error| SchemaError::InvalidSchema(error.to_string()))?
            .validate(instance)
            .map_err(|error| SchemaError::InvalidInstance(error.to_string()))
    }

    pub fn raw(&self) -> &Value {
        &self.raw
    }
}

fn parse_required(
    value: Option<&Value>,
    properties: &serde_json::Map<String, Value>,
) -> Result<BTreeSet<String>, SchemaError> {
    let Some(value) = value else {
        return Ok(BTreeSet::new());
    };
    let values = value.as_array().ok_or(SchemaError::InvalidRequired)?;
    let mut required = BTreeSet::new();
    for item in values {
        let name = item.as_str().ok_or(SchemaError::InvalidRequired)?;
        if !properties.contains_key(name) || !required.insert(name.to_owned()) {
            return Err(SchemaError::InvalidRequired);
        }
    }
    Ok(required)
}

fn field_kind(definition: &serde_json::Map<String, Value>) -> Option<FieldKind> {
    match definition.get("type")?.as_str()? {
        "string" => match definition.get("format").and_then(Value::as_str) {
            Some("uuid") => Some(FieldKind::Uuid),
            Some("date-time") => Some(FieldKind::Timestamp),
            Some(_) | None => Some(FieldKind::String),
        },
        "integer" => Some(FieldKind::Integer),
        "number" => Some(FieldKind::Number),
        "boolean" => Some(FieldKind::Boolean),
        "object" | "array" => Some(FieldKind::Json),
        _ => None,
    }
}

pub(crate) fn validate_identifier(value: &str) -> Result<(), SchemaError> {
    let mut chars = value.chars();
    let valid_first = chars
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_lowercase());
    if !valid_first
        || value.len() > 63
        || !chars.all(|character| {
            character == '_' || character.is_ascii_lowercase() || character.is_ascii_digit()
        })
    {
        return Err(SchemaError::InvalidIdentifier(value.to_owned()));
    }
    Ok(())
}

pub(crate) fn to_snake_case(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase() && !output.is_empty() && !output.ends_with('_') {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
        } else if !output.is_empty() && !output.ends_with('_') {
            output.push('_');
        }
    }
    output.trim_matches('_').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn schema() -> Value {
        json!({
            "$schema": DRAFT_2020_12,
            "title": "Transfer Record",
            "type": "object",
            "additionalProperties": false,
            "required": ["id", "sizeBytes"],
            "properties": {
                "sizeBytes": {"type": "integer"},
                "id": {"type": "string", "format": "uuid", "x-ftnl-primary-key": true},
                "metadata": {"type": "object"}
            },
            "x-ftnl-table": "transfer_records"
        })
    }

    #[test]
    fn normalizes_fields_deterministically() {
        let schema = CanonicalSchema::from_value(schema()).unwrap();
        assert_eq!(schema.table_name, "transfer_records");
        assert_eq!(
            schema.fields.keys().cloned().collect::<Vec<_>>(),
            ["id", "metadata", "sizeBytes"]
        );
        assert_eq!(schema.fields["sizeBytes"].column_name, "size_bytes");
    }

    #[test]
    fn rejects_identifier_injection() {
        let mut schema = schema();
        schema["x-ftnl-table"] = json!("records; DROP TABLE users");
        assert!(matches!(
            CanonicalSchema::from_value(schema),
            Err(SchemaError::InvalidIdentifier(_))
        ));
    }

    #[test]
    fn validates_instances_against_the_source_schema() {
        let schema = CanonicalSchema::from_value(schema()).unwrap();
        assert!(schema
            .validate_instance(
                &json!({"id": "f5cc96e7-9a11-4b9f-97fb-d0f504494c4e", "sizeBytes": 12})
            )
            .is_ok());
        assert!(schema
            .validate_instance(&json!({"id": "not-a-uuid", "sizeBytes": 12}))
            .is_err());
    }
}
