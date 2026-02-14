use ratatui::{layout::Rect, style::Style, widgets::Paragraph, Frame};

use crate::app::App;

pub fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    let filter_status = if app.seismic_filter_inactive() {
        " [FILTER: active files only]"
    } else {
        ""
    };

    let status = match app.selected_file() {
        Some(path) => {
            format!(" {path} │ [R]estore [F]ilter [j/k]scroll [Q]uit{filter_status} ")
        }
        None => {
            format!(" [scroll]time [j/k]files [F]ilter [1-6]views [Q]uit{filter_status} ")
        }
    };

    f.render_widget(
        Paragraph::new(status).style(Style::default().fg(ratatui::style::Color::DarkGray)),
        area,
    );
}
