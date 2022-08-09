use std::collections::HashSet;
use std::error::Error;
use std::thread;

use json::JsonValue;
use tui_tree_widget::{flatten, TreeState};
use url::Url;

use crate::interactive::mqtt_thread::MqttThread;
use crate::interactive::topic_tree_entry::get_visible;
use crate::{json_view, topic};

pub enum ElementInFocus {
    TopicOverview,
    JsonPayload,
    CleanRetainedPopup(String),
}

enum Direction {
    Up,
    Down,
}

enum CursorMove {
    Absolute(usize),
    Relative(Direction),
}

pub struct App {
    pub display_broker: Url,
    pub subscribe_topic: String,
    pub mqtt_thread: MqttThread,

    pub focus: ElementInFocus,
    pub json_view_state: TreeState,
    pub opened_topics: HashSet<String>,
    pub selected_topic: Option<String>,
    pub should_quit: bool,
    pub topic_overview_state: TreeState,

    last_index_clicked: Option<usize>,
    pub overview_area_width: u16,
}

impl App {
    pub fn new(display_broker: Url, subscribe_topic: String, mqtt_thread: MqttThread) -> Self {
        Self {
            display_broker,
            subscribe_topic,
            mqtt_thread,

            focus: ElementInFocus::TopicOverview,
            json_view_state: TreeState::default(),
            opened_topics: HashSet::new(),
            selected_topic: None,
            should_quit: false,
            topic_overview_state: TreeState::default(),
            last_index_clicked: None,

            overview_area_width: 0,
        }
    }

    fn change_selected_topic(&mut self, cursor_move: CursorMove) -> Result<bool, Box<dyn Error>> {
        let tmlp_tree = self.mqtt_thread.get_history()?.to_tte();
        let visible = get_visible(&self.opened_topics, &tmlp_tree);

        let current_index = self
            .selected_topic
            .as_ref()
            .and_then(|selected_topic| visible.iter().position(|o| &o.topic == selected_topic));
        let new_index = match cursor_move {
            CursorMove::Absolute(index) => index,
            CursorMove::Relative(direction) => current_index.map_or_else(
                || match direction {
                    Direction::Down => 0,
                    Direction::Up => usize::MAX,
                },
                |current_index| match direction {
                    Direction::Up => current_index.overflowing_sub(1).0,
                    Direction::Down => current_index.saturating_add(1) % visible.len(),
                },
            ),
        }
        .min(visible.len().saturating_sub(1));

        let next_selected_topic = visible.get(new_index).map(|o| o.topic.clone());
        let different = self.selected_topic != next_selected_topic;
        self.selected_topic = next_selected_topic;
        Ok(different)
    }

    fn get_json_of_current_topic(&self) -> Result<Option<JsonValue>, Box<dyn Error>> {
        if let Some(topic) = &self.selected_topic {
            let json = self
                .mqtt_thread
                .get_history()?
                .get_last(topic)
                .and_then(|last| last.payload.as_optional_json().cloned());
            Ok(json)
        } else {
            Ok(None)
        }
    }

    fn change_selected_json_property(
        &mut self,
        cursor_move: CursorMove,
    ) -> Result<bool, Box<dyn Error>> {
        let json = self.get_json_of_current_topic()?.unwrap_or(JsonValue::Null);
        let tree_items = json_view::root_tree_items_from_json(&json);

        let visible = flatten(&self.json_view_state.get_all_opened(), &tree_items);
        let current_identifier = self.json_view_state.selected();
        let current_index = visible
            .iter()
            .position(|o| o.identifier == current_identifier);

        let new_index = match cursor_move {
            CursorMove::Relative(direction) => current_index.map_or(0, |current_index| {
                match direction {
                    Direction::Up => current_index.saturating_sub(1),
                    Direction::Down => current_index.saturating_add(1),
                }
                .min(visible.len() - 1)
            }),
            CursorMove::Absolute(index) => index.min(visible.len() - 1),
        };
        let changed = Some(new_index) != self.last_index_clicked;
        self.last_index_clicked = Some(new_index);

        let new_identifier = visible.get(new_index).unwrap().identifier.clone();
        self.json_view_state.select(new_identifier);
        Ok(changed)
    }

