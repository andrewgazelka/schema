use schema::Schema;
use schema_anthropic::{create_tool_schema, to_anthropic_schema};
use serde_json::json;

#[derive(Schema)]
#[allow(dead_code)]
struct ClickElement {
    selector: String,
    index: Option<i32>,
}

#[derive(Schema)]
#[allow(dead_code)]
enum ElementAction {
    Click,
    Fill { value: String },
    Select { option: String },
    Clear,
    Focus,
    Submit,
}

#[test]
fn test_struct_to_anthropic() {
    let schema = ClickElement::schema();
    let anthropic = to_anthropic_schema(&schema);

    let expected = json!({
        "type": "object",
        "properties": {
            "selector": { "type": "string" },
            "index": { "type": "integer" }
        },
        "required": ["selector"]
    });

    assert_eq!(anthropic, expected);
}

#[test]
fn test_tagged_union_to_anthropic() {
    let schema = ElementAction::schema();
    let anthropic = to_anthropic_schema(&schema);

    // Should be a flat object with discriminator + data fields
    let obj = anthropic.as_object().unwrap();

    assert_eq!(obj.get("type").unwrap(), &json!("object"));

    let properties = obj.get("properties").unwrap().as_object().unwrap();

    // Should have "type" discriminator field
    assert!(properties.contains_key("type"));
    let type_field = properties.get("type").unwrap();
    assert_eq!(type_field.get("enum").unwrap().as_array().unwrap().len(), 6);

    // Should have all data fields as optional properties
    assert!(properties.contains_key("value"));
    assert!(properties.contains_key("option"));

    // Only "type" should be required
    let required = obj.get("required").unwrap().as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], json!("type"));
}

#[test]
fn test_create_tool_schema() {
    let schema = ClickElement::schema();
    let tool = create_tool_schema(
        "click_element",
        "Click on an element matching the selector",
        &schema,
    );

    assert_eq!(tool.get("name").unwrap(), "click_element");
    assert!(tool.get("description").is_some());
    assert!(tool.get("input_schema").is_some());
}

#[test]
fn test_no_oneof_in_output() {
    let schema = ElementAction::schema();
    let anthropic = to_anthropic_schema(&schema);
    let json_str = serde_json::to_string(&anthropic).unwrap();

    // Should NOT contain "oneOf" anywhere
    assert!(!json_str.contains("oneOf"));
    assert!(!json_str.contains("one_of"));
}
