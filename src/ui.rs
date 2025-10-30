// FILE: git-trek/src/ui.rs
use crate::app::{format_oid, format_summary, App, AppState, WINDOW_SIZE};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{self, Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

// FILE: git-trek/src/ui.rs | FUNCTION: draw
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref()) // <-- CORRECTED from Constraint.Length
        .split(f.area());

    draw_main(f, chunks[0], app);
    draw_footer(f, chunks[1], app);

    if app.state == AppState::ShowingHelp {
        draw_help(f, f.area());
    } else if app.state == AppState::ConfirmingCheckout {
        draw_confirm(f, f.area(), app);
    }
}

fn draw_main(f: &mut Frame, area: Rect, app: &App) {
    if app.state == AppState::ViewingDetail {
        draw_detail(f, area, app);
        return;
    }
    draw_list(f, area, app);
}

fn draw_list(f: &mut Frame, area: Rect, app: &App) {
    let header_cells = ["", "oid", "summary"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_to_render = app.commits.iter().skip(app.scroll).take(WINDOW_SIZE);

    let rows = items_to_render.enumerate().map(|(i, c)| {
        let display_idx = app.scroll + i;
        let oid_str = format_oid(c.oid);
        let summary = format_summary(&c.summary);
        let anchor = if app.anchor == Some(display_idx) { "✓" } else { " " }.to_string();
        let style = if display_idx == app.idx {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(anchor).style(Style::default().fg(Color::Yellow)),
            Cell::from(oid_str).style(style),
            Cell::from(summary).style(style),
        ])
    });

    let widths = [
        Constraint::Length(1),
        Constraint::Length(10),
        Constraint::Min(40),
    ];
    let t = Table::new(rows, widths)
        .header(header)
        .block(title("commits", Color::White));

    f.render_widget(t, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let mut hotkeys = vec![
        ("q", "quit"),
        ("j/k/↓/↑", "navigate"),
        ("p", "pin anchor"),
        ("Enter", "details"),
    ];
    if app.state == AppState::ViewingDetail {
        hotkeys = vec![
            ("q/Esc", "back"),
            ("d", "toggle diff"),
            ("p/f", "pass/fail"),
            ("c", "confirm checkout"),
        ]; // <-- ADDED semicolon
    }
    if app.state == AppState::ConfirmingCheckout {
        hotkeys = vec![("y", "confirm"), ("n/Esc", "cancel")]; // <-- ADDED semicolon
    }
    if app.state == AppState::ShowingHelp {
        hotkeys = vec![("q/Esc/?/h", "close help")]; // <-- ADDED semicolon
    }
    let spans: Vec<Span> = hotkeys
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(*key, Style::default().fg(Color::Yellow)),
                // Inlined format args
                Span::raw(format!(": {desc} ")),
            ]
        })
        .collect();
    let text = Line::from(spans);
    let p = Paragraph::new(text);
    f.render_widget(p, area);
}

fn draw_detail(f: &mut Frame, area: Rect, app: &App) {
    let d = &app.detail;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(10)].as_ref())
        .split(chunks[0]);

    let meta = vec![
        Line::from(vec![
            Span::styled("oid: ", Style::default().fg(Color::Yellow)),
            Span::raw(d.hash.clone()),
        ]),
        Line::from(vec![
            Span::styled("author: ", Style::default().fg(Color::Yellow)),
            Span::raw(d.author.clone()),
        ]),
        Line::from(vec![
            Span::styled("time: ", Style::default().fg(Color::Yellow)),
            Span::raw(d.date.clone()),
        ]),
    ];
    let p = Paragraph::new(meta).block(title("meta", Color::White));
    f.render_widget(p, left[0]);
    let msg = Paragraph::new(d.message.clone())
        .block(title("message", Color::White))
        .wrap(widgets::Wrap { trim: false });
    f.render_widget(msg, left[1]);
    let diff_title = format!("diff (+{} -{})", d.insertions, d.deletions);
    let p = Paragraph::new("Diff text rendering would go here.").block(title(&diff_title, Color::White));
    f.render_widget(p, chunks[1]);
}

fn draw_confirm(f: &mut Frame, area: Rect, app: &App) {
    let p = &app.commits[app.idx];
    let text = format!("Are you sure you want to checkout {}?", format_oid(p.oid));
    let p = Paragraph::new(text).block(title("Confirm Checkout", Color::Red));
    let area = centered_rect(60, 20, area);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let text = "Help:\n\nThis is a help message.";
    let p = Paragraph::new(text).block(title("Help", Color::Green));
    let area = centered_rect(60, 50, area);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

fn title(t: &str, color: Color) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(t, Style::default().fg(color)))
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}