use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Oid, Repository};
use std::{collections::HashMap, time::Duration};

use crate::{git_ops::{format_oid, Point}, shell};

#[derive(Clone, Default)]
pub struct Detail {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub insertions: usize,
    pub deletions: usize,
    pub test_ok: Option<bool>,
    pub test_ms: Option<u128>,
    pub manual: Option<bool>,
}

pub fn load_detail(
    repo: &Repository,
    point: &Point,
    tests: &HashMap<Oid, (Option<bool>, Option<u128>)>,
    marks: &HashMap<Oid, bool>,
) -> Result<Detail> {
    let oid = point.oid;
    let commit = repo.find_commit(oid)?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit.tree()?), None)?;
    let stats = diff.stats()?;
    let ts = DateTime::<Utc>::from_timestamp(commit.time().seconds(), 0).context("ts")?;
    let (ok, ms) = tests.get(&oid).copied().unwrap_or((None, None));
    let manual = marks.get(&oid).copied();

    Ok(Detail {
        hash: format_oid(commit.id()),
        author: commit.author().to_string(),
        date: ts.to_rfc2822(),
        message: commit.message().unwrap_or("").to_string(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
        test_ok: ok,
        test_ms: ms,
        manual,
    })
}

pub fn run_cmd(app: &mut crate::app::App) -> Result<()> {
    let oid = app.commits[app.idx].oid;

    if let Some((ok, ms)) = app.tests.get(&oid).copied() {
        app.detail.test_ok = ok;
        app.detail.test_ms = ms;
        return Ok(());
    }

    let Some(cmd) = app.opts.cmd.clone() else {
        return Ok(());
    };

    let timeout = (app.opts.cmd_timeout > 0).then(|| Duration::from_secs(app.opts.cmd_timeout));
    let start = std::time::Instant::now();
    let ok = shell::run(&cmd, timeout, &app.repo_dir)?;
    let ms = start.elapsed().as_millis();

    app.tests.insert(oid, (Some(ok), Some(ms)));
    app.detail.test_ok = Some(ok);
    app.detail.test_ms = Some(ms);
    Ok(())
}