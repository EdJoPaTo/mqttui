use crate::mqtt_history::TopicMessagesLastPayload;
use crate::{format, topic};
use std::collections::{HashMap, HashSet};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui_tree_widget::identifier::TreeIdentifierVec;
use tui_tree_widget::TreeItem;

#[derive(Debug)]
pub struct TopicTreeEntry {
    pub topic: String,
    pub messages: usize,
    pub last_payload: Option<Vec<u8>>,
    pub entries_below: Vec<TopicTreeEntry>,
}

impl TopicTreeEntry {
    pub fn leaf(&self) -> &str {
        topic::get_leaf(&self.topic)
    }

    pub fn topics_below(&self) -> usize {
        let mut counter = 0;
        for below in &self.entries_below {
            if below.messages > 0 {
                counter += 1;
            }

            counter += below.topics_below();
        }

        counter
    }

    pub fn messages_below(&self) -> usize {
        let mut counter = self.messages;
        for below in &self.entries_below {
            counter += below.messages_below();
        }

        counter
    }
}

pub fn get_tmlp_as_tree<'a, I>(entries: I) -> Vec<TopicTreeEntry>
where
    I: IntoIterator<Item = &'a TopicMessagesLastPayload>,
{
    let mut map: HashMap<&str, &TopicMessagesLastPayload> = HashMap::new();
    for entry in entries {
        map.insert(&entry.topic, entry);
    }

    let mut keys = map.keys().cloned().collect::<Vec<_>>();
    keys.sort_unstable();

    let roots = topic::get_all_roots(keys.clone());
    let topics = topic::get_all_with_parents(keys);

    let mut result = Vec::new();
    for root in roots {
        result.push(build_recursive(&map, &topics, root));
    }

    result
}

fn build_recursive(
    map: &HashMap<&str, &TopicMessagesLastPayload>,
    all_topics: &[&str],
    topic: &str,
) -> TopicTreeEntry {
    let mut entries_below: Vec<TopicTreeEntry> = Vec::new();
    for child in topic::get_direct_children(topic, all_topics) {
        entries_below.push(build_recursive(map, all_topics, child));
    }

    let info = map.get(topic);

    TopicTreeEntry {
        topic: topic.to_owned(),
        messages: info.map_or(0, |o| o.messages),
        last_payload: info.map(|o| o.last_payload.clone()),
        entries_below,
    }
}

pub fn get_identifier_of_topic(
    tree_items: &[TopicTreeEntry],
    topic: &str,
) -> Option<TreeIdentifierVec> {
    let mut identifier = Vec::new();
    let mut current = tree_items;
    for part in topic::get_parts(topic) {
        let index = current
            .iter()
            .position(|o| topic::get_leaf(&o.topic) == part)?;
        current = &current.get(index).unwrap().entries_below;
        identifier.push(index);
    }

    Some(identifier)
}

pub fn tree_items_from_tmlp_tree(entries: &[TopicTreeEntry]) -> Vec<TreeItem> {
    let mut result = Vec::new();

    for entry in entries {
        let children = tree_items_from_tmlp_tree(&entry.entries_below);

        let meta = if let Some(payload) = &entry.last_payload {
            format!("= {}", format::payload_as_utf8(payload.clone()))
        } else {
            format!(
                "({} topics, {} messages)",
                entry.topics_below(),
                entry.messages_below()
            )
        };

        let text = vec![Spans::from(vec![
            Span::styled(entry.leaf(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(meta, Style::default().fg(Color::DarkGray)),
        ])];

        result.push(TreeItem::new(text, children));
    }

    result
}

pub fn is_topic_opened(opened: &HashSet<String>, topic: &str) -> bool {
    topic::get_all_parents(topic)
        .iter()
        .cloned()
        .all(|t| opened.contains(t))
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
