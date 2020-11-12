use std::cmp::Ordering;

use chrono::{DateTime, Local};
use rumqttc::QoS;
use tui::backend::Backend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row, Table, TableState};
use tui::{symbols, Frame};

use crate::format::{self};
use crate::mqtt_history::HistoryEntry;

#[derive(Debug, PartialEq)]
pub enum PacketTime {
    Retained,
    Local(DateTime<Local>),
}

pub struct DataPoint {
    pub time: PacketTime,
    pub qos: QoS,
    pub payload: Vec<u8>,
}

impl DataPoint {
    pub fn parse_from_history_entry(entry: &HistoryEntry) -> Self {
        let time = if entry.packet.retain {
            PacketTime::Retained
        } else {
            PacketTime::Local(entry.time)
        };

        let qos = entry.packet.qos;
        let payload = entry.packet.payload.to_vec();
        DataPoint { time, qos, payload }
    }

    pub fn parse_from_history_entries(entries: &[HistoryEntry]) -> Vec<Self> {
        let mut data = Vec::new();
        for entry in entries {
            data.push(DataPoint::parse_from_history_entry(entry));
        }
        data
    }

    fn optional_time(&self) -> Option<DateTime<Local>> {
        if let PacketTime::Local(time) = self.time {
            Some(time)
        } else {
            None
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn parse_time_to_chart_x(time: &DateTime<Local>) -> f64 {
    time.timestamp_millis() as f64
}

struct GraphDataPoint {
    time: DateTime<Local>,
    y: f64,
}

impl GraphDataPoint {
    fn parse_from_datapoint(entry: &DataPoint) -> Option<GraphDataPoint> {
        // TODO: Impl into instead of randomly named function?

        let time = entry.optional_time()?;
        let y = format::payload_as_float(entry.payload.to_owned())?;
        Some(GraphDataPoint { time, y })
    }

    fn as_graph_point(&self) -> (f64, f64) {
        let x = parse_time_to_chart_x(&self.time);
        (x, self.y)
    }
}

/// Dataset of Points showable by the graph. Ensures to create a useful graph (has at least 2 points)
struct GraphDataPoints {
    data: Vec<GraphDataPoint>,
}

impl GraphDataPoints {
    fn parse_from_datapoints(entries: &[DataPoint]) -> Option<Self> {
        let mut data = Vec::new();
        for entry in entries {
            if let Some(point) = GraphDataPoint::parse_from_datapoint(entry) {
                data.push(point);
            }
        }

        if data.len() < 2 {
            None
        } else {
            Some(GraphDataPoints { data })
        }
    }

    fn get_y_bounds(&self) -> [f64; 2] {
        let y = self.data.iter().map(|o| o.y).collect::<Vec<_>>();

        // TODO: Use total_cmp when stable
        let min = y
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .expect("DataPoints ensures to have points")
            .to_owned();
        let max = y
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .expect("DataPoints ensures to have points")
            .to_owned();

        [min, max]
    }

    fn to_simple_vec(&self) -> Vec<(f64, f64)> {
        self.data
            .iter()
            .map(GraphDataPoint::as_graph_point)
            .collect()
    }

    fn first_time(&self) -> &DateTime<Local> {
        &self
            .data
            .first()
            .expect("DataPoints ensures to have points")
            .time
    }

    fn last_time(&self) -> &DateTime<Local> {
        &self
            .data
            .last()
            .expect("DataPoints ensures to have points")
            .time
    }
}

pub fn draw<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry])
where
    B: Backend,
{
    let data = DataPoint::parse_from_history_entries(&topic_history);

    let table_area = GraphDataPoints::parse_from_datapoints(&data).map_or(area, |points| {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        draw_graph(f, chunks[1], &points);
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

    let without_retain: Vec<_> = topic_history
        .iter()
        .filter(|o| o.time != PacketTime::Retained)
        .collect();
    let amount_without_retain = without_retain.len().saturating_sub(1);
    if amount_without_retain > 0 {
        title += ", every ~";

        let first = without_retain
            .first()
            .expect("is not empty")
            .optional_time()
            .expect("only not retained")
            .timestamp();
        let last = without_retain
            .last()
            .expect("is not empty")
            .optional_time()
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

    let header = ["Time", "QoS", "Payload"];

    let mut rows_content: Vec<Vec<String>> = Vec::new();
    for entry in topic_history {
        let time = match entry.time {
            PacketTime::Retained => String::from(format::TIMESTAMP_RETAINED),
            PacketTime::Local(time) => time.format(format::TIMESTAMP_FORMAT).to_string(),
        };
        let qos = format::qos(entry.qos);
        let payload = format::payload_as_utf8(entry.payload.to_vec());
        rows_content.push(vec![time, qos, payload]);
    }
    let rows = rows_content.iter().map(|i| Row::Data(i.iter()));

    let t = Table::new(header.iter(), rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Min(10),
        ]);

    let mut state = TableState::default();
    state.select(Some(topic_history.len() - 1));

    f.render_stateful_widget(t, area, &mut state);
}

fn draw_graph<B>(f: &mut Frame<B>, area: Rect, points: &GraphDataPoints)
where
    B: Backend,
{
    let simple_data = points.to_simple_vec();
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::LightGreen))
        .graph_type(GraphType::Line)
        .data(&simple_data)];

    let first_time = points.first_time();
    let last_time = points.last_time();

    let ybounds = points.get_y_bounds();

    let chart = Chart::new(datasets)
        .block(Block::default().title("Graph").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .labels(vec![
                    Span::raw(first_time.format("%H:%M:%S").to_string()),
                    Span::raw(last_time.format("%H:%M:%S").to_string()),
                ])
                .bounds([
                    parse_time_to_chart_x(&first_time),
                    parse_time_to_chart_x(&last_time),
                ]),
        )
        .y_axis(
            Axis::default()
                .labels(vec![
                    Span::raw(format!("{}", ybounds[0])),
                    Span::raw(format!("{}", ybounds[1])),
                ])
                .bounds(ybounds),
        );
    f.render_widget(chart, area);
}
