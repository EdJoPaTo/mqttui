use crate::{
    mqtt_history::{get_sorted_vec, HistoryArc},
    topic_logic::{get_parent, get_shown_topics},
};
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
    pub topics_overview_state: ListState,

    pub should_quit: bool,
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
            topics_overview_state: ListState::default(),

            should_quit: false,
        }
    }

    fn change_seleted_topic(&mut self, increase: bool) -> Result<(), Box<dyn Error>> {
        let topics = get_sorted_vec(
            self.history
                .lock()
                .map_err(|err| format!("failed to aquire lock of mqtt history: {}", err))?
                .keys(),
        );
        let shown = get_shown_topics(&topics, &self.opened_topics);

        let new_index = if let Some(topic) = &self.selected_topic {
            shown
                .iter()
                .position(|o| o == topic)
                .map(|current_pos| {
                    if increase {
                        current_pos.checked_add(1)
                    } else {
                        current_pos.checked_sub(1)
                    }
                    .unwrap_or(current_pos)
                })
                .unwrap_or(0)
        } else if increase {
            0
        } else {
            usize::MAX
        };

        self.selected_topic = shown
            .get(min(new_index, shown.len() - 1))
            .map(|s| s.to_owned().to_owned());

        Ok(())
    }

    pub fn on_up(&mut self) -> Result<(), Box<dyn Error>> {
        self.change_seleted_topic(false)
    }

    pub fn on_down(&mut self) -> Result<(), Box<dyn Error>> {
        self.change_seleted_topic(true)
    }

    pub fn on_right(&mut self) {
        if let Some(topic) = &self.selected_topic {
            self.opened_topics.insert(topic.to_owned());
        }
    }

    pub fn on_left(&mut self) {
        if let Some(topic) = &self.selected_topic {
            if let false = self.opened_topics.remove(topic) {
                self.selected_topic = get_parent(topic).map(|s| s.to_owned());
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
