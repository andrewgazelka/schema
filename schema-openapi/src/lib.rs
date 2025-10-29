use schema::{Schema, SchemaType, TypeKind};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Convert a Schema to OpenAPI 3.0 schema format
pub fn to_openapi_schema<T: Schema>() -> Value {
    schema_type_to_openapi(&T::schema())
}

fn schema_type_to_openapi(schema: &SchemaType) -> Value {
    let mut result = match &schema.kind {
        TypeKind::String => json!({ "type": "string" }),
        TypeKind::Number(_) => json!({ "type": "number" }),
        TypeKind::Integer(_) => json!({ "type": "integer" }),
        TypeKind::Boolean => json!({ "type": "boolean" }),
        TypeKind::Null => json!({ "type": "null" }),
        TypeKind::Array { items } => {
            json!({
                "type": "array",
                "items": schema_type_to_openapi(items)
            })
        }
        TypeKind::Object {
            properties,
            required,
        } => {
            let props: HashMap<String, Value> = properties
                .iter()
                .map(|(k, v)| (k.clone(), schema_type_to_openapi(v)))
                .collect();

            let mut obj = json!({
                "type": "object",
                "properties": props
            });

            if !required.is_empty() {
                obj["required"] = json!(required);
            }

            obj
        }
        TypeKind::Enum { variants } => {
            json!({
                "type": "string",
                "enum": variants
            })
        }
        TypeKind::TaggedUnion {
            tag_field,
            tag_variants,
            data_fields,
        } => {
            // Legacy: For OpenAPI, represent as oneOf with discriminator
            let mut schemas = Vec::new();

            for variant in tag_variants {
                let mut props: HashMap<String, Value> = data_fields
                    .iter()
                    .map(|(k, v)| (k.clone(), schema_type_to_openapi(v)))
                    .collect();

                // Add tag field
                props.insert(
                    tag_field.clone(),
                    json!({
                        "type": "string",
                        "enum": [variant]
                    }),
                );

                schemas.push(json!({
                    "type": "object",
                    "properties": props,
                    "required": [tag_field]
                }));
            }

            json!({
                "oneOf": schemas,
                "discriminator": {
                    "propertyName": tag_field
                }
            })
        }
        TypeKind::Variant { cases } => {
            // Proper variant type - OpenAPI oneOf without forced discriminator
            let schemas: Vec<Value> = cases
                .iter()
                .map(|case| {
                    match &case.data {
                        None => {
                            // Unit variant - represent as const string
                            json!({
                                "type": "string",
                                "const": case.name
                            })
                        }
                        Some(data) => {
                            // Variant with data - wrap in object with tag
                            let data_schema = schema_type_to_openapi(data);
                            let mut obj = json!({
                                "type": "object",
                                "properties": {
                                    "type": {
                                        "type": "string",
                                        "const": case.name
                                    },
                                    "data": data_schema
                                },
                                "required": ["type", "data"]
                            });

                            if let Some(desc) = &case.description {
                                obj["description"] = json!(desc);
                            }
                            obj
                        }
                    }
                })
                .collect();

            json!({ "oneOf": schemas })
        }
        TypeKind::Result { ok, err } => {
            // Result type - OpenAPI oneOf with ok/error variants
            json!({
                "oneOf": [
                    {
                        "type": "object",
                        "properties": {
                            "ok": schema_type_to_openapi(ok)
                        },
                        "required": ["ok"]
                    },
                    {
                        "type": "object",
                        "properties": {
                            "error": schema_type_to_openapi(err)
                        },
                        "required": ["error"]
                    }
                ]
            })
        }
        TypeKind::Tuple { fields } => {
            // Tuple - OpenAPI array with fixed items
            if fields.is_empty() {
                json!({ "type": "array", "maxItems": 0 })
            } else {
                let items: Vec<Value> = fields
                    .iter()
                    .map(schema_type_to_openapi)
                    .collect();
                json!({
                    "type": "array",
                    "prefixItems": items,
                    "minItems": fields.len(),
                    "maxItems": fields.len()
                })
            }
        }
        TypeKind::Ref { name } => {
            json!({
                "$ref": format!("#/components/schemas/{}", name)
            })
        }
    };

    // Add description if present
    if let Some(desc) = &schema.description {
        result["description"] = json!(desc);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        #[derive(Schema)]
        struct Person {
            name: String,
            age: u32,
        }

        let openapi = to_openapi_schema::<Person>();
        assert_eq!(openapi["type"], "object");
        assert_eq!(openapi["properties"]["name"]["type"], "string");
        assert_eq!(openapi["properties"]["age"]["type"], "integer");
        assert_eq!(openapi["required"][0], "name");
        assert_eq!(openapi["required"][1], "age");
    }

    #[test]
    fn test_optional_fields() {
        #[derive(Schema)]
        struct User {
            id: String,
            email: Option<String>,
            bio: Option<String>,
        }

        let openapi = to_openapi_schema::<User>();
        assert_eq!(openapi["type"], "object");
        assert_eq!(openapi["required"].as_array().unwrap().len(), 1);
        assert_eq!(openapi["required"][0], "id");
    }

    #[test]
    fn test_nested_objects() {
        #[derive(Schema)]
        struct Address {
            street: String,
            city: String,
        }

        #[derive(Schema)]
        struct Person {
            name: String,
            address: Address,
        }

        let openapi = to_openapi_schema::<Person>();
        assert_eq!(openapi["type"], "object");
        assert_eq!(openapi["properties"]["address"]["type"], "object");
        assert_eq!(
            openapi["properties"]["address"]["properties"]["street"]["type"],
            "string"
        );
        assert_eq!(
            openapi["properties"]["address"]["properties"]["city"]["type"],
            "string"
        );
    }

    #[test]
    fn test_arrays() {
        #[derive(Schema)]
        struct Team {
            name: String,
            members: Vec<String>,
        }

        let openapi = to_openapi_schema::<Team>();
        assert_eq!(openapi["type"], "object");
        assert_eq!(openapi["properties"]["members"]["type"], "array");
        assert_eq!(openapi["properties"]["members"]["items"]["type"], "string");
    }

    #[test]
    fn test_simple_enum() {
        #[derive(Schema)]
        enum Status {
            Active,
            Inactive,
            Pending,
        }

        let openapi = to_openapi_schema::<Status>();
        assert_eq!(openapi["type"], "string");
        let variants = openapi["enum"].as_array().unwrap();
        assert_eq!(variants.len(), 3);
        assert!(variants.iter().any(|v| v == "active"));
        assert!(variants.iter().any(|v| v == "inactive"));
        assert!(variants.iter().any(|v| v == "pending"));
    }

    #[test]
    fn test_variant() {
        #[derive(Schema)]
        enum Message {
            Text {
                content: String,
            },
            Image {
                url: String,
                width: u32,
                height: u32,
            },
        }

        let openapi = to_openapi_schema::<Message>();
        assert!(openapi.get("oneOf").is_some());

        // New Variant type generates proper per-case structure
        let cases = openapi["oneOf"].as_array().unwrap();
        assert_eq!(cases.len(), 2);

        // Each case should be an object with type and data fields
        for case in cases {
            assert_eq!(case["type"], "object");
            assert!(case["properties"].get("type").is_some());
            assert!(case["properties"].get("data").is_some());
        }
    }

    #[test]
    fn test_descriptions() {
        #[derive(Schema)]
        /// A user account
        struct User {
            /// Unique identifier
            id: String,
            /// User's email address
            email: String,
        }

        let openapi = to_openapi_schema::<User>();
        assert_eq!(openapi["description"], "A user account");
        assert_eq!(
            openapi["properties"]["id"]["description"],
            "Unique identifier"
        );
        assert_eq!(
            openapi["properties"]["email"]["description"],
            "User's email address"
        );
    }

    #[test]
    fn test_number_types() {
        #[derive(Schema)]
        struct Metrics {
            count: u32,
            ratio: f32,
            precise: f64,
        }

        let openapi = to_openapi_schema::<Metrics>();
        assert_eq!(openapi["properties"]["count"]["type"], "integer");
        assert_eq!(openapi["properties"]["ratio"]["type"], "number");
        assert_eq!(openapi["properties"]["precise"]["type"], "number");
    }

    #[test]
    fn test_boolean() {
        #[derive(Schema)]
        struct Settings {
            enabled: bool,
            verified: bool,
        }

        let openapi = to_openapi_schema::<Settings>();
        assert_eq!(openapi["properties"]["enabled"]["type"], "boolean");
        assert_eq!(openapi["properties"]["verified"]["type"], "boolean");
    }
}
