use crate::interactive::app::App;
use crate::mqtt_history::HistoryArc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod ui;

enum Event<I> {
    Input(I),
    Tick,
}

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

    let tick_rate = Duration::from_millis(500);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
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
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char(c) => match c {
                    'q' => app.should_quit = true,
                    ' ' => app.on_toggle(),
                    'h' => app.on_left(),
                    'j' => app.on_down()?,
                    'k' => app.on_up()?,
                    'l' => app.on_right(),
                    _ => {}
                },
                KeyCode::Enter => app.on_toggle(),
                KeyCode::Left => app.on_left(),
                KeyCode::Up => app.on_up()?,
                KeyCode::Right => app.on_right(),
                KeyCode::Down => app.on_down()?,
                _ => {}
            },
            Event::Tick => {}
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
