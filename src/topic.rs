pub fn get_all_parents(topic: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut current = topic;

    while let Some(parent) = get_parent(current) {
        result.push(parent);
        current = parent;
    }

    result.reverse();
    result
}

pub fn get_parent(topic: &str) -> Option<&str> {
    topic.rfind('/').map(|i| &topic[0..i])
}

pub fn get_leaf(topic: &str) -> &str {
    topic.rfind('/').map_or(topic, |i| &topic[i + 1..])
}

pub fn get_depth(topic: &str) -> usize {
    topic.matches('/').count()
}

#[test]
fn all_parents_works() {
    assert_eq!(get_all_parents("a").len(), 0);
    assert_eq!(get_all_parents("a/b"), ["a"]);
    assert_eq!(get_all_parents("a/b/c"), ["a", "a/b"]);
    assert_eq!(get_all_parents("a/b/c/d"), ["a", "a/b", "a/b/c"]);
}

#[test]
fn parent_works() {
    assert_eq!(None, get_parent("a"));
    assert_eq!(Some("a"), get_parent("a/b"));
    assert_eq!(Some("a/b"), get_parent("a/b/c"));
    assert_eq!(Some("a/b/c"), get_parent("a/b/c/d"));
}

#[test]
fn leaf_works() {
    assert_eq!("a", get_leaf("a"));
    assert_eq!("b", get_leaf("a/b"));
    assert_eq!("c", get_leaf("a/b/c"));
    assert_eq!("d", get_leaf("a/b/c/d"));
}

#[test]
fn depth_works() {
    assert_eq!(0, get_depth("a"));
    assert_eq!(1, get_depth("a/b"));
    assert_eq!(2, get_depth("a/b/c"));
    assert_eq!(3, get_depth("a/b/c/d"));
}
