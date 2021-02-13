use crate::mqtt_history::HistoryArc;
use crate::{format, json_view, mqtt_history, topic, topic_view};
use json::JsonValue;
use std::collections::HashSet;
use std::error::Error;
use tui_tree_widget::{flatten, TreeState};

#[derive(Debug, PartialEq)]
pub enum ElementInFocus {
    TopicOverview,
    JsonPayload,
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
}

#[derive(Debug)]
enum CursorMove {
    Absolute(usize),
    Relative(Direction),
}

pub struct App<'a> {
    pub host: &'a str,
    pub port: u16,
    pub subscribe_topic: &'a str,
    pub history: HistoryArc,

    pub focus: ElementInFocus,
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

            focus: ElementInFocus::TopicOverview,
            json_view_state: TreeState::default(),
            opened_topics: HashSet::new(),
            selected_topic: None,
            should_quit: false,
            topic_overview_state: TreeState::default(),
        }
    }

    fn change_selected_topic(&mut self, cursor_move: CursorMove) -> Result<bool, Box<dyn Error>> {
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

        let new_index = match cursor_move {
            CursorMove::Absolute(index) => index,
            CursorMove::Relative(direction) => current_index.map_or_else(
                || match direction {
                    Direction::Down => 0,
                    Direction::Up => usize::MAX,
                },
                |current_index| match direction {
                    Direction::Up => current_index.overflowing_sub(1).0,
                    Direction::Down => current_index.saturating_add(1) % visible_topics.len(),
                },
            ),
        }
        .min(visible_topics.len() - 1);

        let next_selected_topic = visible_topics.get(new_index).map(|o| (*o).to_string());
        let different = self.selected_topic != next_selected_topic;
        self.selected_topic = next_selected_topic;
        Ok(different)
    }

    fn get_json_of_current_topic(&self) -> Result<Option<JsonValue>, Box<dyn Error>> {
        let history = self
            .history
            .lock()
            .map_err(|err| format!("failed to aquire lock of mqtt history: {}", err))?;

        let json = self
            .selected_topic
            .as_ref()
            .and_then(|topic| history.get(topic))
            .map(|o| o.last().expect("History always has at least one entry"))
            .and_then(|value| format::payload_as_json(value.packet.payload.to_vec()));

        Ok(json)
    }

    fn change_selected_json_property(
        &mut self,
        direction: &Direction,
    ) -> Result<(), Box<dyn Error>> {
        let json = self.get_json_of_current_topic()?.unwrap_or(JsonValue::Null);
        let tree_items = json_view::root_tree_items_from_json(&json);

        let visible = flatten(&self.json_view_state.get_all_opened(), &tree_items);
        let current_identifier = self.json_view_state.selected();
        let current_index = visible
            .iter()
            .position(|o| o.identifier == current_identifier);
        let new_index = current_index.map_or(0, |current_index| {
            match direction {
                Direction::Up => current_index.saturating_sub(1),
                Direction::Down => current_index.saturating_add(1),
            }
            .min(visible.len() - 1)
        });
        let new_identifier = visible.get(new_index).unwrap().identifier.to_owned();
        self.json_view_state.select(new_identifier);
        Ok(())
    }

    pub fn on_up(&mut self) -> Result<(), Box<dyn Error>> {
        let direction = Direction::Up;
        match self.focus {
            ElementInFocus::TopicOverview => {
                self.change_selected_topic(CursorMove::Relative(direction))?;
            }
            ElementInFocus::JsonPayload => self.change_selected_json_property(&direction)?,
        }

        Ok(())
    }

    pub fn on_down(&mut self) -> Result<(), Box<dyn Error>> {
        let direction = Direction::Down;
        match self.focus {
            ElementInFocus::TopicOverview => {
                self.change_selected_topic(CursorMove::Relative(direction))?;
            }
            ElementInFocus::JsonPayload => self.change_selected_json_property(&direction)?,
        }

        Ok(())
    }

    pub fn on_right(&mut self) {
        match self.focus {
            ElementInFocus::TopicOverview => {
                if let Some(topic) = &self.selected_topic {
                    self.opened_topics.insert(topic.to_owned());
                }
            }
            ElementInFocus::JsonPayload => {
                self.json_view_state.open(self.json_view_state.selected());
            }
        }
    }

    pub fn on_left(&mut self) {
        match self.focus {
            ElementInFocus::TopicOverview => {
                if let Some(topic) = &self.selected_topic {
                    if let false = self.opened_topics.remove(topic) {
                        self.selected_topic =
                            topic::get_parent(topic).map(std::borrow::ToOwned::to_owned);
                    }
                }
            }
            ElementInFocus::JsonPayload => {
                let selected = self.json_view_state.selected();
                if !self.json_view_state.close(&selected) {
                    let (head, _) = tui_tree_widget::identifier::get_without_leaf(&selected);
                    self.json_view_state.select(head);
                }
            }
        }
    }

    pub fn on_toggle(&mut self) {
        if ElementInFocus::TopicOverview == self.focus {
            if let Some(topic) = &self.selected_topic {
                if self.opened_topics.contains(topic) {
                    self.opened_topics.remove(topic);
                } else {
                    self.opened_topics.insert(topic.to_owned());
                }
            }
        }
    }

    pub fn on_tab(&mut self) -> Result<(), Box<dyn Error>> {
        let is_json_on_topic = self.get_json_of_current_topic()?.is_some();
        self.focus = if is_json_on_topic {
            match self.focus {
                ElementInFocus::TopicOverview => ElementInFocus::JsonPayload,
                ElementInFocus::JsonPayload => ElementInFocus::TopicOverview,
            }
        } else {
            ElementInFocus::TopicOverview
        };

        Ok(())
    }

    pub fn on_click(&mut self, row: u16, _column: u16) -> Result<(), Box<dyn Error>> {
        const VIEW_OFFSET_TOP: u16 = 6;

        if self.focus == ElementInFocus::TopicOverview {
            let overview_offset = self.topic_overview_state.get_offset();

            if let Some(row_in_tree) = row.checked_sub(VIEW_OFFSET_TOP) {
                let index = overview_offset.saturating_add(row_in_tree as usize);
                let changed = self.change_selected_topic(CursorMove::Absolute(index))?;

                if !changed {
                    self.on_toggle();
                }
            }
        }

        Ok(())
    }
}
