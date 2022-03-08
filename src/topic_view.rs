use crate::mqtt_history::TopicMessagesLastPayload;
use crate::{format, topic};
use std::collections::{HashMap, HashSet};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui_tree_widget::{TreeIdentifierVec, TreeItem};

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
        self.entries_below
            .iter()
            .map(|below| {
                let has_messages = if below.messages > 0 { 1 } else { 0 };
                let topics_below = below.topics_below();
                has_messages + topics_below
            })
            .sum()
    }

    pub fn messages_below(&self) -> usize {
        let below = self
            .entries_below
            .iter()
            .map(Self::messages_below)
            .sum::<usize>();
        self.messages + below
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

    let mut keys = map.keys().copied().collect::<Vec<_>>();
    keys.sort_unstable();

    let topics = topic::get_all_with_parents(keys.clone());
    topic::get_all_roots(keys)
        .iter()
        .map(|root| build_recursive(&map, &topics, root))
        .collect()
}

fn build_recursive(
    map: &HashMap<&str, &TopicMessagesLastPayload>,
    all_topics: &[&str],
    topic: &str,
) -> TopicTreeEntry {
    let entries_below = topic::get_direct_children(topic, all_topics)
        .iter()
        .map(|child| build_recursive(map, all_topics, child))
        .collect();

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
    for part in topic.split('/') {
        let index = current
            .iter()
            .position(|o| topic::get_leaf(&o.topic) == part)?;
        current = &current.get(index).unwrap().entries_below;
        identifier.push(index);
    }

    Some(identifier)
}

pub fn tree_items_from_tmlp_tree<'a, I>(entries: I) -> Vec<TreeItem<'a>>
where
    I: IntoIterator<Item = &'a TopicTreeEntry>,
{
    entries
        .into_iter()
        .map(|entry| {
            let children = tree_items_from_tmlp_tree(&entry.entries_below);

            let meta = entry.last_payload.as_ref().map_or_else(
                || {
                    format!(
                        "({} topics, {} messages)",
                        entry.topics_below(),
                        entry.messages_below()
                    )
                },
                |payload| format!("= {}", format::payload_as_utf8(payload.clone())),
            );

            let text = vec![Spans::from(vec![
                Span::styled(entry.leaf(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(meta, Style::default().fg(Color::DarkGray)),
            ])];

            TreeItem::new(text, children)
        })
        .collect()
}

pub fn is_topic_opened(opened: &HashSet<String>, topic: &str) -> bool {
    topic::get_all_parents(topic)
        .iter()
        .copied()
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
    let actual = ALL_EXAMPLES
        .iter()
        .copied()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect::<Vec<_>>();
    assert_eq!(actual, ["a", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_some() {
    let mut opened = HashSet::new();
    opened.insert("a".to_string());
    let actual = ALL_EXAMPLES
        .iter()
        .copied()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect::<Vec<_>>();
    assert_eq!(actual, ["a", "a/b", "a/d", "e"]);
}

#[test]
fn filter_topics_by_opened_shows_only_when_all_parents_are_opened() {
    let mut opened = HashSet::new();
    opened.insert("a/b".to_string());
    let actual = ALL_EXAMPLES
        .iter()
        .copied()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect::<Vec<_>>();
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

    let actual = ALL_EXAMPLES
        .iter()
        .copied()
        .filter(|entry| is_topic_opened(&opened, entry))
        .collect::<Vec<_>>();
    assert_eq!(actual, ALL_EXAMPLES);
}
