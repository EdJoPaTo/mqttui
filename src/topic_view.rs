use crate::mqtt_history::TopicMessagesLastPayload;
use crate::topic;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct TopicTreeEntry<'a> {
    pub topic: &'a str,
    pub topics_below: usize,
    pub messages_below: usize,
    pub last_payload: Option<&'a [u8]>,
}

pub fn get_tree_with_metadata<'a>(
    entries: &'a [TopicMessagesLastPayload],
) -> Vec<TopicTreeEntry<'a>> {
    let mut result: HashMap<&'a str, TopicTreeEntry<'a>> = HashMap::new();

    for tmlp in entries {
        for parent in topic::get_all_parents(&tmlp.topic) {
            if let Some(entry) = result.get_mut(parent) {
                entry.messages_below += tmlp.messages;
                entry.topics_below += 1;
            } else {
                result.insert(
                    parent,
                    TopicTreeEntry {
                        topic: parent,
                        messages_below: tmlp.messages,
                        topics_below: 1,
                        last_payload: None,
                    },
                );
            }
        }

        if let Some(entry) = result.get_mut(&tmlp.topic[0..]) {
            entry.messages_below += tmlp.messages;
            entry.last_payload = Some(&tmlp.last_payload);
        } else {
            result.insert(
                &tmlp.topic,
                TopicTreeEntry {
                    topic: &tmlp.topic,
                    messages_below: tmlp.messages,
                    topics_below: 0,
                    last_payload: Some(&tmlp.last_payload),
                },
            );
        }
    }

    let mut vec: Vec<_> = result.values().cloned().collect();
    vec.sort_by_key(|o| o.topic);
    vec
}

pub fn is_topic_opened(opened: &HashSet<String>, topic: &str) -> bool {
    topic::get_all_parents(topic)
        .iter()
        .cloned()
        .all(|t| opened.contains(t))
}

#[test]
fn tree_with_metadata_works() {
    let mut entries: Vec<TopicMessagesLastPayload> = Vec::new();
    entries.push(TopicMessagesLastPayload {
        topic: "a/b".to_string(),
        messages: 4,
        last_payload: b"bla".to_vec(),
    });
    entries.push(TopicMessagesLastPayload {
        topic: "a/c".to_string(),
        messages: 6,
        last_payload: b"blubb".to_vec(),
    });
    entries.push(TopicMessagesLastPayload {
        topic: "d".to_string(),
        messages: 5,
        last_payload: b"fish".to_vec(),
    });

    assert_eq!(
        get_tree_with_metadata(&entries),
        [
            TopicTreeEntry {
                topic: "a",
                topics_below: 2,
                messages_below: 10,
                last_payload: None,
            },
            TopicTreeEntry {
                topic: "a/b",
                topics_below: 0,
                messages_below: 4,
                last_payload: Some(b"bla"),
            },
            TopicTreeEntry {
                topic: "a/c",
                topics_below: 0,
                messages_below: 6,
                last_payload: Some(b"blubb"),
            },
            TopicTreeEntry {
                topic: "d",
                topics_below: 0,
                messages_below: 5,
                last_payload: Some(b"fish"),
            },
        ]
    );
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
    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
    assert_eq!(actual, ["a", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_some() {
    let mut opened = HashSet::new();
    opened.insert("a".to_string());
    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
    assert_eq!(actual, ["a", "a/b", "a/d", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_only_when_all_parents_are_opened() {
    let mut opened = HashSet::new();
    opened.insert("a/b".to_string());
    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
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

    let actual: Vec<_> = ALL_EXAMPLES
        .iter()
        .cloned()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect();
    assert_eq!(actual, ALL_EXAMPLES);
}
