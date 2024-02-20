use std::cmp::min;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List};
use ratatui::Frame;
use tui_tree_widget::{Tree, TreeState};

use crate::interactive::details::json_selector::JsonSelector;
use crate::interactive::details::tree_items_from_json;
use crate::interactive::ui::{focus_color, get_row_inside, split_area_vertically};
use crate::mqtt::{HistoryEntry, Payload};

#[derive(Default)]
pub struct PayloadView {
    pub json_state: TreeState<JsonSelector>,
    pub last_area: Rect,
}

impl PayloadView {
    pub fn draw(
        &mut self,
        f: &mut Frame,
        area: Rect,
        topic_history: &[HistoryEntry],
        has_focus: bool,
    ) -> Rect {
        let last = topic_history
            .last()
            .expect("when Details are drawn they should always have at least one HistoryEntry");
        let size = last.payload_size;
        match &last.payload {
            Payload::Json(json) => self.draw_json(f, area, size, json, has_focus),
            Payload::MessagePack(messagepack) => {
                self.draw_messagepack(f, area, size, messagepack, has_focus)
            }
            Payload::NotUtf8(err) => self.draw_string(f, area, size, &err.to_string()),
            Payload::String(str) => self.draw_string(f, area, size, str),
        }
    }

    pub fn json_index_of_click(&self, column: u16, row: u16) -> Option<usize> {
        get_row_inside(self.last_area, column, row).map(|index| {
            let offset = self.json_state.get_offset();
            (index as usize) + offset
        })
    }

    fn areas(&mut self, area: Rect, content_height: usize) -> (Rect, Rect) {
        let max_payload_height = area.height / 3;
        #[allow(clippy::cast_possible_truncation)]
        let payload_height = min(
            max_payload_height as usize,
            content_height.saturating_add(2),
        ) as u16;
        let (payload_area, remaining_area) = split_area_vertically(area, payload_height);
        self.last_area = payload_area;
        (payload_area, remaining_area)
    }

    fn draw_json(
        &mut self,
        f: &mut Frame,
        area: Rect,
        payload_bytes: usize,
        json: &serde_json::Value,
        has_focus: bool,
    ) -> Rect {
        let title = format!("JSON Payload (Bytes: {payload_bytes})");
        let items = tree_items_from_json(json);

        let visible = self.json_state.flatten(&items);
        let content_height = visible.into_iter().map(|o| o.item.height()).sum::<usize>();
        let (payload_area, remaining_area) = self.areas(area, content_height);

        let focus_color = focus_color(has_focus);
        let widget = Tree::new(items)
            .unwrap()
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::bordered()
                    .border_style(Style::new().fg(focus_color))
                    .title(title),
            );
        f.render_stateful_widget(widget, payload_area, &mut self.json_state);
        remaining_area
    }

    fn draw_messagepack(
        &mut self,
        f: &mut Frame,
        area: Rect,
        payload_bytes: usize,
        messagepack: &rmpv::Value,
        has_focus: bool,
    ) -> Rect {
        let title = format!("MessagePack Payload (Bytes: {payload_bytes})");

        let messagepack_string = messagepack.to_string();
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&messagepack_string) else {
            // TODO: use raw messagepack implementation?
            return self.draw_string(f, area, payload_bytes, &messagepack_string);
        };

        let items = tree_items_from_json(&json);

        let visible = self.json_state.flatten(&items);
        let content_height = visible.into_iter().map(|o| o.item.height()).sum::<usize>();
        let (payload_area, remaining_area) = self.areas(area, content_height);

        let focus_color = focus_color(has_focus);
        let widget = Tree::new(items)
            .unwrap()
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::bordered()
                    .border_style(Style::new().fg(focus_color))
                    .title(title),
            );
        f.render_stateful_widget(widget, payload_area, &mut self.json_state);
        remaining_area
    }

    fn draw_string(
        &mut self,
        f: &mut Frame,
        area: Rect,
        payload_bytes: usize,
        payload: &str,
    ) -> Rect {
        let title = format!("Payload (Bytes: {payload_bytes})");
        let items = payload.lines().collect::<Vec<_>>();

        let (payload_area, remaining_area) = self.areas(area, items.len());

        let widget = List::new(items).block(Block::bordered().title(title));
        f.render_widget(widget, payload_area);
        remaining_area
    }
}
