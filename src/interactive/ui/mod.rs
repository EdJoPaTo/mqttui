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

use crate::format;
use crate::interactive::app::{App, ElementInFocus};
use crate::json_view::root_tree_items_from_json;
use crate::mqtt_history::HistoryEntry;
use crate::topic_view::{self, TopicTreeEntry};

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
    let host = format!("MQTT Broker: {} (Port {})", app.host, app.port);
    let subscribed = format!("Subscribed Topic: {}", app.subscribe_topic);
    let mut text = vec![Spans::from(host), Spans::from(subscribed)];

    if let Some(err) = app.history.has_connection_err().unwrap() {
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
    let topics = app.history.to_tmlp()?;
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
    let overview_area = if let Some(selected_topic) = &app.selected_topic {
        if let Some(topic_history) = app.history.get(selected_topic)? {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
                .direction(Direction::Horizontal)
                .split(area);

            draw_details(
                f,
                chunks[1],
                &topic_history,
                app.focus == ElementInFocus::JsonPayload,
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
        topics.len(),
        &tree_items,
        app.focus == ElementInFocus::TopicOverview,
        &mut app.topic_overview_state,
    );
    Ok(())
}

fn draw_overview<B>(
    f: &mut Frame<B>,
    area: Rect,
    topic_amount: usize,
    tree_items: &[TopicTreeEntry],
    has_focus: bool,
    state: &mut TreeState,
) where
    B: Backend,
{
    let title = format!("Topics ({})", topic_amount);

    let tree_items = topic_view::tree_items_from_tmlp_tree(tree_items);

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
    let payload_length = last.packet.payload.len();
    let payload_json = format::payload_as_json(last.packet.payload.to_vec());

    #[allow(clippy::option_if_let_else)]
    let history_area = if let Some(json) = payload_json {
        let chunks = Layout::default()
            .constraints(
                [
                    #[allow(clippy::cast_possible_truncation)]
                    Constraint::Percentage(25),
                    Constraint::Min(16),
                ]
                .as_ref(),
            )
            .split(area);

        draw_payload_json(
            f,
            chunks[0],
            payload_length,
            &json,
            json_payload_has_focus,
            json_view_state,
        );
        chunks[1]
    } else {
        let payload = format::payload_as_utf8(last.packet.payload.to_vec());
        let lines = payload.matches('\n').count().saturating_add(1);

        let max_payload_height = area.height / 3;
        let chunks = Layout::default()
            .constraints(
                [
                    #[allow(clippy::cast_possible_truncation)]
                    Constraint::Length(min(max_payload_height as usize, 2 + lines) as u16),
                    Constraint::Min(16),
                ]
                .as_ref(),
            )
            .split(area);

        draw_payload_string(f, chunks[0], payload_length, &payload);
        chunks[1]
    };

    history::draw(f, history_area, topic_history, &json_view_state.selected());
}

fn draw_payload_string<B>(f: &mut Frame<B>, area: Rect, bytes: usize, payload: &str)
where
    B: Backend,
{
    let title = format!("Payload (Bytes: {})", bytes);
    let items = payload.lines().map(ListItem::new).collect::<Vec<_>>();
    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, area);
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
