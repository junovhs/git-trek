// ui.rs - COPY THIS ENTIRE FILE

use crate::app::{format_oid, format_summary, App, AppState, VERSION, VISIBLE_CARDS};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Wrap},
    Frame,
};

const COLOR_MAGENTA: Color = Color::Rgb(255, 0, 255);
const COLOR_CYAN: Color = Color::Rgb(0, 255, 255);
const COLOR_YELLOW: Color = Color::Rgb(255, 255, 0);
const COLOR_GREEN: Color = Color::Rgb(0, 255, 128);
const COLOR_ORANGE: Color = Color::Rgb(255, 128, 0);
const COLOR_PURPLE: Color = Color::Rgb(200, 100, 255);
const COLOR_BLUE: Color = Color::Rgb(100, 200, 255);
const COLOR_DIM: Color = Color::Rgb(80, 80, 80);
const COLOR_BLACK: Color = Color::Black;
const COLOR_WHITE: Color = Color::White;

pub fn draw(f: &mut Frame, app: &App) {
    if app.state == AppState::DirtyTreeWarning {
        draw_cards(f, f.area(), app);
        draw_footer(f, f.area(), app);
        draw_dirty_warning_modal(f, f.area());
        return;
    }
    
    if app.state == AppState::ConfirmingCheckout {
        draw_cards(f, f.area(), app);
        draw_footer(f, f.area(), app);
        draw_confirm_modal(f, f.area(), app);
        return;
    }
    
    if app.state == AppState::ShowingHelp {
        draw_cards(f, f.area(), app);
        draw_footer(f, f.area(), app);
        draw_help_modal(f, f.area());
        return;
    }
    
    if app.state == AppState::ViewingDetail {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(49),
                Constraint::Length(1),
            ])
            .split(f.area());
        
        draw_cards(f, chunks[0], app);
        draw_detail_panel(f, chunks[1], app);
        draw_footer(f, chunks[2], app);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(f.area());
        
        draw_cards(f, chunks[0], app);
        draw_footer(f, chunks[1], app);
    }
}

fn draw_cards(f: &mut Frame, area: Rect, app: &App) {
    if app.commits.is_empty() {
        return;
    }
    
    let cards_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);
    
    let current_idx = app.idx;
    
    // LEFT CARD
    if current_idx > 0 {
        let left_idx = current_idx - 1;
        draw_card(f, cards_layout[0], app, left_idx, false);
    }
    
    // CENTER CARD (SELECTED)
    draw_card(f, cards_layout[1], app, current_idx, true);
    
    // RIGHT CARD
    if current_idx < app.commits.len() - 1 {
        let right_idx = current_idx + 1;
        draw_card(f, cards_layout[2], app, right_idx, false);
    }
}

fn draw_card(f: &mut Frame, area: Rect, app: &App, idx: usize, selected: bool) {
    let commit = &app.commits[idx];
    let oid_str = format_oid(commit.oid);
    let summary = format_summary(&commit.summary);
    
    let (border_color, bg_color, text_color) = if selected {
        (COLOR_MAGENTA, COLOR_MAGENTA, COLOR_BLACK)
    } else {
        (COLOR_DIM, COLOR_BLACK, COLOR_DIM)
    };
    
    let marker = if app.anchor == Some(idx) {
        "⚡"
    } else {
        ""
    };
    
    let card_content = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {}  ", marker),
                Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {}  ", oid_str),
                Style::default().fg(text_color).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {}  ", summary),
                Style::default().fg(text_color)
            ),
        ]),
    ];
    
    let card = Paragraph::new(card_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(bg_color))
        )
        .alignment(Alignment::Center);
    
    f.render_widget(card, area);
}

