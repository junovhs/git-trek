mod header;
mod layout;
mod render;
mod status;
mod timeline;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;
use crate::views::Render;

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

    let (header_area, timeline_area, terrain_area, status_area) = (
        chunks.first().copied().unwrap_or_default(),
        chunks.get(1).copied().unwrap_or_default(),
        chunks.get(2).copied().unwrap_or_default(),
        chunks.get(3).copied().unwrap_or_default(),
    );

    header::draw(f, header_area, app, &mut render);
    timeline::draw(f, timeline_area, app);
    render::draw(f, terrain_area, app, &mut render);
    status::draw(f, status_area, app);

    render
}
