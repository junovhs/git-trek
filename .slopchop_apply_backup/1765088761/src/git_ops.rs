use anyhow::{Context, Result};
use git2::{ObjectType, Oid, Repository, Sort, Tree, TreeEntry};
use std::collections::HashMap;

use crate::data::{CommitInfo, FileSnapshot, RepoData, TrackedFile};

pub fn load_repo_data(repo: &Repository, limit: usize) -> Result<RepoData> {
    let mut data = RepoData::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;

    let oids: Vec<Oid> = revwalk.filter_map(Result::ok).take(limit).collect();

    for (idx, oid) in oids.iter().enumerate() {
        let commit = repo.find_commit(*oid)?;
        let tree = commit.tree()?;
        record_tree_files(repo, &tree, idx, &mut data.files)?;
        data.commits.push(CommitInfo { oid: *oid });
    }
    Ok(data)
}

fn record_tree_files(
    repo: &Repository,
    tree: &Tree,
    idx: usize,
    files: &mut HashMap<String, TrackedFile>,
) -> Result<()> {
    tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        process_tree_entry(repo, dir, entry, idx, files);
        git2::TreeWalkResult::Ok
    })?;
    Ok(())
}

fn process_tree_entry(
    repo: &Repository,
    dir: &str,
    entry: &TreeEntry,
    idx: usize,
    files: &mut HashMap<String, TrackedFile>,
) {
    if entry.kind() != Some(ObjectType::Blob) {
        return;
    }
    let Some(name) = entry.name() else { return };
    let Ok(blob) = repo.find_blob(entry.id()) else { return };
    let path = format!("{dir}{name}");
    let content = blob.content();
    #[allow(clippy::naive_bytecount)]
    let lines = content.iter().filter(|&&c| c == b'\n').count();
    files
        .entry(path)
        .or_insert_with(TrackedFile::new)
        .history
        .insert(idx, FileSnapshot { lines });
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