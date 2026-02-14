use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::Health;
use crate::mouse::{HitBox, HitTarget};
use crate::views::Render;

use super::layout;

const CLR_STABLE: Color = Color::Rgb(46, 49, 55);
const CLR_GREW: Color = Color::Rgb(80, 180, 120);
const CLR_SHRANK: Color = Color::Rgb(180, 160, 80);
const CLR_TRAUMA: Color = Color::Rgb(200, 60, 60);
const CLR_NEW: Color = Color::Rgb(80, 140, 200);
const CLR_HOVER: Color = Color::Rgb(255, 100, 255);
const CLR_SELECTED: Color = Color::Rgb(0, 220, 220);

pub fn draw(f: &mut Frame, area: Rect, app: &App, render: &mut Render) {
    let title = format!(" TERRAIN @ {} ", app.commit_label());

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let files = app.files_at_current();
    if files.is_empty() {
        return;
    }

    let layout_data = layout::compute(&files, inner);

    for (path, lines, rect) in layout_data {
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

fn health_color(health: Health) -> Color {
    match health {
        Health::Stable | Health::Deleted => CLR_STABLE,
        Health::Grew => CLR_GREW,
        Health::Shrank => CLR_SHRANK,
        Health::Trauma => CLR_TRAUMA,
        Health::New => CLR_NEW,
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
