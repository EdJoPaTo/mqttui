use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::interactive::ui::{focus_color, get_row_inside};

#[derive(Default)]
pub struct TopicOverview {
    pub last_area: Rect,
    pub state: TreeState<String>,
}

impl TopicOverview {
    pub fn get_selected(&self) -> Option<String> {
        let selected = self.state.selected();
        if selected.is_empty() {
            return None;
        }
        Some(selected.join("/"))
    }

    pub fn draw(
        &mut self,
        f: &mut Frame,
        area: Rect,
        topic_amount: usize,
        tree_items: Vec<TreeItem<String>>,
        has_focus: bool,
    ) {
        let title = format!("Topics ({topic_amount})");
        let focus_color = focus_color(has_focus);
        let widget = Tree::new(tree_items)
            .unwrap()
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .border_style(Style::new().fg(focus_color))
                    .title_alignment(Alignment::Center)
                    .title(title),
            );
        f.render_stateful_widget(widget, area, &mut self.state);
        self.last_area = area;
    }

    pub fn index_of_click(&mut self, column: u16, row: u16) -> Option<usize> {
        if let Some(index) = get_row_inside(self.last_area, column, row) {
            let offset = self.state.get_offset();
            let new_index = (index as usize) + offset;
            Some(new_index)
        } else {
            None
        }
    }
}
