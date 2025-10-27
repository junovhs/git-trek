#![deny(warnings)]

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

mod cli;
mod app;
mod ui;
mod shell;

use app::{App, AppState};
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse_checked()?;
    let mut app = App::new(cli)?;
    setup_terminal()?;
    app.refresh_view()?; // initial load + optional cmd

    while !app.should_quit {
        draw(&app)?;
        if let Some(msg) = step(&mut app)? {
            app.final_message = Some(msg);
        }
    }

    teardown_terminal()?;
    if let Some(m) = app.final_message.take() {
        println!("{m}");
    }
    Ok(())
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode().context("enable_raw_mode")?;
    execute!(io::stdout(), EnterAlternateScreen, event::EnableMouseCapture).context("enter alt")?;
    Ok(())
}

fn teardown_terminal() -> Result<()> {
    disable_raw_mode().context("disable_raw_mode")?;
    execute!(io::stdout(), LeaveAlternateScreen, event::DisableMouseCapture).context("leave alt")?;
    Ok(())
}

fn draw(app: &App) -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("terminal new")?;
    ui::draw(&mut terminal, app)?;
    Ok(())
}

fn step(app: &mut App) -> Result<Option<String>> {
    if !event::poll(Duration::from_millis(app.event_poll_ms()))? {
        return Ok(None);
    }
    let Event::Key(key) = event::read()? else { return Ok(None) };
    if key.kind != KeyEventKind::Press { return Ok(None); }

    match app.state {
        AppState::Navigating => handle_nav(app, key.code),
        AppState::Detail => handle_detail(app, key.code),
        AppState::Confirm => handle_confirm(app, key.code),
    }
}

fn handle_nav(app: &mut App, code: KeyCode) -> Result<Option<String>> {
    use KeyCode::*;
    match code {
        Char('q') | Esc => app.stop().map(Some),
        Up | Char('w') => { app.move_sel(-1)?; Ok(None) }
        Down | Char('s') => { app.move_sel(1)?; Ok(None) }
        PageUp => { app.page(-1)?; Ok(None) }
        PageDown => { app.page(1)?; Ok(None) }
        Home => { app.home()?; Ok(None) }
        End => { app.end()?; Ok(None) }
        Enter => { app.enter_detail()?; Ok(None) }
        Char('p') | Char('P') => { app.pin_anchor(); Ok(None) }
        Char(c) if c.is_ascii_alphabetic() => { app.jump_letter(c)?; Ok(None) }
        _ => Ok(None),
    }
}

fn handle_detail(app: &mut App, code: KeyCode) -> Result<Option<String>> {
    use KeyCode::*;
    match code {
        Esc | Backspace | Char('q') => { app.exit_detail(); Ok(None) }
        Enter | Char('c') => { app.enter_confirm(); Ok(None) }
        Char('d') | Char('D') => { app.toggle_diff(); Ok(None) }
        Char('p') | Char('P') => { app.mark_manual(true); Ok(None) }
        Char('f') | Char('F') => { app.mark_manual(false); Ok(None) }
        _ => Ok(None),
    }
}

fn handle_confirm(app: &mut App, code: KeyCode) -> Result<Option<String>> {
    use KeyCode::*;
    match code {
        Char('y') | Char('Y') => app.checkout().map(Some),
        Char('n') | Char('N') | Esc | Backspace => { app.exit_confirm(); Ok(None) }
        _ => Ok(None),
    }
}
