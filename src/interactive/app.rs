use crate::mqtt_history::{self, HistoryArc};
use crate::topic;
use crate::topic_view;
use std::collections::HashSet;
use std::error::Error;
use tui_tree_widget::{flatten, TreeState};

pub struct App<'a> {
    pub host: &'a str,
    pub port: u16,
    pub subscribe_topic: &'a str,
    pub history: HistoryArc,

    pub json_view_state: TreeState,
    pub opened_topics: HashSet<String>,
    pub selected_topic: Option<String>,
    pub should_quit: bool,
    pub topic_overview_state: TreeState,
}

impl<'a> App<'a> {
    pub fn new(host: &'a str, port: u16, subscribe_topic: &'a str, history: HistoryArc) -> App<'a> {
        App {
            host,
            port,
            subscribe_topic,
            history,

            json_view_state: TreeState::default(),
            opened_topics: HashSet::new(),
            selected_topic: None,
            should_quit: false,
            topic_overview_state: TreeState::default(),
        }
    }

    fn change_selected_topic(&mut self, down: bool) -> Result<(), Box<dyn Error>> {
        let history = self
            .history
            .lock()
            .map_err(|err| format!("failed to aquire lock of mqtt history: {}", err))?;

        let topics = mqtt_history::history_to_tmlp(history.iter());
        let topics_with_parents =
            topic::get_all_with_parents(topics.iter().map(|o| o.topic.as_ref()));
        let visible_topics = topics_with_parents
            .iter()
            .filter(|topic| topic_view::is_topic_opened(&self.opened_topics, topic))
            .cloned()
            .collect::<Vec<_>>();

        let tmlp_tree = topic_view::get_tmlp_as_tree(&topics);

        let tree_items = topic_view::tree_items_from_tmlp_tree(&tmlp_tree);
        let visible = flatten(&self.topic_overview_state.get_all_opened(), &tree_items);

        let current_identifier = self.selected_topic.as_ref().and_then(|selected_topic| {
            topic_view::get_identifier_of_topic(&tmlp_tree, selected_topic)
        });
        let current_index = current_identifier
            .and_then(|identifier| visible.iter().position(|o| o.identifier == identifier));

        let new_index = if let Some(current_index) = current_index {
            if down {
                current_index.saturating_add(1) % visible_topics.len()
            } else {
                current_index.overflowing_sub(1).0
            }
        } else if down {
            0
        } else {
            usize::MAX
        }
        .min(visible_topics.len() - 1);

        self.selected_topic = visible_topics.get(new_index).map(|o| (*o).to_string());
        Ok(())
    }

    pub fn on_up(&mut self) -> Result<(), Box<dyn Error>> {
        let increase = false;
        self.change_selected_topic(increase)
    }

    pub fn on_down(&mut self) -> Result<(), Box<dyn Error>> {
        let increase = true;
        self.change_selected_topic(increase)
    }

    pub fn on_right(&mut self) {
        if let Some(topic) = &self.selected_topic {
            self.opened_topics.insert(topic.to_owned());
        }
    }

    pub fn on_left(&mut self) {
        if let Some(topic) = &self.selected_topic {
            if let false = self.opened_topics.remove(topic) {
                self.selected_topic = topic::get_parent(topic).map(std::borrow::ToOwned::to_owned);
            }
        }
    }

    pub fn on_toggle(&mut self) {
        if let Some(topic) = &self.selected_topic {
            if self.opened_topics.contains(topic) {
                self.opened_topics.remove(topic);
            } else {
                self.opened_topics.insert(topic.to_owned());
            }
        }
    }
}
