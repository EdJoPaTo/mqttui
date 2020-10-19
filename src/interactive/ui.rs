use crate::format::*;
use crate::interactive::app::App;
use crate::mqtt_history::get_sorted_vec;
use crate::mqtt_history::HistoryEntry;
use chrono::{DateTime, Local};
use std::cmp::Ordering;
use std::error::Error;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    text::Spans,
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph, Row,
        Table, TableState, Wrap,
    },
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    let chunks = Layout::default()
        .constraints([Constraint::Length(2 + 3), Constraint::Min(8)].as_ref())
        .split(f.size());
    draw_info_header(f, chunks[0], app);
    draw_main(f, chunks[1], app)?;
    Ok(())
}

fn draw_info_header<B>(f: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let host = format!("MQTT Broker: {} (Port {})", app.host, app.port);
    let subscribed = format!("Subscribed Topic: {}", app.subscribe_topic);
    let mut text = vec![Spans::from(host), Spans::from(subscribed)];

    if let Some(topic) = &app.selected_topic {
        text.push(Spans::from(format!("Selected Topic: {}", topic)));
    }

    let title = format!("MQTT CLI {}", env!("CARGO_PKG_VERSION"));
    let block = Block::default().borders(Borders::ALL).title(title);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_main<B>(f: &mut Frame<B>, area: Rect, app: &mut App) -> Result<(), Box<dyn Error>>
where
    B: Backend,
{
    let history = &app
        .history
        .lock()
        .map_err(|err| format!("failed to aquire lock of mqtt history: {}", err))?;
    let topics = get_sorted_vec(history.keys());

    let overview_area = if let Some(selected_topic) = &app.selected_topic {
        let pos = topics.iter().position(|t| t == selected_topic);
        app.topics_overview_state.select(pos);

        if let Some(topic_history) = history.get(selected_topic) {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(area);

            draw_details(f, chunks[1], topic_history);

            chunks[0]
        } else {
            area
        }
    } else {
        area
    };

    draw_overview(f, overview_area, &topics, &mut app.topics_overview_state);
    Ok(())
}

fn draw_overview<B>(f: &mut Frame<B>, area: Rect, topics: &[String], state: &mut ListState)
where
    B: Backend,
{
    let title = format!("Topics ({})", topics.len());

    let items: Vec<ListItem> = topics
        .iter()
        .map(|i| {
            let lines: Vec<Spans> = vec![Spans::from(i.to_owned())];
            ListItem::new(lines)
        })
        .collect();
    let list_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list_widget, area, state);
}

fn draw_details<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry])
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let graph_works = draw_details_chart(f, chunks[1], topic_history).is_some();

    let table_area = if graph_works { chunks[0] } else { area };
    draw_details_table(f, table_area, topic_history);
}

fn draw_details_table<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry])
where
    B: Backend,
{
    let title = format!("History ({})", topic_history.len());
    let header = ["Time", "QoS", "Payload"];

    let mut rows_content: Vec<Vec<String>> = Vec::new();
    for entry in topic_history {
        let time = format_timestamp(entry.packet.retain, &entry.time);
        let qos = format_qos(&entry.packet.qos);
        let payload = format_payload(entry.packet.payload.to_vec());
        rows_content.push(vec![time, qos, payload]);
    }
    let rows = rows_content.iter().map(|i| Row::Data(i.iter()));

    let t = Table::new(header.iter(), rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().fg(Color::White))
        .widths(&[
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Min(10),
        ]);

    let mut state = TableState::default();
    state.select(Some(topic_history.len() - 1));

    f.render_stateful_widget(t, area, &mut state);
}

fn draw_details_chart<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry]) -> Option<()>
where
    B: Backend,
{
    let mut data: Vec<(f64, f64)> = Vec::new();
    for entry in topic_history {
        if let Some(point) = parse_history_entry_to_chart_point(entry) {
            data.push(point);
        }
    }

    if data.len() < 2 {
        return None;
    }

    let ybounds = get_y_bounds(&data)?;

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::LightGreen))
        .graph_type(GraphType::Line)
        .data(&data)];

    let first_time = topic_history.first()?.time;
    let last_time = topic_history.last()?.time;

    let chart = Chart::new(datasets)
        .block(Block::default().title("Graph").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .labels(vec![
                    Span::raw(first_time.format("%H:%M:%S").to_string()),
                    Span::raw(last_time.format("%H:%M:%S").to_string()),
                ])
                .bounds([
                    parse_time_to_chart_y(first_time),
                    parse_time_to_chart_y(last_time),
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

    Some(())
}

fn get_y_bounds(data: &[(f64, f64)]) -> Option<[f64; 2]> {
    let mut y_sorted = data.to_vec();
    // TODO: Use total_cmp when stable
    y_sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
    Some([y_sorted.first()?.1, y_sorted.last()?.1])
}

fn parse_history_entry_to_chart_point(entry: &HistoryEntry) -> Option<(f64, f64)> {
    if entry.packet.retain {
        return None;
    }

    let y = format_payload_as_float(entry.packet.payload.to_vec())?;
    let x = parse_time_to_chart_y(entry.time);
    Some((x, y))
}

fn parse_time_to_chart_y(time: DateTime<Local>) -> f64 {
    time.timestamp_millis() as f64
}
