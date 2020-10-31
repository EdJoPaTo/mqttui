pub fn get_all_with_parents<'a, I>(all: I) -> Vec<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    // HashSet seems more useful as duplicates arnt wanted but it looses the order
    let mut result = Vec::new();
    for topic in all {
        for parent in get_all_parents(topic) {
            if !result.contains(&parent) {
                result.push(parent);
            }
        }

        if !result.contains(&topic) {
            result.push(topic);
        }
    }

    result
}

pub fn get_all_roots<'a, I>(all: I) -> Vec<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut result = all.into_iter().map(|o| get_root(o)).collect::<Vec<_>>();
    result.sort_unstable();
    result.dedup();
    result
}

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

pub fn get_root(topic: &str) -> &str {
    topic.find('/').map_or(topic, |i| &topic[0..i])
}

pub fn get_parent(topic: &str) -> Option<&str> {
    topic.rfind('/').map(|i| &topic[0..i])
}

pub fn get_direct_children<'a>(topic: &str, all: &[&'a str]) -> Vec<&'a str> {
    all.iter()
        .filter(|o| is_direct_child(topic, o))
        .cloned()
        .collect()
}

pub fn get_parts(topic: &str) -> Vec<&str> {
    topic.split('/').collect::<Vec<_>>()
}

pub fn get_leaf(topic: &str) -> &str {
    topic.rfind('/').map_or(topic, |i| &topic[i + 1..])
}

pub fn get_depth(topic: &str) -> usize {
    topic.matches('/').count()
}

pub fn is_below(parent: &str, child: &str) -> bool {
    if let Some(first_char_after) = child.chars().nth(parent.len()) {
        if first_char_after != '/' {
            return false;
        }
    }

    child.starts_with(parent)
}

pub fn is_direct_child(parent: &str, child: &str) -> bool {
    if get_depth(parent) + 1 != get_depth(child) {
        return false;
    }

    is_below(parent, child)
}

#[test]
fn all_with_parents_works() {
    let actual = get_all_with_parents(vec!["a/b", "a/b/c", "d/e"]);
    assert_eq!(actual, ["a", "a/b", "a/b/c", "d", "d/e"]);
}

#[test]
fn all_roots_works() {
    let actual = get_all_roots(vec!["a/b", "a/b/c", "d/e"]);
    assert_eq!(actual, ["a", "d"]);
}

#[test]
fn all_parents_works() {
    assert_eq!(get_all_parents("a").len(), 0);
    assert_eq!(get_all_parents("a/b"), ["a"]);
    assert_eq!(get_all_parents("a/b/c"), ["a", "a/b"]);
    assert_eq!(get_all_parents("a/b/c/d"), ["a", "a/b", "a/b/c"]);
}

#[test]
fn root_works() {
    assert_eq!("a", get_root("a"));
    assert_eq!("a", get_root("a/b"));
    assert_eq!("a", get_root("a/b/c"));
    assert_eq!("a", get_root("a/b/c/d"));
}

#[test]
fn parent_works() {
    assert_eq!(None, get_parent("a"));
    assert_eq!(Some("a"), get_parent("a/b"));
    assert_eq!(Some("a/b"), get_parent("a/b/c"));
    assert_eq!(Some("a/b/c"), get_parent("a/b/c/d"));
}

#[test]
fn direct_children_works() {
    let actual = get_direct_children("a", &["a/b", "a/b/c", "a/d", "e"]);
    assert_eq!(actual, ["a/b", "a/d"])
}

#[test]
fn parts_works() {
    assert_eq!(get_parts("a"), ["a"]);
    assert_eq!(get_parts("a/b"), ["a", "b"]);
    assert_eq!(get_parts("a/b/c"), ["a", "b", "c"]);
    assert_eq!(get_parts("a/b/c/d"), ["a", "b", "c", "d"]);
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

#[test]
fn is_below_works() {
    assert_eq!(true, is_below("a", "a/b"));
    assert_eq!(true, is_below("a", "a/b/c"));
    assert_eq!(false, is_below("a", "b"));
    assert_eq!(false, is_below("a", "b/c"));
    assert_eq!(false, is_below("a", "ab/c"));
}

#[test]
fn is_direct_child_works() {
    assert_eq!(true, is_direct_child("a", "a/b"));
    assert_eq!(false, is_direct_child("a", "a/b/c"));
    assert_eq!(false, is_direct_child("a", "b/c"));
}
