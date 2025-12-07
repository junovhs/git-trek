use git2::Oid;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Stable,
    Grew,
    Shrank,
    MaybeFucked,
    New,
    Deleted,
}

impl HealthStatus {
    pub fn from_size_change(old: Option<usize>, new: Option<usize>) -> Self {
        match (old, new) {
            (None, None) => Self::Stable,
            (None, Some(_)) => Self::New,
            (Some(_), None) => Self::Deleted,
            (Some(o), Some(n)) => Self::from_ratio(o, n),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn from_ratio(old: usize, new: usize) -> Self {
        if old == 0 { return Self::New; }
        let ratio = new as f64 / old as f64;
        if ratio < 0.7 { Self::MaybeFucked }
        else if ratio < 0.95 { Self::Shrank }
        else if ratio > 1.05 { Self::Grew }
        else { Self::Stable }
    }
}

#[derive(Clone, Debug)]
pub struct FileSnapshot {
    pub lines: usize,
    #[allow(dead_code)]
    pub bytes: usize,
}

#[derive(Clone, Debug)]
pub struct TrackedFile {
    #[allow(dead_code)]
    pub path: String,
    pub history: HashMap<usize, FileSnapshot>,
}

impl TrackedFile {
    pub fn new(path: String) -> Self {
        Self { path, history: HashMap::new() }
    }

    pub fn lines_at(&self, commit_idx: usize) -> Option<usize> {
        self.history.get(&commit_idx).map(|s| s.lines)
    }

    pub fn health_at(&self, commit_idx: usize, prev_idx: Option<usize>) -> HealthStatus {
        let new_lines = self.lines_at(commit_idx);
        let old_lines = prev_idx.and_then(|i| self.lines_at(i));
        HealthStatus::from_size_change(old_lines, new_lines)
    }
}

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub oid: Oid,
    pub summary: String,
    #[allow(dead_code)]
    pub author: String,
    #[allow(dead_code)]
    pub timestamp: i64,
    #[allow(dead_code)]
    pub files_changed: Vec<String>,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Clone, Default)]
pub struct RepoData {
    pub commits: Vec<CommitInfo>,
    pub files: HashMap<String, TrackedFile>,
}

impl RepoData {
    pub fn new() -> Self { Self::default() }
}