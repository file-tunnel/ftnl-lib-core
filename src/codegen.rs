use crate::{CanonicalSchema, FieldKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCode {
    pub language: &'static str,
    pub source: String,
}

pub fn generate_rust(schema: &CanonicalSchema) -> GeneratedCode {
    let name = pascal_case(&schema.title);
    let fields = schema
        .fields
        .values()
        .map(|field| {
            let identifier = rust_identifier(&field.json_name);
            let rename = if identifier == field.json_name {
                String::new()
            } else {
                format!("    #[serde(rename = {:?})]\n", field.json_name)
            };
            let ty = rust_type(&field.kind);
            let ty = if field.required {
                ty.to_owned()
            } else {
                format!("Option<{ty}>")
            };
            format!("{rename}    pub {identifier}: {ty},\n")
        })
        .collect::<String>();
    GeneratedCode {
        language: "rust",
        source: format!(
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\npub struct {name} {{\n{fields}}}\n"
        ),
    }
}

pub fn generate_typescript(schema: &CanonicalSchema) -> GeneratedCode {
    let name = pascal_case(&schema.title);
    let fields = schema
        .fields
        .values()
        .map(|field| {
            let key = serde_json::to_string(&field.json_name).expect("JSON string serialization");
            let optional = if field.required { "" } else { "?" };
            format!("  {key}{optional}: {};\n", typescript_type(&field.kind))
        })
        .collect::<String>();
    GeneratedCode {
        language: "typescript",
        source: format!("export interface {name} {{\n{fields}}}\n"),
    }
}

pub fn generate_dart(schema: &CanonicalSchema) -> GeneratedCode {
    let name = pascal_case(&schema.title);
    let declarations = schema
        .fields
        .values()
        .map(|field| {
            let nullable = if field.required { "" } else { "?" };
            format!(
                "  final {}{nullable} {};\n",
                dart_type(&field.kind),
                dart_identifier(&field.json_name)
            )
        })
        .collect::<String>();
    let parameters = schema
        .fields
        .values()
        .map(|field| {
            let required = if field.required { "required " } else { "" };
            format!(
                "    {required}this.{},\n",
                dart_identifier(&field.json_name)
            )
        })
        .collect::<String>();
    GeneratedCode {
        language: "dart",
        source: format!(
            "final class {name} {{\n{declarations}\n  const {name}({{\n{parameters}  }});\n}}\n"
        ),
    }
}

fn rust_type(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::String | FieldKind::Uuid | FieldKind::Timestamp => "String",
        FieldKind::Integer => "i64",
        FieldKind::Number => "f64",
        FieldKind::Boolean => "bool",
        FieldKind::Json => "serde_json::Value",
    }
}

fn typescript_type(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::String | FieldKind::Uuid | FieldKind::Timestamp => "string",
        FieldKind::Integer | FieldKind::Number => "number",
        FieldKind::Boolean => "boolean",
        FieldKind::Json => "unknown",
    }
}

fn dart_type(kind: &FieldKind) -> &'static str {
    match kind {
        FieldKind::String | FieldKind::Uuid | FieldKind::Timestamp => "String",
        FieldKind::Integer => "int",
        FieldKind::Number => "double",
        FieldKind::Boolean => "bool",
        FieldKind::Json => "Object",
    }
}

fn pascal_case(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + characters.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn safe_identifier(value: &str) -> String {
    let sanitized = value
        .chars()
        .enumerate()
        .flat_map(|(index, character)| {
            let safe = if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            };
            let leading_underscore = (index == 0 && safe.is_ascii_digit()).then_some('_');
            leading_underscore.into_iter().chain(std::iter::once(safe))
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "field".to_owned()
    } else {
        sanitized
    }
}

fn rust_identifier(value: &str) -> String {
    let value = safe_identifier(value);
    match value.as_str() {
        "as" | "break" | "const" | "crate" | "else" | "enum" | "extern" | "false" | "fn"
        | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
        | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" | "trait"
        | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await" | "dyn" => {
            format!("r#{value}")
        }
        _ => value,
    }
}

fn dart_identifier(value: &str) -> String {
    let value = safe_identifier(value);
    match value.as_str() {
        "class" | "const" | "final" | "var" | "void" | "dynamic" | "import" | "export"
        | "return" | "this" | "type" => format!("{value}Value"),
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use crate::CanonicalSchema;

    use super::*;

    #[test]
    fn generators_are_deterministic_and_escape_identifiers() {
        let schema = CanonicalSchema::from_json(
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","title":"Transfer Job","type":"object","required":["type"],"properties":{"type":{"type":"string"},"sizeBytes":{"type":"integer"}}}"#,
        )
        .unwrap();
        let rust = generate_rust(&schema).source;
        assert!(rust.contains("pub r#type: String"));
        assert!(rust.contains("pub sizeBytes: Option<i64>"));
        assert_eq!(rust, generate_rust(&schema).source);
        assert!(generate_typescript(&schema)
            .source
            .contains("\"sizeBytes\"?: number"));
        assert!(generate_dart(&schema)
            .source
            .contains("final String typeValue"));
    }

    #[test]
    fn generated_sources_keep_their_exact_layout() {
        let schema = CanonicalSchema::from_json(
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","title":"Transfer Job","type":"object","required":["type"],"properties":{"type":{"type":"string"},"sizeBytes":{"type":"integer"}}}"#,
        )
        .unwrap();
        assert_eq!(
            generate_rust(&schema).source,
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\npub struct TransferJob {\n    pub sizeBytes: Option<i64>,\n    #[serde(rename = \"type\")]\n    pub r#type: String,\n}\n"
        );
        assert_eq!(
            generate_typescript(&schema).source,
            "export interface TransferJob {\n  \"sizeBytes\"?: number;\n  \"type\": string;\n}\n"
        );
        assert_eq!(
            generate_dart(&schema).source,
            "final class TransferJob {\n  final int? sizeBytes;\n  final String typeValue;\n\n  const TransferJob({\n    this.sizeBytes,\n    required this.typeValue,\n  });\n}\n"
        );
    }

    #[test]
    fn identifiers_are_sanitized_and_never_start_with_a_digit() {
        assert_eq!(safe_identifier("2fa-token"), "_2fa_token");
        assert_eq!(safe_identifier("plain_name"), "plain_name");
        assert_eq!(safe_identifier(""), "field");
    }
}
