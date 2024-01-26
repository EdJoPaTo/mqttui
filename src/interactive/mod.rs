use std::io::stdout;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent, KeyModifiers,
    MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use ratatui::{backend::CrosstermBackend, Terminal};
use rumqttc::{Client, Connection};
use tui_tree_widget::TreeItem;

use crate::cli::Broker;
use crate::interactive::details::json_view::root_tree_items_from_json;
use crate::interactive::ui::ElementInFocus;
use crate::mqtt::Payload;

mod clean_retained;
mod details;
mod footer;
mod mqtt_error_widget;
mod mqtt_history;
mod mqtt_thread;
mod topic_overview;
mod ui;

enum Event {
    Key(KeyEvent),
    MouseClick { column: u16, row: u16 },
    MouseScrollUp,
    MouseScrollDown,
    Tick,
}

enum Refresh {
    /// Update the TUI
    Update,
    /// Skip the update of the TUI
    Skip,
    /// Quit the TUI and return to the shell
    Quit,
}

#[derive(Clone, Copy)]
enum SearchSelection {
    Before,
    Stay,
    After,
}

pub fn show(
    client: Client,
    connection: Connection,
    broker: &Broker,
    subscribe_topic: Vec<String>,
) -> anyhow::Result<()> {
    let mqtt_thread = mqtt_thread::MqttThread::new(client, connection, subscribe_topic)?;
    let mut app = App::new(broker, mqtt_thread);

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        const TICK_RATE: Duration = Duration::from_millis(500);

        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = TICK_RATE
                .checked_sub(last_tick.elapsed())
                .unwrap_or_default();
            if crossterm::event::poll(timeout).unwrap() {
                match crossterm::event::read().unwrap() {
                    CEvent::Key(key) => tx.send(Event::Key(key)).unwrap(),
                    CEvent::Mouse(mouse) => match mouse.kind {
                        MouseEventKind::ScrollUp => tx.send(Event::MouseScrollUp).unwrap(),
                        MouseEventKind::ScrollDown => tx.send(Event::MouseScrollDown).unwrap(),
                        MouseEventKind::Down(MouseButton::Left) => tx
                            .send(Event::MouseClick {
                                column: mouse.column,
                                row: mouse.row,
                            })
                            .unwrap(),
                        _ => {}
                    },
                    CEvent::FocusGained
                    | CEvent::FocusLost
                    | CEvent::Paste(_)
                    | CEvent::Resize(_, _) => {}
                }
            }
            if last_tick.elapsed() >= TICK_RATE {
                if tx.send(Event::Tick).is_err() {
                    // The receiver is gone → the main thread is finished.
                    // Just end the loop here, reporting this error is not helpful in any form.
                    // If the main loop exited successfully this is planned. If not we cant give a helpful error message here anyway.
                    break;
                }
                last_tick = Instant::now();
            }
        }
    });

    terminal.clear()?;

    let main_loop_result = main_loop(&mut app, &rx, &mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    main_loop_result
}

fn main_loop<B>(
    app: &mut App,
    rx: &Receiver<Event>,
    terminal: &mut Terminal<B>,
) -> anyhow::Result<()>
where
    B: Backend,
{
    terminal.draw(|f| app.draw(f))?;
    loop {
        let refresh = match rx.recv()? {
            Event::Key(event) => app.on_key(event)?,
            Event::MouseClick { column, row } => app.on_click(column, row),
            Event::MouseScrollDown => app.on_down(),
            Event::MouseScrollUp => app.on_up(),
            Event::Tick => Refresh::Update,
        };
        match refresh {
            Refresh::Update => {
                terminal.draw(|f| app.draw(f))?;
            }
            Refresh::Skip => {}
            Refresh::Quit => break,
        }
    }
    Ok(())
}

pub struct App {
    details: details::Details,
    focus: ElementInFocus,
    footer: footer::Footer,
    mqtt_thread: mqtt_thread::MqttThread,
    topic_overview: topic_overview::TopicOverview,
}

impl App {
    fn new(broker: &Broker, mqtt_thread: mqtt_thread::MqttThread) -> Self {
        Self {
            details: details::Details::default(),
            focus: ElementInFocus::TopicOverview,
            footer: footer::Footer::new(broker),
            mqtt_thread,
            topic_overview: topic_overview::TopicOverview::default(),
        }
    }

