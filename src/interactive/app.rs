use crate::mqtt_history::{self, HistoryArc};
use crate::topic;
use crate::topic_view;
use std::cmp::min;
use std::collections::HashSet;
use std::error::Error;
use tui::widgets::ListState;

pub struct App<'a> {
    pub host: &'a str,
    pub port: u16,
    pub subscribe_topic: &'a str,
    pub history: HistoryArc,

    pub opened_topics: HashSet<String>,
    pub selected_topic: Option<String>,
    pub should_quit: bool,
    pub topics_overview_state: ListState,
}

impl<'a> App<'a> {
    pub fn new(host: &'a str, port: u16, subscribe_topic: &'a str, history: HistoryArc) -> App<'a> {
        App {
            host,
            port,
            subscribe_topic,
            history,

            opened_topics: HashSet::new(),
            selected_topic: None,
            should_quit: false,
            topics_overview_state: ListState::default(),
        }
    }

    fn change_selected_topic(&mut self, increase: bool) -> Result<(), Box<dyn Error>> {
        let history = self
            .history
            .lock()
            .map_err(|err| format!("failed to aquire lock of mqtt history: {}", err))?;

        let topics = mqtt_history::history_to_tmlp(history.iter());
        let entries = topic_view::get_tree_with_metadata(&topics);
        let visible_entries: Vec<_> = entries
            .iter()
            .filter(|o| topic_view::is_topic_opened(&self.opened_topics, o.topic))
            .collect();

        let new_index = if let Some(topic) = &self.selected_topic {
            visible_entries
                .iter()
                .position(|o| o.topic == topic)
                .map_or(0, |current_pos| {
                    if increase {
                        current_pos.saturating_add(1) % visible_entries.len()
                    } else {
                        current_pos.overflowing_sub(1).0
                    }
                })
        } else if increase {
            0
        } else {
            usize::MAX
        };

        self.selected_topic = visible_entries
            .get(min(new_index, visible_entries.len() - 1))
            .map(|s| s.topic.to_owned());

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
