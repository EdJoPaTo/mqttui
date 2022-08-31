use std::cmp::min;

use json::JsonValue;
use tui::backend::Backend;
use tui::layout::{Constraint, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Frame;
use tui_tree_widget::{Tree, TreeState};

use crate::interactive::ui::focus_color;
use crate::json_view::root_tree_items_from_json;
use crate::mqtt::{HistoryEntry, Payload};

mod history;

pub fn draw<B>(
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
