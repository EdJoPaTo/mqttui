use std::error::Error;
use std::io::stdout;
use std::sync::mpsc;
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
use rumqttc::{Client, Connection};
use tui::{backend::CrosstermBackend, Terminal};

use crate::cli::Broker;
use crate::interactive::app::App;
use crate::interactive::mqtt_thread::MqttThread;

mod app;
mod mqtt_history;
mod mqtt_thread;
mod topic_tree_entry;
mod ui;

enum MouseScrollDirection {
    Up,
    Down,
}

struct MousePosition {
    column: u16,
    row: u16,
}

enum Event {
    Key(KeyEvent),
    MouseClick(MousePosition),
    MouseScroll(MouseScrollDirection),
    Tick,
}

const TICK_RATE: Duration = Duration::from_millis(500);

pub fn show(
    client: Client,
    connection: Connection,
    broker: Broker,
    subscribe_topic: String,
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
                        MouseEventKind::ScrollUp => tx
                            .send(Event::MouseScroll(MouseScrollDirection::Up))
                            .unwrap(),
                        MouseEventKind::ScrollDown => tx
                            .send(Event::MouseScroll(MouseScrollDirection::Down))
                            .unwrap(),
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
        terminal.draw(|f| ui::draw(f, &mut app).expect("failed to draw ui"))?;
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
            Event::MouseScroll(direction) => match direction {
                MouseScrollDirection::Up => app.on_up()?,
                MouseScrollDirection::Down => app.on_down()?,
            },
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
