use std::io::stdout;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyModifiers, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use json::JsonValue;
use rumqttc::{Client, Connection};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Frame;
use tui::{backend::CrosstermBackend, Terminal};
use tui_tree_widget::flatten;

use crate::cli::Broker;
use crate::interactive::ui::{CursorMove, Event};
use crate::json_view::root_tree_items_from_json;

mod clear_retained;
mod details;
mod info_header;
mod mqtt_history;
mod mqtt_thread;
mod topic_overview;
mod topic_tree_entry;
mod ui;

const TICK_RATE: Duration = Duration::from_millis(500);

enum ElementInFocus {
    TopicOverview,
    JsonPayload,
    CleanRetainedPopup(String),
}

pub fn show(
    client: Client,
    connection: Connection,
    broker: &Broker,
    subscribe_topic: &str,
) -> anyhow::Result<()> {
    let mqtt_thread =
        mqtt_thread::MqttThread::new(client, connection, subscribe_topic.to_string())?;
    let mut app = App::new(broker, subscribe_topic, mqtt_thread);

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = TICK_RATE
                .checked_sub(last_tick.elapsed())
                .unwrap_or_default();
            if crossterm::event::poll(timeout).unwrap() {
                match crossterm::event::read().unwrap() {
                    CEvent::Key(key) => {
                        tx.send(Event::Key(key)).unwrap();
                    }
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

fn terminal_draw<B>(app: &mut App, terminal: &mut Terminal<B>) -> anyhow::Result<()>
where
    B: Backend,
{
    let mut draw_error = None;
    terminal.draw(|f| {
        if let Err(error) = app.draw(f) {
            draw_error = Some(error);
        }
    })?;
    draw_error.map_or(Ok(()), Err)
}

fn main_loop<B>(
    app: &mut App,
    rx: &Receiver<Event>,
    terminal: &mut Terminal<B>,
) -> anyhow::Result<()>
where
    B: Backend,
{
    loop {
        terminal_draw(app, terminal)?;
        match rx.recv()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    break;
                }
                KeyCode::Enter | KeyCode::Char(' ') => app.on_confirm()?,
                KeyCode::Left | KeyCode::Char('h') => app.on_left(),
                KeyCode::Down | KeyCode::Char('j') => app.on_down()?,
                KeyCode::Up | KeyCode::Char('k') => app.on_up()?,
                KeyCode::Right | KeyCode::Char('l') => app.on_right(),
                KeyCode::Tab | KeyCode::BackTab => app.on_tab()?,
                KeyCode::Backspace | KeyCode::Delete => app.on_delete(),
                _ => app.on_other(),
            },
            Event::MouseClick { column, row } => app.on_click(column, row)?,
            Event::MouseScrollDown => app.on_down()?,
            Event::MouseScrollUp => app.on_up()?,
            Event::Tick => {}
        }
    }
    Ok(())
}

struct App {
    details: details::Details,
    focus: ElementInFocus,
    info_header: info_header::InfoHeader,
    mqtt_thread: mqtt_thread::MqttThread,
    topic_overview: topic_overview::TopicOverview,
}

impl App {
    fn new(broker: &Broker, subscribe_topic: &str, mqtt_thread: mqtt_thread::MqttThread) -> Self {
        Self {
            details: details::Details::default(),
            focus: ElementInFocus::TopicOverview,
            info_header: info_header::InfoHeader::new(broker, subscribe_topic),
            mqtt_thread,
            topic_overview: topic_overview::TopicOverview::default(),
        }
    }

    fn get_json_of_current_topic(&self) -> anyhow::Result<Option<JsonValue>> {
        if let Some(topic) = self.topic_overview.get_selected() {
            let json = self
                .mqtt_thread
                .get_history()?
                .get_last(topic)
                .and_then(|last| last.payload.as_optional_json().cloned());
            Ok(json)
        } else {
            Ok(None)
        }
    }

