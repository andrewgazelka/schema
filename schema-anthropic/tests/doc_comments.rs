use schema::Schema;
use schema_anthropic::to_anthropic_schema;
use serde_json::Value;

/// Click on an element
#[derive(Schema)]
#[allow(dead_code)]
struct ClickElement {
    /// CSS selector to find the element
    selector: String,
    /// Zero-based index if multiple matches
    index: Option<i32>,
}

/// Actions that can be performed
#[derive(Schema)]
#[allow(dead_code)]
enum ElementAction {
    Click,
    /// Fill a form field with text
    Fill {
        /// The text to enter
        value: String,
    },
    /// Select an option from dropdown
    Select {
        /// The option label
        option: String,
    },
}

#[test]
fn test_struct_descriptions_in_anthropic() {
    let schema = ClickElement::schema();
    let anthropic = to_anthropic_schema(&schema);

    let obj = anthropic.as_object().unwrap();

    // Check struct description
    assert_eq!(
        obj.get("description").unwrap().as_str().unwrap(),
        "Click on an element"
    );

    // Check field descriptions
    let properties = obj.get("properties").unwrap().as_object().unwrap();

    let selector = properties.get("selector").unwrap().as_object().unwrap();
    assert_eq!(
        selector.get("description").unwrap().as_str().unwrap(),
        "CSS selector to find the element"
    );

    let index = properties.get("index").unwrap().as_object().unwrap();
    assert_eq!(
        index.get("description").unwrap().as_str().unwrap(),
        "Zero-based index if multiple matches"
    );
}

#[test]
fn test_tagged_union_descriptions_in_anthropic() {
    let schema = ElementAction::schema();
    let anthropic = to_anthropic_schema(&schema);

    let obj = anthropic.as_object().unwrap();

    // Check union description
    assert_eq!(
        obj.get("description").unwrap().as_str().unwrap(),
        "Actions that can be performed"
    );

    // Check data field descriptions
    let properties = obj.get("properties").unwrap().as_object().unwrap();

    let value = properties.get("value").unwrap().as_object().unwrap();
    assert_eq!(
        value.get("description").unwrap().as_str().unwrap(),
        "The text to enter"
    );

    let option = properties.get("option").unwrap().as_object().unwrap();
    assert_eq!(
        option.get("description").unwrap().as_str().unwrap(),
        "The option label"
    );
}

#[test]
fn test_descriptions_format_for_anthropic() {
    let schema = ClickElement::schema();
    let anthropic = to_anthropic_schema(&schema);
    let json_str = serde_json::to_string_pretty(&anthropic).unwrap();

    // Verify it follows Anthropic's expected format
    assert!(json_str.contains("\"description\""));
    assert!(json_str.contains("Click on an element"));

    // Parse back to verify structure
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.get("type").is_some());
    assert!(parsed.get("description").is_some());
    assert!(parsed.get("properties").is_some());
}
