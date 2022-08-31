use std::error::Error;
use std::thread;

use json::JsonValue;
use tui_tree_widget::{flatten, TreeState};

use crate::cli::Broker;
use crate::interactive::mqtt_thread::MqttThread;
use crate::interactive::topic_overview::TopicOverview;
use crate::interactive::ui::{CursorMove, Direction};
use crate::json_view;

pub enum ElementInFocus {
    TopicOverview,
    JsonPayload,
    CleanRetainedPopup(String),
}

pub struct App {
    pub broker: Broker,
    pub subscribe_topic: String,
    pub mqtt_thread: MqttThread,

    pub focus: ElementInFocus,
    pub json_view_state: TreeState,
    pub should_quit: bool,
    pub topic_overview: TopicOverview,
}

impl App {
    pub fn new(broker: Broker, subscribe_topic: String, mqtt_thread: MqttThread) -> Self {
        Self {
            broker,
            subscribe_topic,
            mqtt_thread,

            focus: ElementInFocus::TopicOverview,
            json_view_state: TreeState::default(),
            should_quit: false,
            topic_overview: TopicOverview::default(),
        }
    }

    fn get_json_of_current_topic(&self) -> Result<Option<JsonValue>, Box<dyn Error>> {
        if let Some(topic) = self.topic_overview.get_selected() {
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
        let new_identifier = visible.get(new_index).unwrap().identifier.clone();
        self.json_view_state.select(new_identifier);
        Ok(())
    }

    pub fn on_up(&mut self) -> Result<(), Box<dyn Error>> {
        let direction = Direction::Up;
        match self.focus {
            ElementInFocus::TopicOverview => {
                let tree_items = self.mqtt_thread.get_history()?.to_tte();
                self.topic_overview
                    .change_selected(&tree_items, CursorMove::Relative(direction));
            }
            ElementInFocus::JsonPayload => self.change_selected_json_property(&direction)?,
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }

        Ok(())
    }

    pub fn on_down(&mut self) -> Result<(), Box<dyn Error>> {
        let direction = Direction::Down;
        match self.focus {
            ElementInFocus::TopicOverview => {
                let tree_items = self.mqtt_thread.get_history()?.to_tte();
                self.topic_overview
                    .change_selected(&tree_items, CursorMove::Relative(direction));
            }
            ElementInFocus::JsonPayload => self.change_selected_json_property(&direction)?,
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }

        Ok(())
    }

    pub fn on_right(&mut self) {
        match self.focus {
            ElementInFocus::TopicOverview => {
                self.topic_overview.open();
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
                self.topic_overview.close();
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
                self.topic_overview.toggle();
            }
            ElementInFocus::JsonPayload => {}
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

    pub fn on_click(&mut self, row: u16, _column: u16) -> Result<(), Box<dyn Error>> {
        const VIEW_OFFSET_TOP: u16 = 6;

        if matches!(self.focus, ElementInFocus::TopicOverview) {
            let overview_offset = self.topic_overview.state.get_offset();

            if let Some(row_in_tree) = row.checked_sub(VIEW_OFFSET_TOP) {
                let index = overview_offset.saturating_add(row_in_tree as usize);
                let tree_items = self.mqtt_thread.get_history()?.to_tte();
                let changed = self
                    .topic_overview
                    .change_selected(&tree_items, CursorMove::Absolute(index));

                if !changed {
                    self.on_confirm()?;
                }
            }
        }
        Ok(())
    }

    pub fn on_delete(&mut self) {
        if matches!(self.focus, ElementInFocus::TopicOverview) {
            if let Some(topic) = self.topic_overview.get_selected() {
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
