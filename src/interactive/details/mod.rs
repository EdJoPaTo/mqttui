use ratatui::layout::{Position, Rect};
use ratatui::widgets::TableState;
use ratatui::Frame;

use crate::interactive::details::graph_data::GraphData;
use crate::interactive::ui::{split_area_vertically, ElementInFocus};
use crate::mqtt::HistoryEntry;

mod graph_data;
mod history;
mod payload_view;

#[derive(Default)]
pub struct Details {
    pub table_state: TableState,
    pub last_table_area: Option<Rect>,
    pub payload: payload_view::PayloadView,
}

impl Details {
    pub fn selected_history_index(&self, topic_history_length: usize) -> usize {
        self.table_state
            .selected()
            .unwrap_or(usize::MAX)
            .min(topic_history_length.saturating_sub(1))
    }

    fn table_index_of_click(&self, column: u16, row: u16) -> Option<usize> {
        let area = self.last_table_area?;
        if !area.contains(Position { x: column, y: row }) {
            return None;
        }
        let visible = row.saturating_sub(area.top()).saturating_sub(2); // subtract block & header
        let offset = self.table_state.offset();
        let index = (visible as usize) + offset;
        Some(index)
    }

    /// Handles a click. Checks if its on the table. When it is the index get selected and true is returned.
    pub fn table_click(&mut self, column: u16, row: u16) -> bool {
        let Some(index) = self.table_index_of_click(column, row) else {
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
            entry,
            matches!(focus, ElementInFocus::Payload),
        );
        let binary_address = self.payload.binary_state.selected();
        let json_selector = self.payload.json_state.selected();

        let table_area =
            GraphData::parse(topic_history, binary_address.unwrap_or(0), &json_selector).map_or(
                history_area,
                |data| {
                    let (table_area, graph_area) =
                        split_area_vertically(history_area, history_area.height / 2);
                    history::draw_graph(frame, graph_area, &data);
                    table_area
                },
            );
        self.last_table_area = Some(table_area);
        history::draw_table(
            frame,
            table_area,
            topic_history,
            binary_address,
            &json_selector,
            &mut self.table_state,
            matches!(focus, ElementInFocus::HistoryTable),
        );
    }
}
