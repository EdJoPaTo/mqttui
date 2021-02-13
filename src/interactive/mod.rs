use crate::interactive::app::App;
use crate::mqtt_history::HistoryArc;
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
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
    host: &str,
    port: u16,
    subscribe_topic: &str,
    history: HistoryArc,
) -> Result<(), Box<dyn Error>> {
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
                .unwrap_or_else(|| Duration::from_secs(0));
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
                    CEvent::Resize(_, _) => {}
                }
            }
            if last_tick.elapsed() >= TICK_RATE {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = App::new(host, port, subscribe_topic, history);

    terminal.clear()?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app).expect("failed to draw ui"))?;
        match rx.recv()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Enter | KeyCode::Char(' ') => app.on_toggle(),
                KeyCode::Left | KeyCode::Char('h') => app.on_left(),
                KeyCode::Down | KeyCode::Char('j') => app.on_down()?,
                KeyCode::Up | KeyCode::Char('k') => app.on_up()?,
                KeyCode::Right | KeyCode::Char('l') => app.on_right(),
                KeyCode::Tab | KeyCode::BackTab => app.on_tab()?,
                _ => {}
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
