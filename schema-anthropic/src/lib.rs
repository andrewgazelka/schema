use schema::SchemaType;
use serde_json::{Value, json};

/// Convert a Schema to Anthropic-compatible JSON Schema
///
/// Key differences from standard JSON Schema:
/// - Avoids oneOf for tagged unions
/// - Uses discriminator pattern instead
/// - Simpler enum representation
pub fn to_anthropic_schema(schema: &SchemaType) -> Value {
    use schema::TypeKind;

    let mut obj = serde_json::Map::new();

    // Add description if present
    if let Some(desc) = &schema.description {
        obj.insert("description".to_string(), json!(desc));
    }

    match &schema.kind {
        TypeKind::String => {
            obj.insert("type".to_string(), json!("string"));
        }

        TypeKind::Integer(_) => {
            obj.insert("type".to_string(), json!("integer"));
        }

        TypeKind::Number(_) => {
            obj.insert("type".to_string(), json!("number"));
        }

        TypeKind::Boolean => {
            obj.insert("type".to_string(), json!("boolean"));
        }

        TypeKind::Null => {
            obj.insert("type".to_string(), json!("null"));
        }

        TypeKind::Object {
            properties,
            required,
        } => {
            let mut props = serde_json::Map::new();
            for (key, value) in properties {
                props.insert(key.clone(), to_anthropic_schema(value));
            }

            obj.insert("type".to_string(), json!("object"));
            obj.insert("properties".to_string(), Value::Object(props));
            obj.insert("required".to_string(), json!(required));
        }

        TypeKind::Array { items } => {
            obj.insert("type".to_string(), json!("array"));
            obj.insert("items".to_string(), to_anthropic_schema(items));
        }

        TypeKind::Enum { variants } => {
            obj.insert("type".to_string(), json!("string"));
            obj.insert("enum".to_string(), json!(variants));
        }

        TypeKind::TaggedUnion {
            tag_field,
            tag_variants,
            data_fields,
        } => {
            // Instead of oneOf, create a flat object with:
            // - A discriminator field (tag_field)
            // - All possible data fields marked as optional
            let mut properties = serde_json::Map::new();

            // Add discriminator field
            properties.insert(
                tag_field.clone(),
                json!({
                    "type": "string",
                    "enum": tag_variants,
                }),
            );

            // Add all data fields (they're all optional since they depend on tag)
            for (field_name, field_schema) in data_fields {
                properties.insert(field_name.clone(), to_anthropic_schema(field_schema));
            }

            obj.insert("type".to_string(), json!("object"));
            obj.insert("properties".to_string(), Value::Object(properties));
            obj.insert("required".to_string(), json!([tag_field]));
        }

        TypeKind::Ref { name } => {
            return json!({ "$ref": format!("#/definitions/{}", name) });
        }
    }

    Value::Object(obj)
}

/// Helper to create a full tool schema for Anthropic
pub fn create_tool_schema(name: &str, description: &str, input_schema: &SchemaType) -> Value {
    json!({
        "name": name,
        "description": description,
        "input_schema": to_anthropic_schema(input_schema),
    })
}
