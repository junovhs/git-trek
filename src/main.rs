// FILE: git-trek/src/main.rs
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    time::Duration,
};

mod app;
mod cli;
mod shell;
mod ui;

use crate::{
    app::{App, AppState, EVENT_POLL_MS},
    cli::Cli,
};

fn main() -> Result<()> {
    let cli = Cli::parse_checked()?;
    let mut terminal = setup_terminal().context("terminal setup")?;
    let mut app = App::new(cli).context("app setup")?;
    let res = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal).context("terminal restore")?;
    
    if let Some(msg) = app.final_message {
        println!("{msg}");
    }
    
    if let Err(e) = res {
        eprintln!("Error: {e:?}");
        return Err(e);
    }
    
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, app))?;
        
        if event::poll(Duration::from_millis(EVENT_POLL_MS))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Browsing => handle_browsing(app, key.code)?,
                    AppState::ViewingDetail => handle_detail(app, key.code),
                    AppState::ConfirmingCheckout => handle_confirm(app, key.code)?,
                    AppState::ShowingHelp => {
                        if matches!(key.code, KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q' | '?' | 'h' | 'H')) {
                            app.toggle_help();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_browsing(app: &mut App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Esc | KeyCode::Char('q' | 'Q') => app.stop()?,
        KeyCode::Up | KeyCode::Char('k' | 'K' | 'w' | 'W') => app.move_sel(-1)?,
        KeyCode::Down | KeyCode::Char('j' | 'J' | 's' | 'S') => app.move_sel(1)?,
        KeyCode::PageUp => app.page(-1)?,
        KeyCode::PageDown => app.page(1)?,
        KeyCode::Home => app.home()?,
        KeyCode::End => app.end()?,
        KeyCode::Enter => app.enter_detail(),
        KeyCode::Char('p' | 'P') => app.pin_anchor(),
        KeyCode::Char('?' | 'h' | 'H') => app.toggle_help(),
        _ => {}
    }
    Ok(())
}

fn handle_detail(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q' | 'Q') => app.exit_detail(),
        KeyCode::Enter | KeyCode::Char('c' | 'C') => app.enter_confirm(),
        KeyCode::Char('d' | 'D') => app.diff_full = !app.diff_full,
        KeyCode::Char('p' | 'P') => app.mark_manual(true),
        KeyCode::Char('f' | 'F') => app.mark_manual(false),
        KeyCode::Char('?' | 'h' | 'H') => app.toggle_help(),
        _ => {}
    }
}

fn handle_confirm(app: &mut App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('y' | 'Y') => app.checkout()?,
        KeyCode::Char('n' | 'N') | KeyCode::Esc | KeyCode::Backspace => app.exit_confirm(),
        KeyCode::Char('?' | 'h' | 'H') => app.toggle_help(),
        _ => {}
    }
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}