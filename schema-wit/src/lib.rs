use schema::{IntegerKind, NumberKind, Schema, SchemaType, TypeKind};

/// Convert a Schema to WIT type definition
pub fn to_wit_type<T: Schema>() -> String {
    let schema = T::schema();
    schema_type_to_wit(&schema, T::type_name())
}

/// Convert a SchemaType to WIT, optionally with a type name for records/variants/enums
fn schema_type_to_wit(schema: &SchemaType, type_name: Option<&str>) -> String {
    match &schema.kind {
        TypeKind::String => "string".to_string(),
        TypeKind::Boolean => "bool".to_string(),
        TypeKind::Null => "unit".to_string(), // WIT doesn't have null, use empty record
        TypeKind::Integer(kind) => integer_to_wit(kind),
        TypeKind::Number(kind) => number_to_wit(kind),
        TypeKind::Array { items } => {
            let item_type = schema_type_to_wit(items, None);
            format!("list<{}>", item_type)
        }
        TypeKind::Object {
            properties,
            required,
        } => record_to_wit(properties, required, type_name, schema.description.as_deref()),
        TypeKind::Enum { variants } => enum_to_wit(variants, type_name, schema.description.as_deref()),
        TypeKind::Variant { cases } => {
            variant_to_wit(cases, type_name, schema.description.as_deref())
        }
        TypeKind::Result { ok, err } => {
            let ok_type = schema_type_to_wit(ok, None);
            let err_type = schema_type_to_wit(err, None);
            format!("result<{}, {}>", ok_type, err_type)
        }
        TypeKind::Tuple { fields } => tuple_to_wit(fields),
        TypeKind::TaggedUnion { .. } => {
            // Legacy - not recommended for WIT generation
            "/* TaggedUnion not supported - use Variant instead */".to_string()
        }
        TypeKind::Ref { name } => to_kebab_case(name),
    }
}

fn integer_to_wit(kind: &IntegerKind) -> String {
    match kind {
        IntegerKind::I32 => "s32",
        IntegerKind::I64 => "s64",
        IntegerKind::U8 => "u8",
        IntegerKind::U32 => "u32",
        IntegerKind::U64 => "u64",
        IntegerKind::Usize => "u64", // usize maps to u64 for portability
    }
    .to_string()
}

fn number_to_wit(kind: &NumberKind) -> String {
    match kind {
        NumberKind::F32 => "f32",
        NumberKind::F64 => "f64",
    }
    .to_string()
}

fn record_to_wit(
    properties: &std::collections::HashMap<String, SchemaType>,
    required: &[String],
    type_name: Option<&str>,
    description: Option<&str>,
) -> String {
    let mut output = String::new();

    // Add description as comment if present
    if let Some(desc) = description {
        for line in desc.lines() {
            output.push_str(&format!("/// {}\n", line));
        }
    }

    let name = type_name.unwrap_or("anonymous-record");
    output.push_str(&format!("record {} {{\n", to_kebab_case(name)));

    // Sort fields for deterministic output
    let mut fields: Vec<_> = properties.iter().collect();
    fields.sort_by_key(|(name, _)| *name);

    for (field_name, field_schema) in fields {
        // Add field description if present
        if let Some(desc) = &field_schema.description {
            for line in desc.lines() {
                output.push_str(&format!("    /// {}\n", line));
            }
        }

        let field_type = schema_type_to_wit(field_schema, None);
        let is_optional = !required.contains(field_name);

        let final_type = if is_optional {
            format!("option<{}>", field_type)
        } else {
            field_type
        };

        output.push_str(&format!("    {}: {},\n", to_kebab_case(field_name), final_type));
    }

    output.push('}');
    output
}

fn enum_to_wit(variants: &[String], type_name: Option<&str>, description: Option<&str>) -> String {
    let mut output = String::new();

    if let Some(desc) = description {
        for line in desc.lines() {
            output.push_str(&format!("/// {}\n", line));
        }
    }

    let name = type_name.unwrap_or("anonymous-enum");
    output.push_str(&format!("enum {} {{\n", to_kebab_case(name)));

    for variant in variants {
        output.push_str(&format!("    {},\n", to_kebab_case(variant)));
    }

    output.push('}');
    output
}

