use std::collections::HashMap;

use git2::Oid;

/// Health status of a file based on change magnitude.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Health {
    #[default]
    Stable,
    Grew,
    Shrank,
    Trauma,
    New,
    Deleted,
}

impl Health {
    /// Determine health from old and new line counts.
    pub fn from_change(old: Option<usize>, new: Option<usize>) -> Self {
        match (old, new) {
            (None, None) => Self::Stable,
            (None, Some(_)) => Self::New,
            (Some(_), None) => Self::Deleted,
            (Some(o), Some(n)) => Self::from_ratio(o, n),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn from_ratio(old: usize, new: usize) -> Self {
        if old == 0 {
            return Self::New;
        }
        let ratio = new as f64 / old as f64;
        if ratio < 0.7 {
            Self::Trauma
        } else if ratio < 0.95 {
            Self::Shrank
        } else if ratio > 1.05 {
            Self::Grew
        } else {
            Self::Stable
        }
    }
}

/// Snapshot of a file at a specific commit.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub lines: usize,
}

/// A file tracked across commits.
#[derive(Clone, Debug, Default)]
pub struct FileHistory {
    pub snapshots: HashMap<usize, Snapshot>,
}

impl FileHistory {
    pub fn lines_at(&self, commit_idx: usize) -> Option<usize> {
        self.snapshots.get(&commit_idx).map(|s| s.lines)
    }

    pub fn health_at(&self, commit_idx: usize, prev_idx: Option<usize>) -> Health {
        let new_lines = self.lines_at(commit_idx);
        let old_lines = prev_idx.and_then(|i| self.lines_at(i));
        Health::from_change(old_lines, new_lines)
    }
}

/// Information about a single commit.
#[derive(Clone, Debug)]
pub struct Commit {
    pub oid: Oid,
    pub short_oid: String,
    pub summary: String,
}

impl Commit {
    pub fn new(oid: Oid, summary: String) -> Self {
        let short_oid = oid.to_string()[..8].to_string();
        Self {
            oid,
            short_oid,
            summary,
        }
    }
}

/// Complete repository data loaded for visualization.
#[derive(Clone, Default)]
pub struct History {
    pub commits: Vec<Commit>,
    pub files: HashMap<String, FileHistory>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn files_at_commit(&self, commit_idx: usize) -> Vec<(&str, usize)> {
        let mut files: Vec<_> = self
            .files
            .iter()
            .filter_map(|(path, history)| {
                history
                    .lines_at(commit_idx)
                    .map(|lines| (path.as_str(), lines))
            })
            .collect();
        files.sort_by(|a, b| b.1.cmp(&a.1));
        files
    }
}
