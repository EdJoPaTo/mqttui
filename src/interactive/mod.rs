use std::error::Error;
use std::io::stdout;
use std::sync::mpsc;
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
use rumqttc::{Client, Connection};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::Frame;
use tui::{backend::CrosstermBackend, Terminal};

use crate::cli::Broker;
use crate::interactive::app::{App, ElementInFocus};
use crate::interactive::mqtt_thread::MqttThread;
use crate::interactive::ui::{Event, MousePosition};

mod app;
mod clear_retained;
mod details;
mod info_header;
mod mqtt_history;
mod mqtt_thread;
mod topic_overview;
mod topic_tree_entry;
mod ui;

const TICK_RATE: Duration = Duration::from_millis(500);

pub fn show(
    client: Client,
    connection: Connection,
    broker: &Broker,
    subscribe_topic: &str,
) -> Result<(), Box<dyn Error>> {
    let mqtt_thread = MqttThread::new(client, connection, subscribe_topic.to_string())?;
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
                            .send(Event::MouseClick(MousePosition {
                                column: mouse.column,
                                row: mouse.row,
                            }))
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

    loop {
        terminal.draw(|f| draw(f, &mut app).expect("failed to draw ui"))?;
        match rx.recv()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.should_quit = true;
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
            Event::MouseClick(position) => app.on_click(position.row, position.column)?,
            Event::MouseScrollDown => app.on_down()?,
            Event::MouseScrollUp => app.on_up()?,
            Event::Tick => {}
        }
        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    let chunks = Layout::default()
        .constraints([Constraint::Length(2 + 3), Constraint::Min(8)].as_ref())
        .split(f.size());
    app.info_header.draw(
        f,
        chunks[0],
        app.mqtt_thread.has_connection_err().unwrap(),
        app.topic_overview.get_selected(),
    );
    draw_main(f, chunks[1], app)?;
    if let ElementInFocus::CleanRetainedPopup(topic) = &app.focus {
        clear_retained::draw_popup(f, topic);
    }
    Ok(())
}

fn draw_main<B>(f: &mut Frame<B>, area: Rect, app: &mut App) -> Result<(), Box<dyn Error>>
where
    B: Backend,
{
    let history = app.mqtt_thread.get_history()?;
    let tree_items = history.to_tte();

    app.topic_overview.ensure_state(&history);

    #[allow(clippy::option_if_let_else)]
    let overview_area = if let Some(selected_topic) = app.topic_overview.get_selected() {
        if let Some(topic_history) = history.get(selected_topic) {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
                .direction(Direction::Horizontal)
                .split(area);

            app.details.draw(
                f,
                chunks[1],
                topic_history,
                matches!(app.focus, ElementInFocus::JsonPayload),
            );

            chunks[0]
        } else {
            area
        }
    } else {
        area
    };

    app.topic_overview.draw(
        f,
        overview_area,
        &tree_items,
        matches!(app.focus, ElementInFocus::TopicOverview),
    );
    Ok(())
}
