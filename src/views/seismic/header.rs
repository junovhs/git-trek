use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::{HitBox, HitTarget};
use crate::views::ViewMode;

pub fn draw_header(f: &mut Frame, area: Rect, app: &App, render: &mut crate::views::Render) {
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
    for (i, mode) in ViewMode::ALL.iter().enumerate() {
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
