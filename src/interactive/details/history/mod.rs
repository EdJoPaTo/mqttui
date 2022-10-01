use std::fmt::Write;

use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row, Table, TableState};
use tui::{symbols, Frame};

use crate::interactive::ui::{split_area_vertically, STYLE_BOLD};
use crate::mqtt::{HistoryEntry, Payload, Time};
use crate::{format, json_view};
use graph_data::GraphData;

mod graph_data;

pub fn draw<B>(
    f: &mut Frame<B>,
    area: Rect,
    topic_history: &[HistoryEntry],
    json_selector: &[usize],
) where
    B: Backend,
{
    let table_area = GraphData::parse(topic_history, json_selector).map_or(area, |data| {
        let (table_area, graph_area) = split_area_vertically(area, area.height / 2);
        draw_graph(f, graph_area, &data);
        table_area
    });
    draw_table(f, table_area, topic_history, json_selector);
}

#[allow(clippy::cast_precision_loss)]
fn draw_table<B>(
    f: &mut Frame<B>,
    area: Rect,
    topic_history: &[HistoryEntry],
    json_selector: &[usize],
) where
    B: Backend,
{
    let mut title = format!("History ({}", topic_history.len());

    let without_retain = topic_history
        .iter()
        .filter(|o| !matches!(o.time, Time::Retained))
        .collect::<Vec<_>>();
    let amount_without_retain = without_retain.len().saturating_sub(1);
    if amount_without_retain > 0 {
        title += ", every ~";

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
        if message_every_n_seconds < 100.0 {
            let _ = write!(title, "{:.1} seconds", message_every_n_seconds);
        } else {
            let _ = write!(title, "{:.1} minutes", message_every_n_seconds / 60.0);
        }
    }
    title += ")";

    let rows = topic_history.iter().map(|entry| {
        let time = entry.time.to_string();
        let qos = format::qos(entry.qos).to_string();
        let value = match &entry.payload {
            Payload::NotUtf8(err) => format!("invalid UTF-8: {}", err),
            Payload::String(str) => str.to_string(),
            Payload::Json(json) => json_view::get_selected_subvalue(json, json_selector)
                .unwrap_or(json)
                .dump(),
        };
        Row::new(vec![time, qos, value])
    });

    let t = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .header(Row::new(vec!["Time", "QoS", "Value"]).style(STYLE_BOLD))
        .highlight_style(STYLE_BOLD)
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Percentage(100),
        ]);

    let mut state = TableState::default();
    state.select(Some(topic_history.len() - 1));

    f.render_stateful_widget(t, area, &mut state);
}

fn draw_graph<B>(f: &mut Frame<B>, area: Rect, points: &GraphData)
where
    B: Backend,
{
    const STYLE: Style = Style {
        fg: Some(Color::LightGreen),
        bg: None,
        add_modifier: Modifier::empty(),
        sub_modifier: Modifier::empty(),
    };
    let datasets = vec![Dataset::default()
        .graph_type(GraphType::Line)
        .marker(symbols::Marker::Braille)
        .style(STYLE)
        .data(&points.data)];

    let chart = Chart::new(datasets)
        .block(Block::default().title("Graph").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .labels(vec![
                    Span::raw(points.first_time.format("%H:%M:%S").to_string()),
                    Span::raw(points.last_time.format("%H:%M:%S").to_string()),
                ])
                .bounds([points.x_min, points.x_max]),
        )
        .y_axis(
            Axis::default()
                .labels(vec![
                    Span::raw(points.y_min.to_string()),
                    Span::raw(points.y_max.to_string()),
                ])
                .bounds([points.y_min, points.y_max]),
        );
    f.render_widget(chart, area);
}
