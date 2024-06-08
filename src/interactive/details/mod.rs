use ratatui::layout::{Position, Rect};
use ratatui::widgets::TableState;
use ratatui::Frame;

use crate::interactive::ui::{split_area_vertically, ElementInFocus};
use crate::mqtt::HistoryEntry;

mod graph;
mod payload_view;
mod table;

#[derive(Default)]
pub struct Details {
    pub table_state: TableState,
    pub last_table_area: Rect,
    pub payload: payload_view::PayloadView,
}

impl Details {
    pub fn selected_history_index(&self, topic_history_length: usize) -> usize {
        self.table_state
            .selected()
            .unwrap_or(usize::MAX)
            .min(topic_history_length.saturating_sub(1))
    }

    const fn table_index_of_click(&self, position: Position) -> Option<usize> {
        let area = self.last_table_area;
        if !area.contains(position) {
            return None;
        }
        let visible_index = position.y.saturating_sub(area.top()).saturating_sub(2); // subtract block & header
        let offset = self.table_state.offset();
        let index = (visible_index as usize) + offset;
        Some(index)
    }

    /// Handles a click. Checks if its on the table. When it is the index get selected and true is returned.
    pub fn table_click(&mut self, position: Position) -> bool {
        let Some(index) = self.table_index_of_click(position) else {
            return false;
        };
        self.table_state.select(Some(index));
        true
    }

    pub fn draw(
        &mut self,
        frame: &mut Frame,
        full_area: Rect,
        topic_history: &[HistoryEntry],
        focus: &ElementInFocus,
    ) {
        let entry = topic_history
            .get(self.selected_history_index(topic_history.len()))
            .expect("when Details are drawn they should always have at least one HistoryEntry");
        let history_area = self.payload.draw(
            frame,
            full_area,
            matches!(focus, ElementInFocus::Payload),
            entry,
        );

        let table_area =
            graph::Graph::parse(topic_history, &self.payload).map_or(history_area, |graph| {
                let (table_area, graph_area) =
                    split_area_vertically(history_area, history_area.height / 2);
                graph.draw(frame, graph_area);
                table_area
            });
        self.last_table_area = table_area;
        table::draw(
            frame,
            table_area,
            topic_history,
            &self.payload,
            &mut self.table_state,
            matches!(focus, ElementInFocus::HistoryTable),
        );
    }
}
