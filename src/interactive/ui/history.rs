use rumqttc::QoS;
use tui::backend::Backend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row, Table, TableState};
use tui::{symbols, Frame};

use crate::format;
use crate::interactive::ui::graph_data::GraphData;
use crate::json_view;
use crate::mqtt_packet::{HistoryEntry, Payload, Time};

pub struct DataPoint {
    pub time: Time,
    pub qos: QoS,
    // TODO: Why result? Maybe &HistoryEntry can be used directly?
    pub value: Result<String, String>,
}

impl DataPoint {
    pub fn parse_from_history_entry(entry: &HistoryEntry, json_selector: &[usize]) -> Self {
        let value = match &entry.payload {
            Payload::NotUtf8(err) => Err(format!("invalid UTF8: {}", err)),
            Payload::String(str) => Ok(str.to_string()),
            Payload::Json(json) => Ok(json_view::get_selected_subvalue(json, json_selector)
                .unwrap_or(json)
                .dump()),
        };
        Self {
            time: entry.time,
            qos: entry.qos,
            value,
        }
    }
}

pub fn draw<'h, B, H>(f: &mut Frame<B>, area: Rect, topic_history: H, json_selector: &[usize])
where
    B: Backend,
    H: IntoIterator<Item = &'h HistoryEntry>,
{
    let data = topic_history
        .into_iter()
        .map(|entry| DataPoint::parse_from_history_entry(entry, json_selector))
        .collect::<Vec<_>>();

    let table_area = GraphData::parse_from_datapoints(&data).map_or(area, |data| {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        draw_graph(f, chunks[1], &data);
        chunks[0]
    });

    draw_table(f, table_area, &data);
}

#[allow(clippy::cast_precision_loss)]
fn draw_table<B>(f: &mut Frame<B>, area: Rect, topic_history: &[DataPoint])
where
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
            title += &format!("{:.1} seconds", message_every_n_seconds);
        } else {
            title += &format!("{:.1} minutes", message_every_n_seconds / 60.0);
        }
    }
    title += ")";

    let rows = topic_history.iter().map(|entry| {
        let time = entry.time.to_string();
        let qos = format::qos(entry.qos);
        let value = entry.value.clone().unwrap_or_else(|err| err);
        Row::new(vec![time, qos, value])
    });

    let t = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .header(
            Row::new(vec!["Time", "QoS", "Value"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
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
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::LightGreen))
        .graph_type(GraphType::Line)
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
