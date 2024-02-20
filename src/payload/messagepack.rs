use rmpv::Value;

/// Attempts to decode [`MessagePack`](rmpv::Value) from the payload.
/// Tries to find out if data seems valid.
pub fn decode(mut payload: &[u8]) -> Option<Value> {
    let value = rmpv::decode::read_value(&mut payload).ok()?;
    if value.to_string().len() < payload.len() {
        // The JSON should be bigger than the bytes.
        // Otherwise there is data missing from the bytes so its unlikely MessagePack.
        return None;
    }
    if has_duplicate_keys(&value) {
        // Can not be displayed by the Tree widget currently anyway
        return None;
    }
    Some(value)
}

#[test]
fn decode_empty() {
    assert_eq!(decode(&[0, 0, 0, 0, 0]), None);
}

#[test]
fn decode_true() {
    assert_eq!(decode(&[0xC3]), Some(Value::Boolean(true)));
}

fn has_duplicate_keys(value: &Value) -> bool {
    match value {
        Value::Nil
        | Value::Boolean(_)
        | Value::Integer(_)
        | Value::F32(_)
        | Value::F64(_)
        | Value::String(_)
        | Value::Binary(_)
        | Value::Ext(_, _) => false,
        Value::Array(values) => values.iter().any(has_duplicate_keys),
        Value::Map(map) => {
            let mut keys = map
                .iter()
                .map(|(key, _value)| {
                    key.as_str()
                        .map_or_else(|| key.to_string(), ToOwned::to_owned)
                })
                .collect::<Vec<_>>();
            let before = keys.len();
            keys.sort_unstable();
            keys.dedup();
            if before > keys.len() {
                return true;
            }
            map.iter()
                .map(|(_key, value)| value)
                .any(has_duplicate_keys)
        }
    }
}

#[test]
fn duplicates_simple_array() {
    let value = Value::Array(vec![
        Value::F32(42.0),
        Value::Boolean(true),
        Value::String("hello world".into()),
        Value::Binary(vec![1, 3, 3, 7]),
    ]);
    assert!(!has_duplicate_keys(&value));
}

#[test]
fn duplicates_simple_map() {
    let value = Value::Map(vec![
        (Value::String("foo".into()), Value::Boolean(false)),
        (Value::String("bar".into()), Value::Boolean(true)),
    ]);
    assert!(!has_duplicate_keys(&value));
}

#[test]
fn duplicates_map_with_sameish_float() {
    let value = Value::Map(vec![
        (Value::F32(42.0), Value::Boolean(false)),
        (Value::F64(42.0), Value::Boolean(true)),
    ]);
    assert!(has_duplicate_keys(&value));
}

#[test]
fn duplicates_deep_map_works() {
    let value = Value::Array(vec![
        Value::Boolean(true),
        Value::Map(vec![
            (Value::F32(42.0), Value::Boolean(false)),
            (Value::F64(42.0), Value::Boolean(true)),
        ]),
    ]);
    assert!(has_duplicate_keys(&value));
}
