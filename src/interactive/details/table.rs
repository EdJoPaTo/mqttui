use std::fmt::Write;

use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{
    Block, BorderType, Row, ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use ratatui::Frame;
use tui_tree_widget::third_party::messagepack;
use tui_tree_widget::KeyValueTreeItem;

use super::payload_view::PayloadView;
use crate::format;
use crate::interactive::ui::{focus_color, BORDERS_TOP_RIGHT, STYLE_BOLD};
use crate::mqtt::HistoryEntry;
use crate::payload::Payload;

#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    topic_history: &[HistoryEntry],
    payload_view: &PayloadView,
    state: &mut TableState,
    has_focus: bool,
) {
    let mut title = format!("History ({}", topic_history.len());

    {
        let without_retain = topic_history
            .iter()
            .filter_map(|entry| entry.time.as_optional())
            .collect::<Box<[_]>>();
        if let [first, .., last] = *without_retain {
            let seconds = (*last - *first)
                .to_std()
                .expect("later message should be after earlier message")
                .as_secs_f64();
            let every_n_seconds = seconds / without_retain.len().saturating_sub(1) as f64;
            if every_n_seconds < 1.0 {
                let messages_per_second = 1.0 / every_n_seconds;
                write!(title, ", ~{messages_per_second:.1} per second")
            } else if every_n_seconds < 100.0 {
                write!(title, ", every ~{every_n_seconds:.1} seconds")
            } else {
                let every_n_minutes = every_n_seconds / 60.0;
                write!(title, ", every ~{every_n_minutes:.1} minutes")
            }
            .expect("write to string should never fail");
        }
    }
    title += ")";

    let last_index = topic_history.len().saturating_sub(1);
    let rows = topic_history.iter().enumerate().map(|(index, entry)| {
        let time = entry.time.to_string();
        let qos = format::qos(entry.qos).to_owned();
        let value = match &entry.payload {
            Payload::Binary(data) => payload_view
                .binary_state
                .selected_address()
                .and_then(|address| data.get(address).copied())
                .map_or_else(|| format!("{data:?}"), |data| format!("{data}")),
            Payload::Json(json) => payload_view
                .json_state
                .selected()
                .and_then(|selector| json.get_value_deep(selector))
                .unwrap_or(json)
                .to_string(),
            Payload::MessagePack(messagepack) => payload_view
                .indexed_tree_state
                .selected()
                .and_then(|selector| messagepack::get_value(messagepack, selector))
                .unwrap_or(messagepack)
                .to_string(),
            Payload::String(str) => str.to_string(),
        };
        let row = Row::new(vec![time, qos, value]);
        if index == last_index {
            row.style(STYLE_BOLD)
        } else {
            row
        }
    });

    let focus_color = focus_color(has_focus);

    let mut table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Percentage(100),
        ],
    )
    .header(Row::new(["Time", "QoS", "Value"]).style(STYLE_BOLD))
    .block(
        Block::new()
            .border_type(BorderType::Rounded)
            .borders(BORDERS_TOP_RIGHT)
            .title_alignment(Alignment::Center)
            .border_style(Style::new().fg(focus_color))
            .title(title),
    );

    // Ensure selection is possible
    if let Some(selection) = state.selected_mut() {
        *selection = (*selection).min(topic_history.len().saturating_sub(1));
    }

    // Scroll down offset as much as possible
    let height = area.height.saturating_sub(2); // remove block and title
    let offset_with_last_in_view = topic_history.len().saturating_sub(height as usize);
    if let Some(selection) = state.selected() {
        // Only scroll when the change will include both end and selection.
        // When the user manually scrolled away from the end keep the offset.
        if selection >= offset_with_last_in_view {
            *state.offset_mut() = offset_with_last_in_view;
        }
    } else {
        *state.offset_mut() = offset_with_last_in_view;
    }

    // Workaround selection, see https://github.com/ratatui-org/ratatui/issues/174
    if state.selected().is_none() {
        let mut state = TableState::new().with_selected(Some(topic_history.len() - 1));
        frame.render_stateful_widget(table, area, &mut state);
    } else {
        table = table.highlight_style(Style::new().fg(Color::Black).bg(focus_color));
        frame.render_stateful_widget(table, area, state);
    }

    {
        let scrollbar = ratatui::widgets::Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None);
        // Work around overscroll by removing height from total
        let mut scrollbar_state =
            ScrollbarState::new(topic_history.len().saturating_sub(usize::from(height)))
                .position(state.offset())
                .viewport_content_length(height as usize);
        let scrollbar_area = Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(2),
            ..area
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}
