use crate::app::{App, AppState, VERSION};
use crate::ui_cards::{draw_card_row, draw_detail_panel};
use crate::ui_modals::{draw_confirm_modal, draw_dirty_warning_modal, draw_help_modal};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub const COLOR_MAGENTA: Color = Color::Rgb(255, 0, 255);
pub const COLOR_CYAN: Color = Color::Rgb(0, 255, 255);
pub const COLOR_YELLOW: Color = Color::Rgb(255, 255, 0);
pub const COLOR_GREEN: Color = Color::Rgb(0, 255, 128);
pub const COLOR_ORANGE: Color = Color::Rgb(255, 128, 0);
pub const COLOR_PURPLE: Color = Color::Rgb(200, 100, 255);
pub const COLOR_BLUE: Color = Color::Rgb(100, 200, 255);
pub const COLOR_DIM: Color = Color::Rgb(60, 60, 60);
pub const COLOR_CARD_BG: Color = Color::Rgb(30, 30, 30);
pub const COLOR_BLACK: Color = Color::Black;
pub const COLOR_WHITE: Color = Color::White;

pub fn draw(f: &mut Frame, app: &App) {
    draw_snes_base(f, f.area(), app);
    match app.state {
        AppState::DirtyTreeWarning => draw_dirty_warning_modal(f, f.area()),
        AppState::ConfirmingCheckout => draw_confirm_modal(f, f.area(), app),
        AppState::ShowingHelp => draw_help_modal(f, f.area()),
        _ => {}
    }
}

fn draw_snes_base(f: &mut Frame, area: Rect, app: &App) {
    if app.state == AppState::ViewingDetail {
        draw_snes_layout_with_detail(f, area, app);
    } else {
        draw_snes_layout(f, area, app);
    }
}

fn draw_snes_layout(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);
    draw_title_bar(f, chunks[0]);
    draw_card_row(f, chunks[1], app);
    draw_control_hints(f, chunks[2], app);
    draw_status_bar(f, chunks[3], app);
}

fn draw_snes_layout_with_detail(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(42),
            Constraint::Length(5),
        ])
        .split(area);
    draw_title_bar(f, chunks[0]);
    draw_card_row(f, chunks[1], app);
    draw_detail_panel(f, chunks[2], app);
    draw_control_hints(f, chunks[3], app);
}

fn draw_title_bar(f: &mut Frame, area: Rect) {
    let title = vec![Line::from(vec![Span::styled(
        "  ═══ GIT TREK - A Journey Through Time ═══  ",
        Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
    )])];
    let widget = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_BLUE)).border_type(BorderType::Double));
    f.render_widget(widget, area);
}

fn draw_control_hints(f: &mut Frame, area: Rect, app: &App) {
    let hints = match app.state {
        AppState::Browsing => browsing_hints(),
        AppState::ViewingDetail => detail_hints(),
        _ => vec![Line::from("")],
    };
    let widget = Paragraph::new(hints)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_DIM)));
    f.render_widget(widget, area);
}

fn browsing_hints() -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::raw("Use "),
            Span::styled("[LEFT]", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled("[RIGHT]", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::raw(" to Navigate, "),
            Span::styled("[A]", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
            Span::raw(" to Inspect, "),
            Span::styled("[Q]", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw(" to Go Back"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    [P]", Style::default().fg(COLOR_PURPLE)),
            Span::raw(" Pin Anchor  "),
            Span::styled("[?]", Style::default().fg(COLOR_GREEN)),
            Span::raw(" Help"),
        ]),
    ]
}

fn detail_hints() -> Vec<Line<'static>> {
    vec![Line::from(vec![
        Span::raw("Press "),
        Span::styled("[ESC]", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
        Span::raw(" to Return, "),
        Span::styled("[C]", Style::default().fg(COLOR_MAGENTA).add_modifier(Modifier::BOLD)),
        Span::raw(" to Checkout, "),
        Span::styled("[T]", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
        Span::raw(" to Toggle Diff"),
    ])]
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let commit = &app.commits[app.idx];
    let oid_str = crate::git_ops::format_oid(commit.oid);
    let text = vec![Line::from(vec![
        Span::styled("⚡ ", Style::default().fg(COLOR_YELLOW)),
        Span::raw("Current Location: "),
        Span::styled(
            format!("COMMIT {} - {}", oid_str.to_uppercase(), commit.summary),
            Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
        ),
    ])];
    let widget = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_BLUE)));
    f.render_widget(widget, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}