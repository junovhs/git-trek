use anyhow::{Context, Result};
use git2::{Oid, Repository, Sort};

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

        let author_str = commit.author().to_string();
        let summary_str = commit.summary().unwrap_or("").to_string();
        
        let mut info = CommitInfo {
            oid: *oid,
            summary: summary_str,
            author: author_str,
            timestamp: commit.time().seconds(),
            files_changed: Vec::new(),
            insertions: 0,
            deletions: 0,
        };

        if commit.parent_count() > 0 {
            if let Ok(parent) = commit.parent(0) {
                let parent_tree = parent.tree()?;
                let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;
                let stats = diff.stats()?;
                info.insertions = stats.insertions();
                info.deletions = stats.deletions();
            }
        }

        tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                let path = format!("{}{}", dir, entry.name().unwrap_or(""));
                if let Ok(blob) = repo.find_blob(entry.id()) {
                    let content = blob.content();
                    #[allow(clippy::naive_bytecount)]
                    let lines = content.iter().filter(|&&c| c == b'\n').count();
                    let bytes = content.len();
                    data.files
                        .entry(path.clone())
                        .or_insert_with(|| TrackedFile::new(path))
                        .history
                        .insert(idx, FileSnapshot { lines, bytes });
                }
            }
            git2::TreeWalkResult::Ok
        })?;

        data.commits.push(info);
    }
    Ok(data)
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