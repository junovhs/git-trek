use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use git2::{DiffOptions, Oid, Repository, Sort, StatusOptions, Tree};
use std::path::PathBuf;

use crate::cli::Cli;

#[derive(Clone)]
pub struct Point {
    pub oid: Oid,
    pub summary: String,
}

pub fn format_oid(oid: Oid) -> String {
    oid.to_string()[..8].to_string()
}

pub fn format_summary(summary: &str) -> String {
    summary.chars().take(50).collect()
}

pub fn parse_since(since: Option<&str>) -> Result<Option<i64>> {
    let Some(s) = since else { return Ok(None) };
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").context("--since format")?;
    let ts = DateTime::<Utc>::from_naive_utc_and_offset(
        d.and_hms_opt(0, 0, 0).context("invalid time")?,
        Utc,
    )
    .timestamp();
    Ok(Some(ts))
}

pub fn check_if_dirty(repo: &mut Repository) -> Result<bool> {
    let mut so = StatusOptions::new();
    so.include_untracked(true).recurse_untracked_dirs(true);
    Ok(!repo.statuses(Some(&mut so))?.is_empty())
}

pub fn do_autostash(repo: &mut Repository) -> Result<Oid> {
    let sig = repo.signature()?;
    let oid = repo.stash_save(&sig, "git-trek autostash", Some(git2::StashFlags::INCLUDE_UNTRACKED))?;
    Ok(oid)
}

pub fn spawn_worktree() -> Result<(PathBuf, Repository)> {
    let dir = std::env::current_dir()?.join(".git-trek-worktree");
    let ok = std::process::Command::new("git")
        .args(["worktree", "add", "--force", "--detach", &dir.to_string_lossy(), "HEAD"])
        .status()?
        .success();
    if !ok {
        return Err(anyhow!("git worktree add failed"));
    }
    let wt = Repository::open(&dir)?;
    Ok((dir, wt))
}

pub fn head_info(repo: &Repository) -> Result<(Oid, String)> {
    let head = repo.head()?;
    let oid = head.target().context("no HEAD target")?;
    let name = head.shorthand().unwrap_or("HEAD").to_string();
    Ok((oid, name))
}

pub fn new_session(repo: &Repository, head: Oid) -> Result<String> {
    let name = format!("_trek_session_{}", std::time::UNIX_EPOCH.elapsed()?.as_millis());
    let commit = repo.find_commit(head)?;
    repo.branch(&name, &commit, true)?;
    repo.set_head(&format!("refs/heads/{name}"))?;
    Ok(name)
}

pub fn load_commits(repo: &Repository, cli: &Cli, since_ts: Option<i64>) -> Result<Vec<Point>> {
    let mut revwalk = repo.revwalk()?;
    if cli.flags.all() {
        revwalk.push_glob("refs/*")?;
    } else {
        revwalk.push_head()?;
    }
    revwalk.set_sorting(Sort::TOPOLOGICAL)?;

    let mut commits = Vec::with_capacity(cli.limit);
    for id in revwalk {
        let oid = id?;
        if let Some(p) = filter_commit(repo, oid, cli, since_ts)? {
            commits.push(p);
            if commits.len() >= cli.limit {
                break;
            }
        }
    }
    Ok(commits)
}

fn filter_commit(repo: &Repository, oid: Oid, cli: &Cli, since_ts: Option<i64>) -> Result<Option<Point>> {
    let commit = repo.find_commit(oid)?;

    if let Some(ts) = since_ts {
        if commit.time().seconds() < ts {
            return Ok(None);
        }
    }
    if let Some(author) = &cli.author {
        if !commit.author().to_string().to_lowercase().contains(&author.to_lowercase()) {
            return Ok(None);
        }
    }
    if let Some(path) = &cli.path {
        if !touches_path(repo, &commit, path)? {
            return Ok(None);
        }
    }
    let summary = format_summary(commit.summary().unwrap_or(""));
    Ok(Some(Point { oid, summary }))
}

fn touches_path(repo: &Repository, commit: &git2::Commit, path: &str) -> Result<bool> {
    let tree = commit.tree()?;
    let parent_tree: Option<Tree> = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };
    let mut opts = DiffOptions::new();
    opts.pathspec(path);
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))?;
    Ok(diff.stats()?.files_changed() > 0)
}