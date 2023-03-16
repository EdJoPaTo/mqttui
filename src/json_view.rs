use json::JsonValue;
use tui_tree_widget::TreeItem;

fn get_nth_subvalue(root: &JsonValue, select: usize) -> Option<&JsonValue> {
    match root {
        JsonValue::Object(object) => object.iter().nth(select).map(|(_key, value)| value),
        JsonValue::Array(array) => array.get(select),
        _ => None,
    }
}

pub fn get_selected_subvalue<'a>(
    root: &'a JsonValue,
    selection: &[usize],
) -> Option<&'a JsonValue> {
    let mut current = root;
    for select in selection {
        current = get_nth_subvalue(current, *select)?;
    }

    Some(current)
}

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

fn tree_items_from_json_object(object: &json::object::Object) -> Vec<TreeItem<'_>> {
    object
        .iter()
        .map(|(key, value)| tree_items_from_json(key, value))
        .collect::<Vec<_>>()
}

fn tree_items_from_json_array<'a, I>(array: I) -> Vec<TreeItem<'a>>
where
    I: IntoIterator<Item = &'a JsonValue>,
{
    array
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let index = index.to_string();
            tree_items_from_json(&index, value)
        })
        .collect::<Vec<_>>()
}

#[test]
fn can_not_get_nth_other_value() {
    let root = JsonValue::Boolean(false);
    let result = get_selected_subvalue(&root, &[2]);
    assert_eq!(result, None);
}

#[test]
fn can_get_nth_array_value() {
    let root = JsonValue::Array(vec![
        JsonValue::String("bla".to_string()),
        JsonValue::Boolean(true),
    ]);
    let result = get_nth_subvalue(&root, 1);
    assert_eq!(result, Some(&JsonValue::Boolean(true)));
}

#[test]
fn can_not_get_nth_array_value_out_of_range() {
    let root = JsonValue::Array(vec![
        JsonValue::String("bla".to_string()),
        JsonValue::Boolean(true),
    ]);
    let result = get_nth_subvalue(&root, 42);
    assert_eq!(result, None);
}

#[test]
fn can_get_nth_object_value() {
    let mut object = json::object::Object::new();
    object.insert("bla", JsonValue::Boolean(false));
    object.insert("blubb", JsonValue::Boolean(true));

    let root = JsonValue::Object(object);
    let result = get_nth_subvalue(&root, 1);
    assert_eq!(result, Some(&JsonValue::Boolean(true)));
}

#[test]
fn can_not_get_nth_object_value_out_of_range() {
    let mut object = json::object::Object::new();
    object.insert("bla", JsonValue::Boolean(false));
    object.insert("blubb", JsonValue::Boolean(true));

    let root = JsonValue::Object(object);
    let result = get_nth_subvalue(&root, 42);
    assert_eq!(result, None);
}

#[test]
fn can_get_selected_value() {
    let mut inner = json::object::Object::new();
    inner.insert("bla", JsonValue::Boolean(false));
    inner.insert("blubb", JsonValue::Boolean(true));

    let root = JsonValue::Array(vec![
        JsonValue::Boolean(false),
        JsonValue::Object(inner),
        JsonValue::Boolean(false),
    ]);

    let result = get_selected_subvalue(&root, &[1, 1]);
    assert_eq!(result, Some(&JsonValue::Boolean(true)));
}
