use std::fmt::Write as _;

use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType};
use ratatui::Frame;
use ratatui_logline_table::{State as TableState, Table};

use crate::format;
use crate::interactive::ui::{focus_color, BORDERS_TOP_RIGHT, STYLE_BOLD};
use crate::mqtt::HistoryEntry;
use crate::payload::{JsonSelector, Payload};

#[allow(clippy::cast_precision_loss)]
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    topic_history: &[HistoryEntry],
    binary_address: Option<usize>,
    json_selector: &[JsonSelector],
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
    let focus_color = focus_color(has_focus);
    let json_selector = json_selector.to_vec();

    let table = Table::new(
        topic_history,
        [
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Percentage(100),
        ],
        move |index, entry| {
            let time = entry.time.to_string();
            let qos = format::qos(entry.qos).to_owned();
            let value = match &entry.payload {
                Payload::Binary(data) => binary_address
                    .and_then(|address| data.get(address).copied())
                    .map_or_else(|| format!("{data:?}"), |data| format!("{data}")),
                Payload::Json(json) => JsonSelector::get_json(json, &json_selector)
                    .unwrap_or(json)
                    .to_string(),
                Payload::MessagePack(messagepack) => {
                    JsonSelector::get_messagepack(messagepack, &json_selector)
                        .unwrap_or(messagepack)
                        .to_string()
                }
                Payload::String(str) => str.to_string(),
            };

            if index == last_index {
                [
                    Line::styled(time, STYLE_BOLD),
                    Line::styled(qos, STYLE_BOLD),
                    Line::styled(value, STYLE_BOLD),
                ]
            } else {
                [Line::raw(time), Line::raw(qos), Line::raw(value)]
            }
        },
    )
    .header([Line::raw("Time"), Line::raw("QoS"), Line::raw("Value")])
    .header_style(STYLE_BOLD)
    .row_highlight_style(Style::new().fg(Color::Black).bg(focus_color))
    .block(
        Block::new()
            .border_type(BorderType::Rounded)
            .borders(BORDERS_TOP_RIGHT)
            .title_alignment(Alignment::Center)
            .border_style(Style::new().fg(focus_color))
            .title(title),
    );
    frame.render_stateful_widget(table, area, state);
}
