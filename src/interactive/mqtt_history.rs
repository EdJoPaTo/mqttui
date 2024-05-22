use std::collections::{HashMap, HashSet};

use ego_tree::{NodeId, NodeRef, Tree};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use tui_tree_widget::{Node, TreeData};

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

    pub fn total_topics(&self) -> usize {
        self.ids.len()
    }

    pub fn total_messages(&self) -> usize {
        self.tree
            .values()
            .map(|topic| topic.history.len())
            .fold(0, usize::saturating_add)
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

    pub fn count_topics_and_messages_below(&self, topic: &str) -> (usize, usize) {
        let mut topics: usize = 0;
        let mut messages: usize = 0;
        for (_, id) in self
            .ids
            .iter()
            .filter(|(key, _)| is_topic_below(topic, key))
        {
            let node = self.tree.get(*id).unwrap();
            let Topic { history, .. } = node.value();

            messages = messages.saturating_add(history.len());
            if !history.is_empty() {
                topics = topics.saturating_add(1);
            }
        }
        (topics, messages)
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

impl TreeData for MqttHistory {
    type Identifier = String;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Vec<Self::Identifier>>,
    ) -> Vec<Node<Self::Identifier>> {
        fn recursive(
            result: &mut Vec<Node<String>>,
            open_identifiers: &HashSet<Vec<String>>,
            node: NodeRef<Topic>,
            prefix: &[&str],
        ) {
            let Topic { leaf, .. } = node.value();
            let mut topic = prefix.to_vec();
            topic.push(leaf);

            let identifier = topic.iter().map(ToString::to_string).collect();

            let is_open = open_identifiers.contains(&identifier);

            result.push(Node {
                identifier,
                has_children: node.has_children(),
                height: 1,
            });

            if is_open {
                for node in node.children() {
                    recursive(result, open_identifiers, node, &topic);
                }
            }
        }

        let mut result = Vec::new();
        for node in self.tree.root().children() {
            recursive(&mut result, open_identifiers, node, &[]);
        }
        result
    }

    fn render(
        &self,
        identifier: &[Self::Identifier],
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        let leaf = identifier.last().unwrap();
        let topic = identifier.join("/");

        let payload = self
            .ids
            .get(&topic)
            .and_then(|id| self.tree.get(*id))
            .and_then(|node| node.value().history.last())
            .map(|entry| &entry.payload);
        let meta = payload.map_or_else(
            || {
                let (topics, messages) = self.count_topics_and_messages_below(&topic);
                format!("({topics} topics, {messages} messages)")
            },
            |payload| format!("= {payload}"),
        );
        let text = Line::from(vec![
            Span::styled(leaf, STYLE_BOLD),
            Span::raw(" "),
            Span::styled(meta, STYLE_DARKGRAY),
        ]);
        ratatui::widgets::Widget::render(text, area, buffer);
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
fn count_works() {
    let example = MqttHistory::example();
    let (topics, messages) = example.count_topics_and_messages_below("foo");
    assert_eq!(topics, 2);
    assert_eq!(messages, 2);

    let (topics, messages) = example.count_topics_and_messages_below("test");
    assert_eq!(topics, 1);
    assert_eq!(messages, 2);
}

#[test]
fn total_works() {
    let example = MqttHistory::example();
    assert_eq!(example.total_topics(), 4);
    assert_eq!(example.total_messages(), 5);
}

#[test]
fn tree_data_all_closed_works() {
    let example = MqttHistory::example();
    let open = HashSet::new();
    let nodes = example.get_nodes(&open);
    dbg!(&nodes);
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].identifier, &["foo"]);
    assert_eq!(nodes[1].identifier, &["test"]);
    assert_eq!(nodes[2].identifier, &["testing"]);
}

#[test]
fn tree_data_some_open_works() {
    let example = MqttHistory::example();
    let mut open = HashSet::new();
    open.insert(vec!["foo".to_owned()]);
    let nodes = example.get_nodes(&open);
    dbg!(&nodes);
    assert_eq!(nodes.len(), 5);
    assert_eq!(nodes[0].identifier, &["foo"]);
    assert_eq!(nodes[1].identifier, &["foo", "bar"]);
    assert_eq!(nodes[2].identifier, &["foo", "test"]);
    assert_eq!(nodes[3].identifier, &["test"]);
    assert_eq!(nodes[4].identifier, &["testing"]);
}
