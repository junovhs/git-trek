mod activity;
mod render;

use std::collections::HashSet;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::app::App;
use crate::views::Render;

struct GridContext {
    start_commit: usize,
    end_commit: usize,
    current: usize,
    file_col: Rect,
    grid_col: Rect,
}

pub fn draw(f: &mut Frame, area: Rect, app: &App, render: &mut Render) {
    let title = format!(" SEISMIC MONITOR @ {} ", app.commit_label());

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let history = app.history();
    let commits = &history.commits;

    if commits.is_empty() {
        return;
    }

    let file_col_width = 18u16;
    let grid_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(file_col_width), Constraint::Min(1)])
        .split(inner);

    let file_col = grid_area.first().copied().unwrap_or_default();
    let grid_col = grid_area.get(1).copied().unwrap_or_default();

    let cell_width = 2u16;
    let visible_commits = if cell_width == 0 {
        0
    } else {
        (grid_col.width / cell_width) as usize
    };

    if visible_commits == 0 {
        return;
    }

    let total = commits.len();
    let current = app.commit_idx();
    let half_visible = visible_commits / 2;

    let start_commit = calc_start_commit(total, visible_commits, current, half_visible);
    let end_commit = (start_commit + visible_commits).min(total);
    let all_files = collect_files(
        history,
        start_commit,
        end_commit,
        app.seismic_filter_inactive(),
    );
    let sorted_files = activity::sort_by_activity(history, &all_files, start_commit, end_commit);

    let ctx = GridContext {
        start_commit,
        end_commit,
        current,
        file_col,
        grid_col,
    };

    render::draw_rows(f, app, &sorted_files, history, &ctx, render);
}

fn calc_start_commit(total: usize, visible: usize, current: usize, half: usize) -> usize {
    if total <= visible || current < half {
        return 0;
    }
    if current + half >= total {
        return total.saturating_sub(visible);
    }
    current.saturating_sub(half)
}

fn collect_files(
    history: &crate::data::History,
    start: usize,
    end: usize,
    filter_inactive: bool,
) -> Vec<String> {
    let mut file_set = HashSet::new();

    for commit_idx in start..end {
        for (path, _) in history.files_at_commit(commit_idx) {
            file_set.insert(path.to_string());
        }
    }

    let mut files: Vec<_> = file_set.into_iter().collect();

    if filter_inactive {
        files.retain(|path| activity::has_activity(history, path, start, end));
    }

    files
}
