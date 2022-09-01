use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Local};
use ego_tree::{NodeId, NodeRef, Tree};
use rumqttc::Publish;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui_tree_widget::{TreeIdentifierVec, TreeItem};

use crate::mqtt::{HistoryEntry, Payload};

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

struct RecursiveTreeItemGenerator<'a> {
    messages_below: usize,
    messages: usize,
    topics_below: usize,
    tree_item: TreeItem<'a>,
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

    fn entry(&mut self, topic: &str) -> NodeId {
        if let Some(id) = self.ids.get(topic) {
            *id
        } else {
            let mut parent = self.tree.root().id();
            for part in topic.split('/') {
                let noderef = self.tree.get(parent).unwrap();
                let equal_or_after = noderef.children().find(|o| &*o.value().leaf >= part);
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
            self.ids.insert(topic.to_string(), parent);
            parent
        }
    }

    pub fn add(&mut self, packet: &Publish, time: DateTime<Local>) {
        let id = self.entry(&packet.topic);
        self.tree
            .get_mut(id)
            .unwrap()
            .value()
            .history
            .push(HistoryEntry::new(packet, time));
    }

    pub fn get(&self, topic: &str) -> Option<&Vec<HistoryEntry>> {
        let id = self.ids.get(topic)?;
        self.tree.get(*id).map(|node| &node.value().history)
    }

    pub fn get_last(&self, topic: &str) -> Option<&HistoryEntry> {
        let id = self.ids.get(topic)?;
        self.tree
            .get(*id)
            .and_then(|node| node.value().history.last())
    }

    pub fn get_tree_identifier(&self, topic: &str) -> Option<TreeIdentifierVec> {
        let mut identifier = Vec::new();
        let mut parent = self.tree.root();
        for part in topic.split('/') {
            let (index, child) = parent
                .children()
                .enumerate()
                .find(|(_i, o)| &*o.value().leaf == part)?;
            identifier.push(index);
            parent = child;
        }
        Some(identifier)
    }

    pub fn get_visible_topics(&self, opened_topics: &HashSet<String>) -> Vec<String> {
        fn build_recursive(
            opened_topics: &HashSet<String>,
            prefix: &[&str],
            node: NodeRef<Topic>,
        ) -> Vec<String> {
            let mut topic = prefix.to_vec();
            topic.push(&node.value().leaf);

            let topic_string = topic.join("/");

            if opened_topics.contains(&topic_string) {
                let mut entries_below = node
                    .children()
                    .flat_map(|c| build_recursive(opened_topics, &topic, c))
                    .collect::<Vec<_>>();
                entries_below.insert(0, topic_string);
                entries_below
            } else {
                vec![topic_string]
            }
        }

        self.tree
            .root()
            .children()
            .flat_map(|o| build_recursive(opened_topics, &[], o))
            .collect::<Vec<_>>()
    }

    /// Returns (`topic_amount`, `TreeItem`s)
    pub fn to_tree_items(&self) -> (usize, Vec<TreeItem>) {
        fn build_recursive<'a>(
            prefix: &[&str],
            node: NodeRef<'a, Topic>,
        ) -> RecursiveTreeItemGenerator<'a> {
            let Topic { leaf, history } = node.value();
            let mut topic = prefix.to_vec();
            topic.push(leaf);

            let entries_below = node
                .children()
                .map(|c| build_recursive(&topic, c))
                .collect::<Vec<_>>();
            let messages_below = entries_below
                .iter()
                .map(|below| below.messages + below.messages_below)
                .sum();
            let topics_below = entries_below
                .iter()
                .map(|below| {
                    let has_messages = if below.messages > 0 { 1 } else { 0 };
                    has_messages + below.topics_below
                })
                .sum();
            let children = entries_below
                .into_iter()
                .map(|o| o.tree_item)
                .collect::<Vec<_>>();

            let meta = match history.last().map(|o| &o.payload) {
                Some(Payload::String(str)) => format!("= {}", str),
                Some(Payload::Json(json)) => format!("= {}", json.dump()),
                Some(Payload::NotUtf8(_)) => "Payload not UTF-8".to_string(),
                None => format!("({} topics, {} messages)", topics_below, messages_below),
            };
            let text = vec![Spans::from(vec![
                Span::styled(leaf.as_ref(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(meta, Style::default().fg(Color::DarkGray)),
            ])];

            RecursiveTreeItemGenerator {
                messages_below,
                messages: history.len(),
                topics_below,
                tree_item: TreeItem::new(text, children),
            }
        }

        let children = self
            .tree
            .root()
            .children()
            .map(|o| build_recursive(&[], o))
            .collect::<Vec<_>>();
        let topics = children
            .iter()
            .map(|o| o.topics_below + if o.messages > 0 { 1 } else { 0 })
            .sum();
        let items = children
            .into_iter()
            .map(|o| o.tree_item)
            .collect::<Vec<_>>();
        (topics, items)
    }

    #[cfg(test)]
    pub fn example() -> Self {
        let mut history = Self::new();
        history.add(
            &Publish::new("test", rumqttc::QoS::AtLeastOnce, "A"),
            Local::now(),
        );
        history.add(
            &Publish::new("foo/test", rumqttc::QoS::AtLeastOnce, "B"),
            Local::now(),
        );
        history.add(
            &Publish::new("test", rumqttc::QoS::AtLeastOnce, "C"),
            Local::now(),
        );
        history.add(
            &Publish::new("foo/bar", rumqttc::QoS::AtLeastOnce, "D"),
            Local::now(),
        );
        history
    }
}

#[test]
fn tree_identifier_works() {
    let history = MqttHistory::example();
    assert_eq!(history.get_tree_identifier("whatever"), None);
    assert_eq!(history.get_tree_identifier("test").unwrap(), [1]);
    assert_eq!(history.get_tree_identifier("foo/bar").unwrap(), [0, 0]);
    assert_eq!(history.get_tree_identifier("foo/test").unwrap(), [0, 1]);
}

#[test]
fn visible_all_closed_works() {
    let opened_topics = HashSet::new();
    let actual = MqttHistory::example().get_visible_topics(&opened_topics);
    assert_eq!(actual, ["foo", "test"]);
}

#[test]
fn visible_opened_works() {
    let mut opened_topics = HashSet::new();
    opened_topics.insert("foo".into());
    let actual = MqttHistory::example().get_visible_topics(&opened_topics);
    assert_eq!(actual, ["foo", "foo/bar", "foo/test", "test"]);
}

#[test]
fn tree_items_works() {
    let example = MqttHistory::example();
    let (topics, items) = example.to_tree_items();
    assert_eq!(topics, 3);
    dbg!(&items);
    assert_eq!(items.len(), 2);
    assert!(items[0].child(0).is_some());
    assert!(items[0].child(1).is_some());
    assert!(items[0].child(2).is_none());
    assert!(items[1].child(0).is_none());
}
