// FILE: git-trek/src/ui.rs
// ===== PSYCHEDELIC SPACE CONSOLE UI =====
// Vibrant, sci-fi themed interface with modal overlays
use crate::app::{format_oid, format_summary, App, AppState, WINDOW_SIZE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

// ===== PSYCHEDELIC COLOR PALETTE =====
const COLOR_MAGENTA: Color = Color::Rgb(255, 0, 255);      // Bright Magenta
const COLOR_CYAN: Color = Color::Rgb(0, 255, 255);         // Bright Cyan
const COLOR_YELLOW: Color = Color::Rgb(255, 255, 0);       // Bright Yellow
const COLOR_GREEN: Color = Color::Rgb(0, 255, 128);        // Bright Green
const COLOR_ORANGE: Color = Color::Rgb(255, 128, 0);       // Bright Orange
const COLOR_PURPLE: Color = Color::Rgb(200, 100, 255);     // Bright Purple
const COLOR_BLUE: Color = Color::Rgb(100, 200, 255);       // Bright Blue
const COLOR_DIM: Color = Color::Rgb(100, 100, 100);        // Dimmed text
const COLOR_BLACK: Color = Color::Black;                    // Background
const COLOR_WHITE: Color = Color::White;                    // Default text

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    draw_main(f, chunks[0], app);
    draw_footer(f, chunks[1], app);

    // Overlays (modals on top of main view)
    if app.state == AppState::ViewingDetail {
        draw_detail_modal(f, f.area(), app);
    } else if app.state == AppState::ConfirmingCheckout {
        draw_confirm_modal(f, f.area(), app);
    } else if app.state == AppState::ShowingHelp {
        draw_help_modal(f, f.area());
    }
}

fn draw_main(f: &mut Frame, area: Rect, app: &App) {
    draw_chrono_scanner(f, area, app);
}

// ===== CHRONO-SCANNER (Main List View) =====
fn draw_chrono_scanner(f: &mut Frame, area: Rect, app: &App) {
    // Build header with sci-fi styling
    let header_cells = ["◢", "CHRONO-ID", "TEMPORAL SIGNATURE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_to_render = app.commits.iter().skip(app.scroll).take(WINDOW_SIZE);

    let rows = items_to_render.enumerate().map(|(i, c)| {
        let display_idx = app.scroll + i;
        let oid_str = format_oid(c.oid);
        let summary = format_summary(&c.summary);
        
        // Anchor indicator
        let anchor = if app.anchor == Some(display_idx) { 
            "⚡" 
        } else { 
            " " 
        }.to_string();
        
        // Selected item gets full background + sci-fi cursor
        let is_selected = display_idx == app.idx;
        
        if is_selected {
            // SELECTED ROW: Full magenta background with black text
            Row::new(vec![
                Cell::from(anchor).style(Style::default().fg(COLOR_YELLOW).bg(COLOR_MAGENTA)),
                Cell::from(format!(">> {}", oid_str)).style(
                    Style::default()
                        .fg(COLOR_BLACK)
                        .bg(COLOR_MAGENTA)
                        .add_modifier(Modifier::BOLD)
                ),
                Cell::from(summary).style(
                    Style::default()
                        .fg(COLOR_BLACK)
                        .bg(COLOR_MAGENTA)
                        .add_modifier(Modifier::BOLD)
                ),
            ])
        } else {
            // UNSELECTED ROW: OID dimmed, message bright
            Row::new(vec![
                Cell::from(anchor).style(Style::default().fg(COLOR_YELLOW)),
                Cell::from(format!("   {}", oid_str)).style(Style::default().fg(COLOR_DIM)),
                Cell::from(summary).style(Style::default().fg(COLOR_WHITE)),
            ])
        }
    });

    let widths = [
        Constraint::Length(2),
        Constraint::Length(14),
        Constraint::Min(40),
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_MAGENTA))
                .title(Span::styled(
                    " ◢◣ CHRONO-SCANNER ◢◣ ",
                    Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)
                ))
        );

    f.render_widget(table, area);
}

// ===== FOOTER (Hotkey Bar) =====
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let hotkeys = match app.state {
        AppState::Browsing => vec![
            ("Q", "EXIT"),
            ("↑↓/WS", "NAVIGATE"),
            ("P", "PIN"),
            ("ENTER", "INSPECT"),
            ("?", "HELP"),
        ],
        AppState::ViewingDetail => vec![
            ("ESC", "BACK"),
            ("C", "CHECKOUT"),
            ("D", "DIFF"),
            ("P/F", "PASS/FAIL"),
        ],
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
            vec![
                Span::styled(*key, Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled(format!(":{} ", desc), Style::default().fg(COLOR_WHITE)),
            ]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line)
        .style(Style::default().bg(COLOR_BLACK).fg(COLOR_WHITE));
    
    f.render_widget(paragraph, area);
}

