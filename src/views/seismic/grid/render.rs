use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::mouse::{HitBox, HitTarget};
use crate::views::Render;

use super::super::intensity::{format_cell, Intensity};
use super::GridContext;
use crate::data::History;

pub fn draw_rows(
    f: &mut Frame,
    app: &App,
    files: &[String],
    history: &History,
    ctx: &GridContext,
    render: &mut Render,
) {
    let start_y = ctx.file_col.y + 1;
    let max_rows = (ctx.file_col.height.saturating_sub(1)) as usize;
    let scroll_offset = app.seismic_scroll();

    for (row_idx, path) in files.iter().skip(scroll_offset).take(max_rows).enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let y = start_y + row_idx as u16;

        draw_file_name(f, app, path, ctx.file_col, y, render);
        draw_grid_cells(f, history, path, ctx, y, render);
    }
}

fn draw_file_name(
    f: &mut Frame,
    app: &App,
    path: &str,
    file_col: Rect,
    y: u16,
    render: &mut Render,
) {
    let display_name = truncate_path(path, (file_col.width as usize).saturating_sub(1));
    let is_selected = app.selected_file() == Some(path);
    let name_style = if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let file_rect = Rect::new(file_col.x, y, file_col.width, 1);
    f.render_widget(
        Paragraph::new(Line::styled(display_name, name_style)),
        file_rect,
    );
    render
        .hit_boxes
        .push(HitBox::new(file_rect, HitTarget::File(path.to_string())));
}

fn draw_grid_cells(
    f: &mut Frame,
    history: &History,
    path: &str,
    ctx: &GridContext,
    y: u16,
    render: &mut Render,
) {
    let mut row_line = Vec::new();
    for (i, _commit) in history
        .commits
        .iter()
        .enumerate()
        .take(ctx.end_commit)
        .skip(ctx.start_commit)
    {
        let intensity = Intensity::calc(history, path, i);
        let is_current = i == ctx.current;
        let (cell, style) = format_cell(intensity, is_current);
        row_line.push(Span::styled(cell, style));
    }
    f.render_widget(
        Paragraph::new(Line::from(row_line)),
        Rect::new(ctx.grid_col.x, y, ctx.grid_col.width, 1),
    );

    for (col_idx, i) in (ctx.start_commit..ctx.end_commit).enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let cell_x = ctx.grid_col.x + (col_idx as u16 * 2);
        let cell_rect = Rect::new(cell_x, y, 2, 1);
        render
            .hit_boxes
            .push(HitBox::new(cell_rect, HitTarget::SeismicCell(i)));
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
