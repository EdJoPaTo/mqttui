use std::cmp::min;
use std::error::Error;

use json::JsonValue;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Modifier;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use tui::Frame;
use tui_tree_widget::{Tree, TreeState};

use crate::interactive::app::{App, ElementInFocus};
use crate::interactive::topic_tree_entry::TopicTreeEntry;
use crate::json_view::root_tree_items_from_json;
use crate::mqtt::{HistoryEntry, Payload};

mod clear_retained;
mod graph_data;
mod history;

const fn focus_color(has_focus: bool) -> Color {
    if has_focus {
        Color::LightGreen
    } else {
        Color::Gray
    }
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    let chunks = Layout::default()
        .constraints([Constraint::Length(2 + 3), Constraint::Min(8)].as_ref())
        .split(f.size());
    draw_info_header(f, chunks[0], app);
    draw_main(f, chunks[1], app)?;
    if let ElementInFocus::CleanRetainedPopup(topic) = &app.focus {
        clear_retained::draw_popup(f, topic);
    }
    Ok(())
}

fn draw_info_header<B>(f: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let host = format!("MQTT Broker: {}", app.display_broker);
    let subscribed = format!("Subscribed Topic: {}", app.subscribe_topic);
    let mut text = vec![Spans::from(host), Spans::from(subscribed)];

    if let Some(err) = app.mqtt_thread.has_connection_err().unwrap() {
        text.push(Spans::from(Span::styled(
            format!("MQTT Connection Error: {}", err),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        )));
    }

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
    let history = app.mqtt_thread.get_history()?;
    let tree_items = history.to_tte();

    // Move opened_topics over to TreeState
    app.topic_overview_state.close_all();
    for topic in &app.opened_topics {
        app.topic_overview_state
            .open(history.get_tree_identifier(topic).unwrap_or_default());
    }

    // Ensure selected topic is selected index
    app.topic_overview_state.select(
        app.selected_topic
            .as_ref()
            .and_then(|selected_topic| history.get_tree_identifier(selected_topic))
            .unwrap_or_default(),
    );

    #[allow(clippy::option_if_let_else)]
    let overview_area = if let Some(selected_topic) = &app.selected_topic {
        if let Some(topic_history) = history.get(selected_topic) {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
                .direction(Direction::Horizontal)
                .split(area);

            draw_details(
                f,
                chunks[1],
                topic_history,
                matches!(app.focus, ElementInFocus::JsonPayload),
                &mut app.json_view_state,
            );

            chunks[0]
        } else {
            area
        }
    } else {
        area
    };

    draw_overview(
        f,
        overview_area,
        &tree_items,
        matches!(app.focus, ElementInFocus::TopicOverview),
        &mut app.topic_overview_state,
    );
    Ok(())
}

fn draw_overview<B>(
    f: &mut Frame<B>,
    area: Rect,
    tree_items: &[TopicTreeEntry],
    has_focus: bool,
    state: &mut TreeState,
) where
    B: Backend,
{
    let topic_amount = tree_items.iter().map(|o| o.topics_below).sum::<usize>();
    let title = format!("Topics ({})", topic_amount);

    let tree_items = tree_items
        .iter()
        .map(std::convert::Into::into)
        .collect::<Vec<_>>();

    let focus_color = focus_color(has_focus);
    let widget = Tree::new(tree_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(focus_color))
                .title(title),
        )
        .highlight_style(Style::default().fg(Color::Black).bg(focus_color));
    f.render_stateful_widget(widget, area, state);
}

fn draw_details<B>(
    f: &mut Frame<B>,
    area: Rect,
    topic_history: &[HistoryEntry],
    json_payload_has_focus: bool,
    json_view_state: &mut TreeState,
) where
    B: Backend,
{
    let last = topic_history.last().unwrap();
    let size = last.payload_size;
    let history_area = match &last.payload {
        Payload::Json(json) => {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(25), Constraint::Min(16)].as_ref())
                .split(area);

            draw_payload_json(
                f,
                chunks[0],
                size,
                json,
                json_payload_has_focus,
                json_view_state,
            );
            chunks[1]
        }
        Payload::NotUtf8(err) => draw_payload_string(f, area, size, &err.to_string()),
        Payload::String(str) => draw_payload_string(f, area, size, str),
    };

    history::draw(f, history_area, topic_history, &json_view_state.selected());
}

/// Returns remaining rect to be used for history
fn draw_payload_string<B>(f: &mut Frame<B>, area: Rect, payload_bytes: usize, payload: &str) -> Rect
where
    B: Backend,
{
    let title = format!("Payload (Bytes: {})", payload_bytes);
    let items = payload.lines().map(ListItem::new).collect::<Vec<_>>();

    let max_payload_height = area.height / 3;
    let chunks = Layout::default()
        .constraints(
            [
                #[allow(clippy::cast_possible_truncation)]
                Constraint::Length(min(max_payload_height as usize, 2 + items.len()) as u16),
                Constraint::Min(16),
            ]
            .as_ref(),
        )
        .split(area);

    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, chunks[0]);
    chunks[1]
}

fn draw_payload_json<B>(
    f: &mut Frame<B>,
    area: Rect,
    bytes: usize,
    json: &JsonValue,
    has_focus: bool,
    view_state: &mut TreeState,
) where
    B: Backend,
{
    let title = format!("JSON Payload (Bytes: {})  (TAB to switch)", bytes);
    let items = root_tree_items_from_json(json);
    let focus_color = focus_color(has_focus);
    let widget = Tree::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(focus_color))
                .title(title),
        )
        .highlight_style(Style::default().fg(Color::Black).bg(focus_color));
    f.render_stateful_widget(widget, area, view_state);
}
