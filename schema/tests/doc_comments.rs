use schema::{Schema, TypeKind};

/// A person with contact information
#[derive(Schema)]
#[allow(dead_code)]
struct Person {
    /// Full name of the person
    name: String,
    /// Age in years
    age: i32,
    /// Optional email address
    email: Option<String>,
}

/// Current status of an entity
#[derive(Schema)]
#[allow(dead_code)]
enum Status {
    Active,
    Inactive,
    Pending,
}

/// Actions that can be performed on elements
#[derive(Schema)]
#[allow(dead_code)]
enum Action {
    Click,
    /// Fill a form field
    Fill {
        /// The value to enter
        value: String,
    },
    /// Select from a dropdown
    Select {
        /// The option to select
        option: String,
    },
}

#[test]
fn test_struct_with_docs() {
    let schema = Person::schema();

    // Check struct description
    assert_eq!(
        schema.description,
        Some("A person with contact information".to_string())
    );

    match schema.kind {
        TypeKind::Object {
            properties,
            required,
        } => {
            // Check field descriptions
            let name_schema = properties.get("name").unwrap();
            assert_eq!(
                name_schema.description,
                Some("Full name of the person".to_string())
            );
            assert!(matches!(name_schema.kind, TypeKind::String));

            let age_schema = properties.get("age").unwrap();
            assert_eq!(age_schema.description, Some("Age in years".to_string()));
            assert!(matches!(age_schema.kind, TypeKind::Integer(_)));

            let email_schema = properties.get("email").unwrap();
            assert_eq!(
                email_schema.description,
                Some("Optional email address".to_string())
            );
            assert!(matches!(email_schema.kind, TypeKind::String));

            // Check required fields
            assert_eq!(required.len(), 2);
            assert!(required.contains(&"name".to_string()));
            assert!(required.contains(&"age".to_string()));
        }
        _ => panic!("Expected Object schema"),
    }
}

#[test]
fn test_simple_enum_with_docs() {
    let schema = Status::schema();

    assert_eq!(
        schema.description,
        Some("Current status of an entity".to_string())
    );

    match schema.kind {
        TypeKind::Enum { variants } => {
            assert_eq!(variants.len(), 3);
        }
        _ => panic!("Expected Enum schema"),
    }
}

#[test]
fn test_tagged_union_with_docs() {
    let schema = Action::schema();

    // Check enum description
    assert_eq!(
        schema.description,
        Some("Actions that can be performed on elements".to_string())
    );

    match schema.kind {
        TypeKind::TaggedUnion {
            tag_field,
            tag_variants,
            data_fields,
        } => {
            assert_eq!(tag_field, "type");
            assert_eq!(tag_variants.len(), 3);

            // Check data field descriptions
            let value_schema = data_fields.get("value").unwrap();
            assert_eq!(
                value_schema.description,
                Some("The value to enter".to_string())
            );
            assert!(matches!(value_schema.kind, TypeKind::String));

            let option_schema = data_fields.get("option").unwrap();
            assert_eq!(
                option_schema.description,
                Some("The option to select".to_string())
            );
            assert!(matches!(option_schema.kind, TypeKind::String));
        }
        _ => panic!("Expected TaggedUnion schema"),
    }
}
