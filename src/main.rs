use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

mod app;
mod cli;
mod data;
mod git_ops;
mod mouse;
mod views;

use crate::{app::App, cli::Cli, mouse::hit_test};

fn main() -> Result<()> {
    let cli = Cli::parse_checked()?;
    let mut terminal = setup_terminal().context("terminal setup")?;
    let mut app = App::new(cli).context("app init")?;

    let result = run_app(&mut terminal, &mut app);

    restore_terminal(&mut terminal)?;
    if let Some(msg) = &app.message {
        println!("{msg}");
    }
    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let mut hit_boxes = Vec::new();

    while !app.should_quit {
        terminal.draw(|f| {
            let result = views::draw(f, app);
            hit_boxes = result.hit_boxes;
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => handle_key(app, key.code)?,
                Event::Mouse(mouse) => handle_mouse(app, mouse, &hit_boxes),
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('q' | 'Q') => app.should_quit = true,
        KeyCode::Char('1') => app.set_view(views::ViewMode::Treemap),
        KeyCode::Char('2') => app.set_view(views::ViewMode::Heatmap),
        KeyCode::Char('3') => app.set_view(views::ViewMode::Minimap),
        KeyCode::Char('4') => app.set_view(views::ViewMode::River),
        KeyCode::Char('5') => app.set_view(views::ViewMode::Focus),
        KeyCode::Tab => app.next_view(),
        KeyCode::BackTab => app.prev_view(),
        KeyCode::Left => app.scroll_timeline(1),
        KeyCode::Right => app.scroll_timeline(-1),
        KeyCode::Char('r' | 'R') => app.restore_selected()?,
        KeyCode::Esc => app.selected_file = None,
        _ => {}
    }
    Ok(())
}

fn handle_mouse(app: &mut App, mouse: event::MouseEvent, hit_boxes: &[mouse::HitBox]) {
    app.mouse.update_position(mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::Moved => {
            let hit = hit_test(mouse.column, mouse.row, hit_boxes);
            app.mouse.set_hover(hit);
        }
        MouseEventKind::Down(_) => {
            let hit = hit_test(mouse.column, mouse.row, hit_boxes);
            app.handle_click(hit);
        }
        MouseEventKind::ScrollUp => app.scroll_timeline(-1),
        MouseEventKind::ScrollDown => app.scroll_timeline(1),
        _ => {}
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}