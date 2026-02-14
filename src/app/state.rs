use git2::Repository;

use crate::data::History;
use crate::mouse::MouseState;
use crate::views::ViewMode;

pub struct App {
    pub repo: Repository,
    pub history: History,
    pub view: ViewMode,
    pub commit_idx: usize,
    pub selected_file: Option<String>,
    pub mouse: MouseState,
    pub should_quit: bool,
    pub message: Option<String>,
    pub seismic_scroll: usize,
    pub seismic_filter_inactive: bool,
}
