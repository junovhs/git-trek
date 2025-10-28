use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use crate::app::{AnimationState, App, AppState, UI_WIDTH};

// --- Main Drawing Entry Point ---

pub fn draw(f: &mut Frame<'_>, app: &mut App) {
    if app.terminal_too_small {
        draw_too_small(f);
        return;
    }

    match app.state {
        AppState::Scanning => draw_scanning_view(f, app),
        AppState::Inspect => draw_inspect_view(f, app),
    }
}

fn draw_scanning_view(f: &mut Frame<'_>, app: &mut App) {
    let centered_rect = get_centered_rect(f.area());

    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ]).split(centered_rect);

    draw_header(f, app, outer_layout[0]);
    draw_command_console(f, app, outer_layout[2]);
    
    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ]).split(outer_layout[1]);

    let list_area = body_layout[0];
    app.adjust_scroll(list_area.height as usize);
    
    draw_commit_list(f, app, list_area);
    draw_analysis_panel(f, app, body_layout[1]);
}

fn draw_inspect_view(f: &mut Frame<'_>, app: &mut App) {
    let centered_rect = get_centered_rect(f.area());
    let block = Block::default()
        .title(" [LOG DETAIL] ")
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner_area = block.inner(centered_rect);
    f.render_widget(block, centered_rect);

    let paragraph = Paragraph::new(app.detail.message.as_str())
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
        
    f.render_widget(paragraph, inner_area);

    let key_text = Line::from(vec![
        key_style("[Enter/Esc]"), Span::raw(" RETURN TO SCANNER"),
    ]).alignment(Alignment::Center);

    let key_area = Rect { y: centered_rect.bottom() - 2, height: 1, ..centered_rect };
    f.render_widget(Paragraph::new(key_text), key_area);
}


fn draw_header(f: &mut Frame<'_>, app: &App, area: Rect) {
    f.render_widget(Block::default().style(Style::default().bg(Color::Black)), area);
    let top_border = "▄".repeat(area.width as usize);
    let bottom_border = "▀".repeat(area.width as usize);

    let sync_percent = if app.opts.limit > 0 { (app.commits.len() as f32 / app.opts.limit as f32) * 100.0 } else { 100.0 };
    
    let (drive_state, kernel_state) = if app.animation.is_some() {
        (Span::styled("SEEKING", Style::default().fg(Color::Yellow)), Span::styled("ROTATING", Style::default().fg(Color::Yellow)))
    } else {
        (Span::raw("IDLE"), Span::raw("ONLINE"))
    };

    let content_spans = Line::from(vec![
        Span::styled(format!(" █ GIT-TREK v{} ", env!("CARGO_PKG_VERSION")), Style::default().bg(Color::White).fg(Color::Black)),
        Span::styled(format!(" █  DRUM SYNC: {:.0}%  ", sync_percent), Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw("█  DRIVE: "), drive_state, Span::raw("  "),
        Span::raw("█  KERNEL: "), kernel_state, Span::raw("  "),
        Span::raw("█"),
    ]);

    f.render_widget(Paragraph::new(top_border), Rect::new(area.x, area.y, area.width, 1));
    f.render_widget(Paragraph::new(content_spans), Rect::new(area.x, area.y + 1, area.width, 1));
    f.render_widget(Paragraph::new(bottom_border), Rect::new(area.x, area.y + 2, area.width, 1));
}

fn draw_commit_list(f: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default().borders(ratatui::widgets::Borders::ALL).border_style(Style::default().fg(Color::DarkGray));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    let visible_range = app.list_scroll_offset..app.commits.len().min(app.list_scroll_offset + inner_area.height as usize);

    for i in visible_range {
        let commit = &app.commits[i];
        if i == app.idx {
            let is_animating = app.animation.is_some();
            lines.push(Line::from(Span::styled(if is_animating {"┌─[ SCANNING ]─"} else {"┌──────────────"}, Style::default().fg(Color::Yellow))));
            lines.push(Line::from(vec![
                Span::styled("│ > ", Style::default().fg(Color::Yellow)),
                Span::styled(commit.summary.clone(), Style::default().fg(Color::White).bold()),
            ]));
            lines.push(Line::from(Span::styled(if is_animating {"└─[ LOCK-IN... ]─"} else {"└──────────────"}, Style::default().fg(Color::Yellow))));
        } else {
            lines.push(Line::from(vec![Span::raw("  "), Span::styled(commit.summary.clone(), Style::default().fg(Color::DarkGray))]));
            lines.push(Line::raw(""));
            lines.push(Line::raw(""));
        }
    }
    
    f.render_widget(Paragraph::new(lines), inner_area);
}


fn draw_analysis_panel(f: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default()
        .title(" [ANALYSIS] ")
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let content = if let Some(AnimationState{frame: 0, ..}) = app.animation {
        vec![
            Line::from("TARGET LOST..."),
            Line::from("ID:      ▒▒▒▒▒▒▒▒▒▒▒▒"),
            Line::from("AUTHOR:  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒"),
            Line::from("MESSAGE:"),
            Line::from("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒"),
        ]
    } else {
        let author = app.detail.author.split('<').next().unwrap_or("").trim();
        vec![
            Line::from(Span::styled("TARGET LOCKED", Style::default().fg(Color::Cyan).bold())),
            Line::from(vec![Span::raw("ID:      "), Span::styled(app.detail.hash[..12].to_string(), Style::default().fg(Color::Yellow))]),
            Line::from(vec![Span::raw("AUTHOR:  "), Span::styled(author, Style::default().fg(Color::White))]),
            Line::from(""),
            Line::from("MESSAGE:"),
            Line::from(Span::styled(app.detail.message.lines().next().unwrap_or(""), Style::default().fg(Color::Gray))),
        ]
    };

    f.render_widget(Paragraph::new(content).wrap(Wrap{trim: true}), inner_area);
}


fn draw_command_console(f: &mut Frame<'_>, _app: &App, area: Rect) {
    let block = Block::default().title(Line::from("───[ CONSOLE ]───").alignment(Alignment::Center));
    f.render_widget(block, area);

    let keys = Line::from(vec![
        key_style("[↑↓]"), Span::raw(" SHIFT TARGET   "),
        key_style("[RET]"), Span::raw(" EXPAND LOG    "),
        key_style("[Q]"), Span::raw(" DECOUPLE"),
    ]).alignment(Alignment::Center);

    let layout = Layout::default().constraints([Constraint::Length(1)]).split(Rect::new(area.x, area.y + 1, area.width, 1));
    f.render_widget(Paragraph::new(keys), layout[0]);
}

fn draw_too_small(f: &mut Frame<'_>) {
    let message = Paragraph::new(
        Line::from(vec![
            Span::styled("[ WIDTH TOO NARROW: ", Style::default().fg(Color::Red)),
            Span::styled(format!("{}", UI_WIDTH), Style::default().fg(Color::White).bold()),
            Span::styled(" COLUMNS REQUIRED ]", Style::default().fg(Color::Red)),
        ])
    ).alignment(Alignment::Center);
    f.render_widget(message, f.area());
}

fn get_centered_rect(area: Rect) -> Rect {
    let padding = (area.width.saturating_sub(UI_WIDTH)) / 2;
    Rect::new(area.x + padding, area.y, UI_WIDTH, area.height)
}

fn key_style(s: &str) -> Span {
    Span::styled(s, Style::default().fg(Color::Yellow).bold())
}