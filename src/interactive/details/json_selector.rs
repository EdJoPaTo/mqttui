use serde_json::Value as JsonValue;

fn get_nth(root: &JsonValue, select: usize) -> Option<&JsonValue> {
    match root {
        JsonValue::Object(object) => object.iter().nth(select).map(|(_key, value)| value),
        JsonValue::Array(array) => array.get(select),
        _ => None,
    }
}

pub fn select<'a>(root: &'a JsonValue, selector: &[usize]) -> Option<&'a JsonValue> {
    let mut current = root;
    for select in selector {
        current = get_nth(current, *select)?;
    }
    Some(current)
}

#[test]
fn can_not_get_other_value() {
    let root = JsonValue::Bool(false);
    let result = get_nth(&root, 2);
    assert_eq!(result, None);
}

#[test]
fn can_get_nth_array_value() {
    let root = JsonValue::Array(vec![
        JsonValue::String("bla".to_string()),
        JsonValue::Bool(true),
    ]);
    let result = get_nth(&root, 1);
    assert_eq!(result, Some(&JsonValue::Bool(true)));
}

#[test]
fn can_not_get_array_index_out_of_range() {
    let root = JsonValue::Array(vec![
        JsonValue::String("bla".to_string()),
        JsonValue::Bool(true),
    ]);
    let result = get_nth(&root, 42);
    assert_eq!(result, None);
}

#[test]
fn can_get_object_value() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_string(), JsonValue::Bool(false));
    object.insert("blubb".to_string(), JsonValue::Bool(true));
    let root = JsonValue::Object(object);
    let result = get_nth(&root, 1);
    assert_eq!(result, Some(&JsonValue::Bool(true)));
}

#[test]
fn can_not_get_object_missing_key() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_string(), JsonValue::Bool(false));
    object.insert("blubb".to_string(), JsonValue::Bool(true));
    let root = JsonValue::Object(object);
    let result = get_nth(&root, 42);
    assert_eq!(result, None);
}

#[test]
fn can_get_selected_value() {
    let mut inner = serde_json::Map::new();
    inner.insert("bla".to_string(), JsonValue::Bool(false));
    inner.insert("blubb".to_string(), JsonValue::Bool(true));

    let root = JsonValue::Array(vec![
        JsonValue::Bool(false),
        JsonValue::Object(inner),
        JsonValue::Bool(false),
    ]);

    let result = select(&root, &[1, 1]);
    assert_eq!(result, Some(&JsonValue::Bool(true)));
}
