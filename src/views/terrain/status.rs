use ratatui::{layout::Rect, style::Style, widgets::Paragraph, Frame};

use crate::app::App;

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let status = match app.selected_file() {
        Some(path) => format!(" {path} │ [R]estore [Esc]clear [Q]uit "),
        None => " [click]select [scroll]time [1-6]views [Q]uit ".to_string(),
    };

    f.render_widget(
        Paragraph::new(status).style(Style::default().fg(ratatui::style::Color::DarkGray)),
        area,
    );
}
