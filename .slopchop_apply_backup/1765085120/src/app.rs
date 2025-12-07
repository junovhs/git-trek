use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{build::CheckoutBuilder, Oid, Repository, ResetType};
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    cli::Cli,
    git_ops::{
        check_if_dirty, do_autostash, head_info, load_commits, new_session,
        parse_since, spawn_worktree, Point,
    },
    shell,
};

pub use crate::git_ops::format_oid;

pub const EVENT_POLL_MS: u64 = 100;
pub const CHECKOUT_DEBOUNCE_MS: u64 = 200;
pub const VERSION: &str = "2.2";

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum AppState {
    DirtyTreeWarning,
    Browsing,
    ViewingDetail,
    ConfirmingCheckout,
    ShowingHelp,
}

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

#[allow(clippy::struct_excessive_bools)]
pub struct App {
    pub state: AppState,
    pub repo: Repository,
    pub repo_dir: PathBuf,
    pub commits: Vec<Point>,
    pub idx: usize,
    pub anchor: Option<usize>,
    pub original_branch: String,
    pub session_branch: Option<String>,
    pub autostash: bool,
    pub used_worktree: bool,
    pub worktree_dir: Option<PathBuf>,
    pub opts: Cli,
    pub detail: Detail,
    pub diff_full: bool,
    pub tests: HashMap<Oid, (Option<bool>, Option<u128>)>,
    pub marks: HashMap<Oid, bool>,
    pub should_quit: bool,
    pub final_message: Option<String>,
    pub read_only: bool,
    pub tree_was_dirty: bool,
    pub last_nav_time: Instant,
    pub pending_checkout: bool,
    pub last_checkout_idx: Option<usize>,
}

impl App {
    pub fn new(cli: Cli) -> Result<Self> {
        let mut cwd_repo = Repository::open_from_env().context("not a git repo")?;
        let since_ts = parse_since(cli.since.as_deref())?;
        let tree_is_dirty = check_if_dirty(&mut cwd_repo)?;

        let autostash_oid = if cli.flags.autostash() && tree_is_dirty {
            Some(do_autostash(&mut cwd_repo)?)
        } else {
            None
        };

        let (repo, repo_dir, worktree_dir, used_worktree) = if cli.flags.worktree() {
            let (dir, wt) = spawn_worktree()?;
            (wt, dir.clone(), Some(dir), true)
        } else {
            (cwd_repo, std::env::current_dir()?, None, false)
        };

        let (head_oid, branch) = head_info(&repo)?;
        let session_branch = if cli.flags.worktree() {
            None
        } else {
            Some(new_session(&repo, head_oid)?)
        };

        let commits = load_commits(&repo, &cli, since_ts)?;
        let idx = commits.iter().position(|c| c.oid == head_oid).unwrap_or(0);
        let initial_state = if tree_is_dirty && !cli.flags.autostash() {
            AppState::DirtyTreeWarning
        } else {
            AppState::Browsing
        };

        let mut app = Self {
            state: initial_state, repo, repo_dir, commits, idx, anchor: None,
            original_branch: branch, session_branch, autostash: autostash_oid.is_some(),
            used_worktree, worktree_dir, opts: cli, detail: Detail::default(),
            diff_full: false, tests: HashMap::new(), marks: HashMap::new(),
            should_quit: false, final_message: None, read_only: false,
            tree_was_dirty: tree_is_dirty, last_nav_time: Instant::now(),
            pending_checkout: false, last_checkout_idx: None,
        };
        app.refresh_view()?;
        app.update_checkout()?;
        app.last_checkout_idx = Some(idx);
        Ok(app)
    }

    pub fn move_sel(&mut self, delta: isize) -> Result<()> {
        let len = self.commits.len();
        if len == 0 { return Ok(()); }
        let new_idx = if delta < 0 {
            self.idx.saturating_sub(delta.unsigned_abs())
        } else {
            self.idx.saturating_add(delta.unsigned_abs())
        };
        self.idx = new_idx.clamp(0, len - 1);
        self.last_nav_time = Instant::now();
        self.pending_checkout = true;
        self.refresh_view()
    }

    pub fn maybe_do_pending_checkout(&mut self) -> Result<()> {
        if !self.pending_checkout { return Ok(()); }
        if self.last_nav_time.elapsed() < Duration::from_millis(CHECKOUT_DEBOUNCE_MS) { return Ok(()); }
        if self.last_checkout_idx == Some(self.idx) { self.pending_checkout = false; return Ok(()); }
        self.update_checkout()?;
        self.last_checkout_idx = Some(self.idx);
        self.pending_checkout = false;
        Ok(())
    }

    pub fn pin_anchor(&mut self) { self.anchor = Some(self.idx); }
    pub fn mark_manual(&mut self, pass: bool) {
        let oid = self.commits[self.idx].oid;
        self.marks.insert(oid, pass);
        self.detail.manual = Some(pass);
    }

