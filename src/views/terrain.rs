mod layout;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::Health;
use crate::mouse::{HitBox, HitTarget};
use crate::views::Render;

const CLR_STABLE: Color = Color::Rgb(46, 49, 55);
const CLR_GREW: Color = Color::Rgb(80, 180, 120);
const CLR_SHRANK: Color = Color::Rgb(180, 160, 80);
const CLR_TRAUMA: Color = Color::Rgb(200, 60, 60);
const CLR_NEW: Color = Color::Rgb(80, 140, 200);
const CLR_HOVER: Color = Color::Rgb(255, 100, 255);
const CLR_SELECTED: Color = Color::Rgb(0, 220, 220);

pub fn draw(f: &mut Frame, app: &App) -> Render {
    let mut render = Render::new();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(f.area());

    let (header, timeline, terrain, status) = (
        chunks.first().copied().unwrap_or_default(),
        chunks.get(1).copied().unwrap_or_default(),
        chunks.get(2).copied().unwrap_or_default(),
        chunks.get(3).copied().unwrap_or_default(),
    );

    draw_header(f, header, app, &mut render);
    draw_timeline(f, timeline, app);
    draw_terrain(f, terrain, app, &mut render);
    draw_status(f, status, app);

    render
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, render: &mut Render) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(60)])
        .split(area);

    let header_area = chunks.first().copied().unwrap_or_default();
    let tabs_area = chunks.get(1).copied().unwrap_or_default();

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "GIT-TREK ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v3.0", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, header_area);

    let tab_width = 10u16;
    for (i, mode) in crate::views::ViewMode::ALL.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let x = tabs_area.x + (i as u16 * tab_width);
        let rect = Rect::new(x, tabs_area.y, tab_width, tabs_area.height);

        let is_active = app.view() == *mode;
        let is_hover = app.mouse().hover == HitTarget::ViewTab(i);

        let style = if is_active {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if is_hover {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let label = format!("[{}] {}", i + 1, mode.name());
        f.render_widget(Paragraph::new(label).style(style), rect);
        render
            .hit_boxes
            .push(HitBox::new(rect, HitTarget::ViewTab(i)));
    }
}

fn draw_timeline(f: &mut Frame, area: Rect, app: &App) {
    let total = app.commit_count();
    let current = app.commit_idx();

    let title = match app.current_commit() {
        Some(c) => format!(
            " {} / {} │ {} ",
            current + 1,
            total,
            truncate_text(&c.summary, 50)
        ),
        None => " Timeline ".to_string(),
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if total == 0 || inner.width == 0 {
        return;
    }

    let width = inner.width as usize;
    let marker_pos = if total <= 1 {
        width / 2
    } else {
        (current * width.saturating_sub(1)) / total.saturating_sub(1)
    };

    let line: String = (0..width)
        .map(|i| if i == marker_pos { '◉' } else { '─' })
        .collect();

    f.render_widget(
        Paragraph::new(Span::styled(line, Style::default().fg(Color::Cyan))),
        inner,
    );
}

fn draw_terrain(f: &mut Frame, area: Rect, app: &App, render: &mut Render) {
    let title = format!(" TERRAIN @ {} ", app.commit_label());

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let files = app.files_at_current();
    if files.is_empty() {
        return;
    }

    let layout = layout::compute(&files, inner);

    for (path, lines, rect) in layout {
        let health = app.file_health(&path);
        let is_hover = app.mouse().hover == HitTarget::File(path.clone());
        let is_selected = app.selected_file() == Some(&path);

        let bg = if is_selected {
            CLR_SELECTED
        } else if is_hover {
            CLR_HOVER
        } else {
            health_color(health)
        };

        let display_name = truncate_path(&path, rect.width as usize);
        let content = vec![
            Line::styled(display_name, Style::default().fg(Color::Black)),
            Line::styled(format!("{lines}"), Style::default().fg(Color::Black)),
        ];

        let para = Paragraph::new(content)
            .style(Style::default().bg(bg))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(para, rect);
        render
            .hit_boxes
            .push(HitBox::new(rect, HitTarget::File(path)));
    }
}

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    let status = match app.selected_file() {
        Some(path) => format!(" {path} │ [R]estore [Esc]clear [Q]uit "),
        None => " [click]select [scroll]time [1-6]views [Q]uit ".to_string(),
    };

    f.render_widget(
        Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn health_color(health: Health) -> Color {
    match health {
        Health::Stable | Health::Deleted => CLR_STABLE,
        Health::Grew => CLR_GREW,
        Health::Shrank => CLR_SHRANK,
        Health::Trauma => CLR_TRAUMA,
        Health::New => CLR_NEW,
    }
}

fn truncate_text(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}...", &text[..max.saturating_sub(3)])
    }
}

fn truncate_path(path: &str, max: usize) -> String {
    if path.len() <= max {
        return path.to_string();
    }
    if max <= 2 {
        return path.chars().take(max).collect();
    }

    let filename = path.rsplit('/').next().unwrap_or(path);
    if filename.len() <= max {
        return filename.to_string();
    }

    format!(
        "{}..",
        filename
            .chars()
            .take(max.saturating_sub(2))
            .collect::<String>()
    )
}
