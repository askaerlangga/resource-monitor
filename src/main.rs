mod app;
mod cli;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io, time::{Duration, Instant}};

fn main() -> Result<()> {
    let config = cli::parse_args();
    let tick_rate = Duration::from_millis(config.tick_rate_ms);
    let no_mouse = config.no_mouse;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if no_mouse {
        execute!(stdout, EnterAlternateScreen)?;
    } else {
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&config);

    let res = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    if no_mouse {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    } else {
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    }
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> Result<()>
where
    B::Error: std::error::Error + Send + Sync + 'static,
{
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if app.show_kill_confirm {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_kill(),
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_kill(),
                        _ => {}
                    }
                } else if app.show_sort_menu {
                    match key.code {
                        KeyCode::Char('1') => {
                            app.sort_by = app::SortBy::Cpu;
                            app.show_sort_menu = false;
                        }
                        KeyCode::Char('2') => {
                            app.sort_by = app::SortBy::Memory;
                            app.show_sort_menu = false;
                        }
                        KeyCode::Char('3') => {
                            app.sort_by = app::SortBy::DiskRead;
                            app.show_sort_menu = false;
                        }
                        KeyCode::Char('4') => {
                            app.sort_by = app::SortBy::DiskWrite;
                            app.show_sort_menu = false;
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('s') => {
                            app.show_sort_menu = false;
                        }
                        _ => {}
                    }
                } else if app.search_mode {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            app.search_mode = false;
                        }
                        KeyCode::Backspace => {
                            app.search_pop();
                        }
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                            app.search_clear();
                        }
                        KeyCode::Down | KeyCode::Up => {
                            if key.code == KeyCode::Down { app.next_process(); } else { app.previous_process(); }
                        }
                        KeyCode::Char(c) => {
                            app.search_push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::Char('1') => app.active_tab = 0,
                        KeyCode::Char('2') => app.active_tab = 1,
                        KeyCode::Char('3') => app.active_tab = 2,
                        KeyCode::Char('4') => app.active_tab = 3,
                        KeyCode::Char('5') => app.active_tab = 4,
                        KeyCode::Down | KeyCode::Char('j') => app.next_process(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous_process(),
                        KeyCode::Char('x') => app.kill_selected(),
                        KeyCode::Char('s') => {
                            if app.active_tab == 0 {
                                app.show_sort_menu = true;
                            }
                        }
                        KeyCode::Char('/') => {
                            if app.active_tab == 0 {
                                app.search_mode = true;
                            }
                        }
                        KeyCode::Char('e') => {
                            let _ = app.export_snapshot();
                        }
                        KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                            && app.active_tab == 0
                            && !app.search_query.is_empty() =>
                        {
                            app.search_clear();
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }
}
