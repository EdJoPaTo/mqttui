use std::fmt::Write;

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType, Row, Table, TableState};
use ratatui::{symbols, Frame};

use crate::format;
use crate::interactive::ui::{split_area_vertically, STYLE_BOLD};
use crate::mqtt::HistoryEntry;
use crate::payload::{JsonSelector, Payload};
use graph_data::GraphData;

mod graph_data;

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    topic_history: &[HistoryEntry],
    binary_address: Option<usize>,
    json_selector: &[JsonSelector],
) {
    let table_area = GraphData::parse(topic_history, binary_address.unwrap_or(0), json_selector)
        .map_or(area, |data| {
            let (table_area, graph_area) = split_area_vertically(area, area.height / 2);
            draw_graph(frame, graph_area, &data);
            table_area
        });
    draw_table(
        frame,
        table_area,
        topic_history,
        binary_address,
        json_selector,
    );
}

#[allow(clippy::cast_precision_loss)]
fn draw_table(
    frame: &mut Frame,
    area: Rect,
    topic_history: &[HistoryEntry],
    binary_address: Option<usize>,
    json_selector: &[JsonSelector],
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
            Payload::Binary(data) => binary_address
                .and_then(|address| data.get(address).copied())
                .map_or_else(|| format!("{data:?}"), |data| format!("{data}")),
            Payload::Json(json) => JsonSelector::get_json(json, json_selector)
                .unwrap_or(json)
                .to_string(),
            Payload::MessagePack(messagepack) => {
                JsonSelector::get_messagepack(messagepack, json_selector)
                    .unwrap_or(messagepack)
                    .to_string()
            }
            Payload::String(str) => str.to_string(),
        };
        let row = Row::new(vec![time, qos, value]);
        if index == last_index {
            row.style(STYLE_BOLD)
        } else {
            row
        }
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Percentage(100),
        ],
    )
    .header(Row::new(vec!["Time", "QoS", "Value"]).style(STYLE_BOLD))
    .block(Block::bordered().title(title));

    let mut state = TableState::default();
    state.select(Some(topic_history.len() - 1));

    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_graph(frame: &mut Frame, area: Rect, points: &GraphData) {
    const STYLE: Style = Style::new().fg(Color::LightGreen);
    let datasets = vec![Dataset::default()
        .graph_type(GraphType::Line)
        .marker(symbols::Marker::Braille)
        .style(STYLE)
        .data(&points.data)];

    let chart = Chart::new(datasets)
        .block(Block::bordered().title("Graph"))
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
    frame.render_widget(chart, area);
}
