use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;
use crate::cli::Cli;
use crate::mouse::{hit_test, HitBox};
use crate::views::ViewMode;

mod app;
mod cli;
mod data;
mod error;
mod git_ops;
mod mouse;
mod views;

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    if cli.check {
        App::new(&cli)?;
        println!("git-trek initialized successfully");
        return Ok(());
    }

    let mut terminal = setup_terminal()?;
    let mut app = App::new(&cli)?;

    let result = run_app(&mut terminal, &mut app);

    restore_terminal(&mut terminal)?;

    if let Some(msg) = app.message() {
        println!("{msg}");
    }

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let mut hit_boxes: Vec<HitBox> = Vec::new();

    while !app.should_quit() {
        terminal.draw(|f| {
            let render = views::draw(f, app);
            hit_boxes = render.hit_boxes;
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => handle_key(app, key)?,
                Event::Mouse(mouse) => handle_mouse(app, mouse, &hit_boxes),
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(mode) = view_from_key(key.code) {
        app.set_view(mode);
        return Ok(());
    }

    if handle_navigation(app, key.code) {
        return Ok(());
    }

    match key.code {
        KeyCode::Char('q' | 'Q') => {
            app.quit();
        }
        KeyCode::Char('r' | 'R') => {
            app.restore_selected()?;
        }
        KeyCode::Esc => {
            app.clear_selection();
        }
        _ => {}
    }

    Ok(())
}

fn view_from_key(code: KeyCode) -> Option<ViewMode> {
    match code {
        KeyCode::Char('1') => Some(ViewMode::Terrain),
        KeyCode::Char('2') => Some(ViewMode::Seismic),
        KeyCode::Char('3') => Some(ViewMode::Strata),
        KeyCode::Char('4') => Some(ViewMode::Flow),
        KeyCode::Char('5') => Some(ViewMode::Constellation),
        KeyCode::Char('6') => Some(ViewMode::Surgery),
        _ => None,
    }
}

fn handle_navigation(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Left => {
            app.scroll_timeline(1);
            true
        }
        KeyCode::Right => {
            app.scroll_timeline(-1);
            true
        }
        KeyCode::Tab => {
            app.next_view();
            true
        }
        KeyCode::BackTab => {
            app.prev_view();
            true
        }
        _ => false,
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent, hit_boxes: &[HitBox]) {
    app.mouse_mut().update_position(mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::Moved => {
            let target = hit_test(mouse.column, mouse.row, hit_boxes);
            app.mouse_mut().update_hover(target);
        }
        MouseEventKind::Down(_) => {
            let target = hit_test(mouse.column, mouse.row, hit_boxes);
            app.handle_click(target);
        }
        MouseEventKind::ScrollUp => {
            app.scroll_timeline(-1);
        }
        MouseEventKind::ScrollDown => {
            app.scroll_timeline(1);
        }
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
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
