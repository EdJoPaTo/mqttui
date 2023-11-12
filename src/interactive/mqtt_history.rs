use std::collections::HashMap;

use ego_tree::{NodeId, NodeRef, Tree};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use tui_tree_widget::TreeItem;

use crate::interactive::ui::STYLE_BOLD;
use crate::mqtt::{HistoryEntry, Payload};

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
            self.ids.insert(topic.to_owned(), parent);
            parent
        }
    }

    pub fn add(&mut self, topic: &str, history_entry: HistoryEntry) {
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

    pub fn get_last(&self, topic: &str) -> Option<&HistoryEntry> {
        let id = self.ids.get(topic)?;
        self.tree
            .get(*id)
            .and_then(|node| node.value().history.last())
    }

    pub fn get_topics_below(&self, topic: &str) -> Vec<String> {
        fn build_recursive(prefix: &[&str], node: NodeRef<Topic>) -> Vec<String> {
            let mut topic = prefix.to_vec();
            topic.push(&node.value().leaf);

            let mut entries_below = node
                .children()
                .flat_map(|c| build_recursive(&topic, c))
                .collect::<Vec<_>>();
            if !node.value().history.is_empty() {
                entries_below.insert(0, topic.join("/"));
            }
            entries_below
        }

        // Get the node of the given topic in the tree
        let mut noderef = self.tree.root();
        for part in topic.split('/') {
            let node = noderef.children().find(|o| &*o.value().leaf == part);
            if let Some(node) = node {
                noderef = node;
            } else {
                // Node not found -> there are no topics below
                return vec![];
            }
        }

        let mut prefix = topic.split('/').collect::<Vec<_>>();
        prefix.pop(); // The node itself will also be added so its not part of the prefix
        build_recursive(&prefix, noderef)
    }

    /// Returns (`topic_amount`, `TreeItem`s)
    pub fn to_tree_items(&self, filter: &str) -> (usize, Vec<TreeItem<'static, String>>) {
        fn build_recursive(
            prefix: &[&str],
            node: NodeRef<Topic>,
            filter: &str,
        ) -> Option<RecursiveTreeItemGenerator> {
            let Topic { leaf, history } = node.value();
            let mut topic = prefix.to_vec();
            topic.push(leaf);

            let entries_below = node
                .children()
                .filter_map(|c| build_recursive(&topic, c, filter))
                .collect::<Vec<_>>();

            let is_filter_match = topic.join("/").contains(filter);

            if !is_filter_match && entries_below.is_empty() {
                return None;
            }

            let messages_below = entries_below
                .iter()
                .map(|below| below.messages + below.messages_below)
                .sum();
            let topics_below = entries_below
                .iter()
                .map(|below| {
                    let has_messages = usize::from(below.messages > 0);
                    has_messages + below.topics_below
                })
                .sum();
            let children = entries_below
                .into_iter()
                .map(|o| o.tree_item)
                .collect::<Vec<_>>();

            let meta = match history.last().map(|o| &o.payload) {
                Some(Payload::String(str)) => format!("= {str}"),
                Some(Payload::Json(json)) => format!("= {json}"),
                Some(Payload::NotUtf8(_)) => "Payload not UTF-8".to_owned(),
                None => format!("({topics_below} topics, {messages_below} messages)"),
            };
            let text = Line::from(vec![
                Span::styled(leaf.to_string(), STYLE_BOLD),
                Span::raw(" "),
                Span::styled(meta, STYLE_DARKGRAY),
            ]);

            Some(RecursiveTreeItemGenerator {
                messages_below,
                messages: history.len(),
                topics_below,
                tree_item: TreeItem::new(leaf.to_string(), text, children).unwrap(),
            })
        }

        let children = self
            .tree
            .root()
            .children()
            .filter_map(|o| build_recursive(&[], o, filter))
            .collect::<Vec<_>>();
        let topics = children
            .iter()
            .map(|o| o.topics_below + usize::from(o.messages > 0))
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
            "test",
            HistoryEntry::new_now(false, rumqttc::QoS::AtLeastOnce, &"A".into()),
        );
        history.add(
            "foo/test",
            HistoryEntry::new_now(false, rumqttc::QoS::AtLeastOnce, &"B".into()),
        );
        history.add(
            "test",
            HistoryEntry::new_now(false, rumqttc::QoS::AtLeastOnce, &"C".into()),
        );
        history.add(
            "foo/bar",
            HistoryEntry::new_now(false, rumqttc::QoS::AtLeastOnce, &"D".into()),
        );
        history
    }
}

#[test]
fn topics_below_works() {
    let actual = MqttHistory::example().get_topics_below("foo");
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
    let (topics, items) = example.to_tree_items("");
    assert_eq!(topics, 3);
    dbg!(&items);
    assert_eq!(items.len(), 2);
    assert!(items[0].child(0).is_some());
    assert!(items[0].child(1).is_some());
    assert!(items[0].child(2).is_none());
    assert!(items[1].child(0).is_none());
}