    fn get_topic_tree_items(&self) -> Vec<TreeItem<'static, String>> {
        let (_amount, items) = self.mqtt_thread.get_history().to_tree_items();
        items
    }

    fn can_switch_to_payload(&self) -> bool {
        let Some(topic) = self.topic_overview.get_selected() else {
            return false;
        };
        let history = self.mqtt_thread.get_history();
        let Some(history_entry) = &history.get_last(&topic) else {
            return false;
        };
        let result = matches!(history_entry.payload, Payload::Json(_));
        drop(history);
        result
    }

    fn get_json_of_current_topic(&self) -> Option<serde_json::Value> {
        let topic = self.topic_overview.get_selected()?;
        self.mqtt_thread
            .get_history()
            .get_last(&topic)
            .and_then(|last| last.payload.as_optional_json().cloned())
    }

    #[allow(clippy::too_many_lines)]
    fn on_key(&mut self, key: KeyEvent) -> anyhow::Result<Refresh> {
        let refresh = match &self.focus {
            ElementInFocus::TopicOverview => match key.code {
                KeyCode::Char('q') => Refresh::Quit,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Refresh::Quit
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    if self.can_switch_to_payload() {
                        self.focus = ElementInFocus::JsonPayload;
                    }
                    Refresh::Update
                }
                KeyCode::Char('/') => {
                    self.focus = ElementInFocus::TopicSearch;
                    Refresh::Update
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.topic_overview.state.toggle_selected();
                    Refresh::Update
                }
                KeyCode::Down | KeyCode::Char('j') => self.on_down(),
                KeyCode::Up | KeyCode::Char('k') => self.on_up(),
                KeyCode::Left | KeyCode::Char('h') => {
                    self.topic_overview.state.key_left();
                    Refresh::Update
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.topic_overview.state.key_right();
                    Refresh::Update
                }
                KeyCode::Home => {
                    let items = self.get_topic_tree_items();
                    self.topic_overview.state.select_first(&items);
                    Refresh::Update
                }
                KeyCode::End => {
                    let items = self.get_topic_tree_items();
                    self.topic_overview.state.select_last(&items);
                    Refresh::Update
                }
                KeyCode::PageUp => {
                    let page_jump = (self.topic_overview.last_area.height / 2) as usize;
                    let items = self.get_topic_tree_items();
                    self.topic_overview
                        .state
                        .select_visible_relative(&items, |current| {
                            current.map_or(usize::MAX, |current| current.saturating_sub(page_jump))
                        });
                    Refresh::Update
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let page_jump = (self.topic_overview.last_area.height / 2) as usize;
                    let items = self.get_topic_tree_items();
                    self.topic_overview
                        .state
                        .select_visible_relative(&items, |current| {
                            current.map_or(usize::MAX, |current| current.saturating_sub(page_jump))
                        });
                    Refresh::Update
                }
                KeyCode::PageDown => {
                    let page_jump = (self.topic_overview.last_area.height / 2) as usize;
                    let items = self.get_topic_tree_items();
                    self.topic_overview
                        .state
                        .select_visible_relative(&items, |current| {
                            current.map_or(0, |current| current.saturating_add(page_jump))
                        });
                    Refresh::Update
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let page_jump = (self.topic_overview.last_area.height / 2) as usize;
                    let items = self.get_topic_tree_items();
                    self.topic_overview
                        .state
                        .select_visible_relative(&items, |current| {
                            current.map_or(0, |current| current.saturating_add(page_jump))
                        });
                    Refresh::Update
                }
                KeyCode::Backspace | KeyCode::Delete => {
                    if let Some(topic) = self.topic_overview.get_selected() {
                        self.focus = ElementInFocus::CleanRetainedPopup(topic);
                        Refresh::Update
                    } else {
                        Refresh::Skip
                    }
                }
                _ => Refresh::Skip,
            },
            ElementInFocus::TopicSearch => match key.code {
                KeyCode::Char(char) => {
                    self.topic_overview.search += &char.to_lowercase().to_string();
                    self.search_select(SearchSelection::Stay);
                    Refresh::Update
                }
                KeyCode::Backspace => {
                    self.topic_overview.search.pop();
                    self.search_select(SearchSelection::Stay);
                    Refresh::Update
                }
                KeyCode::Up => {
                    self.search_select(SearchSelection::Before);
                    Refresh::Update
                }
                KeyCode::Down => {
                    self.search_select(SearchSelection::After);
                    Refresh::Update
                }
                KeyCode::Enter => {
                    self.search_select(SearchSelection::After);
                    self.topic_overview.state.close_all();
                    self.open_all_search_matches();
                    Refresh::Update
                }
                KeyCode::Esc => {
                    self.topic_overview.search = String::new();
                    self.focus = ElementInFocus::TopicOverview;
                    Refresh::Update
                }
                KeyCode::Tab => {
                    self.focus = ElementInFocus::TopicOverview;
                    Refresh::Update
                }
                _ => Refresh::Skip,
            },
            ElementInFocus::JsonPayload => match key.code {
                KeyCode::Char('q') => Refresh::Quit,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Refresh::Quit
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    self.focus = ElementInFocus::TopicOverview;
                    Refresh::Update
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.details.json_view.toggle_selected();
                    Refresh::Update
                }
                KeyCode::Down | KeyCode::Char('j') => self.on_down(),
                KeyCode::Up | KeyCode::Char('k') => self.on_up(),
                KeyCode::Left | KeyCode::Char('h') => {
                    self.details.json_view.key_left();
                    Refresh::Update
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.details.json_view.key_right();
                    Refresh::Update
                }
                KeyCode::Home => {
                    let json = self
                        .get_json_of_current_topic()
                        .unwrap_or(serde_json::Value::Null);
                    let items = root_tree_items_from_json(&json);
                    self.details.json_view.select_first(&items);
                    Refresh::Update
                }
                KeyCode::End => {
                    let json = self
                        .get_json_of_current_topic()
                        .unwrap_or(serde_json::Value::Null);
                    let items = root_tree_items_from_json(&json);
                    self.details.json_view.select_last(&items);
                    Refresh::Update
                }
                _ => Refresh::Skip,
            },
            ElementInFocus::CleanRetainedPopup(topic) => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                    self.mqtt_thread.clean_below(topic)?;
                }
                self.focus = ElementInFocus::TopicOverview;
                Refresh::Update
            }
        };
        Ok(refresh)
    }

    /// Handle mouse and keyboard up movement
    fn on_up(&mut self) -> Refresh {
        match self.focus {
            ElementInFocus::TopicOverview => {
                let items = self.get_topic_tree_items();
                self.topic_overview.state.key_up(&items);
            }
            ElementInFocus::TopicSearch => {}
            ElementInFocus::JsonPayload => {
                let json = self
                    .get_json_of_current_topic()
                    .unwrap_or(serde_json::Value::Null);
                let items = root_tree_items_from_json(&json);
                self.details.json_view.key_up(&items);
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }
        Refresh::Update
    }

    /// Handle mouse and keyboard down movement
    fn on_down(&mut self) -> Refresh {
        match self.focus {
            ElementInFocus::TopicOverview => {
                let items = self.get_topic_tree_items();
                self.topic_overview.state.key_down(&items);
            }
            ElementInFocus::TopicSearch => {}
            ElementInFocus::JsonPayload => {
                let json = self
                    .get_json_of_current_topic()
                    .unwrap_or(serde_json::Value::Null);
                let items = root_tree_items_from_json(&json);
                self.details.json_view.key_down(&items);
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }
        Refresh::Update
    }

    fn on_click(&mut self, column: u16, row: u16) -> Refresh {
        if let Some(index) = self.topic_overview.index_of_click(column, row) {
            let items = self.get_topic_tree_items();
            let changed = self
                .topic_overview
                .state
                .select_visible_index(&items, index);
            if !changed {
                self.topic_overview.state.toggle_selected();
            }
            self.focus = ElementInFocus::TopicOverview;
            return Refresh::Update;
        }

        if let Some(index) = self.details.json_index_of_click(column, row) {
            let json = self
                .get_json_of_current_topic()
                .unwrap_or(serde_json::Value::Null);
            let items = root_tree_items_from_json(&json);
            let changed = self.details.json_view.select_visible_index(&items, index);
            if !changed {
                self.details.json_view.toggle_selected();
            }
            self.focus = ElementInFocus::JsonPayload;
            return Refresh::Update;
        }
        Refresh::Skip
    }

    fn search_select(&mut self, advance: SearchSelection) {
        let selection = self.topic_overview.get_selected();
        let history = self.mqtt_thread.get_history();
        let mut topics = history
            .get_all_topics()
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();

        let begin_index = selection
            .and_then(|selection| {
                topics
                    .iter()
                    .find(|(_, topic)| *topic == &selection)
                    .map(|(index, _)| *index)
            })
            .unwrap_or(0);

        // Filter out topics not matching the search
        topics.retain(|(_, topic)| topic.to_lowercase().contains(&self.topic_overview.search));

        let select = match advance {
            SearchSelection::Before => topics
                .iter()
                .rev()
                .find(|(index, _)| *index < begin_index)
                .or_else(|| topics.last()),
            SearchSelection::Stay => topics
                .iter()
                .find(|(index, _)| *index >= begin_index)
                .or_else(|| topics.first()),
            SearchSelection::After => topics
                .iter()
                .find(|(index, _)| *index > begin_index)
                .or_else(|| topics.first()),
        };
        let select = select.map_or(Vec::new(), |(_, topic)| {
            topic
                .split('/')
                .map(std::borrow::ToOwned::to_owned)
                .collect()
        });
        drop(history);

        for i in 0..select.len() {
            self.topic_overview.state.open(select[0..i].to_vec());
        }

        self.topic_overview.state.select(select);
    }

    fn open_all_search_matches(&mut self) {
        let topics = self
            .mqtt_thread
            .get_history()
            .get_all_topics()
            .into_iter()
            .filter(|topic| topic.to_lowercase().contains(&self.topic_overview.search))
            .map(|topic| {
                topic
                    .split('/')
                    .map(std::borrow::ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for splitted in topics {
            for i in 0..splitted.len() {
                self.topic_overview.state.open(splitted[0..i].to_vec());
            }
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        const HEADER_HEIGHT: u16 = 1;
        const FOOTER_HEIGHT: u16 = 1;

        let connection_error = self.mqtt_thread.has_connection_err();

        let area = f.size();
        let Rect { width, height, .. } = area;
        debug_assert_eq!(area.x, 0, "area should fill the whole space");
        debug_assert_eq!(area.y, 0, "area should fill the whole space");

        let header_area = Rect {
            height: HEADER_HEIGHT,
            y: 0,
            ..area
        };
        let footer_area = Rect {
            height: FOOTER_HEIGHT,
            y: height - 1,
            ..area
        };
        let error_height = if connection_error.is_some() { 4 } else { 0 };
        let error_area = Rect {
            height: error_height,
            y: height
                .saturating_sub(FOOTER_HEIGHT)
                .saturating_sub(error_height),
            ..area
        };
        let main_area = Rect {
            height: height
                .saturating_sub(HEADER_HEIGHT + FOOTER_HEIGHT)
                .saturating_sub(error_height),
            y: HEADER_HEIGHT,
            ..area
        };

        if let Some(topic) = self.topic_overview.get_selected() {
            let paragraph = Paragraph::new(Span::styled(topic, ui::STYLE_BOLD));
            f.render_widget(paragraph.alignment(Alignment::Center), header_area);
        }

        self.footer.draw(f, footer_area, self);
        if let Some(connection_error) = connection_error {
            mqtt_error_widget::draw(f, error_area, "MQTT Connection Error", &connection_error);
        }

        let history = self.mqtt_thread.get_history();

        let overview_area = self
            .topic_overview
            .get_selected()
            .as_ref()
            .and_then(|selected_topic| history.get(selected_topic))
            .map_or(main_area, |topic_history| {
                let x = width / 3;
                let details_area = Rect {
                    width: width - x,
                    x,
                    ..main_area
                };

                self.details.draw(
                    f,
                    details_area,
                    topic_history,
                    matches!(self.focus, ElementInFocus::JsonPayload),
                );

                Rect {
                    width: x,
                    x: 0,
                    ..main_area
                }
            });

        let (topic_amount, tree_items) = history.to_tree_items();
        self.topic_overview.draw(
            f,
            overview_area,
            topic_amount,
            tree_items,
            matches!(self.focus, ElementInFocus::TopicOverview),
        );
        drop(history);

        if let ElementInFocus::CleanRetainedPopup(topic) = &self.focus {
            clean_retained::draw_popup(f, topic);
        }
    }
}
