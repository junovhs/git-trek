mod navigation;
mod state;

use git2::Repository;

use crate::cli::Cli;
use crate::data::{Commit, Health, History};
use crate::error::Result;
use crate::git_ops;
use crate::mouse::{HitTarget, MouseState};
use crate::views::ViewMode;

pub use navigation::scroll_timeline;
pub use state::App;

impl App {
    pub fn new(cli: &Cli) -> Result<Self> {
        let repo = git_ops::find_repository()?;
        let history = git_ops::load_history(&repo, cli.limit)?;

        Ok(Self {
            repo,
            history,
            view: ViewMode::default(),
            commit_idx: 0,
            selected_file: None,
            mouse: MouseState::default(),
            should_quit: false,
            message: None,
            seismic_scroll: 0,
            seismic_filter_inactive: false,
        })
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn view(&self) -> ViewMode {
        self.view
    }

    pub fn commit_idx(&self) -> usize {
        self.commit_idx
    }

    pub fn commit_count(&self) -> usize {
        self.history.commits.len()
    }

    pub fn current_commit(&self) -> Option<&Commit> {
        self.history.commits.get(self.commit_idx)
    }

    pub fn commit_label(&self) -> String {
        self.current_commit()
            .map_or_else(|| "---".to_string(), |c| c.short_oid.clone())
    }

    pub fn files_at_current(&self) -> Vec<(&str, usize)> {
        self.history.files_at_commit(self.commit_idx)
    }

    pub fn file_health(&self, path: &str) -> Health {
        let prev_idx = self.prev_commit_idx();
        self.history
            .files
            .get(path)
            .map(|f| f.health_at(self.commit_idx, prev_idx))
            .unwrap_or_default()
    }

    pub fn selected_file(&self) -> Option<&str> {
        self.selected_file.as_deref()
    }

    pub fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    pub fn mouse_mut(&mut self) -> &mut MouseState {
        &mut self.mouse
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn prev_commit_idx(&self) -> Option<usize> {
        if self.commit_idx + 1 < self.history.commits.len() {
            Some(self.commit_idx + 1)
        } else {
            None
        }
    }

    pub fn restore_selected(&mut self) -> Result<()> {
        let Some(path) = &self.selected_file else {
            return Ok(());
        };

        let Some(commit) = self.current_commit() else {
            return Ok(());
        };

        git_ops::restore_file(&self.repo, commit.oid, path)?;

        self.message = Some(format!("Restored {} from {}", path, commit.short_oid));
        Ok(())
    }
}
