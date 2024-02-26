use chrono::NaiveDateTime;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use ratatui::{symbols, Frame};

use crate::mqtt::HistoryEntry;
use crate::payload::JsonSelector;

use point::Point;

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
        json_selector: &[JsonSelector],
    ) -> Option<Self> {
        let points = entries
            .iter()
            .filter_map(|entry| Point::parse(entry, binary_address, json_selector))
            .collect::<Box<[_]>>();

        let [ref first, .., ref last] = *points else {
            return None;
        };

        let first_time = first.time;
        let last_time = last.time;
        let x_min = first.as_graph_x();
        let x_max = last.as_graph_x();

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
            first_time,
            last_time,
            x_max,
            x_min,
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
