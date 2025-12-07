use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::{app::App, data::HealthStatus, mouse::{HitBox, HitId}, views::RenderResult};

const CLR_STABLE: Color = Color::Rgb(60, 60, 60);
const CLR_GREW: Color = Color::Rgb(80, 200, 120);
const CLR_SHRANK: Color = Color::Rgb(200, 200, 80);
const CLR_FUCKED: Color = Color::Rgb(255, 80, 80);
const CLR_NEW: Color = Color::Rgb(80, 180, 255);
const CLR_HOVER: Color = Color::Rgb(255, 0, 255);
const CLR_SELECTED: Color = Color::Rgb(0, 255, 255);

pub fn draw(f: &mut Frame, app: &App) -> RenderResult {
    let mut result = RenderResult::new();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(f.area());

    draw_header(f, chunks[0], app, &mut result);
    draw_timeline(f, chunks[1], app);
    draw_treemap_area(f, chunks[2], app, &mut result);
    draw_status(f, chunks[3], app);
    result
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, result: &mut RenderResult) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(60)])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("GIT-TREK ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("v3.0", Style::default().fg(Color::DarkGray)),
    ])).block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, chunks[0]);

    let tabs = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 5); 5])
        .split(chunks[1]);

    for (i, mode) in crate::views::ViewMode::ALL.iter().enumerate() {
        let is_active = app.view == *mode;
        let is_hover = app.mouse.hover == HitId::ViewTab(i);
        let style = if is_active {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if is_hover {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let tab = Paragraph::new(format!("[{}] {}", i + 1, mode.name())).style(style);
        f.render_widget(tab, tabs[i]);
        result.hit_boxes.push(HitBox { rect: tabs[i], id: HitId::ViewTab(i) });
    }
}

fn draw_timeline(f: &mut Frame, area: Rect, app: &App) {
    let total = app.data.commits.len();
    let commit_info = app.data.commits.get(app.commit_idx);
    let title = match commit_info {
        Some(c) => format!(" {} / {} │ {} ", app.commit_idx + 1, total, c.summary.chars().take(40).collect::<String>()),
        None => " Timeline ".to_string(),
    };
    
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if total == 0 { return; }
    
    let width = inner.width as usize;
    if width == 0 { return; }
    
    let marker_pos = if total <= 1 {
        width / 2
    } else {
        (app.commit_idx * width.saturating_sub(1)) / total.saturating_sub(1)
    };
    
    let line: String = (0..width)
        .map(|i| if i == marker_pos { '◉' } else { '─' })
        .collect();
    
    let p = Paragraph::new(Span::styled(line, Style::default().fg(Color::Cyan)));
    f.render_widget(p, inner);
}

fn draw_treemap_area(f: &mut Frame, area: Rect, app: &App, result: &mut RenderResult) {
    let block = Block::default().borders(Borders::ALL).title(format!(" Files @ {} ", app.current_commit_label()));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let files = app.files_at_current_commit();
    if files.is_empty() { return; }

    let rects = compute_treemap_layout(&files, inner);
    for (path, lines, rect) in &rects {
        let health = app.file_health(path);
        let is_hover = app.mouse.hover == HitId::File(path.clone());
        let is_selected = app.selected_file.as_ref() == Some(path);
        let bg = if is_selected { CLR_SELECTED } else if is_hover { CLR_HOVER } else { health_color(health) };
        let name = truncate_path(path, rect.width as usize);
        let content = Paragraph::new(vec![Line::from(name), Line::from(format!("{} ln", lines))])
            .style(Style::default().bg(bg).fg(Color::Black))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(content, *rect);
        result.hit_boxes.push(HitBox { rect: *rect, id: HitId::File(path.clone()) });
    }
}

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    let status = match &app.selected_file {
        Some(path) => format!("Selected: {} | [R]estore [Q]uit", path),
        None => "[click] select | [scroll] time travel | [1-5] views | [Q]uit".to_string(),
    };
    f.render_widget(Paragraph::new(status).style(Style::default().fg(Color::DarkGray)), area);
}

fn health_color(status: HealthStatus) -> Color {
    match status {
        HealthStatus::Stable => CLR_STABLE,
        HealthStatus::Grew => CLR_GREW,
        HealthStatus::Shrank => CLR_SHRANK,
        HealthStatus::MaybeFucked => CLR_FUCKED,
        HealthStatus::New => CLR_NEW,
        HealthStatus::Deleted => CLR_STABLE,
    }
}

fn truncate_path(path: &str, max: usize) -> String {
    if path.len() <= max { return path.to_string(); }
    path.rsplit('/').next().unwrap_or(path).chars().take(max.saturating_sub(2)).collect::<String>() + ".."
}

fn compute_treemap_layout(files: &[(String, usize)], area: Rect) -> Vec<(String, usize, Rect)> {
    let total: usize = files.iter().map(|(_, s)| s).sum();
    if total == 0 || area.width < 2 || area.height < 2 { return vec![]; }

    let mut result = Vec::new();
    let mut remaining = area;

    for (path, lines) in files.iter().take(20) {
        if remaining.width < 3 || remaining.height < 2 { break; }
        let ratio = (*lines as f64) / (total as f64);
        let horizontal = remaining.width >= remaining.height;
        let rect = if horizontal {
            let w = ((remaining.width as f64) * ratio).max(4.0).min(remaining.width as f64) as u16;
            let r = Rect::new(remaining.x, remaining.y, w, remaining.height);
            remaining.x += w;
            remaining.width = remaining.width.saturating_sub(w);
            r
        } else {
            let h = ((remaining.height as f64) * ratio).max(3.0).min(remaining.height as f64) as u16;
            let r = Rect::new(remaining.x, remaining.y, remaining.width, h);
            remaining.y += h;
            remaining.height = remaining.height.saturating_sub(h);
            r
        };
        result.push((path.clone(), *lines, rect));
    }
    result
}