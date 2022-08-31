use json::JsonValue;

use crate::cli::Broker;
use crate::interactive::details::Details;
use crate::interactive::info_header::InfoHeader;
use crate::interactive::mqtt_thread::MqttThread;
use crate::interactive::topic_overview::TopicOverview;
use crate::interactive::ui::CursorMove;
use crate::json_view::root_tree_items_from_json;

pub enum ElementInFocus {
    TopicOverview,
    JsonPayload,
    CleanRetainedPopup(String),
}

pub struct App {
    pub details: Details,
    pub focus: ElementInFocus,
    pub info_header: InfoHeader,
    pub mqtt_thread: MqttThread,
    pub topic_overview: TopicOverview,
}

impl App {
    pub fn new(broker: &Broker, subscribe_topic: &str, mqtt_thread: MqttThread) -> Self {
        Self {
            details: Details::default(),
            focus: ElementInFocus::TopicOverview,
            info_header: InfoHeader::new(broker, subscribe_topic),
            mqtt_thread,
            topic_overview: TopicOverview::default(),
        }
    }

    fn get_json_of_current_topic(&self) -> anyhow::Result<Option<JsonValue>> {
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

    pub fn on_up(&mut self) -> anyhow::Result<()> {
        const DIRECTION: CursorMove = CursorMove::RelativeUp;
        match self.focus {
            ElementInFocus::TopicOverview => {
                let tree_items = self.mqtt_thread.get_history()?.to_tte();
                self.topic_overview.change_selected(&tree_items, DIRECTION);
            }
            ElementInFocus::JsonPayload => {
                let json = self.get_json_of_current_topic()?.unwrap_or(JsonValue::Null);
                let items = root_tree_items_from_json(&json);
                self.details.json_view.key_up(&items);
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }

        Ok(())
    }

    pub fn on_down(&mut self) -> anyhow::Result<()> {
        const DIRECTION: CursorMove = CursorMove::RelativeDown;
        match self.focus {
            ElementInFocus::TopicOverview => {
                let tree_items = self.mqtt_thread.get_history()?.to_tte();
                self.topic_overview.change_selected(&tree_items, DIRECTION);
            }
            ElementInFocus::JsonPayload => {
                let json = self.get_json_of_current_topic()?.unwrap_or(JsonValue::Null);
                let items = root_tree_items_from_json(&json);
                self.details.json_view.key_down(&items);
            }
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
                self.details.json_view.key_right();
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
                self.details.json_view.key_left();
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }
    }

    pub fn on_confirm(&mut self) -> anyhow::Result<()> {
        match &self.focus {
            ElementInFocus::TopicOverview => {
                self.topic_overview.toggle();
            }
            ElementInFocus::JsonPayload => {
                self.details.json_view.toggle_selected();
            }
            ElementInFocus::CleanRetainedPopup(topic) => {
                let base = self.mqtt_thread.get_mqtt_options();
                super::clear_retained::do_clear(base, topic)?;
                self.focus = ElementInFocus::TopicOverview;
            }
        }
        Ok(())
    }

    pub fn on_tab(&mut self) -> anyhow::Result<()> {
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

    pub fn on_click(&mut self, row: u16, _column: u16) -> anyhow::Result<()> {
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
