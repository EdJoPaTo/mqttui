use std::collections::HashSet;

pub fn get_shown_topics<'a>(existing: &'a [String], opened: &HashSet<String>) -> Vec<&'a str> {
    let all = build_all_tree_variants(existing);
    filter_topics_by_opened(&all, opened)
}

pub fn build_all_tree_variants<'a>(existing: &'a [String]) -> Vec<&'a str> {
    let mut result = Vec::new();

    for entry in existing {
        for parent in get_all_parents(entry) {
            result.push(parent);
        }

        result.push(entry);
    }

    result.sort_unstable();
    result.dedup();

    result
}

pub fn filter_topics_by_opened<'a>(all: &[&'a str], opened: &HashSet<String>) -> Vec<&'a str> {
    let mut shown = Vec::new();

    for entry in all {
        let show = get_all_parents(entry)
            .iter()
            .cloned()
            .all(|t| opened.contains(t));

        if show {
            shown.push(entry.to_owned());
        }
    }

    shown
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

pub fn get_parent(topic: &str) -> Option<&str> {
    topic.rfind('/').map(|i| &topic[0..i])
}

pub fn get_leaf(topic: &str) -> &str {
    topic.rfind('/').map_or(topic, |i| &topic[i + 1..])
}

pub fn get_topic_depth(topic: &str) -> usize {
    topic.matches('/').count()
}

#[test]
fn tree_variants_empty_stays_emty() {
    let actual = build_all_tree_variants(&[]);
    assert_eq!(0, actual.len());
}

#[test]
fn tree_variants_shortest_path() {
    let topics = ["foo".to_owned()];
    let actual = build_all_tree_variants(&topics);
    assert_eq!(actual, ["foo"]);
}

#[test]
fn tree_variants_path_gets_splitted() {
    let topics = ["foo/bar".to_owned()];
    let actual = build_all_tree_variants(&topics);
    assert_eq!(actual, ["foo", "foo/bar"]);
}

#[test]
fn tree_variants_dont_duplicate() {
    let topics = ["a/b".to_owned(), "a/b/c".to_owned(), "a/d".to_owned()];
    let actual = build_all_tree_variants(&topics);
    assert_eq!(actual, ["a", "a/b", "a/b/c", "a/d"]);
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
fn topic_depth_works() {
    assert_eq!(0, get_topic_depth("a"));
    assert_eq!(1, get_topic_depth("a/b"));
    assert_eq!(2, get_topic_depth("a/b/c"));
    assert_eq!(3, get_topic_depth("a/b/c/d"));
}

#[cfg(test)]
const ALL_EXAMPLES: [&str; 10] = [
    "a",
    "a/b",
    "a/b/c",
    "a/d",
    "e",
    "e/f",
    "e/f/g",
    "e/f/g/h",
    "e/f/g/h/i",
    "e/j",
];

#[test]
fn filter_topics_by_opened_shows_only_top_level() {
    let opened = HashSet::new();
    let actual = filter_topics_by_opened(&ALL_EXAMPLES, &opened);
    assert_eq!(actual, ["a", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_some() {
    let mut opened = HashSet::new();
    opened.insert("a".to_string());

    let actual = filter_topics_by_opened(&ALL_EXAMPLES, &opened);
    assert_eq!(actual, ["a", "a/b", "a/d", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_only_when_all_parents_are_opened() {
    let mut opened = HashSet::new();
    opened.insert("a/b".to_string());

    let actual = filter_topics_by_opened(&ALL_EXAMPLES, &opened);
    assert_eq!(actual, ["a", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_all() {
    let mut opened = HashSet::new();
    opened.insert("a".to_string());
    opened.insert("a/b".to_string());
    opened.insert("e".to_string());
    opened.insert("e/f".to_string());
    opened.insert("e/f/g".to_string());
    opened.insert("e/f/g/h".to_string());

    let actual = filter_topics_by_opened(&ALL_EXAMPLES, &opened);
    assert_eq!(actual, ALL_EXAMPLES);
}