    pub fn on_up(&mut self) -> Result<(), Box<dyn Error>> {
        let direction = Direction::Up;
        match self.focus {
            ElementInFocus::TopicOverview => {
                self.change_selected_topic(CursorMove::Relative(direction))?;
            }
            ElementInFocus::JsonPayload => {
                self.change_selected_json_property(CursorMove::Relative(direction))?;
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }

        Ok(())
    }

    pub fn on_down(&mut self) -> Result<(), Box<dyn Error>> {
        let direction = Direction::Down;
        match self.focus {
            ElementInFocus::TopicOverview => {
                self.change_selected_topic(CursorMove::Relative(direction))?;
            }
            ElementInFocus::JsonPayload => {
                self.change_selected_json_property(CursorMove::Relative(direction))?;
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }

        Ok(())
    }

    pub fn on_right(&mut self) {
        match self.focus {
            ElementInFocus::TopicOverview => {
                if let Some(topic) = &self.selected_topic {
                    self.opened_topics.insert(topic.clone());
                }
            }
            ElementInFocus::JsonPayload => {
                self.json_view_state.open(self.json_view_state.selected());
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }
    }

    pub fn on_left(&mut self) {
        match self.focus {
            ElementInFocus::TopicOverview => {
                if let Some(topic) = &self.selected_topic {
                    if !self.opened_topics.remove(topic) {
                        self.selected_topic =
                            topic::get_parent(topic).map(std::borrow::ToOwned::to_owned);
                    }
                }
            }
            ElementInFocus::JsonPayload => {
                let selected = self.json_view_state.selected();
                if !self.json_view_state.close(&selected) {
                    let (head, _) = tui_tree_widget::get_identifier_without_leaf(&selected);
                    self.json_view_state.select(head);
                }
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }
    }

    pub fn on_confirm(&mut self) -> Result<(), Box<dyn Error>> {
        match &self.focus {
            ElementInFocus::TopicOverview => {
                if let Some(topic) = &self.selected_topic {
                    if self.opened_topics.contains(topic) {
                        self.opened_topics.remove(topic);
                    } else {
                        self.opened_topics.insert(topic.clone());
                    }
                }
            }
            ElementInFocus::JsonPayload => {
                self.json_view_state.toggle();
            }
            ElementInFocus::CleanRetainedPopup(topic) => {
                let base = self.mqtt_thread.get_mqtt_options();

                let client_id = format!("mqttui-clean-{:x}", rand::random::<u32>());

                let (host, port) = base.broker_address();
                let mut options = rumqttc::MqttOptions::new(client_id, host, port);
                if let Some((username, password)) = base.credentials() {
                    options.set_credentials(username, password);
                }

                let (mut client, connection) = rumqttc::Client::new(options, 100);
                client.subscribe(topic, rumqttc::QoS::AtLeastOnce)?;
                client.subscribe(format!("{}/#", topic), rumqttc::QoS::AtLeastOnce)?;

                thread::Builder::new()
                    .name(format!("clean retained {}", topic))
                    .spawn(move || {
                        crate::clean_retained::clean_retained(
                            client,
                            connection,
                            crate::clean_retained::Mode::Silent,
                        );
                    })?;

                self.focus = ElementInFocus::TopicOverview;
            }
        }
        Ok(())
    }

    pub fn on_tab(&mut self) -> Result<(), Box<dyn Error>> {
        let is_json_on_topic = self.get_json_of_current_topic()?.is_some();
        self.focus = if is_json_on_topic {
            match self.focus {
                ElementInFocus::TopicOverview => ElementInFocus::JsonPayload,
                ElementInFocus::JsonPayload | ElementInFocus::CleanRetainedPopup(_) => {
                    ElementInFocus::TopicOverview
                }
            }
        } else {
            ElementInFocus::TopicOverview
        };
        Ok(())
    }

    pub fn on_click(&mut self, row: u16, column: u16) -> Result<(), Box<dyn Error>> {
        const VIEW_OFFSET_TOP: u16 = 6;

        // allow for switching columns with click, as long as not clicking on the overview bar
        if row > VIEW_OFFSET_TOP {
            // the area width is set in draw logic
            if column > self.overview_area_width {
                // This might be redundant, since the area width is calculated after this in ui logic...
                let is_json_on_topic = self.get_json_of_current_topic()?.is_some();
                if is_json_on_topic {
                    if let ElementInFocus::TopicOverview = self.focus {
                        self.focus = ElementInFocus::JsonPayload;
                    }
                }
            } else {
                if let ElementInFocus::JsonPayload = self.focus {
                    self.focus = ElementInFocus::TopicOverview;
                }
            }
        }

        match self.focus {
            ElementInFocus::TopicOverview => {
                let overview_offset = self.topic_overview_state.get_offset();
                if let Some(row_in_tree) = row.checked_sub(VIEW_OFFSET_TOP) {
                    let index = overview_offset.saturating_add(row_in_tree as usize);
                    let changed = self.change_selected_topic(CursorMove::Absolute(index))?;
                    if !changed {
                        self.on_confirm()?;
                    }
                }
            }
            ElementInFocus::JsonPayload => {
                let jsonpayload_offset = self.json_view_state.get_offset();
                if let Some(row_in_tree) = row.checked_sub(VIEW_OFFSET_TOP) {
                    let index = jsonpayload_offset.saturating_add(row_in_tree as usize);
                    let changed =
                        self.change_selected_json_property(CursorMove::Absolute(index))?;
                    if !changed {
                        self.on_confirm()?;
                    }
                }
            }
            _ => {}
        };

        Ok(())
    }

    pub fn on_delete(&mut self) {
        if matches!(self.focus, ElementInFocus::TopicOverview) {
            if let Some(topic) = &self.selected_topic {
                self.focus = ElementInFocus::CleanRetainedPopup(topic.to_string());
            }
        }
    }

    pub fn on_other(&mut self) {
        if matches!(self.focus, ElementInFocus::CleanRetainedPopup(_)) {
            self.focus = ElementInFocus::TopicOverview;
        }
    }
}
