use anyhow::Result;
use git2::Repository;
use std::path::PathBuf;

use crate::{
    cli::Cli,
    data::{HealthStatus, RepoData},
    git_ops,
    mouse::{HitId, MouseState},
    views::ViewMode,
};

pub struct App {
    pub repo: Repository,
    pub repo_dir: PathBuf,
    pub data: RepoData,
    pub view: ViewMode,
    pub commit_idx: usize,
    pub selected_file: Option<String>,
    pub mouse: MouseState,
    pub should_quit: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(cli: Cli) -> Result<Self> {
        let repo = Repository::open_from_env()?;
        let repo_dir = std::env::current_dir()?;
        let data = git_ops::load_repo_data(&repo, cli.limit)?;

        Ok(Self {
            repo,
            repo_dir,
            data,
            view: ViewMode::default(),
            commit_idx: 0,
            selected_file: None,
            mouse: MouseState::default(),
            should_quit: false,
            message: None,
        })
    }

    pub fn current_commit_label(&self) -> String {
        self.data.commits.get(self.commit_idx)
            .map(|c| git_ops::format_oid(c.oid))
            .unwrap_or_else(|| "---".to_string())
    }

    pub fn files_at_current_commit(&self) -> Vec<(String, usize)> {
        let mut files: Vec<_> = self.data.files.iter()
            .filter_map(|(path, tracked)| {
                tracked.lines_at(self.commit_idx).map(|lines| (path.clone(), lines))
            })
            .collect();
        files.sort_by(|a, b| b.1.cmp(&a.1));
        files
    }

    pub fn file_health(&self, path: &str) -> HealthStatus {
        let prev_idx = if self.commit_idx + 1 < self.data.commits.len() {
            Some(self.commit_idx + 1)
        } else { None };
        self.data.files.get(path)
            .map(|f| f.health_at(self.commit_idx, prev_idx))
            .unwrap_or(HealthStatus::Stable)
    }

    pub fn handle_click(&mut self, id: HitId) {
        match id {
            HitId::File(path) => { self.selected_file = Some(path); }
            HitId::ViewTab(i) => { self.view = ViewMode::from_index(i); }
            HitId::Commit(i) | HitId::TimelinePoint(i) => { self.commit_idx = i; }
            HitId::None => {}
        }
    }

    pub fn scroll_timeline(&mut self, delta: isize) {
        let max = self.data.commits.len().saturating_sub(1);
        let new_idx = if delta > 0 {
            self.commit_idx.saturating_add(delta as usize)
        } else {
            self.commit_idx.saturating_sub(delta.unsigned_abs())
        };
        self.commit_idx = new_idx.min(max);
    }

    pub fn set_view(&mut self, mode: ViewMode) { self.view = mode; }
    pub fn next_view(&mut self) { self.view = self.view.next(); }
    pub fn prev_view(&mut self) { self.view = self.view.prev(); }

    pub fn restore_selected(&mut self) -> Result<()> {
        let Some(path) = &self.selected_file else { return Ok(()); };
        let oid = self.data.commits[self.commit_idx].oid;
        git_ops::restore_file(&self.repo, oid, path)?;
        self.message = Some(format!("Restored {} from {}", path, git_ops::format_oid(oid)));
        Ok(())
    }
}