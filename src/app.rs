use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use git2::{
    build::CheckoutBuilder, DiffDelta, DiffOptions, Oid, Repository, ResetType, Signature, Sort,
    StatusOptions,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::Cli;
use crate::shell;

pub const WINDOW_SIZE: usize = 26;
pub const MAX_FILES_SHOWN: usize = 30;
pub const EVENT_POLL_MS: u64 = 100;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AppState {
    Navigating,
    Detail,
    Confirm,
}

#[derive(Clone, Default)]
pub struct FileChange {
    pub path: String,
    pub status: String,
}

#[derive(Clone, Default)]
pub struct Detail {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub files: Vec<FileChange>,
    pub show_files: bool,
    pub test_ok: Option<bool>,
    pub test_ms: Option<u128>,
    pub manual: Option<bool>,
}

#[derive(Clone)]
pub struct Item {
    pub oid: Oid,
    pub summary: String,
}

pub struct App {
    pub state: AppState,
    pub repo: Repository,
    pub repo_dir: PathBuf,
    pub commits: Vec<Item>,
    pub labels: HashMap<Oid, Vec<String>>,
    pub idx: usize,
    pub anchor: usize,
    pub pinned: bool,
    pub scroll: usize,
    pub original_branch: String,
    pub session_branch: Option<String>,
    pub autostash: bool,
    pub used_worktree: bool,
    pub worktree_dir: Option<PathBuf>,
    pub opts: Cli,

    pub detail: Detail,
    pub tests: HashMap<Oid, (Option<bool>, Option<u128>)>,
    pub marks: HashMap<Oid, bool>,
    pub should_quit: bool,
    pub final_message: Option<String>,
}

impl App {
    pub fn event_poll_ms(&self) -> u64 {
        EVENT_POLL_MS
    }

    pub fn new(cli: Cli) -> Result<Self> {
        let mut cwd_repo = Repository::open_from_env().context("not a git repo")?;
        let since_ts = parse_since(&cli.since)?;
        let autostash = maybe_autostash(&mut cwd_repo, cli.autostash)?;

        let (repo, repo_dir, worktree_dir, used_worktree) = if cli.worktree {
            let (dir, wt) = spawn_worktree()?;
            (wt, dir.clone(), Some(dir), true)
        } else {
            (cwd_repo, std::env::current_dir()?, None, false)
        };

        let (head_oid, branch) = head_info(&repo)?;
        let session_branch = if !cli.worktree {
            Some(new_session(&repo, head_oid)?)
        } else {
            None
        };

        let commits = load_commits(&repo, &cli, since_ts)?;
        let labels = collect_labels(&repo)?;
        let idx = commits
            .iter()
            .position(|c| c.oid == head_oid)
            .unwrap_or(0);

        Ok(Self {
            state: AppState::Navigating,
            repo,
            repo_dir,
            commits,
            labels,
            idx,
            anchor: idx,
            pinned: false,
            scroll: 0,
            original_branch: branch,
            session_branch,
            autostash: autostash.is_some(),
            used_worktree,
            worktree_dir,
            opts: cli,

            detail: Detail::default(),
            tests: HashMap::new(),
            marks: HashMap::new(),
            should_quit: false,
            final_message: None,
        })
    }

    pub fn refresh_view(&mut self) -> Result<()> {
        self.load_detail()?;
        if self.opts.cmd.is_some() {
            self.run_cmd()?;
        }
        Ok(())
    }

    pub fn move_sel(&mut self, delta: isize) -> Result<()> {
        let new = self.idx as isize + delta;
        if new < 0 || (new as usize) >= self.commits.len() {
            return Ok(());
        }
        self.idx = new as usize;
        self.update_checkout()?;
        self.adjust_scroll();
        self.refresh_view()?;
        Ok(())
    }

    pub fn page(&mut self, pages: isize) -> Result<()> {
        self.move_sel(WINDOW_SIZE as isize * pages)
    }
    pub fn home(&mut self) -> Result<()> {
        self.idx = 0;
        self.update_checkout()?;
        self.adjust_scroll();
        self.refresh_view()
    }
    pub fn end(&mut self) -> Result<()> {
        if !self.commits.is_empty() {
            self.idx = self.commits.len() - 1;
            self.update_checkout()?;
            self.adjust_scroll();
            self.refresh_view()?;
        }
        Ok(())
    }

    pub fn jump_letter(&mut self, c: char) -> Result<()> {
        let up = c.to_ascii_uppercase();
        if !(('A'..='Z').contains(&up)) {
            return Ok(());
        }
        let pos = (up as u8 - b'A') as usize + self.scroll;
        if pos < self.commits.len() {
            self.idx = pos;
            self.update_checkout()?;
            self.adjust_scroll();
            self.refresh_view()?;
        }
        Ok(())
    }

    pub fn pin_anchor(&mut self) {
        self.anchor = self.idx;
        self.pinned = true;
    }
    pub fn toggle_diff(&mut self) {
        self.detail.show_files = !self.detail.show_files;
    }
    pub fn mark_manual(&mut self, pass: bool) {
        let oid = self.commits[self.idx].oid;
        self.marks.insert(oid, pass);
        self.detail.manual = Some(pass);
    }

    pub fn enter_detail(&mut self) -> Result<()> {
        self.load_detail()?;
        self.state = AppState::Detail;
        Ok(())
    }
    pub fn exit_detail(&mut self) {
        self.state = AppState::Navigating;
    }
    pub fn enter_confirm(&mut self) {
        self.state = AppState::Confirm;
    }
    pub fn exit_confirm(&mut self) {
        self.state = AppState::Detail;
    }

    pub fn stop(&mut self) -> Result<String> {
        self.cleanup()?;
        self.should_quit = true;
        Ok("Returned to original timeline.".into())
    }

    pub fn checkout(&mut self) -> Result<String> {
        let oid = self.commits[self.idx].oid;
        self.cleanup()?;
        if self.used_worktree {
            self.should_quit = true;
            return Ok(format!("Now at {} (worktree).", short(&oid)));
        }
        self.repo.set_head_detached(oid)?;
        self.repo
            .checkout_head(Some(CheckoutBuilder::new().force()))?;
        self.should_quit = true;
        Ok(format!(
            "Now at {}.\nTo return: git switch {}",
            short(&oid),
            self.original_branch
        ))
    }

    pub fn x_of_y(&self) -> (usize, usize) {
        (self.idx + 1, self.commits.len())
    }

    pub fn detail_for(&self, oid: Oid) -> Option<Detail> {
        if self.commits.get(self.idx).map(|c| c.oid) == Some(oid) {
            return Some(self.detail.clone());
        }
        let mut d = Detail::default();
        if let Some((ok, ms)) = self.tests.get(&oid) {
            d.test_ok = *ok;
            d.test_ms = *ms;
        }
        if let Some(m) = self.marks.get(&oid) {
            d.manual = Some(*m);
        }
        Some(d)
    }

    // --- internals ---

    fn run_cmd(&mut self) -> Result<()> {
        let oid = self.commits[self.idx].oid;
        if self.tests.contains_key(&oid) {
            let (ok, ms) = self.tests[&oid];
            self.detail.test_ok = ok;
            self.detail.test_ms = ms;
            return Ok(());
        }
        let cmd = self.opts.cmd.clone().unwrap();
        let to = (self.opts.cmd_timeout > 0)
            .then(|| std::time::Duration::from_secs(self.opts.cmd_timeout));
        let start = std::time::Instant::now();
        let ok = shell::run(&cmd, to, &self.repo_dir)?;
        let ms = start.elapsed().as_millis();
        self.tests.insert(oid, (Some(ok), Some(ms)));
        self.detail.test_ok = Some(ok);
        self.detail.test_ms = Some(ms);
        Ok(())
    }

    fn update_checkout(&self) -> Result<()> {
        let oid = self.commits[self.idx].oid;
        let c = self.repo.find_commit(oid)?;
        self.repo.reset(c.as_object(), ResetType::Hard, None)?;
        Ok(())
    }

    fn adjust_scroll(&mut self) {
        debug_assert!(self.idx < self.commits.len());
        if self.idx < self.scroll {
            self.scroll = self.idx;
        }
        if self.idx >= self.scroll + WINDOW_SIZE {
            self.scroll = self.idx - (WINDOW_SIZE - 1);
        }
        if !self.pinned {
            self.anchor = self.idx;
        }
    }

    fn load_detail(&mut self) -> Result<()> {
        let oid = self.commits[self.idx].oid;
        let c = self.repo.find_commit(oid)?;
        let tree = c.tree()?;
        let (files_changed, insertions, deletions, files) = if c.parent_count() > 0 {
            let p = c.parent(0)?;
            let (fc, ins, del, f) = diff_stats(&self.repo, Some(&p.tree()?), Some(&tree))?;
            (fc, ins, del, f)
        } else {
            let (fc, ins, del, f) = diff_stats(&self.repo, None, Some(&tree))?;
            (fc, ins, del, f)
        };
        let ts =
            DateTime::<Utc>::from_timestamp(c.time().seconds(), 0).context("ts")?;
        let (ok, ms) = self.tests.get(&oid).cloned().unwrap_or((None, None));
        let manual = self.marks.get(&oid).copied();

        self.detail = Detail {
            hash: c.id().to_string(),
            author: c.author().to_string(),
            date: ts.to_rfc2822(),
            message: c.message().unwrap_or("").to_string(),
            files_changed,
            insertions,
            deletions,
            files,
            show_files: false,
            test_ok: ok,
            test_ms: ms,
            manual,
        };
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        if let Some(name) = &self.session_branch {
            self.repo
                .set_head(&format!("refs/heads/{}", self.original_branch))?;
            self.repo
                .checkout_head(Some(CheckoutBuilder::new().force()))?;
            if let Ok(mut b) = self.repo.find_branch(name, git2::BranchType::Local) {
                let _ = b.delete();
            }
        }
        if self.autostash {
            let _ = self.repo.stash_pop(0, None);
        }
        if let Some(dir) = &self.worktree_dir {
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", dir.to_str().unwrap()])
                .status();
        }
        Ok(())
    }
}

// --- helpers (pure) ---

fn parse_since(since: &Option<String>) -> Result<Option<i64>> {
    let Some(s) = since else { return Ok(None) };
    let d =
        NaiveDate::parse_from_str(s, "%Y-%m-%d").context("--since format")?;
    let ts = DateTime::<Utc>::from_naive_utc_and_offset(
        d.and_hms_opt(0, 0, 0).unwrap(),
        Utc,
    )
    .timestamp();
    Ok(Some(ts))
}

// FIX: drop immutable borrow before mut stash_save
fn maybe_autostash(repo: &mut Repository, autostash: bool) -> Result<Option<Oid>> {
    let mut so = StatusOptions::new();
    so.include_untracked(true)
        .recurse_untracked_dirs(true);

    // scope statuses so it drops before stash_save (avoids E0502)
    let dirty = {
        let statuses = repo.statuses(Some(&mut so))?;
        !statuses.is_empty()
    };
    if !dirty {
        return Ok(None);
    }
    if !autostash {
        return Err(anyhow!(
            "working tree dirty; commit/stash or --autostash"
        ));
    }
    let sig = repo
        .signature()
        .unwrap_or(Signature::now("git-trek", "git-trek@local")?);
    let oid = repo.stash_save(
        &sig,
        "git-trek autostash",
        Some(git2::StashFlags::INCLUDE_UNTRACKED),
    )?;
    Ok(Some(oid))
}

fn spawn_worktree() -> Result<(PathBuf, Repository)> {
    let dir = std::env::current_dir()?.join(".git-trek-worktree");
    let ok = Command::new("git")
        .args([
            "worktree",
            "add",
            "--force",
            "--detach",
            dir.to_str().unwrap(),
            "HEAD",
        ])
        .status()
        .context("git worktree add")?
        .success();
    if !ok {
        return Err(anyhow!("git worktree add failed"));
    }
    let wt = Repository::open(&dir)?;
    Ok((dir, wt))
}

fn head_info(repo: &Repository) -> Result<(Oid, String)> {
    let h = repo.head()?;
    let oid = h.target().context("no HEAD target")?;
    let name = h.shorthand().unwrap_or("HEAD").to_string();
    Ok((oid, name))
}

fn new_session(repo: &Repository, head: Oid) -> Result<String> {
    let name = format!(
        "_trek_session_{}",
        std::time::UNIX_EPOCH.elapsed()?.as_millis()
    );
    let c = repo.find_commit(head)?;
    repo.branch(&name, &c, true)?;
    repo.set_head(&format!("refs/heads/{}", name))?;
    Ok(name)
}

fn load_commits(repo: &Repository, cli: &Cli, since_ts: Option<i64>) -> Result<Vec<Item>> {
    let mut rev = repo.revwalk()?;
    if cli.all {
        rev.push_glob("refs/*")?;
    } else {
        rev.push_head()?;
    }
    rev.set_sorting(Sort::TOPOLOGICAL)?;

    let mut out = Vec::with_capacity(cli.limit);
    for id in rev.take(cli.limit) {
        let oid = id?;
        let c = repo.find_commit(oid)?;
        if let Some(ts) = since_ts {
            if c.time().seconds() < ts {
                continue;
            }
        }
        if let Some(a) = &cli.author {
            if !c
                .author()
                .to_string()
                .to_lowercase()
                .contains(&a.to_lowercase())
            {
                continue;
            }
        }
        if let Some(p) = &cli.path {
            if !touches_path(repo, oid, p)? {
                continue;
            }
        }
        let sum = c.summary().unwrap_or("").chars().take(70).collect();
        out.push(Item { oid, summary: sum });
    }
    Ok(out)
}

fn touches_path(repo: &Repository, oid: Oid, path: &str) -> Result<bool> {
    let c = repo.find_commit(oid)?;
    let t = c.tree()?;
    let base = if c.parent_count() > 0 {
        Some(c.parent(0)?.tree()?)
    } else {
        None
    };
    let mut opt = DiffOptions::new();
    opt.pathspec(path);
    let diff = repo.diff_tree_to_tree(base.as_ref(), Some(&t), Some(&mut opt))?;
    Ok(diff.deltas().len() > 0)
}

fn collect_labels(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>> {
    let mut map = HashMap::<Oid, Vec<String>>::new();

    if let Ok(tag_names) = repo.tag_names(None) {
        for t in tag_names.iter().flatten() {
            if let Ok(obj) = repo.revparse_single(t) {
                if let Ok(c) = obj.peel_to_commit() {
                    map.entry(c.id())
                        .or_default()
                        .push(format!("#{}", t));
                }
            }
        }
    }
    let refs = repo.references()?;
    for r in refs.flatten() {
        if r.is_branch() {
            if let Some(tgt) = r.target() {
                let name = r.shorthand().unwrap_or_default().to_string();
                map.entry(tgt).or_default().push(format!("⎇ {}", name));
            }
        }
    }
    Ok(map)
}

fn diff_stats(
    repo: &Repository,
    a: Option<&git2::Tree>,
    b: Option<&git2::Tree>,
) -> Result<(usize, usize, usize, Vec<FileChange>)> {
    let mut opt = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(a, b, Some(&mut opt))?;
    let stats = diff.stats()?;
    let mut files = Vec::<FileChange>::new();
    diff.foreach(
        &mut |_d, _p| true,
        None,
        Some(&mut |delta: DiffDelta, _| {
            let p = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path());
            let status = match delta.status() {
                git2::Delta::Added => "A",
                git2::Delta::Modified => "M",
                git2::Delta::Deleted => "D",
                git2::Delta::Renamed => "R",
                _ => "?",
            }
            .to_string();
            if let Some(pp) = p {
                files.push(FileChange {
                    path: pp.to_string_lossy().to_string(),
                    status,
                });
            }
            true
        }),
        None,
    )?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((
        stats.files_changed(),
        stats.insertions(),
        stats.deletions(),
        files,
    ))
}

fn short(oid: &Oid) -> String {
    oid.to_string()[..8].to_string()
}
