use chrono::NaiveDateTime;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use ratatui::{symbols, Frame};
use tui_tree_widget::Selector;

use self::point::Point;
use crate::mqtt::HistoryEntry;

mod point;

pub struct Graph {
    data: Vec<(f64, f64)>,
    first_time: NaiveDateTime,
    last_time: NaiveDateTime,
    x_max: f64,
    x_min: f64,
    y_max: f64,
    y_min: f64,
}

impl Graph {
    /// Ensures to create a useful graph (has at least 2 points)
    pub fn parse(
        entries: &[HistoryEntry],
        binary_address: usize,
        json_selector: &[Selector],
    ) -> Option<Self> {
        let points = entries
            .iter()
            .filter_map(|entry| Point::parse(entry, binary_address, json_selector))
            .collect::<Box<[_]>>();

        let [ref first, .., ref last] = *points else {
            return None;
        };

        let mut data = Vec::with_capacity(points.len());
        let mut y_min = first.y;
        let mut y_max = y_min;
        for point in points.iter() {
            y_min = y_min.min(point.y);
            y_max = y_max.max(point.y);
            data.push((point.as_graph_x(), point.y));
        }

        Some(Self {
            data,
            first_time: first.time,
            last_time: last.time,
            x_max: last.as_graph_x(),
            x_min: first.as_graph_x(),
            y_max,
            y_min,
        })
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        const STYLE: Style = Style::new().fg(Color::LightGreen);
        let dataset = Dataset::default()
            .graph_type(GraphType::Line)
            .marker(symbols::Marker::Braille)
            .style(STYLE)
            .data(&self.data);
        let chart = Chart::new(vec![dataset])
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .title_alignment(Alignment::Center)
                    .title("Graph"),
            )
            .x_axis(
                Axis::default()
                    .bounds([self.x_min, self.x_max])
                    .labels(vec![
                        Span::raw(self.first_time.format("%H:%M:%S").to_string()),
                        Span::raw(self.last_time.format("%H:%M:%S").to_string()),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .bounds([self.y_min, self.y_max])
                    .labels(vec![
                        Span::raw(self.y_min.to_string()),
                        Span::raw(self.y_max.to_string()),
                    ]),
            );
        frame.render_widget(chart, area);
    }
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use super::*;
    use crate::mqtt::Time;
    use crate::payload::Payload;

    fn entry(time: Time, payload: &str) -> HistoryEntry {
        HistoryEntry {
            qos: rumqttc::QoS::AtMostOnce,
            time,
            payload_size: payload.len(),
            payload: Payload::String(payload.into()),
        }
    }

    #[test]
    fn not_enough_points() {
        let entries = vec![
            entry(Time::Retained, "12.3"),
            entry(Time::Local(Time::datetime_example()), "12.3"),
            // After an MQTT reconnect retained are sent again -> also filter them out
            entry(Time::Retained, "12.3"),
        ];
        let graph = Graph::parse(&entries, 0, &[]);
        assert!(graph.is_none());
    }

    #[test]
    fn retained_filtered_out() {
        let first_date = Time::datetime_example();
        let second_date = first_date.with_second(59).unwrap();
        let entries = vec![
            entry(Time::Retained, "12.3"),
            entry(Time::Local(first_date), "12.4"),
            // After an MQTT reconnect retained are sent again -> also filter them out
            entry(Time::Retained, "12.4"),
            entry(Time::Local(second_date), "12.5"),
        ];

        let graph = Graph::parse(&entries, 0, &[]).expect("Should be possible to create graph");

        assert_eq!(graph.data.len(), 2);
        assert_eq!(graph.first_time, first_date);
        assert_eq!(graph.last_time, second_date);
        assert!((graph.y_min - 12.4).abs() < 0.01);
        assert!((graph.y_max - 12.5).abs() < 0.01);
    }
}
