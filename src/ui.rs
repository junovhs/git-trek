// ui.rs - COPY THIS ENTIRE FILE (SNES SAVE FILE AESTHETIC)

use crate::app::{format_oid, format_summary, App, AppState, VERSION};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Clear, Paragraph, Wrap},
    Frame,
};

const COLOR_MAGENTA: Color = Color::Rgb(255, 0, 255);
const COLOR_CYAN: Color = Color::Rgb(0, 255, 255);
const COLOR_YELLOW: Color = Color::Rgb(255, 255, 0);
const COLOR_GREEN: Color = Color::Rgb(0, 255, 128);
const COLOR_ORANGE: Color = Color::Rgb(255, 128, 0);
const COLOR_PURPLE: Color = Color::Rgb(200, 100, 255);
const COLOR_BLUE: Color = Color::Rgb(100, 200, 255);
const COLOR_DIM: Color = Color::Rgb(60, 60, 60);
const COLOR_CARD_BG: Color = Color::Rgb(30, 30, 30);
const COLOR_BLACK: Color = Color::Black;
const COLOR_WHITE: Color = Color::White;

pub fn draw(f: &mut Frame, app: &App) {
    if app.state == AppState::DirtyTreeWarning {
        draw_snes_layout(f, f.area(), app);
        draw_dirty_warning_modal(f, f.area());
        return;
    }

    if app.state == AppState::ConfirmingCheckout {
        draw_snes_layout(f, f.area(), app);
        draw_confirm_modal(f, f.area(), app);
        return;
    }

    if app.state == AppState::ShowingHelp {
        draw_snes_layout(f, f.area(), app);
        draw_help_modal(f, f.area());
        return;
    }

    if app.state == AppState::ViewingDetail {
        draw_snes_layout_with_detail(f, f.area(), app);
    } else {
        draw_snes_layout(f, f.area(), app);
    }
}

fn draw_snes_layout(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Title bar
            Constraint::Min(0),         // Cards
            Constraint::Length(5),      // Control hints
            Constraint::Length(3),      // Status bar
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
        Style::default()
            .fg(COLOR_CYAN)
            .add_modifier(Modifier::BOLD),
    )])];

    let title_widget = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BLUE))
                .border_type(BorderType::Double),
        );

    f.render_widget(title_widget, area);
}

fn draw_card_row(f: &mut Frame, area: Rect, app: &App) {
    if app.commits.is_empty() {
        return;
    }

    let margin = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    let cards_area = margin[1];

    let cards_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(cards_area);

    let current_idx = app.idx;

    // LEFT CARD (dimmed)
    if current_idx > 0 {
        let left_idx = current_idx - 1;
        draw_save_card(f, cards_layout[0], app, left_idx, CardState::Left);
    }

    // CENTER CARD (selected, glowing)
    draw_save_card(f, cards_layout[1], app, current_idx, CardState::Selected);

    // RIGHT CARD (dimmed)
    if current_idx < app.commits.len() - 1 {
        let right_idx = current_idx + 1;
        draw_save_card(f, cards_layout[2], app, right_idx, CardState::Right);
    }
}

#[derive(PartialEq)]
enum CardState {
    Left,
    Selected,
    Right,
}

fn draw_save_card(f: &mut Frame, area: Rect, app: &App, idx: usize, state: CardState) {
    let commit = &app.commits[idx];
    let oid_str = format_oid(commit.oid);
    let summary = format_summary(&commit.summary);

    let (border_color, border_type, bg_color, title_color, is_bright) = match state {
        CardState::Selected => (
            COLOR_MAGENTA,
            BorderType::Double,
            COLOR_CARD_BG,
            COLOR_CYAN,
            true,
        ),
        _ => (COLOR_DIM, BorderType::Plain, COLOR_BLACK, COLOR_DIM, false),
    };

    let marker = if app.anchor == Some(idx) { "⚡" } else { "" };

    // Generate Braille art pattern based on commit hash
    let art = generate_braille_art(&oid_str, is_bright);

    let mut card_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("COMMIT {}", oid_str.to_uppercase()),
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    // Add Braille art
    for art_line in art {
        card_lines.push(art_line);
    }

    card_lines.push(Line::from(""));
    card_lines.push(Line::from(""));

    // Add marker and summary
    card_lines.push(Line::from(vec![
        Span::styled(marker, Style::default().fg(COLOR_YELLOW)),
        Span::raw(" "),
        Span::styled(
            summary,
            Style::default().fg(if is_bright {
                COLOR_WHITE
            } else {
                COLOR_DIM
            }),
        ),
    ]));

    let card = Paragraph::new(card_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color).add_modifier(
                    if state == CardState::Selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ))
                .border_type(border_type)
                .style(Style::default().bg(bg_color)),
        )
        .alignment(Alignment::Center);

    f.render_widget(card, area);
}

