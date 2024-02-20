use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{
    Event as CEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind,
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
use crate::interactive::details::tree_items_from_json;
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

fn reset_terminal() -> anyhow::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
        crossterm::cursor::Show
    )?;
    Ok(())
}

pub fn show(
    client: Client,
    connection: Connection,
    broker: &Broker,
    subscribe_topic: Vec<String>,
) -> anyhow::Result<()> {
    let mqtt_thread = mqtt_thread::MqttThread::new(client, connection, subscribe_topic)?;
    let mut app = App::new(broker, mqtt_thread);

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal().unwrap();
        original_hook(panic);
    }));

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture,
        crossterm::cursor::Hide
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

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

    reset_terminal()?;

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
    terminal.draw(|frame| app.draw(frame))?;
    loop {
        let refresh = match rx.recv()? {
            Event::Key(event) => app.on_key(event)?,
            Event::MouseClick { column, row } => app.on_click(column, row),
            Event::MouseScrollDown => app.on_scroll_down(),
            Event::MouseScrollUp => app.on_scroll_up(),
            Event::Tick => Refresh::Update,
        };
        match refresh {
            Refresh::Update => {
                terminal.draw(|frame| app.draw(frame))?;
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

    /// Currently always the last payload on the current topic
    /// In the future it might not be the last one (Select index from history table)
    fn get_selected_payload(&self) -> Option<Payload> {
        let topic = self.topic_overview.get_selected()?;
        self.mqtt_thread
            .get_history()
            .get_last(&topic)
            .map(|entry| entry.payload.clone())
    }

    #[allow(clippy::too_many_lines)]
    fn on_key(&mut self, key: KeyEvent) -> anyhow::Result<Refresh> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(Refresh::Quit);
        }

        match &self.focus {
            ElementInFocus::TopicOverview => match key.code {
                KeyCode::Char('q') => return Ok(Refresh::Quit),
                KeyCode::Tab | KeyCode::BackTab if self.can_switch_to_payload() => {
                    self.focus = ElementInFocus::Payload;
                }
                KeyCode::Char('/') => {
                    self.focus = ElementInFocus::TopicSearch;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.topic_overview.state.toggle_selected();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let items = self.get_topic_tree_items();
                    self.topic_overview.state.key_down(&items);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let items = self.get_topic_tree_items();
                    self.topic_overview.state.key_up(&items);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.topic_overview.state.key_left();
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.topic_overview.state.key_right();
                }
                KeyCode::Home => {
                    let items = self.get_topic_tree_items();
                    self.topic_overview.state.select_first(&items);
                }
                KeyCode::End => {
                    let items = self.get_topic_tree_items();
                    self.topic_overview.state.select_last(&items);
                }
                KeyCode::PageUp => {
                    let page_jump = (self.topic_overview.last_area.height / 3) as usize;
                    self.topic_overview.state.scroll_up(page_jump);
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let page_jump = (self.topic_overview.last_area.height / 3) as usize;
                    self.topic_overview.state.scroll_up(page_jump);
                }
                KeyCode::PageDown => {
                    let page_jump = (self.topic_overview.last_area.height / 3) as usize;
                    self.topic_overview.state.scroll_down(page_jump);
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let page_jump = (self.topic_overview.last_area.height / 3) as usize;
                    self.topic_overview.state.scroll_down(page_jump);
                }
                KeyCode::Backspace | KeyCode::Delete => {
                    if let Some(topic) = self.topic_overview.get_selected() {
                        self.focus = ElementInFocus::CleanRetainedPopup(topic);
                    } else {
                        return Ok(Refresh::Skip);
                    }
                }
                _ => return Ok(Refresh::Skip),
            },
            ElementInFocus::TopicSearch => match key.code {
                KeyCode::Char(char) => {
                    self.topic_overview.search += &char.to_lowercase().to_string();
                    self.search_select(SearchSelection::Stay);
                }
                KeyCode::Backspace => {
                    self.topic_overview.search.pop();
                    self.search_select(SearchSelection::Stay);
                }
                KeyCode::Up => {
                    self.search_select(SearchSelection::Before);
                }
                KeyCode::Down => {
                    self.search_select(SearchSelection::After);
                }
                KeyCode::Enter => {
                    self.search_select(SearchSelection::After);
                    self.topic_overview.state.close_all();
                    self.open_all_search_matches();
                }
                KeyCode::Esc => {
                    self.topic_overview.search = String::new();
                    self.focus = ElementInFocus::TopicOverview;
                }
                KeyCode::PageUp => {
                    let page_jump = (self.topic_overview.last_area.height / 3) as usize;
                    self.topic_overview.state.scroll_up(page_jump);
                }
                KeyCode::PageDown => {
                    let page_jump = (self.topic_overview.last_area.height / 3) as usize;
                    self.topic_overview.state.scroll_down(page_jump);
                }
                KeyCode::Tab => {
                    self.focus = ElementInFocus::TopicOverview;
                }
                _ => return Ok(Refresh::Skip),
            },
            ElementInFocus::Payload => {
                if key.code == KeyCode::Char('q') {
                    return Ok(Refresh::Quit);
                }
                if matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
                    self.focus = ElementInFocus::TopicOverview;
                    return Ok(Refresh::Update);
                }
                match self.get_selected_payload() {
                    Some(Payload::NotUtf8(_) | Payload::String(_)) | None => {}
                    Some(Payload::Json(json)) => match key.code {
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            self.details.payload.json_state.toggle_selected();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let items = tree_items_from_json(&json);
                            self.details.payload.json_state.key_down(&items);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let items = tree_items_from_json(&json);
                            self.details.payload.json_state.key_up(&items);
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            self.details.payload.json_state.key_left();
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            self.details.payload.json_state.key_right();
                        }
                        KeyCode::Home => {
                            let items = tree_items_from_json(&json);
                            self.details.payload.json_state.select_first(&items);
                        }
                        KeyCode::End => {
                            let items = tree_items_from_json(&json);
                            self.details.payload.json_state.select_last(&items);
                        }
                        KeyCode::PageUp => {
                            self.details.payload.json_state.scroll_up(3);
                        }
                        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.details.payload.json_state.scroll_up(3);
                        }
                        KeyCode::PageDown => {
                            self.details.payload.json_state.scroll_down(3);
                        }
                        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.details.payload.json_state.scroll_down(3);
                        }
                        _ => return Ok(Refresh::Skip),
                    },
                }
            }
            ElementInFocus::CleanRetainedPopup(topic) => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
                    self.mqtt_thread.clean_below(topic)?;
                }
                self.focus = ElementInFocus::TopicOverview;
            }
        }
        Ok(Refresh::Update)
    }

    fn on_scroll_up(&mut self) -> Refresh {
        match &self.focus {
            ElementInFocus::TopicOverview | ElementInFocus::TopicSearch => {
                self.topic_overview.state.scroll_up(1);
            }
            ElementInFocus::Payload => match self.get_selected_payload() {
                Some(Payload::NotUtf8(_) | Payload::String(_)) | None => return Refresh::Skip,
                Some(Payload::Json(_)) => self.details.payload.json_state.scroll_up(1),
            },
            ElementInFocus::CleanRetainedPopup(_) => return Refresh::Skip,
        }
        Refresh::Update
    }

    fn on_scroll_down(&mut self) -> Refresh {
        match &self.focus {
            ElementInFocus::TopicOverview | ElementInFocus::TopicSearch => {
                self.topic_overview.state.scroll_down(1);
            }
            ElementInFocus::Payload => match self.get_selected_payload() {
                Some(Payload::NotUtf8(_) | Payload::String(_)) | None => return Refresh::Skip,
                Some(Payload::Json(_)) => self.details.payload.json_state.scroll_down(1),
            },
            ElementInFocus::CleanRetainedPopup(_) => return Refresh::Skip,
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

        if let Some(index) = self.details.payload.json_index_of_click(column, row) {
            match self.get_selected_payload() {
                Some(Payload::Json(json)) => {
                    let items = tree_items_from_json(&json);
                    let changed = self
                        .details
                        .payload
                        .json_state
                        .select_visible_index(&items, index);
                    if !changed {
                        self.details.payload.json_state.toggle_selected();
                    }
                    self.focus = ElementInFocus::Payload;
                    return Refresh::Update;
                }
                Some(Payload::NotUtf8(_) | Payload::String(_)) | None => return Refresh::Skip,
            }
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
            topic.split('/').map(ToOwned::to_owned).collect()
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
            .map(|topic| topic.split('/').map(ToOwned::to_owned).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for splitted in topics {
            for i in 0..splitted.len() {
                self.topic_overview.state.open(splitted[0..i].to_vec());
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        const HEADER_HEIGHT: u16 = 1;
        const FOOTER_HEIGHT: u16 = 1;

        let connection_error = self.mqtt_thread.has_connection_err();

        let area = frame.size();
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
            frame.render_widget(paragraph.alignment(Alignment::Center), header_area);
        }

        self.footer.draw(frame, footer_area, self);
        if let Some(connection_error) = connection_error {
            mqtt_error_widget::draw(
                frame,
                error_area,
                "MQTT Connection Error",
                &connection_error,
            );
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
                    frame,
                    details_area,
                    topic_history,
                    matches!(self.focus, ElementInFocus::Payload),
                );

                Rect {
                    width: x,
                    x: 0,
                    ..main_area
                }
            });

        let (topic_amount, tree_items) = history.to_tree_items();
        self.topic_overview.draw(
            frame,
            overview_area,
            topic_amount,
            tree_items,
            matches!(self.focus, ElementInFocus::TopicOverview),
        );
        drop(history);

        if let ElementInFocus::CleanRetainedPopup(topic) = &self.focus {
            clean_retained::draw_popup(frame, topic);
        }
    }
}
