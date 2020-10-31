use crate::format;
use crate::interactive::app::App;
use crate::mqtt_history::{self, HistoryEntry};
use crate::topic_view::{self, TopicTreeEntry};
use chrono::{DateTime, Local};
use std::cmp::{min, Ordering};
use std::error::Error;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    text::Spans,
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph, Row, Table,
        TableState, Wrap,
    },
    Frame,
};
use tui_tree_widget::{Tree, TreeState};

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

    let title = format!("MQTT TUI {}", env!("CARGO_PKG_VERSION"));
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

    let topics = mqtt_history::history_to_tmlp(history.iter());
    let tree_items = topic_view::get_tmlp_as_tree(&topics);

    // Move opened_topics over to TreeState
    app.topic_overview_state.close_all();
    for topic in &app.opened_topics {
        app.topic_overview_state
            .open(topic_view::get_identifier_of_topic(&tree_items, topic).unwrap_or_default());
    }

    // Ensure selected topic is selected index
    app.topic_overview_state.select(
        app.selected_topic
            .as_ref()
            .and_then(|selected_topic| {
                topic_view::get_identifier_of_topic(&tree_items, selected_topic)
            })
            .unwrap_or_default(),
    );

    #[allow(clippy::option_if_let_else)]
    let overview_area = if let Some(topic_history) = app
        .selected_topic
        .as_ref()
        .and_then(|selected_topic| history.get(selected_topic))
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
            .direction(Direction::Horizontal)
            .split(area);

        draw_details(f, chunks[1], topic_history);

        chunks[0]
    } else {
        area
    };

    draw_overview(
        f,
        overview_area,
        topics.len(),
        &tree_items,
        &mut app.topic_overview_state,
    );
    Ok(())
}

fn draw_overview<B>(
    f: &mut Frame<B>,
    area: Rect,
    topic_amount: usize,
    tree_items: &[TopicTreeEntry],
    state: &mut TreeState,
) where
    B: Backend,
{
    let title = format!("Topics ({})", topic_amount);

    let tree_items = topic_view::tree_items_from_tmlp_tree(&tree_items);

    let widget = Tree::new(tree_items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::LightGreen));
    f.render_stateful_widget(widget, area, state);
}

fn draw_details<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry])
where
    B: Backend,
{
    let last = topic_history.last().unwrap();
    let payload_length = last.packet.payload.len();
    let payload_json = format::payload_as_json(last.packet.payload.to_vec());

    let payload = payload_json.map_or(
        format::payload_as_utf8(last.packet.payload.to_vec()),
        |payload| json::stringify_pretty(payload, 2),
    );
    let lines = payload.matches('\n').count().saturating_add(1);

    let chunks = Layout::default()
        .constraints(
            [
                #[allow(clippy::cast_possible_truncation)]
                Constraint::Length(min(area.height as usize / 3, 2 + lines) as u16),
                Constraint::Min(16),
            ]
            .as_ref(),
        )
        .split(area);

    draw_payload(f, chunks[0], payload_length, &payload);
    draw_history(f, chunks[1], topic_history);
}

fn draw_payload<B>(f: &mut Frame<B>, area: Rect, bytes: usize, payload: &str)
where
    B: Backend,
{
    let title = format!("Payload (Bytes: {})", bytes);
    let items = payload.lines().map(ListItem::new).collect::<Vec<_>>();
    let widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::LightGreen));
    f.render_widget(widget, area);
}

fn draw_history<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry])
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

#[allow(clippy::cast_precision_loss)]
fn draw_details_table<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry])
where
    B: Backend,
{
    let mut title = format!("History ({}", topic_history.len());

    let without_retain: Vec<_> = topic_history.iter().filter(|o| !o.packet.retain).collect();
    let amount_without_retain = without_retain.len().saturating_sub(1);
    if amount_without_retain > 0 {
        title += ", every ~";

        let seconds_since_start = without_retain.last().unwrap().time.timestamp()
            - without_retain.first().unwrap().time.timestamp();
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
        let time = format::timestamp(entry.packet.retain, &entry.time);
        let qos = format::qos(entry.packet.qos);
        let payload = format::payload_as_utf8(entry.packet.payload.to_vec());
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

    let y = format::payload_as_float(entry.packet.payload.to_vec())?;
    let x = parse_time_to_chart_y(entry.time);
    Some((x, y))
}

#[allow(clippy::cast_precision_loss)]
fn parse_time_to_chart_y(time: DateTime<Local>) -> f64 {
    time.timestamp_millis() as f64
}
