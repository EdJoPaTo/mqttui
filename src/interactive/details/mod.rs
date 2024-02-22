use ratatui::layout::Rect;
use ratatui::Frame;

use crate::mqtt::HistoryEntry;

mod graph_data;
mod history;
pub mod payload_view;

#[derive(Default)]
pub struct Details {
    pub payload: payload_view::PayloadView,
}

impl Details {
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        topic_history: &[HistoryEntry],
        payload_has_focus: bool,
    ) {
        let history_area = self
            .payload
            .draw(frame, area, topic_history, payload_has_focus);
        let binary_address = self.payload.binary_state.selected();
        let json_selector = self.payload.json_state.selected();
        history::draw(
            frame,
            history_area,
            topic_history,
            binary_address,
            &json_selector,
        );
    }
}
