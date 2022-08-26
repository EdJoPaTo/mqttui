pub fn get_parent(topic: &str) -> Option<&str> {
    topic.rfind('/').map(|i| &topic[0..i])
}

#[test]
fn parent_works() {
    assert_eq!(None, get_parent("a"));
    assert_eq!(Some("a"), get_parent("a/b"));
    assert_eq!(Some("a/b"), get_parent("a/b/c"));
    assert_eq!(Some("a/b/c"), get_parent("a/b/c/d"));
}
