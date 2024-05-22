use rmpv::Value;
use tui_tree_widget::Selector;

fn apply_messagepack<'v>(selector: &Selector, root: &'v Value) -> Option<&'v Value> {
    use rmpv::Value;
    match (root, selector) {
        (Value::Array(array), Selector::ArrayIndex(index)) => array.get(*index),
        (Value::Map(object), Selector::ObjectKey(selectkey)) => object
            .iter()
            .find(|(mapkey, _value)| {
                // similar to messagepack::map_key
                mapkey.as_str().map_or_else(
                    || &mapkey.to_string() == selectkey,
                    |mapkey| mapkey == selectkey,
                )
            })
            .map(|(_key, value)| value),
        _ => None,
    }
}

pub fn select<'v>(root: &'v Value, selector: &[Selector]) -> Option<&'v Value> {
    let mut current = root;
    for select in selector {
        current = apply_messagepack(select, current)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_not_get_other_value() {
        let root = Value::Boolean(false);
        let result = apply_messagepack(&Selector::ArrayIndex(2), &root);
        assert_eq!(result, None);
    }

    #[test]
    fn can_get_nth_array_value() {
        let root = Value::Array(vec![Value::String("bla".into()), Value::Boolean(true)]);
        let result = apply_messagepack(&Selector::ArrayIndex(1), &root);
        assert_eq!(result, Some(&Value::Boolean(true)));
    }

    #[test]
    fn can_get_object_value() {
        let root = Value::Map(vec![
            (Value::String("bla".into()), Value::Boolean(false)),
            (Value::String("blubb".into()), Value::Boolean(true)),
            (Value::Integer(12.into()), Value::Boolean(true)),
        ]);
        let result = apply_messagepack(&Selector::ObjectKey("blubb".to_owned()), &root);
        assert_eq!(result, Some(&Value::Boolean(true)));
    }

    #[test]
    fn can_get_selected_value() {
        let inner = vec![
            (Value::String("bla".into()), Value::Boolean(false)),
            (Value::String("blubb".into()), Value::Boolean(true)),
        ];
        let root = Value::Array(vec![
            Value::Boolean(false),
            Value::Map(inner),
            Value::Boolean(false),
        ]);

        let selector = vec![
            Selector::ArrayIndex(1),
            Selector::ObjectKey("blubb".to_owned()),
        ];

        let result = select(&root, &selector);
        assert_eq!(result, Some(&Value::Boolean(true)));
    }
}
