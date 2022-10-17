use std::cmp::min;

use json::JsonValue;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Frame;
use tui_tree_widget::{Tree, TreeState};

use crate::interactive::ui::{focus_color, get_row_inside, split_area_vertically};
use crate::json_view::root_tree_items_from_json;
use crate::mqtt::{HistoryEntry, Payload};

mod history;

#[derive(Default)]
pub struct Details {
    pub json_view: TreeState,
    pub last_json_area: Option<Rect>,
}

impl Details {
    pub fn draw<B>(
        &mut self,
        f: &mut Frame<B>,
        area: Rect,
        topic_history: &[HistoryEntry],
        json_payload_has_focus: bool,
    ) where
        B: Backend,
    {
        self.last_json_area = None;

        let last = topic_history.last().unwrap();
        let size = last.payload_size;
        let history_area = match &last.payload {
            Payload::Json(json) => {
                let (payload_area, remaining_area) = split_area_vertically(area, area.height / 4);
                self.last_json_area = Some(payload_area);
                draw_payload_json(
                    f,
                    payload_area,
                    size,
                    json,
                    json_payload_has_focus,
                    &mut self.json_view,
                );
                remaining_area
            }
            Payload::NotUtf8(err) => draw_payload_string(f, area, size, &err.to_string()),
            Payload::String(str) => draw_payload_string(f, area, size, str),
        };

        history::draw(f, history_area, topic_history, &self.json_view.selected());
    }

    pub fn json_index_of_click(&mut self, column: u16, row: u16) -> Option<usize> {
        if let Some(index) = self
            .last_json_area
            .and_then(|area| get_row_inside(area, column, row))
        {
            let offset = self.json_view.get_offset();
            let new_index = (index as usize) + offset;
            Some(new_index)
        } else {
            None
        }
    }
}

/// Returns remaining rect to be used for history
fn draw_payload_string<B>(f: &mut Frame<B>, area: Rect, payload_bytes: usize, payload: &str) -> Rect
where
    B: Backend,
{
    let title = format!("Payload (Bytes: {})", payload_bytes);
    let items = payload.lines().map(ListItem::new).collect::<Vec<_>>();

    let max_payload_height = area.height / 3;
    #[allow(clippy::cast_possible_truncation)]
    let payload_height = min(max_payload_height as usize, 2 + items.len()) as u16;
    let (payload_area, remaining_area) = split_area_vertically(area, payload_height);

    let widget = List::new(items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(widget, payload_area);
    remaining_area
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
