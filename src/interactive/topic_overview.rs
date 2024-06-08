use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Scrollbar, ScrollbarOrientation};
use ratatui::Frame;
use tui_tree_widget::{Tree, TreeState};

use super::mqtt_history::MqttHistory;
use super::ui::{focus_color, BORDERS_TOP_RIGHT};

#[derive(Default)]
pub struct TopicOverview {
    pub last_area: Rect,
    pub search: String,
    pub state: TreeState<String>,
}

impl TopicOverview {
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, history: &MqttHistory, has_focus: bool) {
        let topic_amount = history.total_topics();
        let message_amount = history.total_messages();
        let title = format!("Topics ({topic_amount}, {message_amount} messages)");

        let focus_color = focus_color(has_focus);
        let widget = Tree::new(history)
            .experimental_scrollbar(Some(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None),
            ))
            .highlight_style(Style::new().fg(Color::Black).bg(focus_color))
            .block(
                Block::new()
                    .border_type(BorderType::Rounded)
                    .borders(BORDERS_TOP_RIGHT)
                    .border_style(Style::new().fg(focus_color))
                    .title_alignment(Alignment::Center)
                    .title(title),
            );
        frame.render_stateful_widget(widget, area, &mut self.state);
        self.last_area = area;
    }
}
