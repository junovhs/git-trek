#![deny(warnings)]

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

mod app;
mod cli;
mod ui;

use crate::app::{AppState, App, AnimationDirection, ANIMATION_FRAME_MS, UI_WIDTH};
use crate::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse_checked()?;
    let mut app = App::new(cli)?;
    let mut terminal = setup_terminal()?;

    let res = run_app(&mut terminal, &mut app);

    teardown_terminal()?;

    if let Err(err) = res {
        println!("Error: {err:?}");
    } else if let Some(msg) = app.final_message {
        println!("{msg}");
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    while !app.should_quit {
        let size = terminal.size()?;
        app.terminal_too_small = size.width < UI_WIDTH;
        
        terminal.draw(|f| ui::draw(f, app))?;

        if app.animation.is_some() {
            app.on_tick();
            std::thread::sleep(Duration::from_millis(ANIMATION_FRAME_MS));
        } else {
            if app.idx != app.last_checkout_idx {
                app.update_checkout()?;
            }

            if event::poll(Duration::from_millis(1000))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(app, key.code, key.modifiers)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
    if app.terminal_too_small {
        if code == KeyCode::Char('q') { app.should_quit = true; }
        return Ok(());
    }
    
    if code == KeyCode::Char('q') || (code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL) {
        return app.stop();
    }
    
    if app.state == AppState::Scanning {
        match code {
            KeyCode::Up | KeyCode::Char('k') => app.shift_target(AnimationDirection::Up),
            KeyCode::Down | KeyCode::Char('j') => app.shift_target(AnimationDirection::Down),
            KeyCode::Enter => app.toggle_inspect(),
            _ => {}
        }
    } else if app.state == AppState::Inspect {
        match code {
            KeyCode::Enter | KeyCode::Esc => app.toggle_inspect(),
            _ => {}
        }
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(Into::into)
}

fn teardown_terminal() -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(io::stdout(), LeaveAlternateScreen).context("failed to leave alternate screen")?;
    Ok(())
}