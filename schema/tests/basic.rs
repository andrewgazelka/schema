use schema::{Schema, TypeKind};

#[derive(Schema)]
#[allow(dead_code)]
struct Person {
    name: String,
    age: i32,
    email: Option<String>,
}

#[derive(Schema)]
#[allow(dead_code)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Schema)]
#[allow(dead_code)]
enum Action {
    Click,
    Fill { value: String },
    Select { option: String },
}

#[test]
fn test_struct_schema() {
    let schema = Person::schema();

    match schema.kind {
        TypeKind::Object {
            properties,
            required,
        } => {
            assert_eq!(properties.len(), 3);
            assert!(properties.contains_key("name"));
            assert!(properties.contains_key("age"));
            assert!(properties.contains_key("email"));

            // name and age are required, email is optional
            assert_eq!(required.len(), 2);
            assert!(required.contains(&"name".to_string()));
            assert!(required.contains(&"age".to_string()));
            assert!(!required.contains(&"email".to_string()));
        }
        _ => panic!("Expected Object schema"),
    }
}

#[test]
fn test_simple_enum_schema() {
    let schema = Status::schema();

    match schema.kind {
        TypeKind::Enum { variants } => {
            assert_eq!(variants.len(), 3);
            assert!(variants.contains(&"active".to_string()));
            assert!(variants.contains(&"inactive".to_string()));
            assert!(variants.contains(&"pending".to_string()));
        }
        _ => panic!("Expected Enum schema"),
    }
}

#[test]
fn test_tagged_union_schema() {
    let schema = Action::schema();

    match schema.kind {
        TypeKind::TaggedUnion {
            tag_field,
            tag_variants,
            data_fields,
        } => {
            assert_eq!(tag_field, "type");
            assert_eq!(tag_variants.len(), 3);
            assert!(tag_variants.contains(&"click".to_string()));
            assert!(tag_variants.contains(&"fill".to_string()));
            assert!(tag_variants.contains(&"select".to_string()));

            // Should have collected all unique data fields
            assert_eq!(data_fields.len(), 2);
            assert!(data_fields.contains_key("value"));
            assert!(data_fields.contains_key("option"));
        }
        _ => panic!("Expected TaggedUnion schema"),
    }
}