fn variant_to_wit(
    cases: &[schema::VariantCase],
    type_name: Option<&str>,
    description: Option<&str>,
) -> String {
    let mut output = String::new();

    if let Some(desc) = description {
        for line in desc.lines() {
            output.push_str(&format!("/// {}\n", line));
        }
    }

    let name = type_name.unwrap_or("anonymous-variant");
    output.push_str(&format!("variant {} {{\n", to_kebab_case(name)));

    for case in cases {
        // Add case description if present
        if let Some(desc) = &case.description {
            for line in desc.lines() {
                output.push_str(&format!("    /// {}\n", line));
            }
        }

        match &case.data {
            None => {
                // Unit variant
                output.push_str(&format!("    {},\n", to_kebab_case(&case.name)));
            }
            Some(data) => {
                // Variant with data
                let data_type = schema_type_to_wit(data, None);
                output.push_str(&format!(
                    "    {}({}),\n",
                    to_kebab_case(&case.name),
                    data_type
                ));
            }
        }
    }

    output.push('}');
    output
}

fn tuple_to_wit(fields: &[SchemaType]) -> String {
    if fields.is_empty() {
        return "unit".to_string();
    }

    let field_types: Vec<String> = fields.iter().map(|f| schema_type_to_wit(f, None)).collect();
    format!("tuple<{}>", field_types.join(", "))
}

/// Convert snake_case or PascalCase to kebab-case
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            if !result.is_empty() {
                result.push('-');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else if ch == '_' {
            result.push('-');
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kebab_case() {
        assert_eq!(to_kebab_case("snake_case"), "snake-case");
        assert_eq!(to_kebab_case("PascalCase"), "pascal-case");
        assert_eq!(to_kebab_case("camelCase"), "camel-case");
        assert_eq!(to_kebab_case("already-kebab"), "already-kebab");
    }

    #[test]
    fn test_primitives() {
        assert_eq!(to_wit_type::<String>(), "string");
        assert_eq!(to_wit_type::<bool>(), "bool");
        assert_eq!(to_wit_type::<i32>(), "s32");
        assert_eq!(to_wit_type::<u32>(), "u32");
        assert_eq!(to_wit_type::<f64>(), "f64");
    }

    #[test]
    fn test_collections() {
        assert_eq!(to_wit_type::<Vec<String>>(), "list<string>");
        assert_eq!(to_wit_type::<Vec<u32>>(), "list<u32>");
        assert_eq!(to_wit_type::<Option<String>>(), "string");
    }

    #[test]
    fn test_result() {
        assert_eq!(to_wit_type::<Result<String, u32>>(), "result<string, u32>");
    }

    #[test]
    fn test_tuple() {
        assert_eq!(to_wit_type::<(String, u32)>(), "tuple<string, u32>");
        assert_eq!(to_wit_type::<(bool, f32, i64)>(), "tuple<bool, f32, s64>");
    }

    #[test]
    fn test_record() {
        #[derive(schema::Schema)]
        struct Person {
            name: String,
            age: u32,
        }

        let wit = to_wit_type::<Person>();
        assert!(wit.contains("record person {"));
        assert!(wit.contains("name: string"));
        assert!(wit.contains("age: u32"));
    }

    #[test]
    fn test_record_with_optional() {
        #[derive(schema::Schema)]
        struct User {
            id: String,
            email: Option<String>,
        }

        let wit = to_wit_type::<User>();
        assert!(wit.contains("id: string"));
        assert!(wit.contains("email: option<string>"));
    }

    #[test]
    fn test_simple_enum() {
        #[derive(schema::Schema)]
        enum Status {
            Active,
            Inactive,
            Pending,
        }

        let wit = to_wit_type::<Status>();
        assert!(wit.contains("enum status {"));
        assert!(wit.contains("active"));
        assert!(wit.contains("inactive"));
        assert!(wit.contains("pending"));
    }

    #[test]
    fn test_variant() {
        #[derive(schema::Schema)]
        enum Message {
            Text { content: String },
            Image { url: String, width: u32 },
        }

        let wit = to_wit_type::<Message>();
        assert!(wit.contains("variant message {"));
        assert!(wit.contains("text(record"));
        assert!(wit.contains("image(record"));
    }

    #[test]
    fn test_variant_unit() {
        #[derive(schema::Schema)]
        enum Event {
            Start,
            Stop,
            Pause { duration: u32 },
        }

        let wit = to_wit_type::<Event>();
        println!("{}", wit);
        assert!(wit.contains("variant event {"));
        assert!(wit.contains("start,"));
        assert!(wit.contains("stop,"));
        assert!(wit.contains("pause("));
    }

    #[test]
    fn test_nested() {
        #[derive(schema::Schema)]
        struct Address {
            street: String,
            city: String,
        }

        #[derive(schema::Schema)]
        struct Person {
            name: String,
            address: Address,
        }

        let wit = to_wit_type::<Person>();
        println!("Generated WIT:\n{}", wit);
        // Nested types inline as complete record definitions
        // Note: nested types get "anonymous-record" name since type_name isn't passed for nested objects
        assert!(wit.contains("address: record"));
        assert!(wit.contains("street: string"));
        assert!(wit.contains("city: string"));
    }
}
