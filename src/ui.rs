use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap, Clear},
    Frame, Terminal,
};

use crate::app::{App, AppState, MAX_FILES_SHOWN, WINDOW_SIZE};

pub fn draw(terminal: &mut Terminal<impl ratatui::backend::Backend>, app: &App) -> Result<()> {
    terminal.draw(|f| { if app.help { draw_help(f, app); } else { match app.state { AppState::Navigating => draw_timeline(f, app), AppState::Detail | AppState::Confirm => draw_detail(f, app), } } })?;
    Ok(())
}

fn draw_timeline(f: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
        .split(f.area());

    f.render_widget(header(app), layout[0]);
    render_list(f, app, layout[1]);
    controls(
        f,
        layout[2],
        "↑/W ↓/S Move | A–Z Jump | PgUp/PgDn | Home/End | Enter Details | P Pin | Q Quit",
    );
}

fn draw_detail(f: &mut Frame, app: &App) {
    let area = f.area();
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let d = &app.detail;
    let msg = Paragraph::new(d.message.as_str())
        .block(title("LOG ENTRY", Color::Cyan))
        .wrap(Wrap { trim: false });
    f.render_widget(msg, split[0]);

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(split[1]);

    f.render_widget(meta_block(app), halves[0]);
    f.render_widget(sensor_block(app), halves[1]);

    let bar = if app.state == AppState::Confirm {
        "Confirm [Y/N]"
    } else {
        "[Enter] Engage  [Esc] Back  [d] Diff  [P] Mark Pass  [F] Mark Fail"
    };
    controls(
        f,
        Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        },
        bar,
    );
}

fn render_list(f: &mut Frame, app: &App, area: Rect) {
    let block = title("TEMPORAL LOG", Color::Magenta);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let end = (app.scroll + WINDOW_SIZE).min(app.commits.len());
    let slice = app.scroll..end;

    let mut lines = Vec::with_capacity(WINDOW_SIZE * 2);
    for (i, idx) in slice.clone().enumerate() {
        let c = &app.commits[idx];
        let letter = (b'A' + i as u8) as char;
        let (mark, color) = if idx == app.idx {
            ("◉", Color::Cyan)
        } else if idx == app.anchor {
            ("◎", Color::Green)
        } else {
            ("○", Color::DarkGray)
        };

        let lbl = app
            .labels
            .get(&c.oid)
            .map(|v| format!(" [{}]", v.join(" | ")))
            .unwrap_or_default();

        let (badge, bcolor) = app
            .detail_for(c.oid)
            .map(|d| {
                if let Some(m) = d.manual {
                    return (
                        if m { "✅" } else { "❌" },
                        if m { Color::Green } else { Color::Red },
                    );
                }
                if let Some(ok) = d.test_ok {
                    return (
                        if ok { "✅" } else { "❌" },
                        if ok { Color::Green } else { Color::Red },
                    );
                }
                ("•", Color::DarkGray)
            })
            .unwrap_or(("•", Color::DarkGray));

        lines.push(Line::from(vec![
            Span::styled(mark, Style::default().fg(color)),
            Span::styled(
                format!(" [{}] ", letter),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                &c.summary,
                Style::default().fg(if idx == app.idx {
                    Color::White
                } else {
                    Color::Gray
                }),
            ),
            Span::styled(lbl, Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(badge, Style::default().fg(bcolor)),
        ]));
        if i < WINDOW_SIZE - 1 && idx < app.commits.len().saturating_sub(1) {
            lines.push(Line::from(Span::styled(
                "   ·",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
    f.render_widget(Paragraph::new(lines), inner);
}

fn meta_block(app: &App) -> Paragraph<'static> {
    let d = &app.detail;
    let mut lines: Vec<Line<'static>> = vec![
        kv("Hash", d.hash.clone()),
        kv("Author", d.author.clone()),
        kv("Date", d.date.clone()),
    ];
    let test = match d.test_ok {
        Some(true) => Line::from(vec![
            "Test: ".into(),
            "✅ PASS ".green(),
            format!("{} ms", d.test_ms.unwrap_or(0)).into(),
        ]),
        Some(false) => Line::from(vec![
            "Test: ".into(),
            "❌ FAIL ".red(),
            format!("{} ms", d.test_ms.unwrap_or(0)).into(),
        ]),
        None => Line::from("Test: —"),
    };
    lines.push(test);
    if let Some(m) = d.manual {
        lines.push(Line::from(if m {
            "Manual: ✅ PASS"
        } else {
            "Manual: ❌ FAIL"
        }));
    }
    Paragraph::new(lines).block(title("DATABANK RECORD", Color::Green))
}

fn sensor_block(app: &App) -> Paragraph<'static> {
    let d = &app.detail;
    let mut lines: Vec<Line<'static>> = vec![
        kv("Files", d.files_changed.to_string()),
        Line::from(vec![
            "Insertions: ".yellow(),
            format!("+{}", d.insertions).into(),
        ]),
        Line::from(vec![
            "Deletions:  ".yellow(),
            format!("-{}", d.deletions).into(),
        ]),
    ];
    if d.show_files {
        lines.push(Line::from(""));
        lines.push(Line::from("Changed files:"));
        for fc in d.files.iter().take(MAX_FILES_SHOWN) {
            let color = match fc.status.as_str() {
                "A" => Color::Green,
                "M" => Color::Yellow,
                "D" => Color::Red,
                "R" => Color::Blue,
                _ => Color::White,
            };
            lines.push(Line::from(vec![
                Span::raw(format!("{:>2} ", fc.status)),
                Span::styled(fc.path.clone(), Style::default().fg(color)),
            ]));
        }
    }
    Paragraph::new(lines).block(title("SENSOR READINGS", Color::Magenta))
}

fn header(app: &App) -> Paragraph<'static> {
    let (x, y) = app.x_of_y();
    Paragraph::new(Line::from(vec![
        " ".into(),
        Span::styled("🚀 GIT TREK", Style::default().fg(Color::Cyan).bold()),
        format!("  ({x} of {y})").into(),
        " ".into(),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Green)),
    )
    .alignment(Alignment::Center)
}

fn controls(f: &mut Frame, area: Rect, text: &str) {
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(p, area);
}

fn title<'a>(t: &'a str, color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(color))
        .title(format!(" {t} "))
}

