use itertools::Itertools;
use std::collections::HashSet;

pub fn get_shown_topics(existing: &[String], opened: &HashSet<String>) -> Vec<String> {
    let all = build_all_tree_variants(existing);
    filter_topics_by_opened(&all, opened)
}

pub fn build_all_tree_variants(existing: &[String]) -> Vec<String> {
    let mut result = Vec::new();

    for entry in existing {
        let parts = entry.split('/').collect_vec();

        for l in 0..parts.len() {
            let topic: String = parts.iter().take(l + 1).cloned().intersperse("/").collect();
            result.push(topic)
        }
    }

    result.sort();
    result.dedup();

    result
}

pub fn filter_topics_by_opened(all: &[String], opened: &HashSet<String>) -> Vec<String> {
    let mut shown = Vec::new();

    for entry in all {
        let show = match get_parent(entry) {
            Some(parent) => opened.contains(parent),
            None => true,
        };

        if show {
            shown.push(entry.to_owned());
        }
    }

    shown
}

pub fn get_parent(topic: &str) -> Option<&str> {
    topic.rfind('/').map(|i| &topic[0..i])
}

pub fn get_leaf(topic: &str) -> &str {
    topic.rfind('/').map(|i| &topic[i + 1..]).unwrap_or(topic)
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
    let actual = build_all_tree_variants(&["foo".into()]);
    assert_eq!(actual, ["foo"]);
}

#[test]
fn tree_variants_path_gets_splitted() {
    let actual = build_all_tree_variants(&["foo/bar".into()]);
    assert_eq!(actual, ["foo", "foo/bar"]);
}

#[test]
fn tree_variants_dont_duplicate() {
    let actual = build_all_tree_variants(&["a/b".into(), "a/b/c".into(), "a/d".into()]);
    assert_eq!(actual, ["a", "a/b", "a/b/c", "a/d"]);
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
    let all = ALL_EXAMPLES
        .iter()
        .cloned()
        .map(|o| o.to_owned())
        .collect_vec();
    let opened = HashSet::new();
    let actual = filter_topics_by_opened(&all, &opened);
    assert_eq!(actual, ["a", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_some() {
    let all = ALL_EXAMPLES
        .iter()
        .cloned()
        .map(|o| o.to_owned())
        .collect_vec();
    let mut opened = HashSet::new();
    opened.insert("a".to_string());

    let actual = filter_topics_by_opened(&all, &opened);
    assert_eq!(actual, ["a", "a/b", "a/d", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_all() {
    let all = ALL_EXAMPLES
        .iter()
        .cloned()
        .map(|o| o.to_owned())
        .collect_vec();
    let mut opened = HashSet::new();
    opened.insert("a".to_string());
    opened.insert("a/b".to_string());
    opened.insert("e".to_string());
    opened.insert("e/f".to_string());
    opened.insert("e/f/g".to_string());
    opened.insert("e/f/g/h".to_string());

    let actual = filter_topics_by_opened(&all, &opened);
    assert_eq!(actual, ALL_EXAMPLES);
}
