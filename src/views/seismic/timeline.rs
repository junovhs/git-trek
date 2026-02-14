use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
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

fn truncate_text(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}...", &text[..max.saturating_sub(3)])
    }
}
