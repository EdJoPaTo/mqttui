use chrono::{DateTime, Local};

use crate::interactive::ui::history::DataPoint;

#[allow(clippy::cast_precision_loss)]
fn parse_time_to_chart_x(time: &DateTime<Local>) -> f64 {
    time.timestamp_millis() as f64
}
struct Point {
    time: DateTime<Local>,
    y: f64,
}

impl Point {
    fn parse_from_datapoint(entry: &DataPoint) -> Option<Self> {
        let time = entry.optional_time()?;
        let y = entry.value.as_ref().ok()?.parse::<f64>().ok()?;
        if y.is_finite() {
            Some(Self { time, y })
        } else {
            None
        }
    }

    fn as_graph_point(&self) -> (f64, f64) {
        let x = parse_time_to_chart_x(&self.time);
        (x, self.y)
    }
}

/// Dataset of Points showable by the graph. Ensures to create a useful graph (has at least 2 points)
pub struct GraphData {
    pub data: Vec<(f64, f64)>,
    pub first_time: DateTime<Local>,
    pub last_time: DateTime<Local>,
    pub x_max: f64,
    pub x_min: f64,
    pub y_max: f64,
    pub y_min: f64,
}

impl GraphData {
    pub fn parse_from_datapoints<'a, I>(entries: I) -> Option<Self>
    where
        I: IntoIterator<Item = &'a DataPoint>,
    {
        let points = entries
            .into_iter()
            .filter_map(Point::parse_from_datapoint)
            .collect::<Vec<_>>();

        if points.len() < 2 {
            return None;
        }

        let first_time = points.first().unwrap().time;
        let last_time = points.last().unwrap().time;
        let x_min = parse_time_to_chart_x(&first_time);
        let x_max = parse_time_to_chart_x(&last_time);

        let mut data = Vec::with_capacity(points.len());
        let mut y_min = points.first().unwrap().y;
        let mut y_max = y_min;
        for point in points {
            y_min = y_min.min(point.y);
            y_max = y_max.max(point.y);
            data.push(point.as_graph_point());
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
}
