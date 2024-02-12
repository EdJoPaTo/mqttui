use std::cmp::min;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, List};
use ratatui::Frame;
use tui_tree_widget::{Tree, TreeState};

use crate::interactive::details::json_selector::JsonSelector;
use crate::interactive::details::json_view::root_tree_items_from_json;
use crate::interactive::ui::{focus_color, get_row_inside, split_area_vertically};
use crate::mqtt::{HistoryEntry, Payload};

mod history;
mod json_selector;
pub mod json_view;

#[derive(Default)]
pub struct Details {
    pub json_state: TreeState<JsonSelector>,
    pub last_json_area: Option<Rect>,
}

impl Details {
    pub fn draw(
        &mut self,
        f: &mut Frame,
        area: Rect,
        topic_history: &[HistoryEntry],
        json_payload_has_focus: bool,
    ) {
        self.last_json_area = None;

        let last = topic_history
            .last()
            .expect("when Details are drawn they should always have at least one HistoryEntry");
        let size = last.payload_size;
        let history_area = match &last.payload {
            Payload::Json(json) => {
                self.draw_payload_json(f, area, size, json, json_payload_has_focus)
            },
            Payload::MsgPack(_, json) => {
                self.draw_payload_msgpack(f, area, size, json, json_payload_has_focus)
            }
            Payload::NotUtf8(err) => draw_payload_string(f, area, size, &err.to_string()),
            Payload::String(str) => draw_payload_string(f, area, size, str),
        };

        history::draw(f, history_area, topic_history, &self.json_state.selected());
    }

    pub fn json_index_of_click(&mut self, column: u16, row: u16) -> Option<usize> {
        if let Some(index) = self
            .last_json_area
            .and_then(|area| get_row_inside(area, column, row))
        {
            let offset = self.json_state.get_offset();
            let new_index = (index as usize) + offset;
            Some(new_index)
        } else {
            None
        }
    }

    fn draw_payload_json(
        &mut self,
        f: &mut Frame,
        area: Rect,
        bytes: usize,
        json: &serde_json::Value,
        has_focus: bool,
    ) -> Rect {
        let title = format!("JSON Payload (Bytes: {bytes})");
        let items = root_tree_items_from_json(json);

        let visible = self.json_state.flatten(&items);
        let content_height = visible.into_iter().map(|o| o.item.height()).sum::<usize>();
        let max_payload_height = area.height / 3;
        #[allow(clippy::cast_possible_truncation)]
        let payload_height = min(max_payload_height as usize, 2 + content_height) as u16;
        let (payload_area, remaining_area) = split_area_vertically(area, payload_height);
        self.last_json_area = Some(payload_area);

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

    fn draw_payload_msgpack(
        &mut self,
        f: &mut Frame,
        area: Rect,
        bytes: usize,
        json: &serde_json::Value,
        has_focus: bool,
    ) -> Rect {
        let title = format!("MessagePack Payload (Bytes: {bytes})");
        let items = root_tree_items_from_json(json);

        let visible = self.json_state.flatten(&items);
        let content_height = visible.into_iter().map(|o| o.item.height()).sum::<usize>();
        let max_payload_height = area.height / 3;
        #[allow(clippy::cast_possible_truncation)]
        let payload_height = min(max_payload_height as usize, 2 + content_height) as u16;
        let (payload_area, remaining_area) = split_area_vertically(area, payload_height);
        self.last_json_area = Some(payload_area);

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
}

/// Returns remaining rect to be used for history
fn draw_payload_string(f: &mut Frame, area: Rect, payload_bytes: usize, payload: &str) -> Rect {
    let title = format!("Payload (Bytes: {payload_bytes})");
    let items = payload.lines().collect::<Vec<_>>();

    let max_payload_height = area.height / 3;
    #[allow(clippy::cast_possible_truncation)]
    let payload_height = min(max_payload_height as usize, 2 + items.len()) as u16;
    let (payload_area, remaining_area) = split_area_vertically(area, payload_height);

    let widget = List::new(items).block(Block::bordered().title(title));
    f.render_widget(widget, payload_area);
    remaining_area
}
