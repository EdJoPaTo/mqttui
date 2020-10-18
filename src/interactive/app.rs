use crate::mqtt_history::{get_sorted_vec, HistoryArc};
use tui::widgets::ListState;

pub struct App<'a> {
    pub host: &'a str,
    pub port: u16,
    pub subscribe_topic: &'a str,
    pub history: HistoryArc,

    pub selected_topic: Option<String>,
    pub topics_overview_state: ListState,

    pub should_quit: bool,
}

impl<'a> App<'a> {
    pub fn new(host: &'a str, port: u16, subscribe_topic: &'a str, history: HistoryArc) -> App<'a> {
        App {
            host,
            port,
            subscribe_topic,
            should_quit: false,
            history,
            selected_topic: None,
            topics_overview_state: ListState::default(),
        }
    }

    fn ensure_index_to_topic(&mut self) {
        let topics = get_sorted_vec(self.history.lock().unwrap().keys());

        self.selected_topic = self
            .topics_overview_state
            .selected()
            .and_then(|index| topics.get(index))
            .map(|o| o.to_owned());
    }

    pub fn on_up(&mut self) {
        self.topics_overview_state.select(
            if let Some(selected) = self.topics_overview_state.selected() {
                Some(selected - 1)
            } else {
                Some(0)
            },
        );

        self.ensure_index_to_topic();
    }

    pub fn on_down(&mut self) {
        self.topics_overview_state.select(
            if let Some(selected) = self.topics_overview_state.selected() {
                Some(selected + 1)
            } else {
                Some(0)
            },
        );

        self.ensure_index_to_topic();
    }

    pub fn on_right(&mut self) {}

    pub fn on_left(&mut self) {
        self.topics_overview_state.select(None);
        self.selected_topic = None;
    }

    pub fn on_key(&mut self, c: char) {
        if let 'q' = c {
            self.should_quit = true;
        }
    }
}
