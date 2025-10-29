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

        TypeKind::Set { items, .. } => {
            obj.insert("type".to_string(), json!("array"));
            obj.insert("items".to_string(), to_anthropic_schema(items));
            obj.insert("uniqueItems".to_string(), json!(true));
        }

        TypeKind::Map { key, value, .. } => {
            // If key is String, use additionalProperties
            if matches!(key.kind, TypeKind::String) {
                obj.insert("type".to_string(), json!("object"));
                obj.insert(
                    "additionalProperties".to_string(),
                    to_anthropic_schema(value),
                );
            } else {
                // For non-string keys, use array of tuples
                let tuple_schema = SchemaType {
                    kind: TypeKind::Tuple {
                        fields: vec![(**key).clone(), (**value).clone()],
                    },
                    description: None,
                };
                obj.insert("type".to_string(), json!("array"));
                obj.insert("items".to_string(), to_anthropic_schema(&tuple_schema));
            }
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

        TypeKind::Variant { cases } => {
            // Similar to TaggedUnion but with proper per-case structure
            // Flatten for Anthropic compatibility
            let mut properties = serde_json::Map::new();

            // Add discriminator field
            let tag_variants: Vec<String> = cases.iter().map(|c| c.name.clone()).collect();
            properties.insert(
                "type".to_string(),
                json!({
                    "type": "string",
                    "enum": tag_variants,
                }),
            );

            // Collect all unique fields from all cases
            let mut all_fields = std::collections::HashMap::new();
            for case in cases {
                if let Some(data) = &case.data {
                    if let TypeKind::Object {
                        properties: props, ..
                    } = &data.kind
                    {
                        for (field_name, field_schema) in props {
                            all_fields
                                .entry(field_name.clone())
                                .or_insert_with(|| field_schema.clone());
                        }
                    }
                }
            }

            // Add all fields as optional
            for (field_name, field_schema) in all_fields {
                properties.insert(field_name, to_anthropic_schema(&field_schema));
            }

            obj.insert("type".to_string(), json!("object"));
            obj.insert("properties".to_string(), Value::Object(properties));
            obj.insert("required".to_string(), json!(["type"]));
        }

        TypeKind::Result { ok, err } => {
            // Represent as union with ok/error fields
            let mut properties = serde_json::Map::new();
            properties.insert("ok".to_string(), to_anthropic_schema(ok));
            properties.insert("error".to_string(), to_anthropic_schema(err));

            obj.insert("type".to_string(), json!("object"));
            obj.insert("properties".to_string(), Value::Object(properties));
            obj.insert(
                "description".to_string(),
                json!("Result type - exactly one of ok or error will be present"),
            );
        }

        TypeKind::Tuple { fields } => {
            // Represent as fixed-length array
            if fields.is_empty() {
                obj.insert("type".to_string(), json!("array"));
                obj.insert("maxItems".to_string(), json!(0));
            } else {
                let items: Vec<Value> = fields.iter().map(to_anthropic_schema).collect();
                obj.insert("type".to_string(), json!("array"));
                obj.insert("prefixItems".to_string(), json!(items));
                obj.insert("minItems".to_string(), json!(fields.len()));
                obj.insert("maxItems".to_string(), json!(fields.len()));
            }
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
