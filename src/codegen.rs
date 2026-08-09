use crate::{CanonicalSchema, FieldKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCode {
    pub language: &'static str,
    pub source: String,
}

pub fn generate_rust(schema: &CanonicalSchema) -> GeneratedCode {
    let name = pascal_case(&schema.title);
    let mut source =
        String::from("#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\n");
    source.push_str(&format!("pub struct {name} {{\n"));
    for field in schema.fields.values() {
        let identifier = rust_identifier(&field.json_name);
        if identifier != field.json_name {
            source.push_str(&format!("    #[serde(rename = {:?})]\n", field.json_name));
        }
        let ty = rust_type(&field.kind);
        let ty = if field.required {
            ty.to_owned()
        } else {
            format!("Option<{ty}>")
        };
        source.push_str(&format!("    pub {identifier}: {ty},\n"));
    }
    source.push_str("}\n");
    GeneratedCode {
        language: "rust",
        source,
    }
}

pub fn generate_typescript(schema: &CanonicalSchema) -> GeneratedCode {
    let name = pascal_case(&schema.title);
    let mut source = format!("export interface {name} {{\n");
    for field in schema.fields.values() {
        let key = serde_json::to_string(&field.json_name).expect("JSON string serialization");
        let optional = if field.required { "" } else { "?" };
        source.push_str(&format!(
            "  {key}{optional}: {};\n",
            typescript_type(&field.kind)
        ));
    }
    source.push_str("}\n");
    GeneratedCode {
        language: "typescript",
        source,
    }
}

pub fn generate_dart(schema: &CanonicalSchema) -> GeneratedCode {
    let name = pascal_case(&schema.title);
    let mut source = format!("final class {name} {{\n");
    for field in schema.fields.values() {
        let nullable = if field.required { "" } else { "?" };
        source.push_str(&format!(
            "  final {}{nullable} {};\n",
            dart_type(&field.kind),
            dart_identifier(&field.json_name)
        ));
    }
    source.push_str(&format!("\n  const {name}({{\n"));
    for field in schema.fields.values() {
        let required = if field.required { "required " } else { "" };
        source.push_str(&format!(
            "    {required}this.{},\n",
            dart_identifier(&field.json_name)
        ));
    }
    source.push_str("  });\n}\n");
    GeneratedCode {
        language: "dart",
        source,
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
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        let safe = if character.is_ascii_alphanumeric() || character == '_' {
            character
        } else {
            '_'
        };
        if index == 0 && safe.is_ascii_digit() {
            output.push('_');
        }
        output.push(safe);
    }
    if output.is_empty() {
        "field".to_owned()
    } else {
        output
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
}
