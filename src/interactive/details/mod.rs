use ratatui::layout::Rect;
use ratatui::Frame;

use crate::mqtt::HistoryEntry;

mod history;
mod json_selector;
pub mod json_view;
pub mod payload_view;

#[derive(Default)]
pub struct Details {
    pub payload: payload_view::PayloadView,
}

impl Details {
    pub fn draw(
        &mut self,
        f: &mut Frame,
        area: Rect,
        topic_history: &[HistoryEntry],
        payload_has_focus: bool,
    ) {
        let history_area = self.payload.draw(f, area, topic_history, payload_has_focus);
        let json_selector = self.payload.json_state.selected();
        history::draw(f, history_area, topic_history, &json_selector);
    }
}
