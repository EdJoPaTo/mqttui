use std::collections::HashSet;

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::interactive::mqtt_history::MqttHistory;
use crate::interactive::ui::{focus_color, get_row_inside, CursorMove};
use crate::mqtt::topic::get_parent;

#[derive(Default)]
pub struct TopicOverview {
    last_area: Rect,
    opened_topics: HashSet<String>,
    selected_topic: Option<String>,
    state: TreeState,
}

impl TopicOverview {
    pub const fn get_opened(&self) -> &HashSet<String> {
        &self.opened_topics
    }

    pub const fn get_selected(&self) -> &Option<String> {
        &self.selected_topic
    }

    pub fn ensure_state(&mut self, history: &MqttHistory) {
        self.state.close_all();
        for topic in &self.opened_topics {
            self.state
                .open(history.get_tree_identifier(topic).unwrap_or_default());
        }

        // Ensure selected topic is selected index
        self.state.select(
            self.selected_topic
                .as_ref()
                .and_then(|selected_topic| history.get_tree_identifier(selected_topic))
                .unwrap_or_default(),
        );
    }

    pub fn draw(
        &mut self,
        f: &mut Frame,
        area: Rect,
        topic_amount: usize,
        tree_items: Vec<TreeItem>,
        has_focus: bool,
    ) {
        let title = format!("Topics ({topic_amount})");
        let focus_color = focus_color(has_focus);
        let widget = Tree::new(tree_items)
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .border_style(Style::new().fg(focus_color))
                    .title_alignment(Alignment::Center)
                    .title(title),
            );
        f.render_stateful_widget(widget, area, &mut self.state);
        self.last_area = area;
    }

    pub fn change_selected(&mut self, visible: &[String], cursor_move: CursorMove) -> bool {
        let current_index = self
            .selected_topic
            .as_ref()
            .and_then(|selected_topic| visible.iter().position(|o| o == selected_topic));
        let page_jump = (self.last_area.height / 2) as usize;
        let new_index = match cursor_move {
            CursorMove::Absolute(index) => index,
            CursorMove::OneUp => current_index.map_or(usize::MAX, |i| i.saturating_sub(1)),
            CursorMove::OneDown => current_index.map_or(0, |i| i.saturating_add(1)),
            CursorMove::PageUp => current_index.map_or(usize::MAX, |i| i.saturating_sub(page_jump)),
            CursorMove::PageDown => current_index.map_or(0, |i| i.saturating_add(page_jump)),
        }
        .min(visible.len().saturating_sub(1));

        let next_selected_topic = visible.get(new_index).cloned();
        let different = self.selected_topic != next_selected_topic;
        self.selected_topic = next_selected_topic;
        different
    }

    pub fn open(&mut self) {
        if let Some(topic) = &self.selected_topic {
            self.opened_topics.insert(topic.clone());
        }
    }

    pub fn close(&mut self) {
        if let Some(topic) = &self.selected_topic {
            if !self.opened_topics.remove(topic) {
                self.selected_topic = get_parent(topic).map(std::borrow::ToOwned::to_owned);
            }
        }
    }

    pub fn toggle(&mut self) {
        if let Some(topic) = &self.selected_topic {
            if self.opened_topics.contains(topic) {
                self.opened_topics.remove(topic);
            } else {
                self.opened_topics.insert(topic.to_string());
            }
        }
    }

    pub fn index_of_click(&mut self, column: u16, row: u16) -> Option<usize> {
        if let Some(index) = get_row_inside(self.last_area, column, row) {
            let offset = self.state.get_offset();
            let new_index = (index as usize) + offset;
            Some(new_index)
        } else {
            None
        }
    }
}
