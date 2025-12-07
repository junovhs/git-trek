use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    time::Duration,
};

mod app;
mod cli;
mod git_ops;
mod shell;
mod ui;
mod ui_cards;
mod ui_modals;

use crate::{
    app::{App, AppState, EVENT_POLL_MS},
    cli::Cli,
};

fn main() -> Result<()> {
    let cli = Cli::parse_checked()?;

    if cli.flags.dry_run() {
        return run_dry(&cli);
    }

    let mut terminal = setup_terminal().context("terminal setup")?;
    let mut app = App::new(cli).context("app setup")?;
    let res = run_app(&mut terminal, &mut app);

    restore_terminal(&mut terminal).context("terminal restore")?;
    if let Some(msg) = app.final_message {
        println!("{msg}");
    }
    res
}

fn run_dry(cli: &Cli) -> Result<()> {
    let app = App::new(cli.clone()).context("app setup")?;
    let backend = ratatui::backend::TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| ui::draw(f, &app)).context("dry-run render")?;
    println!("App initialized and rendered successfully");
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, app))?;
        app.maybe_do_pending_checkout()?;
        poll_and_handle(app)?;
    }
    Ok(())
}

fn poll_and_handle(app: &mut App) -> Result<()> {
    if !event::poll(Duration::from_millis(EVENT_POLL_MS))? {
        return Ok(());
    }
    if let Event::Key(key) = event::read()? {
        if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT {
            dispatch_key(app, key)?;
        }
    }
    Ok(())
}

fn dispatch_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match app.state {
        AppState::DirtyTreeWarning => handle_dirty(app, key.code),
        AppState::Browsing => handle_browse(app, key.code),
        AppState::ViewingDetail => { handle_detail(app, key.code); Ok(()) }
        AppState::ConfirmingCheckout => handle_confirm(app, key.code),
        AppState::ShowingHelp => { handle_help(app, key.code); Ok(()) }
    }
}

fn handle_dirty(app: &mut App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('s' | 'S') => app.handle_dirty_stash()?,
        KeyCode::Char('c' | 'C') => app.handle_dirty_continue(),
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => app.handle_dirty_quit(),
        _ => {}
    }
    Ok(())
}

fn handle_browse(app: &mut App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('q' | 'Q') => app.stop()?,
        KeyCode::Left | KeyCode::Char('a' | 'A') => app.move_sel(-1)?,
        KeyCode::Right | KeyCode::Char('d' | 'D') => app.move_sel(1)?,
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
        KeyCode::Enter | KeyCode::Char('c' | 'C') if !app.read_only => app.enter_confirm(),
        KeyCode::Char('t' | 'T') => app.diff_full = !app.diff_full,
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

fn handle_help(app: &mut App, code: KeyCode) {
    if matches!(code, KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q' | 'Q' | '?' | 'h' | 'H')) {
        app.toggle_help();
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}