use git2::{Oid, Repository, Sort, Tree};

use crate::data::{Commit, FileHistory, History, Snapshot};
use crate::error::{Result, TrekError};

/// Find and open the git repository.
pub fn find_repository() -> Result<Repository> {
    Repository::open_from_env().map_err(|_| TrekError::NoRepository)
}

/// Load complete repository history up to limit commits.
pub fn load_history(repo: &Repository, limit: usize) -> Result<History> {
    let mut history = History::new();
    let oids = collect_commit_oids(repo, limit)?;

    if oids.is_empty() {
        return Err(TrekError::NoCommits);
    }

    for (idx, oid) in oids.iter().enumerate() {
        let commit = repo.find_commit(*oid)?;
        let info = build_commit_info(&commit);
        collect_file_snapshots(repo, &commit.tree()?, idx, &mut history.files)?;
        history.commits.push(info);
    }

    Ok(history)
}

/// Collect commit OIDs in reverse chronological order.
fn collect_commit_oids(repo: &Repository, limit: usize) -> Result<Vec<Oid>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
    let oids: Vec<Oid> = revwalk
        .filter_map(std::result::Result::ok)
        .take(limit)
        .collect();
    Ok(oids)
}

/// Build commit info from a git commit.
fn build_commit_info(commit: &git2::Commit) -> Commit {
    let summary = commit.summary().unwrap_or("").to_string();
    Commit::new(commit.id(), summary)
}

/// Collect file snapshots from a tree.
fn collect_file_snapshots(
    repo: &Repository,
    tree: &Tree,
    commit_idx: usize,
    files: &mut std::collections::HashMap<String, FileHistory>,
) -> Result<()> {
    tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            if let Ok(blob) = repo.find_blob(entry.id()) {
                let path = format!("{}{}", dir, entry.name().unwrap_or(""));
                let content = blob.content();
                let lines = count_lines(content);

                files
                    .entry(path.clone())
                    .or_default()
                    .snapshots
                    .insert(commit_idx, Snapshot { lines });
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(())
}

/// Count lines in a byte slice.
#[allow(clippy::naive_bytecount)]
fn count_lines(content: &[u8]) -> usize {
    content.iter().filter(|&&c| c == b'\n').count()
}

/// Get file content at a specific commit.
pub fn get_file_content(repo: &Repository, oid: Oid, path: &str) -> Result<String> {
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let entry = tree.get_path(std::path::Path::new(path))?;
    let blob = repo.find_blob(entry.id())?;

    std::str::from_utf8(blob.content())
        .map(std::string::ToString::to_string)
        .map_err(|_| TrekError::InvalidUtf8)
}

/// Restore a file from a specific commit to the working directory.
pub fn restore_file(repo: &Repository, oid: Oid, path: &str) -> Result<()> {
    let content = get_file_content(repo, oid, path)?;
    std::fs::write(path, content)?;
    Ok(())
}