fn draw_detail_panel(f: &mut Frame, area: Rect, app: &App) {
    let d = &app.detail;
    
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
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
    
    let meta_block = Paragraph::new(meta_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_CYAN))
                .title(Span::styled(
                    " ◢ INFO ◣ ",
                    Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)
                ))
        );
    
    f.render_widget(meta_block, h_chunks[0]);
    
    let message_block = Paragraph::new(d.message.clone())
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_PURPLE))
                .title(Span::styled(
                    " ◢ MESSAGE ◣ ",
                    Style::default().fg(COLOR_PURPLE).add_modifier(Modifier::BOLD)
                ))
        )
        .style(Style::default().fg(COLOR_WHITE));
    
    f.render_widget(message_block, h_chunks[1]);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let hotkeys = match app.state {
        AppState::DirtyTreeWarning => vec![
            ("S", "STASH"),
            ("C", "CONTINUE"),
            ("Q", "QUIT"),
        ],
        AppState::Browsing => {
            let mut keys = vec![
                ("Q", "EXIT"),
                ("←→/AD", "NAV"),
                ("P", "PIN"),
                ("ENTER", "DETAILS"),
                ("?", "HELP"),
            ];
            if app.read_only {
                keys.push(("⚠", "READ-ONLY"));
            }
            keys
        },
        AppState::ViewingDetail => {
            let mut keys = vec![
                ("ESC", "BACK"),
                ("T", "DIFF"),
                ("P/F", "MARK"),
            ];
            if !app.read_only {
                keys.insert(1, ("C", "CHECKOUT"));
            } else {
                keys.push(("⚠", "READ-ONLY"));
            }
            keys
        },
        AppState::ConfirmingCheckout => vec![
            ("Y", "CONFIRM"),
            ("N/ESC", "CANCEL"),
        ],
        AppState::ShowingHelp => vec![
            ("ESC/?", "CLOSE"),
        ],
    };

    let spans: Vec<Span> = hotkeys
        .iter()
        .flat_map(|(key, desc)| {
            let key_color = if *key == "⚠" { 
                COLOR_ORANGE 
            } else { 
                COLOR_YELLOW 
            };
            vec![
                Span::styled(
                    *key, 
                    Style::default()
                        .fg(key_color)
                        .add_modifier(Modifier::BOLD)
                ),
                Span::styled(
                    format!(":{} ", desc), 
                    Style::default().fg(COLOR_WHITE)
                ),
            ]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .style(Style::default().bg(COLOR_BLACK).fg(COLOR_WHITE));
    
    f.render_widget(paragraph, area);
}

fn draw_dirty_warning_modal(f: &mut Frame, area: Rect) {
    let modal_area = centered_rect(70, 40, area);
    f.render_widget(Clear, modal_area);
    
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("WORKING TREE IS DIRTY", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled(" ⚠", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("You have uncommitted changes.", Style::default().fg(COLOR_WHITE))),
        Line::from(""),
        Line::from(Span::styled("Choose how to proceed:", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("[S] STASH", Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::raw("Save changes temporarily (safe, reversible)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[C] CONTINUE", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::raw("Browse in read-only mode (no checkout)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Q] QUIT", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::raw("Exit without changes"),
        ]),
    ];
    
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_ORANGE))
                .title(Span::styled(
                    " ◢◣ WARNING ◢◣ ",
                    Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)
                ))
        );
    
    f.render_widget(paragraph, modal_area);
}

fn draw_confirm_modal(f: &mut Frame, area: Rect, app: &App) {
    let modal_area = centered_rect(60, 25, area);
    f.render_widget(Clear, modal_area);
    
    let p = &app.commits[app.idx];
    let oid_display = format_oid(p.oid);
    
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("CHECKOUT COMMIT", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ⚠", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(COLOR_CYAN)),
            Span::styled(oid_display, Style::default().fg(COLOR_MAGENTA).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("This will detach HEAD to this commit.", Style::default().fg(COLOR_WHITE))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Proceed? ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("[Y/N]", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
        ]),
    ];
    
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_ORANGE))
                .title(Span::styled(
                    " ◢◣ CONFIRM ◢◣ ",
                    Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)
                ))
        );
    
    f.render_widget(paragraph, modal_area);
}

fn draw_help_modal(f: &mut Frame, area: Rect) {
    let modal_area = centered_rect(70, 50, area);
    f.render_widget(Clear, modal_area);
    
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("CARD NAVIGATION", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ← →, A D      ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Navigate cards left/right"),
        ]),
        Line::from(vec![
            Span::styled("  ENTER         ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Open details"),
        ]),
        Line::from(vec![
            Span::styled("  P             ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Pin anchor"),
        ]),
        Line::from(vec![
            Span::styled("  Q             ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("DETAIL VIEW", Style::default().fg(COLOR_PURPLE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ESC, Q        ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Back to cards"),
        ]),
        Line::from(vec![
            Span::styled("  C             ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Checkout commit"),
        ]),
        Line::from(vec![
            Span::styled("  T             ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Toggle diff"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("git-trek v{}", VERSION), Style::default().fg(COLOR_DIM)),
        ]),
    ];
    
    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_GREEN))
                .title(Span::styled(
                    " ◢◣ HELP ◢◣ ",
                    Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD)
                ))
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