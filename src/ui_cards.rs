use crate::app::App;
use crate::git_ops::{format_oid, format_summary};
use crate::ui::{
    COLOR_BLACK, COLOR_CARD_BG, COLOR_CYAN, COLOR_DIM, COLOR_GREEN, COLOR_MAGENTA,
    COLOR_ORANGE, COLOR_PURPLE, COLOR_WHITE, COLOR_YELLOW, COLOR_BLUE,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum CardState { Left, Selected, Right }

pub fn draw_card_row(f: &mut Frame, area: Rect, app: &App) {
    if app.commits.is_empty() { return; }
    let margin = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(80), Constraint::Percentage(10)])
        .split(area);
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)])
        .split(margin[1]);

    if app.idx > 0 { draw_save_card(f, cards[0], app, app.idx - 1, CardState::Left); }
    draw_save_card(f, cards[1], app, app.idx, CardState::Selected);
    if app.idx < app.commits.len() - 1 { draw_save_card(f, cards[2], app, app.idx + 1, CardState::Right); }
}

fn draw_save_card(f: &mut Frame, area: Rect, app: &App, idx: usize, state: CardState) {
    let commit = &app.commits[idx];
    let oid_str = format_oid(commit.oid);
    let summary = format_summary(&commit.summary);
    let (border_color, border_type, bg_color, title_color, is_bright) = match state {
        CardState::Selected => (COLOR_MAGENTA, BorderType::Double, COLOR_CARD_BG, COLOR_CYAN, true),
        _ => (COLOR_DIM, BorderType::Plain, COLOR_BLACK, COLOR_DIM, false),
    };
    let marker = if app.anchor == Some(idx) { "⚡" } else { "" };
    let art = generate_braille_art(&oid_str, is_bright);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(format!("COMMIT {}", oid_str.to_uppercase()), Style::default().fg(title_color).add_modifier(Modifier::BOLD))]),
        Line::from(""),
    ];
    for art_line in art { lines.push(art_line); }
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(marker, Style::default().fg(COLOR_YELLOW)),
        Span::raw(" "),
        Span::styled(summary, Style::default().fg(if is_bright { COLOR_WHITE } else { COLOR_DIM })),
    ]));

    let card = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color).add_modifier(if state == CardState::Selected { Modifier::BOLD } else { Modifier::empty() }))
            .border_type(border_type)
            .style(Style::default().bg(bg_color)))
        .alignment(Alignment::Center);
    f.render_widget(card, area);
}

fn generate_braille_art(seed: &str, bright: bool) -> Vec<Line<'static>> {
    let hash_bytes: Vec<u8> = seed.bytes().collect();
    let patterns = if bright {
        vec!["⠿⢿⣿⡿⠿⢿⣿⡿", "⣿⣿⣿⣿⣿⣿⣿⣿", "⠛⠛⠛⠛⠛⠛⠛⠛", "⣀⣀⣀⣀⣀⣀⣀⣀", "⢸⢸⢸⢸⢸⢸⢸⢸"]
    } else {
        vec!["⠿⠿⠿⠿⠿⠿⠿⠿", "⣿⣿⣿⣿⣿⣿⣿⣿", "⠛⠛⠛⠛⠛⠛⠛⠛"]
    };
    let color = if bright {
        match *hash_bytes.first().unwrap_or(&0) as usize % 5 {
            0 => COLOR_CYAN, 1 => COLOR_MAGENTA, 2 => COLOR_GREEN, 3 => COLOR_YELLOW, _ => COLOR_BLUE,
        }
    } else { COLOR_DIM };
    let pat = patterns[(*hash_bytes.get(1).unwrap_or(&0) as usize) % patterns.len()];
    vec![
        Line::from(""),
        Line::from(vec![Span::styled(pat.to_string(), Style::default().fg(color))]),
        Line::from(vec![Span::styled(pat.to_string(), Style::default().fg(color))]),
        Line::from(vec![Span::styled(pat.to_string(), Style::default().fg(color))]),
        Line::from(""),
    ]
}

pub fn draw_detail_panel(f: &mut Frame, area: Rect, app: &App) {
    let d = &app.detail;
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let meta_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("HASH    » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(&d.hash, Style::default().fg(COLOR_MAGENTA).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("AUTHOR  » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(&d.author, Style::default().fg(COLOR_WHITE)),
        ]),
        Line::from(vec![
            Span::styled("TIME    » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(&d.date, Style::default().fg(COLOR_WHITE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("CHANGES » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(format!("+{} ", d.insertions), Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(format!("-{}", d.deletions), Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
        ]),
    ];
    let meta = Paragraph::new(meta_lines).block(
        Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_CYAN))
            .title(Span::styled(" INFO ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD))),
    );
    f.render_widget(meta, h_chunks[0]);

    let msg = Paragraph::new(d.message.clone())
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_PURPLE))
            .title(Span::styled(" MESSAGE ", Style::default().fg(COLOR_PURPLE).add_modifier(Modifier::BOLD))))
        .style(Style::default().fg(COLOR_WHITE));
    f.render_widget(msg, h_chunks[1]);
}