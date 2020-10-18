use crate::format::*;
use crate::interactive::app::App;
use crate::mqtt_history::get_sorted_vec;
use crate::mqtt_history::HistoryEntry;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(2 + 2), Constraint::Min(8)].as_ref())
        .split(f.size());
    draw_connection_info(f, app, chunks[0]);
    draw_main(f, chunks[1], app);
}

fn draw_connection_info<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let host = format!("MQTT Broker: {} (Port {})", app.host, app.port);
    let subscribed = format!("Subscribed Topic: {}", app.subscribe_topic);

    let text = vec![Spans::from(host), Spans::from(subscribed)];
    let title = format!("MQTT CLI {}", env!("CARGO_PKG_VERSION"));
    let block = Block::default().borders(Borders::ALL).title(title);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_main<B>(f: &mut Frame<B>, area: Rect, app: &mut App)
where
    B: Backend,
{
    let history = &app.history.lock().unwrap();
    let topics = get_sorted_vec(history.keys());

    let overview_area = if let Some(selected_topic) = &app.selected_topic {
        let pos = topics.iter().position(|t| t == selected_topic);
        app.topics_overview_state.select(pos);

        if let Some(topic_history) = history.get(selected_topic) {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
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
        .constraints([Constraint::Length(4 + 2), Constraint::Min(5)].as_ref())
        .split(area);

    let last = topic_history.last().unwrap();
    draw_details_current(f, chunks[0], last);

    draw_details_table(f, chunks[1], topic_history);
}

fn draw_details_current<B>(f: &mut Frame<B>, area: Rect, entry: &HistoryEntry)
where
    B: Backend,
{
    let topic = entry.packet.topic.to_owned();

    let qos = format!("QoS: {}", format_qos(&entry.packet.qos));
    let timestamp = format_timestamp(entry.packet.retain, &entry.time);
    let payload = format!(
        "Payload ({:>3}): {}",
        entry.packet.payload.len(),
        format_payload(&entry.packet.payload.to_vec())
    );

    let text = vec![
        Spans::from(topic),
        Spans::from(qos),
        Spans::from(timestamp),
        Spans::from(payload),
    ];
    let block = Block::default().borders(Borders::ALL).title("Details");
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_details_table<B>(f: &mut Frame<B>, area: Rect, topic_history: &[HistoryEntry])
where
    B: Backend,
{
    let header = ["Time", "Payload"];

    let mut rows_content: Vec<Vec<String>> = Vec::new();
    for entry in topic_history {
        let time = format_timestamp(entry.packet.retain, &entry.time);
        let payload = format_payload(&entry.packet.payload.to_vec());
        rows_content.push(vec![time, payload]);
    }
    let rows = rows_content.iter().map(|i| Row::Data(i.iter()));

    let t = Table::new(header.iter(), rows)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_style(Style::default().fg(Color::White))
        .widths(&[Constraint::Length(12), Constraint::Min(10)]);

    let mut state = TableState::default();
    state.select(Some(topic_history.len() - 1));

    f.render_stateful_widget(t, area, &mut state);
}
