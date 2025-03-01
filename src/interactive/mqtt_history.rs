use std::collections::HashMap;

use ego_tree::{NodeId, NodeRef, Tree};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use tui_tree_widget::TreeItem;

use crate::interactive::ui::STYLE_BOLD;
use crate::mqtt::HistoryEntry;

pub const STYLE_DARKGRAY: Style = Style::new().fg(Color::DarkGray);

struct Topic {
    /// Topic `foo/bar` would have the leaf `bar`
    leaf: Box<str>,
    history: Vec<HistoryEntry>,
}

impl Topic {
    const fn new(leaf: Box<str>) -> Self {
        Self {
            leaf,
            history: Vec::new(),
        }
    }
}

struct RecursiveTreeItemGenerator {
    messages_below: usize,
    messages: usize,
    topics_below: usize,
    tree_item: TreeItem<'static, String>,
}

pub struct MqttHistory {
    tree: Tree<Topic>,
    ids: HashMap<String, NodeId>,
}

impl MqttHistory {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(Topic::new("".into())),
            ids: HashMap::new(),
        }
    }

    fn entry(&mut self, topic: String) -> NodeId {
        if let Some(id) = self.ids.get(&topic) {
            *id
        } else {
            let mut parent = self.tree.root().id();
            for part in topic.split('/') {
                let noderef = self.tree.get(parent).unwrap();
                let equal_or_after = noderef.children().find(|node| &*node.value().leaf >= part);
                if let Some(eoa) = equal_or_after {
                    if eoa.value().leaf.as_ref() == part {
                        parent = eoa.id();
                        continue;
                    }
                    let eoa_id = eoa.id();
                    let mut eoamut = self.tree.get_mut(eoa_id).unwrap();
                    parent = eoamut.insert_before(Topic::new(part.into())).id();
                } else {
                    let mut nodemut = self.tree.get_mut(parent).unwrap();
                    parent = nodemut.append(Topic::new(part.into())).id();
                }
            }
            self.ids.insert(topic, parent);
            parent
        }
    }

    pub fn add(&mut self, topic: String, history_entry: HistoryEntry) {
        let id = self.entry(topic);
        self.tree
            .get_mut(id)
            .unwrap()
            .value()
            .history
            .push(history_entry);
    }

    pub fn get(&self, topic: &str) -> Option<&Vec<HistoryEntry>> {
        let id = self.ids.get(topic)?;
        self.tree.get(*id).map(|node| &node.value().history)
    }

    pub fn uncache_topic_entry(&mut self, topic: &str, index: usize) -> Option<HistoryEntry> {
        let id = self.ids.get(topic)?;
        let mut node = self.tree.get_mut(*id)?;
        let topic = node.value();
        // Prevent removing the newest entry (or out of bounds)
        if index >= topic.history.len().saturating_sub(1) {
            return None;
        }
        let entry = topic.history.remove(index);
        Some(entry)
    }

    pub fn get_all_topics(&self) -> Vec<&String> {
        let mut topics = self.ids.keys().collect::<Vec<_>>();
        topics.sort();
        topics
    }

    pub fn get_topics_below(&self, base: &str) -> Vec<String> {
        self.ids
            .keys()
            .filter(|key| is_topic_below(base, key))
            .cloned()
            .collect()
    }

    /// Returns (`topic_amount`, `message_amount`, `TreeItem`s)
    pub fn to_tree_items(&self) -> (usize, usize, Vec<TreeItem<'static, String>>) {
        fn build_recursive(prefix: &[&str], node: NodeRef<Topic>) -> RecursiveTreeItemGenerator {
            let Topic { leaf, history } = node.value();
            let mut topic = prefix.to_vec();
            topic.push(leaf);

            let entries_below = node.children().map(|node| build_recursive(&topic, node));
            let mut messages_below: usize = 0;
            let mut topics_below: usize = 0;
            let mut children = Vec::new();
            for below in entries_below {
                messages_below = messages_below
                    .saturating_add(below.messages)
                    .saturating_add(below.messages_below);
                topics_below = topics_below
                    .saturating_add(usize::from(below.messages > 0))
                    .saturating_add(below.topics_below);
                children.push(below.tree_item);
            }

            let meta = history.last().map(|entry| &entry.payload).map_or_else(
                || format!("({topics_below} topics, {messages_below} messages)"),
                |payload| format!("= {payload}"),
            );
            let text = Line::from(vec![
                Span::styled(leaf.to_string(), STYLE_BOLD),
                Span::raw(" "),
                Span::styled(meta, STYLE_DARKGRAY),
            ]);

            RecursiveTreeItemGenerator {
                messages_below,
                messages: history.len(),
                topics_below,
                tree_item: TreeItem::new(leaf.to_string(), text, children).unwrap(),
            }
        }

        let children = self
            .tree
            .root()
            .children()
            .map(|node| build_recursive(&[], node));
        let mut topics: usize = 0;
        let mut messages: usize = 0;
        let mut items = Vec::new();
        for child in children {
            topics = topics
                .saturating_add(usize::from(child.messages > 0))
                .saturating_add(child.topics_below);
            messages = messages
                .saturating_add(child.messages)
                .saturating_add(child.messages_below);
            items.push(child.tree_item);
        }
        (topics, messages, items)
    }

    #[cfg(test)]
    pub fn example() -> Self {
        fn entry(payload: &str) -> HistoryEntry {
            HistoryEntry {
                qos: rumqttc::QoS::AtLeastOnce,
                time: crate::mqtt::Time::new_now(false),
                payload_size: payload.len(),
                payload: crate::payload::Payload::unlimited(payload.into()),
            }
        }

        let mut history = Self::new();
        history.add("test".to_owned(), entry("A"));
        history.add("foo/test".to_owned(), entry("B"));
        history.add("test".to_owned(), entry("C"));
        history.add("foo/bar".to_owned(), entry("D"));
        history.add("testing/stuff".to_owned(), entry("E"));
        history
    }
}

fn is_topic_below(base: &str, child: &str) -> bool {
    if base == child {
        return true;
    }
    if !child.starts_with(base) {
        return false;
    }
    child
        .get(base.len()..)
        .is_some_and(|after| after.starts_with('/'))
}

#[test]
fn topics_below_works() {
    let mut actual = MqttHistory::example().get_topics_below("foo");
    actual.sort_unstable();
    assert_eq!(actual, ["foo/bar", "foo/test"]);
}

#[test]
fn topics_below_finds_itself_works() {
    let actual = MqttHistory::example().get_topics_below("test");
    assert_eq!(actual, ["test"]);
}

#[test]
fn tree_items_works() {
    let example = MqttHistory::example();
    let (topics, messages, items) = example.to_tree_items();
    assert_eq!(topics, 4);
    assert_eq!(messages, 5);
    dbg!(&items);
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].children().len(), 2);
    assert_eq!(items[1].children().len(), 0);
    assert_eq!(items[2].children().len(), 1);
}