    fn on_up(&mut self) -> anyhow::Result<()> {
        const DIRECTION: CursorMove = CursorMove::RelativeUp;
        match self.focus {
            ElementInFocus::TopicOverview => {
                let tree_items = self.mqtt_thread.get_history()?.to_tte();
                self.topic_overview.change_selected(&tree_items, DIRECTION);
            }
            ElementInFocus::JsonPayload => {
                let json = self.get_json_of_current_topic()?.unwrap_or(JsonValue::Null);
                let items = root_tree_items_from_json(&json);
                self.details.json_view.key_up(&items);
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }

        Ok(())
    }

    fn on_down(&mut self) -> anyhow::Result<()> {
        const DIRECTION: CursorMove = CursorMove::RelativeDown;
        match self.focus {
            ElementInFocus::TopicOverview => {
                let tree_items = self.mqtt_thread.get_history()?.to_tte();
                self.topic_overview.change_selected(&tree_items, DIRECTION);
            }
            ElementInFocus::JsonPayload => {
                let json = self.get_json_of_current_topic()?.unwrap_or(JsonValue::Null);
                let items = root_tree_items_from_json(&json);
                self.details.json_view.key_down(&items);
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }

        Ok(())
    }

    fn on_right(&mut self) {
        match self.focus {
            ElementInFocus::TopicOverview => {
                self.topic_overview.open();
            }
            ElementInFocus::JsonPayload => {
                self.details.json_view.key_right();
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }
    }

    fn on_left(&mut self) {
        match self.focus {
            ElementInFocus::TopicOverview => {
                self.topic_overview.close();
            }
            ElementInFocus::JsonPayload => {
                self.details.json_view.key_left();
            }
            ElementInFocus::CleanRetainedPopup(_) => self.focus = ElementInFocus::TopicOverview,
        }
    }

    fn on_confirm(&mut self) -> anyhow::Result<()> {
        match &self.focus {
            ElementInFocus::TopicOverview => {
                self.topic_overview.toggle();
            }
            ElementInFocus::JsonPayload => {
                self.details.json_view.toggle_selected();
            }
            ElementInFocus::CleanRetainedPopup(topic) => {
                let base = self.mqtt_thread.get_mqtt_options();
                clear_retained::do_clear(base, topic)?;
                self.focus = ElementInFocus::TopicOverview;
            }
        }
        Ok(())
    }

    fn on_tab(&mut self) -> anyhow::Result<()> {
        let is_json_on_topic = self.get_json_of_current_topic()?.is_some();
        self.focus = if is_json_on_topic {
            match self.focus {
                ElementInFocus::TopicOverview => ElementInFocus::JsonPayload,
                ElementInFocus::JsonPayload | ElementInFocus::CleanRetainedPopup(_) => {
                    ElementInFocus::TopicOverview
                }
            }
        } else {
            ElementInFocus::TopicOverview
        };
        Ok(())
    }

    fn on_click(&mut self, column: u16, row: u16) -> anyhow::Result<()> {
        if let Some(index) = self.topic_overview.index_of_click(column, row) {
            let tree_items = self.mqtt_thread.get_history()?.to_tte();
            let changed = self
                .topic_overview
                .change_selected(&tree_items, CursorMove::Absolute(index));
            if !changed {
                self.topic_overview.toggle();
            }
            self.focus = ElementInFocus::TopicOverview;
        }

        if let Some(index) = self.details.json_index_of_click(column, row) {
            let json = self.get_json_of_current_topic()?.unwrap_or(JsonValue::Null);
            let items = root_tree_items_from_json(&json);
            let opened = self.details.json_view.get_all_opened();
            let flattened = flatten(&opened, &items);
            if let Some(picked) = flattened.get(index) {
                if picked.identifier == self.details.json_view.selected() {
                    self.details.json_view.toggle_selected();
                } else {
                    self.details.json_view.select(picked.identifier.clone());
                }
                self.focus = ElementInFocus::JsonPayload;
            }
        }
        Ok(())
    }

    fn on_delete(&mut self) {
        if matches!(self.focus, ElementInFocus::TopicOverview) {
            if let Some(topic) = self.topic_overview.get_selected() {
                self.focus = ElementInFocus::CleanRetainedPopup(topic.to_string());
            }
        }
    }

    fn on_other(&mut self) {
        if matches!(self.focus, ElementInFocus::CleanRetainedPopup(_)) {
            self.focus = ElementInFocus::TopicOverview;
        }
    }

    fn draw<B>(&mut self, f: &mut Frame<B>) -> anyhow::Result<()>
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .constraints([Constraint::Length(2 + 3), Constraint::Min(8)].as_ref())
            .split(f.size());
        self.info_header.draw(
            f,
            chunks[0],
            self.mqtt_thread.has_connection_err().unwrap(),
            self.topic_overview.get_selected(),
        );

        let main_area = chunks[1];
        let history = self.mqtt_thread.get_history()?;
        let tree_items = history.to_tte();

        #[allow(clippy::option_if_let_else)]
        let overview_area = if let Some(selected_topic) = self.topic_overview.get_selected() {
            if let Some(topic_history) = history.get(selected_topic) {
                let chunks = Layout::default()
                    .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
                    .direction(Direction::Horizontal)
                    .split(main_area);

                self.details.draw(
                    f,
                    chunks[1],
                    topic_history,
                    matches!(self.focus, ElementInFocus::JsonPayload),
                );

                chunks[0]
            } else {
                main_area
            }
        } else {
            main_area
        };

        self.topic_overview.ensure_state(&history);
        self.topic_overview.draw(
            f,
            overview_area,
            &tree_items,
            matches!(self.focus, ElementInFocus::TopicOverview),
        );

        if let ElementInFocus::CleanRetainedPopup(topic) = &self.focus {
            clear_retained::draw_popup(f, topic);
        }
        Ok(())
    }
}
