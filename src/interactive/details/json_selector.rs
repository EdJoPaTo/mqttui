#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub enum JsonSelector {
    ObjectKey(String),
    ArrayIndex(usize),
    #[default]
    None,
}

impl JsonSelector {
    fn apply_json<'v>(&self, root: &'v serde_json::Value) -> Option<&'v serde_json::Value> {
        use serde_json::Value;
        match (root, self) {
            (Value::Object(object), Self::ObjectKey(key)) => object.get(key),
            (Value::Array(array), Self::ArrayIndex(index)) => array.get(*index),
            _ => None,
        }
    }

    pub fn get_json<'v>(
        root: &'v serde_json::Value,
        selector: &[Self],
    ) -> Option<&'v serde_json::Value> {
        let mut current = root;
        for select in selector {
            current = select.apply_json(current)?;
        }
        Some(current)
    }

    fn apply_messagepack<'v>(&self, root: &'v rmpv::Value) -> Option<&'v rmpv::Value> {
        use rmpv::Value;
        match (root, self) {
            (Value::Array(array), Self::ArrayIndex(index)) => array.get(*index),
            (Value::Map(object), Self::ObjectKey(selectkey)) => object
                .iter()
                .find(|(mapkey, _value)| {
                    mapkey.as_str().map_or_else(
                        || &mapkey.to_string() == selectkey,
                        |mapkey| mapkey == selectkey,
                    )
                })
                .map(|(_key, value)| value),
            _ => None,
        }
    }

    pub fn get_messagepack<'v>(
        root: &'v rmpv::Value,
        selector: &[Self],
    ) -> Option<&'v rmpv::Value> {
        let mut current = root;
        for select in selector {
            current = select.apply_messagepack(current)?;
        }
        Some(current)
    }
}

impl std::fmt::Display for JsonSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ObjectKey(key) => f.write_str(key),
            Self::ArrayIndex(index) => f.write_str(&index.to_string()),
            Self::None => Ok(()),
        }
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn can_not_get_other_value() {
        let root = Value::Bool(false);
        let result = JsonSelector::ArrayIndex(2).apply_json(&root);
        assert_eq!(result, None);
    }

    #[test]
    fn can_get_nth_array_value() {
        let root = Value::Array(vec![Value::String("bla".to_owned()), Value::Bool(true)]);
        let result = JsonSelector::ArrayIndex(1).apply_json(&root);
        assert_eq!(result, Some(&Value::Bool(true)));
    }

    #[test]
    fn can_not_get_array_index_out_of_range() {
        let root = Value::Array(vec![Value::String("bla".to_owned()), Value::Bool(true)]);
        let result = JsonSelector::ArrayIndex(42).apply_json(&root);
        assert_eq!(result, None);
    }

    #[test]
    fn can_get_object_value() {
        let mut object = serde_json::Map::new();
        object.insert("bla".to_owned(), Value::Bool(false));
        object.insert("blubb".to_owned(), Value::Bool(true));
        let root = Value::Object(object);
        let result = JsonSelector::ObjectKey("blubb".to_owned()).apply_json(&root);
        assert_eq!(result, Some(&Value::Bool(true)));
    }

    #[test]
    fn can_not_get_object_missing_key() {
        let mut object = serde_json::Map::new();
        object.insert("bla".to_owned(), Value::Bool(false));
        object.insert("blubb".to_owned(), Value::Bool(true));
        let root = Value::Object(object);
        let result = JsonSelector::ObjectKey("foo".to_owned()).apply_json(&root);
        assert_eq!(result, None);
    }

    #[test]
    fn can_not_get_object_by_index() {
        let mut object = serde_json::Map::new();
        object.insert("bla".to_owned(), Value::Bool(false));
        object.insert("blubb".to_owned(), Value::Bool(true));
        let root = Value::Object(object);
        let result = JsonSelector::ArrayIndex(42).apply_json(&root);
        assert_eq!(result, None);
    }

    #[test]
    fn can_get_selected_value() {
        let mut inner = serde_json::Map::new();
        inner.insert("bla".to_owned(), Value::Bool(false));
        inner.insert("blubb".to_owned(), Value::Bool(true));

        let root = Value::Array(vec![
            Value::Bool(false),
            Value::Object(inner),
            Value::Bool(false),
        ]);

        let selector = vec![
            JsonSelector::ArrayIndex(1),
            JsonSelector::ObjectKey("blubb".to_owned()),
        ];

        let result = JsonSelector::get_json(&root, &selector);
        assert_eq!(result, Some(&Value::Bool(true)));
    }
}

#[cfg(test)]
mod messagepack_tests {
    use super::*;
    use rmpv::Value;

    #[test]
    fn can_not_get_other_value() {
        let root = Value::Boolean(false);
        let result = JsonSelector::ArrayIndex(2).apply_messagepack(&root);
        assert_eq!(result, None);
    }

    #[test]
    fn can_get_nth_array_value() {
        let root = Value::Array(vec![Value::String("bla".into()), Value::Boolean(true)]);
        let result = JsonSelector::ArrayIndex(1).apply_messagepack(&root);
        assert_eq!(result, Some(&Value::Boolean(true)));
    }

    #[test]
    fn can_get_object_value() {
        let root = Value::Map(vec![
            (Value::String("bla".into()), Value::Boolean(false)),
            (Value::String("blubb".into()), Value::Boolean(true)),
            (Value::Integer(12.into()), Value::Boolean(true)),
        ]);
        let result = JsonSelector::ObjectKey("blubb".to_owned()).apply_messagepack(&root);
        assert_eq!(result, Some(&Value::Boolean(true)));
    }
}
