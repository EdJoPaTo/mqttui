use serde_json::Value as JsonValue;

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub enum JsonSelector {
    ObjectKey(String),
    ArrayIndex(usize),
    #[default]
    None,
}

impl JsonSelector {
    fn apply<'v>(&self, root: &'v JsonValue) -> Option<&'v JsonValue> {
        match (root, self) {
            (JsonValue::Object(object), Self::ObjectKey(key)) => object.get(key),
            (JsonValue::Array(array), Self::ArrayIndex(index)) => array.get(*index),
            _ => None,
        }
    }

    pub fn get_selection<'a>(root: &'a JsonValue, selector: &[Self]) -> Option<&'a JsonValue> {
        let mut current = root;
        for select in selector {
            current = select.apply(current)?;
        }
        Some(current)
    }
}

impl ToString for JsonSelector {
    fn to_string(&self) -> String {
        match self {
            Self::ObjectKey(key) => key.to_string(),
            Self::ArrayIndex(index) => index.to_string(),
            Self::None => String::new(),
        }
    }
}

#[test]
fn can_not_get_other_value() {
    let root = JsonValue::Bool(false);
    let result = JsonSelector::ArrayIndex(2).apply(&root);
    assert_eq!(result, None);
}

#[test]
fn can_get_nth_array_value() {
    let root = JsonValue::Array(vec![
        JsonValue::String("bla".to_string()),
        JsonValue::Bool(true),
    ]);
    let result = JsonSelector::ArrayIndex(1).apply(&root);
    assert_eq!(result, Some(&JsonValue::Bool(true)));
}

#[test]
fn can_not_get_array_index_out_of_range() {
    let root = JsonValue::Array(vec![
        JsonValue::String("bla".to_string()),
        JsonValue::Bool(true),
    ]);
    let result = JsonSelector::ArrayIndex(42).apply(&root);
    assert_eq!(result, None);
}

#[test]
fn can_get_object_value() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_string(), JsonValue::Bool(false));
    object.insert("blubb".to_string(), JsonValue::Bool(true));
    let root = JsonValue::Object(object);
    let result = JsonSelector::ObjectKey("blubb".to_string()).apply(&root);
    assert_eq!(result, Some(&JsonValue::Bool(true)));
}

#[test]
fn can_not_get_object_missing_key() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_string(), JsonValue::Bool(false));
    object.insert("blubb".to_string(), JsonValue::Bool(true));
    let root = JsonValue::Object(object);
    let result = JsonSelector::ObjectKey("foo".to_string()).apply(&root);
    assert_eq!(result, None);
}

#[test]
fn can_not_get_object_by_index() {
    let mut object = serde_json::Map::new();
    object.insert("bla".to_string(), JsonValue::Bool(false));
    object.insert("blubb".to_string(), JsonValue::Bool(true));
    let root = JsonValue::Object(object);
    let result = JsonSelector::ArrayIndex(42).apply(&root);
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

    let selector = vec![
        JsonSelector::ArrayIndex(1),
        JsonSelector::ObjectKey("blubb".to_string()),
    ];

    let result = JsonSelector::get_selection(&root, &selector);
    assert_eq!(result, Some(&JsonValue::Bool(true)));
}
