use serde_json::Value;
use tui_tree_widget::TreeItem;

use crate::interactive::details::json_selector::JsonSelector;

pub fn tree_items_from_json(root: &Value) -> Vec<TreeItem<'_, JsonSelector>> {
    match root {
        Value::Object(object) => from_object(object),
        Value::Array(array) => from_array(array),
        _ => vec![TreeItem::new_leaf(JsonSelector::None, root.to_string())],
    }
}

fn recurse(key: JsonSelector, value: &Value) -> TreeItem<JsonSelector> {
    match value {
        Value::Object(object) => {
            let text = key.to_string();
            TreeItem::new(key, text, from_object(object)).unwrap()
        }
        Value::Array(array) => {
            let text = key.to_string();
            TreeItem::new(key, text, from_array(array)).unwrap()
        }
        _ => {
            let text = format!("{key}: {value}");
            TreeItem::new_leaf(key, text)
        }
    }
}

fn from_object(object: &serde_json::Map<String, Value>) -> Vec<TreeItem<'_, JsonSelector>> {
    object
        .iter()
        .map(|(key, value)| recurse(JsonSelector::ObjectKey(key.clone()), value))
        .collect()
}

fn from_array(array: &[Value]) -> Vec<TreeItem<'_, JsonSelector>> {
    array
        .iter()
        .enumerate()
        .map(|(index, value)| recurse(JsonSelector::ArrayIndex(index), value))
        .collect()
}
