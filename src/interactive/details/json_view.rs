use serde_json::Value as JsonValue;
use tui_tree_widget::TreeItem;

pub fn root_tree_items_from_json(root: &JsonValue) -> Vec<TreeItem<'_>> {
    match root {
        JsonValue::Object(object) => tree_items_from_json_object(object),
        JsonValue::Array(array) => tree_items_from_json_array(array),
        _ => vec![TreeItem::new_leaf(root.to_string())],
    }
}

fn tree_items_from_json<'a>(key: &str, value: &'a JsonValue) -> TreeItem<'a> {
    match value {
        JsonValue::Object(object) => {
            TreeItem::new(key.to_owned(), tree_items_from_json_object(object))
        }
        JsonValue::Array(array) => TreeItem::new(key.to_owned(), tree_items_from_json_array(array)),
        _ => TreeItem::new_leaf(format!("{key}: {value}")),
    }
}

fn tree_items_from_json_object(object: &serde_json::Map<String, JsonValue>) -> Vec<TreeItem<'_>> {
    object
        .iter()
        .map(|(key, value)| tree_items_from_json(key, value))
        .collect::<Vec<_>>()
}

fn tree_items_from_json_array(array: &[JsonValue]) -> Vec<TreeItem<'_>> {
    array
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let index = index.to_string();
            tree_items_from_json(&index, value)
        })
        .collect::<Vec<_>>()
}
