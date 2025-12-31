use std::cmp::min;

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation};
use ratatui_binary_data_widget::{BinaryDataWidget, BinaryDataWidgetState};
use tui_tree_widget::{Tree, TreeState};

use crate::interactive::ui::{BORDERS_TOP_RIGHT, focus_color, split_area_vertically};
use crate::mqtt::HistoryEntry;
use crate::payload::{JsonSelector, Payload, tree_items_from_json, tree_items_from_messagepack};

#[derive(Default)]
pub struct PayloadView {
    pub binary_state: BinaryDataWidgetState,
    pub json_state: TreeState<JsonSelector>,
    pub last_area: Rect,
}

impl PayloadView {
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        has_focus: bool,
        entry: &HistoryEntry,
    ) -> Rect {
        let size = entry.payload_size;
        match &entry.payload {
            Payload::Binary(data) => self.draw_binary(frame, area, has_focus, size, data),
            Payload::Json(json) => self.draw_json(frame, area, has_focus, size, json),
            Payload::MessagePack(messagepack) => {
                self.draw_messagepack(frame, area, has_focus, size, messagepack)
            }
            Payload::String(str) => self.draw_string(frame, area, has_focus, size, str),
        }
    }

    fn areas(&mut self, area: Rect, has_focus: bool, content_height: usize) -> (Rect, Rect) {
        let max_payload_height = if has_focus {
            area.height.saturating_mul(2) / 3
        } else {
            area.height / 3
        };
        #[expect(clippy::cast_possible_truncation)]
        let payload_height = min(
            max_payload_height as usize,
            content_height.saturating_add(2),
        ) as u16;
        let (payload_area, remaining_area) = split_area_vertically(area, payload_height);
        self.last_area = payload_area;
        (payload_area, remaining_area)
    }

    fn draw_binary(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        has_focus: bool,
        payload_bytes: usize,
        data: &[u8],
    ) -> Rect {
        let title = format!("Binary Payload (Bytes: {payload_bytes})");

        let focus_color = focus_color(has_focus);
        let widget = BinaryDataWidget::new(data)
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::new()
                    .border_type(BorderType::Rounded)
                    .borders(BORDERS_TOP_RIGHT)
                    .title_alignment(Alignment::Center)
                    .border_style(Style::new().fg(focus_color))
                    .title(title),
            );

        let max_lines = widget.get_max_lines_of_data_in_area(area);
        let (payload_area, remaining_area) = self.areas(area, has_focus, max_lines);

        frame.render_stateful_widget(widget, payload_area, &mut self.binary_state);
        remaining_area
    }

    fn draw_json(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        has_focus: bool,
        payload_bytes: usize,
        json: &serde_json::Value,
    ) -> Rect {
        let title = format!("JSON Payload (Bytes: {payload_bytes})");
        let items = tree_items_from_json(json);

        let visible = self.json_state.flatten(&items);
        let content_height = visible
            .into_iter()
            .map(|flattened| flattened.item.height())
            .sum::<usize>();
        let (payload_area, remaining_area) = self.areas(area, has_focus, content_height);

        let focus_color = focus_color(has_focus);
        let widget = Tree::new(&items)
            .unwrap()
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None),
            ))
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::new()
                    .border_type(BorderType::Rounded)
                    .borders(BORDERS_TOP_RIGHT)
                    .title_alignment(Alignment::Center)
                    .border_style(Style::new().fg(focus_color))
                    .title(title),
            );
        frame.render_stateful_widget(widget, payload_area, &mut self.json_state);
        remaining_area
    }

    fn draw_messagepack(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        has_focus: bool,
        payload_bytes: usize,
        messagepack: &rmpv::Value,
    ) -> Rect {
        let title = format!("MessagePack Payload (Bytes: {payload_bytes})");
        let items = tree_items_from_messagepack(messagepack);

        let visible = self.json_state.flatten(&items);
        let content_height = visible
            .into_iter()
            .map(|flattened| flattened.item.height())
            .sum::<usize>();
        let (payload_area, remaining_area) = self.areas(area, has_focus, content_height);

        let focus_color = focus_color(has_focus);
        let widget = Tree::new(&items)
            .unwrap()
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None),
            ))
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::new()
                    .border_type(BorderType::Rounded)
                    .borders(BORDERS_TOP_RIGHT)
                    .title_alignment(Alignment::Center)
                    .border_style(Style::new().fg(focus_color))
                    .title(title),
            );
        frame.render_stateful_widget(widget, payload_area, &mut self.json_state);
        remaining_area
    }

    fn draw_string(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        has_focus: bool,
        payload_bytes: usize,
        payload: &str,
    ) -> Rect {
        let title = format!("Payload (Bytes: {payload_bytes})");
        let text = Text::from(payload);
        let (payload_area, remaining_area) = self.areas(area, has_focus, text.height());
        let widget = Paragraph::new(text).block(
            Block::new()
                .border_type(BorderType::Rounded)
                .borders(BORDERS_TOP_RIGHT)
                .title_alignment(Alignment::Center)
                .title(title),
        );
        frame.render_widget(widget, payload_area);
        remaining_area
    }
}
