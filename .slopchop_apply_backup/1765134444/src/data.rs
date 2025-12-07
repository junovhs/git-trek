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

fn ratio_to_health(old: usize, new: usize) -> HealthStatus {
    if old == 0 {
        return HealthStatus::New;
    }
    #[allow(clippy::cast_precision_loss)]
    let ratio = new as f64 / old as f64;
    if ratio < 0.7 {
        HealthStatus::MaybeFucked
    } else if ratio < 0.95 {
        HealthStatus::Shrank
    } else if ratio > 1.05 {
        HealthStatus::Grew
    } else {
        HealthStatus::Stable
    }
}

impl HealthStatus {
    pub fn from_size_change(old: Option<usize>, new: Option<usize>) -> Self {
        match (old, new) {
            (None, None) => Self::Stable,
            (None, Some(_)) => Self::New,
            (Some(_), None) => Self::Deleted,
            (Some(o), Some(n)) => ratio_to_health(o, n),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileSnapshot {
    pub lines: usize,
}

#[derive(Clone, Debug)]
pub struct TrackedFile {
    pub history: HashMap<usize, FileSnapshot>,
}

impl TrackedFile {
    pub fn new() -> Self {
        Self { history: HashMap::new() }
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

impl Default for TrackedFile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub oid: Oid,
}

#[derive(Clone, Default)]
pub struct RepoData {
    pub commits: Vec<CommitInfo>,
    pub files: HashMap<String, TrackedFile>,
}

impl RepoData {
    pub fn new() -> Self {
        Self::default()
    }
}