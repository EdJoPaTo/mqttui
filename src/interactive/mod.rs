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
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;
use tui::{backend::CrosstermBackend, Terminal};

use crate::cli::Broker;
use crate::interactive::app::{App, ElementInFocus};
use crate::interactive::mqtt_thread::MqttThread;

mod app;
mod clear_retained;
mod details;
mod mqtt_history;
mod mqtt_thread;
mod topic_overview;
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

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) -> Result<(), Box<dyn Error>> {
    let chunks = Layout::default()
        .constraints([Constraint::Length(2 + 3), Constraint::Min(8)].as_ref())
        .split(f.size());
    draw_info_header(f, chunks[0], app);
    draw_main(f, chunks[1], app)?;
    if let ElementInFocus::CleanRetainedPopup(topic) = &app.focus {
        clear_retained::draw_popup(f, topic);
    }
    Ok(())
}

fn draw_info_header<B>(f: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let broker = format!("MQTT Broker: {:?}", app.broker);
    let subscribed = format!("Subscribed Topic: {}", app.subscribe_topic);
    let mut text = vec![Spans::from(broker), Spans::from(subscribed)];

    if let Some(err) = app.mqtt_thread.has_connection_err().unwrap() {
        text.push(Spans::from(Span::styled(
            format!("MQTT Connection Error: {}", err),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
        )));
    }

    if let Some(topic) = app.topic_overview.get_selected() {
        text.push(Spans::from(format!("Selected Topic: {}", topic)));
    }

    let title = format!("MQTT TUI {}", env!("CARGO_PKG_VERSION"));
    let block = Block::default().borders(Borders::ALL).title(title);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
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

            details::draw(
                f,
                chunks[1],
                topic_history,
                matches!(app.focus, ElementInFocus::JsonPayload),
                &mut app.json_view_state,
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
