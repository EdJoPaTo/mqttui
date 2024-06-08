use rmpv::Value;

/// Attempts to decode [`MessagePack`](rmpv::Value) from the payload.
/// Tries to find out if data seems valid.
pub(super) fn decode(mut payload: &[u8]) -> Option<Value> {
    let value = rmpv::decode::read_value(&mut payload).ok()?;
    if value.to_string().len() < payload.len() {
        // The JSON should be bigger than the bytes.
        // Otherwise there is data missing from the bytes so its unlikely MessagePack.
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
