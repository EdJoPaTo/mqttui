use std::collections::HashMap;

use chrono::{DateTime, Local};
use ego_tree::{NodeId, NodeRef, Tree};
use rumqttc::Publish;
use tui_tree_widget::TreeIdentifierVec;

use crate::interactive::topic_tree_entry::TopicTreeEntry;
use crate::mqtt_packet::HistoryEntry;

struct Topic {
    /// Topic `foo/bar` would have the leaf `bar`
    leaf: String,
    history: Vec<HistoryEntry>,
}

impl Topic {
    const fn new(leaf: String) -> Self {
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
            tree: Tree::new(Topic::new("".to_string())),
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
                    if eoa.value().leaf == part {
                        parent = eoa.id();
                        continue;
                    }
                    let eoa_id = eoa.id();
                    let mut eoamut = self.tree.get_mut(eoa_id).unwrap();
                    parent = eoamut.insert_before(Topic::new(part.to_string())).id();
                } else {
                    let mut nodemut = self.tree.get_mut(parent).unwrap();
                    parent = nodemut.append(Topic::new(part.to_string())).id();
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

    pub fn to_tte(&self) -> Vec<TopicTreeEntry> {
        fn build_recursive(prefix: &[&str], node: NodeRef<Topic>) -> TopicTreeEntry {
            let value = node.value();
            let mut topic = prefix.to_vec();
            topic.push(&value.leaf);

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

            TopicTreeEntry {
                topic: topic.join("/"),
                leaf: value.leaf.clone(),
                // TODO: without clone?
                last_payload: value.history.last().map(|o| o.payload.clone()),
                messages: value.history.len(),
                topics_below,
                messages_below,
                entries_below,
            }
        }

        self.tree
            .root()
            .children()
            .map(|o| build_recursive(&[], o))
            .collect::<Vec<_>>()
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
fn tte_works() {
    let expected = TopicTreeEntry::examples();
    let actual = MqttHistory::example().to_tte();
    dbg!(&actual);
    assert_eq!(actual, expected);
}