// ===== DETAIL MODAL (Commit Telemetry) =====
fn draw_detail_modal(f: &mut Frame, area: Rect, app: &App) {
    let d = &app.detail;
    
    // Create modal area (80% width, 70% height, centered)
    let modal_area = centered_rect(80, 70, area);
    
    // Clear the background
    f.render_widget(Clear, modal_area);
    
    // Split modal into left (metadata) and right (message + diff)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(modal_area);
    
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(chunks[0]);
    
    // ===== LEFT PANEL: COMMIT METADATA =====
    let meta_lines = vec![
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
                    " ◢ COMMIT TELEMETRY ◣ ",
                    Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)
                ))
        );
    
    f.render_widget(meta_block, left_chunks[0]);
    
    // ===== TEST RESULTS (if available) =====
    if d.test_ok.is_some() || d.manual.is_some() {
        let status_text = if let Some(manual) = d.manual {
            if manual {
                vec![Line::from(vec![
                    Span::styled("STATUS » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled("✓ MANUAL PASS", Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD)),
                ])]
            } else {
                vec![Line::from(vec![
                    Span::styled("STATUS » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled("✗ MANUAL FAIL", Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)),
                ])]
            }
        } else if let Some(ok) = d.test_ok {
            let (symbol, status, color) = if ok {
                ("✓", "PASSED", COLOR_GREEN)
            } else {
                ("✗", "FAILED", COLOR_ORANGE)
            };
            let mut lines = vec![Line::from(vec![
                Span::styled("TEST    » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} {}", symbol, status), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ])];
            if let Some(ms) = d.test_ms {
                lines.push(Line::from(vec![
                    Span::styled("TIME    » ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{}ms", ms), Style::default().fg(COLOR_WHITE)),
                ]));
            }
            lines
        } else {
            vec![]
        };
        
        let status_block = Paragraph::new(status_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_CYAN))
                    .title(Span::styled(
                        " ◢ TEST RESULTS ◣ ",
                        Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)
                    ))
            );
        
        f.render_widget(status_block, left_chunks[1]);
    }
    
    // ===== RIGHT PANEL: MESSAGE + DIFF =====
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // Message
    let message_block = Paragraph::new(d.message.clone())
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_PURPLE))
                .title(Span::styled(
                    " ◢ TEMPORAL LOG ◣ ",
                    Style::default().fg(COLOR_PURPLE).add_modifier(Modifier::BOLD)
                ))
        )
        .style(Style::default().fg(COLOR_WHITE));
    
    f.render_widget(message_block, right_chunks[0]);
    
    // Diff stats
    let diff_text = if app.diff_full {
        "Full diff view would be rendered here.\n\nUse 'D' to toggle this view.".to_string()
    } else {
        format!(
            "Diff Statistics:\n\n\
            Files Changed: {}\n\
            Insertions:    +{}\n\
            Deletions:     -{}\n\n\
            Press 'D' to toggle full diff view.",
            d.insertions + d.deletions, // simplified
            d.insertions,
            d.deletions
        )
    };
    
    let diff_block = Paragraph::new(diff_text)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BLUE))
                .title(Span::styled(
                    " ◢ DELTA ANALYSIS ◣ ",
                    Style::default().fg(COLOR_BLUE).add_modifier(Modifier::BOLD)
                ))
        )
        .style(Style::default().fg(COLOR_WHITE));
    
    f.render_widget(diff_block, right_chunks[1]);
}

// ===== CONFIRM CHECKOUT MODAL =====
fn draw_confirm_modal(f: &mut Frame, area: Rect, app: &App) {
    let modal_area = centered_rect(60, 25, area);
    f.render_widget(Clear, modal_area);
    
    let p = &app.commits[app.idx];
    let oid_display = format_oid(p.oid);
    
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled("SYSTEM COMMAND INITIATED", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(" ⚠", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target Chrono-Point: ", Style::default().fg(COLOR_CYAN)),
            Span::styled(oid_display, Style::default().fg(COLOR_MAGENTA).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("This will detach your HEAD to this commit.", Style::default().fg(COLOR_WHITE))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Proceed with checkout? ", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("[Y/N]", Style::default().fg(COLOR_YELLOW).add_modifier(Modifier::BOLD)),
        ]),
    ];
    
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_ORANGE))
                .title(Span::styled(
                    " ◢◣ CONFIRMATION REQUIRED ◢◣ ",
                    Style::default().fg(COLOR_ORANGE).add_modifier(Modifier::BOLD)
                ))
        );
    
    f.render_widget(paragraph, modal_area);
}

// ===== HELP MODAL =====
fn draw_help_modal(f: &mut Frame, area: Rect) {
    let modal_area = centered_rect(70, 60, area);
    f.render_widget(Clear, modal_area);
    
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("CHRONO-SCANNER MODE", Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓, W/S      ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Navigate timeline"),
        ]),
        Line::from(vec![
            Span::styled("  A-J           ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Jump to labeled commit"),
        ]),
        Line::from(vec![
            Span::styled("  ENTER         ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Open commit telemetry"),
        ]),
        Line::from(vec![
            Span::styled("  P             ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Pin anchor point"),
        ]),
        Line::from(vec![
            Span::styled("  Q, ESC        ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Exit scanner"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("TELEMETRY MODE", Style::default().fg(COLOR_PURPLE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ESC           ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Return to scanner"),
        ]),
        Line::from(vec![
            Span::styled("  C             ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Initiate checkout sequence"),
        ]),
        Line::from(vec![
            Span::styled("  D             ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Toggle diff view"),
        ]),
        Line::from(vec![
            Span::styled("  P / F         ", Style::default().fg(COLOR_YELLOW)),
            Span::raw("Mark pass/fail (manual)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press '?' or ESC to close this help", Style::default().fg(COLOR_DIM)),
        ]),
    ];
    
    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_GREEN))
                .title(Span::styled(
                    " ◢◣ SYSTEM DOCUMENTATION ◢◣ ",
                    Style::default().fg(COLOR_GREEN).add_modifier(Modifier::BOLD)
                ))
        );
    
    f.render_widget(paragraph, modal_area);
}

// ===== UTILITY: CENTERED RECT =====
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