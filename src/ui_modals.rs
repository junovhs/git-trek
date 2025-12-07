use crate::app::{App, VERSION};
use crate::git_ops::format_oid;
use crate::ui::{
    centered_rect, COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_ORANGE, COLOR_WHITE, COLOR_YELLOW,
};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame, layout::Rect,
};

pub fn draw_dirty_warning_modal(f: &mut Frame, area: Rect) {
    let modal = centered_rect(70, 50, area);
    f.render_widget(Clear, modal);

    let lines = vec![
        Line::from(""),
        warning_header("WORKING TREE IS DIRTY"),
        Line::from(""),
        Line::from(Span::styled("You have uncommitted changes.", Style::default().fg(COLOR_WHITE))),
        Line::from(""),
        Line::from(Span::styled("Choose how to proceed:", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        option_line("[S] STASH", COLOR_GREEN),
        Line::from(vec![Span::raw("    Save changes temporarily")]),
        Line::from(""),
        option_line("[C] CONTINUE", COLOR_YELLOW),
        Line::from(vec![Span::raw("    Browse in read-only mode")]),
        Line::from(""),
        option_line("[Q] QUIT", COLOR_ORANGE),
        Line::from(vec![Span::raw("    Exit without changes")]),
    ];
    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(modal_block(" ⚠ WARNING ⚠ ", COLOR_ORANGE));
    f.render_widget(widget, modal);
}

pub fn draw_confirm_modal(f: &mut Frame, area: Rect, app: &App) {
    let modal = centered_rect(60, 30, area);
    f.render_widget(Clear, modal);

    let oid_display = format_oid(app.commits[app.idx].oid);
    let lines = vec![
        Line::from(""),
        warning_header("CHECKOUT COMMIT"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(COLOR_CYAN)),
            Span::styled(oid_display, Style::default().fg(COLOR_MAGENTA).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("This will detach HEAD.", Style::default().fg(COLOR_WHITE))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Proceed? ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("[Y/N]", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
        ]),
    ];
    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(modal_block(" CONFIRM ", COLOR_ORANGE));
    f.render_widget(widget, modal);
}

pub fn draw_help_modal(f: &mut Frame, area: Rect) {
    let modal = centered_rect(70, 60, area);
    f.render_widget(Clear, modal);

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("═══ GIT TREK CONTROLS ═══", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        section_header("CARD NAVIGATION"),
        Line::from(""),
        help_line("← →, A D      ", "Navigate cards"),
        help_line("ENTER         ", "View details"),
        help_line("P             ", "Pin anchor"),
        help_line("Q             ", "Quit"),
        Line::from(""),
        section_header("DETAIL VIEW"),
        Line::from(""),
        help_line("ESC, Q        ", "Back to cards"),
        help_line("C             ", "Checkout"),
        help_line("T             ", "Toggle diff"),
        Line::from(""),
        Line::from(vec![Span::styled(format!("git-trek v{VERSION}"), Style::default().fg(crate::ui::COLOR_DIM))]),
    ];
    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(modal_block(" HELP ", COLOR_GREEN));
    f.render_widget(widget, modal);
}

fn warning_header(text: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("⚠ ", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(text.to_string(), Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(" ⚠", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
    ])
}

fn option_line(text: &str, color: ratatui::style::Color) -> Line<'static> {
    Line::from(vec![Span::styled(text.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD))])
}

fn section_header(text: &str) -> Line<'static> {
    Line::from(vec![Span::styled(text.to_string(), Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::UNDERLINED))])
}

fn help_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {key}"), Style::default().fg(COLOR_MAGENTA)),
        Span::raw(desc.to_string()),
    ])
}

fn modal_block(title: &str, color: ratatui::style::Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .border_type(BorderType::Double)
        .title(Span::styled(title.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD)))
}