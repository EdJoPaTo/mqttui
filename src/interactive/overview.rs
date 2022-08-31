use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders};
use tui::Frame;
use tui_tree_widget::{Tree, TreeState};

use crate::interactive::topic_tree_entry::TopicTreeEntry;
use crate::interactive::ui::focus_color;

pub fn draw<B>(
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
