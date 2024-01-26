use serde_json::Value as JsonValue;
use tui_tree_widget::TreeItem;

use crate::interactive::details::json_selector::JsonSelector;

pub fn root_tree_items_from_json(root: &JsonValue) -> Vec<TreeItem<'_, JsonSelector>> {
    match root {
        JsonValue::Object(object) => tree_items_from_json_object(object),
        JsonValue::Array(array) => tree_items_from_json_array(array),
        _ => vec![TreeItem::new_leaf(JsonSelector::None, root.to_string())],
    }
}

fn tree_items_from_json(key: JsonSelector, value: &JsonValue) -> TreeItem<JsonSelector> {
    match value {
        JsonValue::Object(object) => {
            let text = key.to_string();
            TreeItem::new(key, text, tree_items_from_json_object(object)).unwrap()
        }
        JsonValue::Array(array) => {
            let text = key.to_string();
            TreeItem::new(key, text, tree_items_from_json_array(array)).unwrap()
        }
        _ => {
            let text = format!("{}: {value}", key.to_string());
            TreeItem::new_leaf(key, text)
        }
    }
}

fn tree_items_from_json_object(
    object: &serde_json::Map<String, JsonValue>,
) -> Vec<TreeItem<'_, JsonSelector>> {
    object
        .iter()
        .map(|(key, value)| tree_items_from_json(JsonSelector::ObjectKey(key.clone()), value))
        .collect()
}

fn tree_items_from_json_array(array: &[JsonValue]) -> Vec<TreeItem<'_, JsonSelector>> {
    array
        .iter()
        .enumerate()
        .map(|(index, value)| tree_items_from_json(JsonSelector::ArrayIndex(index), value))
        .collect()
}