    pub fn checkout(&mut self) -> Result<()> {
        if self.read_only { return Ok(()); }
        let oid = self.commits[self.idx].oid;
        self.cleanup()?;
        self.should_quit = true;
        let msg = if self.used_worktree {
            format!("Now at {} (worktree).", format_oid(oid))
        } else {
            self.repo.set_head_detached(oid)?;
            self.repo.checkout_head(Some(CheckoutBuilder::new().force()))?;
            format!("Now at {}.\nTo return: git switch {}", format_oid(oid), self.original_branch)
        };
        self.final_message = Some(msg);
        Ok(())
    }

    pub fn enter_detail(&mut self) { self.state = AppState::ViewingDetail; }
    pub fn exit_detail(&mut self) { self.state = AppState::Browsing; }
    pub fn enter_confirm(&mut self) { if !self.read_only { self.state = AppState::ConfirmingCheckout; } }
    pub fn exit_confirm(&mut self) { self.state = AppState::ViewingDetail; }
    pub fn toggle_help(&mut self) {
        self.state = if self.state == AppState::ShowingHelp { AppState::Browsing } else { AppState::ShowingHelp };
    }
    pub fn stop(&mut self) -> Result<()> {
        self.cleanup()?;
        self.should_quit = true;
        self.final_message = Some("Returned to original timeline.".into());
        Ok(())
    }

    pub fn handle_dirty_stash(&mut self) -> Result<()> {
        let sig = self.repo.signature()?;
        self.repo.stash_save(&sig, "git-trek autostash", Some(git2::StashFlags::INCLUDE_UNTRACKED))?;
        self.autostash = true;
        self.tree_was_dirty = false;
        self.state = AppState::Browsing;
        Ok(())
    }
    pub fn handle_dirty_continue(&mut self) { self.read_only = true; self.state = AppState::Browsing; }
    pub fn handle_dirty_quit(&mut self) { self.should_quit = true; self.final_message = Some("Exited without changes.".into()); }

    fn refresh_view(&mut self) -> Result<()> {
        if self.commits.is_empty() { return Ok(()); }
        self.load_detail()?;
        if self.opts.cmd.is_some() { self.run_cmd()?; }
        Ok(())
    }

    fn update_checkout(&self) -> Result<()> {
        let oid = self.commits[self.idx].oid;
        let commit = self.repo.find_commit(oid)?;
        self.repo.reset(commit.as_object(), ResetType::Hard, None)?;
        Ok(())
    }

    fn run_cmd(&mut self) -> Result<()> {
        let oid = self.commits[self.idx].oid;
        if let Some((ok, ms)) = self.tests.get(&oid).copied() {
            self.detail.test_ok = ok;
            self.detail.test_ms = ms;
            return Ok(());
        }
        let Some(cmd) = self.opts.cmd.clone() else { return Ok(()); };
        let timeout = (self.opts.cmd_timeout > 0).then(|| Duration::from_secs(self.opts.cmd_timeout));
        let start = std::time::Instant::now();
        let ok = shell::run(&cmd, timeout, &self.repo_dir)?;
        let ms = start.elapsed().as_millis();
        self.tests.insert(oid, (Some(ok), Some(ms)));
        self.detail.test_ok = Some(ok);
        self.detail.test_ms = Some(ms);
        Ok(())
    }

    fn load_detail(&mut self) -> Result<()> {
        let oid = self.commits[self.idx].oid;
        let commit = self.repo.find_commit(oid)?;
        let parent_tree = if commit.parent_count() > 0 { Some(commit.parent(0)?.tree()?) } else { None };
        let diff = self.repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit.tree()?), None)?;
        let stats = diff.stats()?;
        let ts = DateTime::<Utc>::from_timestamp(commit.time().seconds(), 0).context("ts")?;
        let (ok, ms) = self.tests.get(&oid).copied().unwrap_or((None, None));
        let manual = self.marks.get(&oid).copied();
        self.detail = Detail {
            hash: format_oid(commit.id()), author: commit.author().to_string(), date: ts.to_rfc2822(),
            message: commit.message().unwrap_or("").to_string(), insertions: stats.insertions(),
            deletions: stats.deletions(), test_ok: ok, test_ms: ms, manual,
        };
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        if let Some(name) = &self.session_branch {
            self.repo.set_head(&format!("refs/heads/{}", self.original_branch))?;
            self.repo.checkout_head(Some(CheckoutBuilder::new().force()))?;
            if let Ok(mut b) = self.repo.find_branch(name, git2::BranchType::Local) { let _ = b.delete(); }
        }
        if self.autostash { let _ = self.repo.stash_pop(0, None); }
        if let Some(dir) = &self.worktree_dir {
            let _ = std::process::Command::new("git").args(["worktree", "remove", "--force", &dir.to_string_lossy()]).status();
        }
        Ok(())
    }
}