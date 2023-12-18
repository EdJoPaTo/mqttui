use std::fmt::Write;

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row, Table, TableState};
use ratatui::{symbols, Frame};

use crate::format;
use crate::interactive::details::json_selector::JsonSelector;
use crate::interactive::ui::{split_area_vertically, STYLE_BOLD};
use crate::mqtt::{HistoryEntry, Payload, Time};
use graph_data::GraphData;

mod graph_data;

pub fn draw(
    f: &mut Frame,
    area: Rect,
    topic_history: &[HistoryEntry],
    json_selector: &[JsonSelector],
) {
    let table_area = GraphData::parse(topic_history, json_selector).map_or(area, |data| {
        let (table_area, graph_area) = split_area_vertically(area, area.height / 2);
        draw_graph(f, graph_area, &data);
        table_area
    });
    draw_table(f, table_area, topic_history, json_selector);
}

#[allow(clippy::cast_precision_loss)]
fn draw_table(
    f: &mut Frame,
    area: Rect,
    topic_history: &[HistoryEntry],
    json_selector: &[JsonSelector],
) {
    let mut title = format!("History ({}", topic_history.len());

    let last_index = topic_history.len().saturating_sub(1);

    let without_retain = topic_history
        .iter()
        .filter(|o| !matches!(o.time, Time::Retained))
        .collect::<Vec<_>>();
    let amount_without_retain = without_retain.len().saturating_sub(1);
    if amount_without_retain > 0 {
        let first = without_retain
            .first()
            .expect("is not empty")
            .time
            .as_optional()
            .expect("only not retained")
            .timestamp();
        let last = without_retain
            .last()
            .expect("is not empty")
            .time
            .as_optional()
            .expect("only not retained")
            .timestamp();

        let seconds_since_start = last - first;
        let message_every_n_seconds = seconds_since_start as f64 / amount_without_retain as f64;
        if message_every_n_seconds < 1.0 {
            let messages_per_second = 1.0 / message_every_n_seconds;
            write!(title, ", ~{messages_per_second:.1} per second")
        } else if message_every_n_seconds < 100.0 {
            write!(title, ", every ~{message_every_n_seconds:.1} seconds")
        } else {
            write!(
                title,
                ", every ~{:.1} minutes",
                message_every_n_seconds / 60.0
            )
        }
        .expect("write to string should never fail");
    }
    title += ")";

    let rows = topic_history.iter().enumerate().map(|(index, entry)| {
        let time = entry.time.to_string();
        let qos = format::qos(entry.qos).to_owned();
        let value = match &entry.payload {
            Payload::NotUtf8(err) => format!("invalid UTF-8: {err}"),
            Payload::String(str) => str.to_string(),
            Payload::Json(json) => JsonSelector::get_selection(json, json_selector)
                .unwrap_or(json)
                .to_string(),
        };
        let row = Row::new(vec![time, qos, value]);
        if index == last_index {
            row.style(STYLE_BOLD)
        } else {
            row
        }
    });

    let t = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Percentage(100),
        ],
    )
    .header(Row::new(vec!["Time", "QoS", "Value"]).style(STYLE_BOLD))
    .block(Block::new().borders(Borders::ALL).title(title));

    let mut state = TableState::default();
    state.select(Some(topic_history.len() - 1));

    f.render_stateful_widget(t, area, &mut state);
}

fn draw_graph(f: &mut Frame, area: Rect, points: &GraphData) {
    const STYLE: Style = Style::new().fg(Color::LightGreen);
    let datasets = vec![Dataset::default()
        .graph_type(GraphType::Line)
        .marker(symbols::Marker::Braille)
        .style(STYLE)
        .data(&points.data)];

    let chart = Chart::new(datasets)
        .block(Block::new().borders(Borders::ALL).title("Graph"))
        .x_axis(
            Axis::default()
                .bounds([points.x_min, points.x_max])
                .labels(vec![
                    Span::raw(points.first_time.format("%H:%M:%S").to_string()),
                    Span::raw(points.last_time.format("%H:%M:%S").to_string()),
                ]),
        )
        .y_axis(
            Axis::default()
                .bounds([points.y_min, points.y_max])
                .labels(vec![
                    Span::raw(points.y_min.to_string()),
                    Span::raw(points.y_max.to_string()),
                ]),
        );
    f.render_widget(chart, area);
}