fn generate_braille_art(seed: &str, bright: bool) -> Vec<Line<'static>> {
    // Generate deterministic Braille pattern based on commit hash
    let hash_bytes: Vec<u8> = seed.bytes().collect();

    let patterns = if bright {
        // Brighter, more complex patterns for selected card
        vec![
            "⠿⢿⣿⡿⠿⢿⣿⡿",
            "⣿⣿⣿⣿⣿⣿⣿⣿",
            "⠛⠛⠛⠛⠛⠛⠛⠛",
            "⣀⣀⣀⣀⣀⣀⣀⣀",
            "⢸⢸⢸⢸⢸⢸⢸⢸",
        ]
    } else {
        // Simpler patterns for side cards
        vec!["⠿⠿⠿⠿⠿⠿⠿⠿", "⣿⣿⣿⣿⣿⣿⣿⣿", "⠛⠛⠛⠛⠛⠛⠛⠛"]
    };

    let color = if bright {
        match *hash_bytes.get(0).unwrap_or(&0) as usize % 5 {
            0 => COLOR_CYAN,
            1 => COLOR_MAGENTA,
            2 => COLOR_GREEN,
            3 => COLOR_YELLOW,
            _ => COLOR_BLUE,
        }
    } else {
        COLOR_DIM
    };

    let selected_pattern =
        patterns[(*hash_bytes.get(1).unwrap_or(&0) as usize) % patterns.len()];

    vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            selected_pattern.to_string(),
            Style::default().fg(color),
        )]),
        Line::from(vec![Span::styled(
            selected_pattern.to_string(),
            Style::default().fg(color),
        )]),
        Line::from(vec![Span::styled(
            selected_pattern.to_string(),
            Style::default().fg(color),
        )]),
        Line::from(""),
    ]
}

