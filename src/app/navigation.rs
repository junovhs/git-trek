use crate::mouse::HitTarget;
use crate::views::ViewMode;

use super::App;

impl App {
    pub fn seismic_scroll(&self) -> usize {
        self.seismic_scroll
    }

    pub fn seismic_scroll_vertical(&mut self, delta: i32) {
        let amount = delta.unsigned_abs() as usize;
        if delta > 0 {
            self.seismic_scroll = self.seismic_scroll.saturating_add(amount);
        } else {
            self.seismic_scroll = self.seismic_scroll.saturating_sub(amount);
        }
    }

    pub fn seismic_filter_inactive(&self) -> bool {
        self.seismic_filter_inactive
    }

    pub fn toggle_seismic_filter(&mut self) {
        self.seismic_filter_inactive = !self.seismic_filter_inactive;
        self.seismic_scroll = 0;
    }

    pub fn handle_click(&mut self, target: HitTarget) {
        match target {
            HitTarget::File(path) => {
                self.selected_file = Some(path);
            }
            HitTarget::ViewTab(i) => {
                self.view = ViewMode::from_index(i);
                self.seismic_scroll = 0;
            }
            HitTarget::SeismicCell(commit_idx) => {
                self.commit_idx = commit_idx;
            }
            HitTarget::None => {}
        }
    }

    pub fn scroll_timeline(&mut self, delta: isize) {
        let max = self.history.commits.len().saturating_sub(1);
        let new_idx = if delta > 0 {
            self.commit_idx.saturating_add(delta.unsigned_abs())
        } else {
            self.commit_idx.saturating_sub(delta.unsigned_abs())
        };
        self.commit_idx = new_idx.min(max);
    }

    pub fn set_view(&mut self, mode: ViewMode) {
        self.view = mode;
        self.seismic_scroll = 0;
    }

    pub fn next_view(&mut self) {
        self.view = self.view.next();
        self.seismic_scroll = 0;
    }

    pub fn prev_view(&mut self) {
        self.view = self.view.prev();
        self.seismic_scroll = 0;
    }

    pub fn clear_selection(&mut self) {
        self.selected_file = None;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
