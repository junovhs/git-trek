use anyhow::{Context, Result};
use git2::{Oid, Repository, Sort, Tree};

use crate::data::{CommitInfo, FileSnapshot, RepoData, TrackedFile};

pub fn load_repo_data(repo: &Repository, limit: usize) -> Result<RepoData> {
    let mut data = RepoData::new();
    let oids = collect_commit_oids(repo, limit)?;

    for (idx, oid) in oids.iter().enumerate() {
        let commit = repo.find_commit(*oid)?;
        let info = build_commit_info(repo, &commit)?;
        collect_file_snapshots(repo, &commit.tree()?, idx, &mut data.files)?;
        data.commits.push(info);
    }
    Ok(data)
}

fn collect_commit_oids(repo: &Repository, limit: usize) -> Result<Vec<Oid>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
    Ok(revwalk.filter_map(Result::ok).take(limit).collect())
}

fn build_commit_info(repo: &Repository, commit: &git2::Commit) -> Result<CommitInfo> {
    let author_str = commit.author().to_string();
    let summary_str = commit.summary().unwrap_or("").to_string();
    let (insertions, deletions) = get_diff_stats(repo, commit)?;

    Ok(CommitInfo {
        oid: commit.id(),
        summary: summary_str,
        author: author_str,
        timestamp: commit.time().seconds(),
        files_changed: Vec::new(),
        insertions,
        deletions,
    })
}

fn get_diff_stats(repo: &Repository, commit: &git2::Commit) -> Result<(usize, usize)> {
    if commit.parent_count() == 0 {
        return Ok((0, 0));
    }
    let parent = commit.parent(0)?;
    let diff = repo.diff_tree_to_tree(Some(&parent.tree()?), Some(&commit.tree()?), None)?;
    let stats = diff.stats()?;
    Ok((stats.insertions(), stats.deletions()))
}

fn collect_file_snapshots(
    repo: &Repository,
    tree: &Tree,
    idx: usize,
    files: &mut std::collections::HashMap<String, TrackedFile>,
) -> Result<()> {
    tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            process_blob_entry(repo, dir, entry, idx, files);
        }
        git2::TreeWalkResult::Ok
    })?;
    Ok(())
}

fn process_blob_entry(
    repo: &Repository,
    dir: &str,
    entry: &git2::TreeEntry,
    idx: usize,
    files: &mut std::collections::HashMap<String, TrackedFile>,
) {
    let path = format!("{}{}", dir, entry.name().unwrap_or(""));
    let Some(blob) = repo.find_blob(entry.id()).ok() else { return };
    let content = blob.content();
    #[allow(clippy::naive_bytecount)]
    let lines = content.iter().filter(|&&c| c == b'\n').count();
    files
        .entry(path.clone())
        .or_insert_with(|| TrackedFile::new(path))
        .history
        .insert(idx, FileSnapshot { lines, bytes: content.len() });
}

pub fn format_oid(oid: Oid) -> String {
    oid.to_string()[..8].to_string()
}

pub fn get_file_content(repo: &Repository, oid: Oid, path: &str) -> Result<String> {
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let entry = tree.get_path(std::path::Path::new(path))?;
    let blob = repo.find_blob(entry.id())?;
    Ok(std::str::from_utf8(blob.content()).context("not UTF-8")?.to_string())
}

pub fn restore_file(repo: &Repository, oid: Oid, path: &str) -> Result<()> {
    let content = get_file_content(repo, oid, path)?;
    std::fs::write(path, content)?;
    Ok(())
}