fn kv(k: &str, v: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{k:<8}: "), Style::default().fg(Color::Yellow)),
        Span::raw(v),
    ])
}

fn draw_help(f: &mut Frame, app: &App) {
    let area = f.area();
    let block = title("HELP", Color::Cyan);
    let inner = block.inner(area);
    f.render_widget(Clear, inner);
    f.render_widget(block, area);

    let (x, y) = app.x_of_y();
    let lines = vec![
        Line::from("git-trek — time travel for debugging"),
        Line::from(""),
        Line::from("NAVIGATION"),
        Line::from("  ↑/W, ↓/S   Move one commit"),
        Line::from("  A–Z        Jump within window (26)"),
        Line::from("  PgUp/PgDn  Page by 26"),
        Line::from("  Home/End   First/Last"),
        Line::from(""),
        Line::from("DETAILS"),
        Line::from("  Enter      Details / Engage (confirm)"),
        Line::from("  d          Toggle changed files list"),
        Line::from("  p          Pin anchor (◎)"),
        Line::from("  P / F      Mark Pass / Fail"),
        Line::from(""),
        Line::from("MODES"),
        Line::from("  ? or h     Toggle this help"),
        Line::from("  Q / Esc    Quit / Back"),
        Line::from(""),
        Line::from(format!("POSITION  ({x} of {y})")),
        Line::from(""),
        Line::from("CLI TIPS"),
        Line::from("  --path <p>    Only commits touching path p"),
        Line::from("  --cmd <c>     Run tests each move (shows ✅/❌ + ms)"),
        Line::from("  --autostash   Stash dirty tree on start; pop on exit"),
        Line::from("  --worktree    Use isolated .git-trek-worktree/"),
    ];
    let p = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(p, inner);
}