fn draw_control_hints(f: &mut Frame, area: Rect, app: &App) {
    let hints = match app.state {
        AppState::Browsing => vec![
            Line::from(vec![
                Span::raw("Use "),
                Span::styled(
                    "[LEFT]",
                    Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    "[RIGHT]",
                    Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to Navigate, "),
                Span::styled(
                    "[A]",
                    Style::default()
                        .fg(COLOR_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to Inspect, "),
                Span::styled(
                    "[Q]",
                    Style::default()
                        .fg(COLOR_ORANGE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to Go Back"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    [P]", Style::default().fg(COLOR_PURPLE)),
                Span::raw(" Pin Anchor  "),
                Span::styled("[?]", Style::default().fg(COLOR_GREEN)),
                Span::raw(" Help"),
            ]),
        ],
        AppState::ViewingDetail => vec![Line::from(vec![
            Span::raw("Press "),
            Span::styled(
                "[ESC]",
                Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to Return, "),
            Span::styled(
                "[C]",
                Style::default()
                    .fg(COLOR_MAGENTA)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to Checkout, "),
            Span::styled(
                "[T]",
                Style::default()
                    .fg(COLOR_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to Toggle Diff"),
        ])],
        _ => vec![Line::from("")],
    };

    let hints_widget = Paragraph::new(hints)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_DIM)),
        );

    f.render_widget(hints_widget, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let commit = &app.commits[app.idx];
    let oid_str = format_oid(commit.oid);
    let summary = &commit.summary;

    let status_text = vec![Line::from(vec![
        Span::styled("⚡ ", Style::default().fg(COLOR_YELLOW)),
        Span::raw("Current Location: "),
        Span::styled(
            format!("COMMIT {} - {}", oid_str.to_uppercase(), summary),
            Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
        ),
    ])];

    let status_widget = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BLUE)),
        );

    f.render_widget(status_widget, area);
}

fn draw_detail_panel(f: &mut Frame, area: Rect, app: &App) {
    let d = &app.detail;

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let meta_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "HASH    » ",
                Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &d.hash,
                Style::default()
                    .fg(COLOR_MAGENTA)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "AUTHOR  » ",
                Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(&d.author, Style::default().fg(COLOR_WHITE)),
        ]),
        Line::from(vec![
            Span::styled(
                "TIME    » ",
                Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(&d.date, Style::default().fg(COLOR_WHITE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "CHANGES » ",
                Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("+{} ", d.insertions),
                Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("-{}", d.deletions),
                Style::default()
                    .fg(COLOR_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let meta_block = Paragraph::new(meta_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_CYAN))
            .title(Span::styled(
                " INFO ",
                Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
            )),
    );

    f.render_widget(meta_block, h_chunks[0]);

    let message_block = Paragraph::new(d.message.clone())
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_PURPLE))
                .title(Span::styled(
                    " MESSAGE ",
                    Style::default()
                        .fg(COLOR_PURPLE)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().fg(COLOR_WHITE));

    f.render_widget(message_block, h_chunks[1]);
}

fn draw_dirty_warning_modal(f: &mut Frame, area: Rect) {
    let modal_area = centered_rect(70, 50, area);
    f.render_widget(Clear, modal_area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "⚠ ",
                Style::default()
                    .fg(COLOR_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "WORKING TREE IS DIRTY",
                Style::default()
                    .fg(COLOR_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ⚠",
                Style::default()
                    .fg(COLOR_ORANGE)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "You have uncommitted changes.",
            Style::default().fg(COLOR_WHITE),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Choose how to proceed:",
            Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[S] STASH",
            Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("    Save changes temporarily")]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[C] CONTINUE",
            Style::default()
                .fg(COLOR_YELLOW)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("    Browse in read-only mode")]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[Q] QUIT",
            Style::default()
                .fg(COLOR_ORANGE)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw("    Exit without changes")]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_ORANGE))
                .border_type(BorderType::Double)
                .title(Span::styled(
                    " ⚠ WARNING ⚠ ",
                    Style::default()
                        .fg(COLOR_ORANGE)
                        .add_modifier(Modifier::BOLD),
                )),
        );

    f.render_widget(paragraph, modal_area);
}

fn draw_confirm_modal(f: &mut Frame, area: Rect, app: &App) {
    let modal_area = centered_rect(60, 30, area);
    f.render_widget(Clear, modal_area);

    let p = &app.commits[app.idx];
    let oid_display = format_oid(p.oid);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "⚠ ",
                Style::default()
                    .fg(COLOR_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "CHECKOUT COMMIT",
                Style::default()
                    .fg(COLOR_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " ⚠",
                Style::default()
                    .fg(COLOR_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(COLOR_CYAN)),
            Span::styled(
                oid_display,
                Style::default()
                    .fg(COLOR_MAGENTA)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "This will detach HEAD.",
            Style::default().fg(COLOR_WHITE),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Proceed? ",
                Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "[Y/N]",
                Style::default()
                    .fg(COLOR_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_ORANGE))
                .border_type(BorderType::Double)
                .title(Span::styled(
                    " CONFIRM ",
                    Style::default()
                        .fg(COLOR_ORANGE)
                        .add_modifier(Modifier::BOLD),
                )),
        );

    f.render_widget(paragraph, modal_area);
}

fn draw_help_modal(f: &mut Frame, area: Rect) {
    let modal_area = centered_rect(70, 60, area);
    f.render_widget(Clear, modal_area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "═══ GIT TREK CONTROLS ═══",
            Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "CARD NAVIGATION",
            Style::default()
                .fg(COLOR_YELLOW)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ← →, A D      ", Style::default().fg(COLOR_MAGENTA)),
            Span::raw("Navigate cards"),
        ]),
        Line::from(vec![
            Span::styled("  ENTER         ", Style::default().fg(COLOR_MAGENTA)),
            Span::raw("View details"),
        ]),
        Line::from(vec![
            Span::styled("  P             ", Style::default().fg(COLOR_MAGENTA)),
            Span::raw("Pin anchor"),
        ]),
        Line::from(vec![
            Span::styled("  Q             ", Style::default().fg(COLOR_MAGENTA)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "DETAIL VIEW",
            Style::default()
                .fg(COLOR_YELLOW)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ESC, Q        ", Style::default().fg(COLOR_MAGENTA)),
            Span::raw("Back to cards"),
        ]),
        Line::from(vec![
            Span::styled("  C             ", Style::default().fg(COLOR_MAGENTA)),
            Span::raw("Checkout"),
        ]),
        Line::from(vec![
            Span::styled("  T             ", Style::default().fg(COLOR_MAGENTA)),
            Span::raw("Toggle diff"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("git-trek v{}", VERSION),
            Style::default().fg(COLOR_DIM),
        )]),
    ];

    let paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_GREEN))
                .border_type(BorderType::Double)
                .title(Span::styled(
                    " HELP ",
                    Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD),
                )),
        );

    f.render_widget(paragraph, modal_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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