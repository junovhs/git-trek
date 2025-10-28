use anyhow::{Context, Result};
use chrono::NaiveDate;
use git2::{build::CheckoutBuilder, Oid, Repository, ResetType, Signature, Sort};
use std::path::PathBuf;
use std::process::Command;
use crate::cli::Cli;

pub const UI_WIDTH: u16 = 100;
pub const ANIMATION_TOTAL_FRAMES: u8 = 2;
pub const ANIMATION_FRAME_MS: u64 = 40;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AppState {
    Scanning,
    Inspect,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AnimationDirection {
    Up,
    Down,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct AnimationState {
    pub kind: AnimationKind,
    pub frame: u8,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AnimationKind {
    Targeting(AnimationDirection),
}

#[derive(Clone, Default)]
pub struct Detail {
    pub hash: String,
    pub author: String,
    pub message: String,
}

#[derive(Clone)]
pub struct Item {
    pub oid: Oid,
    pub summary: String,
}

pub struct App {
    pub state: AppState,
    pub repo: Repository,
    pub commits: Vec<Item>,
    pub idx: usize,
    pub last_checkout_idx: usize,
    pub should_quit: bool,
    pub final_message: Option<String>,
    pub terminal_too_small: bool,
    pub list_scroll_offset: usize,
    pub animation: Option<AnimationState>,
    pub detail: Detail,
    pub original_branch: String,
    pub session_branch: Option<String>,
    pub worktree_dir: Option<PathBuf>,
    pub opts: Cli,
}

impl App {
    pub fn new(cli: Cli) -> Result<Self> {
        let mut cwd_repo = Repository::open_from_env().context("not a git repo")?;
        let since_ts = parse_since(&cli.since)?;
        
        let autostash_oid = if !cli.worktree {
            let is_dirty = {
                let mut status_opts = git2::StatusOptions::new();
                status_opts.include_untracked(true).recurse_untracked_dirs(false);
                let statuses = cwd_repo.statuses(Some(&mut status_opts))?;
                !statuses.is_empty()
            };
            if is_dirty && cli.autostash {
                let sig = cwd_repo.signature().unwrap_or(Signature::now("git-trek", "git-trek@local")?);
                Some(cwd_repo.stash_save(&sig, "git-trek autostash", Some(git2::StashFlags::INCLUDE_UNTRACKED))?)
            } else if is_dirty {
                anyhow::bail!("working tree dirty; commit/stash or use --autostash");
            } else { None }
        } else { None };

        let (repo, worktree_dir) = if cli.worktree {
            let (dir, wt) = spawn_worktree()?;
            (wt, Some(dir))
        } else {
            (cwd_repo, None)
        };

        let (head_oid, branch) = head_info(&repo)?;
        let session_branch = if !cli.worktree {
            Some(new_session(&repo, head_oid)?)
        } else {
            None
        };

        let commits = load_commits(&repo, &cli, since_ts)?;
        let idx = commits.iter().position(|c| c.oid == head_oid).unwrap_or(0);

        let mut app = Self {
            state: AppState::Scanning,
            repo,
            commits,
            idx,
            last_checkout_idx: idx,
            should_quit: false,
            final_message: None,
            terminal_too_small: false,
            list_scroll_offset: 0,
            animation: None,
            detail: Detail::default(),
            original_branch: branch,
            session_branch,
            worktree_dir,
            opts: cli,
        };
        
        if autostash_oid.is_some() {
            if let Some(dir) = &app.worktree_dir {
                Command::new("git").arg("stash").arg("pop").current_dir(dir).status()?;
            }
        }

        app.load_detail()?;
        app.update_checkout()?;
        Ok(app)
    }

    pub fn on_tick(&mut self) {
        if let Some(mut anim) = self.animation {
            anim.frame += 1;
            if anim.frame >= ANIMATION_TOTAL_FRAMES {
                self.animation = None;
            } else {
                self.animation = Some(anim);
            }
        }
    }
    
    pub fn update_checkout(&mut self) -> Result<()> {
        if self.commits.is_empty() { return Ok(()); }
        let oid = self.commits[self.idx].oid;
        let obj = self.repo.find_object(oid, None)?;
        self.repo.reset(&obj, ResetType::Hard, None)?;
        self.last_checkout_idx = self.idx;
        Ok(())
    }

    pub fn shift_target(&mut self, direction: AnimationDirection) {
        if self.animation.is_some() || self.commits.is_empty() { return; }

        let new_idx = match direction {
            AnimationDirection::Up if self.idx > 0 => self.idx - 1,
            AnimationDirection::Down if self.idx < self.commits.len() - 1 => self.idx + 1,
            _ => return,
        };

        self.idx = new_idx;
        self.animation = Some(AnimationState { kind: AnimationKind::Targeting(direction), frame: 0 });
        let _ = self.load_detail();
    }
    
    pub fn adjust_scroll(&mut self, list_height: usize) {
        let window_size = list_height.saturating_sub(2);
        if self.idx < self.list_scroll_offset {
            self.list_scroll_offset = self.idx;
        } else if self.idx >= self.list_scroll_offset + window_size {
            self.list_scroll_offset = self.idx - window_size + 1;
        }
    }
    
    pub fn toggle_inspect(&mut self) {
        self.state = if self.state == AppState::Scanning {
            AppState::Inspect
        } else {
            AppState::Scanning
        };
    }
    
    pub fn stop(&mut self) -> Result<()> {
        self.cleanup()?;
        self.should_quit = true;
        self.final_message = Some("Disengaged. Returned to original timeline.".into());
        Ok(())
    }
    
    fn load_detail(&mut self) -> Result<()> {
        if self.commits.is_empty() { return Ok(()); }
        let oid = self.commits[self.idx].oid;
        let c = self.repo.find_commit(oid)?;
        
        self.detail = Detail {
            hash: c.id().to_string(),
            author: c.author().to_string(),
            message: c.message().unwrap_or("").trim().to_string(),
        };
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        if let Some(name) = &self.session_branch {
            self.repo.set_head(&format!("refs/heads/{}", self.original_branch))?;
            self.repo.checkout_head(Some(CheckoutBuilder::new().force()))?;
            if let Ok(mut b) = self.repo.find_branch(name, git2::BranchType::Local) {
                let _ = b.delete();
            }
        }
        if let Some(dir) = &self.worktree_dir {
            if !self.opts.worktree {
                let _ = Command::new("git")
                    .args(["worktree", "remove", "--force", dir.to_str().unwrap()])
                    .status();
            }
        }
        Ok(())
    }
}

fn parse_since(since: &Option<String>) -> Result<Option<i64>> {
    let Some(s) = since else { return Ok(None) };
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").context("--since format must be YYYY-MM-DD")?;
    let ts = d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
    Ok(Some(ts))
}

fn spawn_worktree() -> Result<(PathBuf, Repository)> {
    let dir = std::env::temp_dir().join(format!("git-trek-{}", std::process::id()));
    let ok = Command::new("git")
        .args(["worktree", "add", "--force", "--detach", dir.to_str().unwrap(), "HEAD"])
        .status()?
        .success();
    if !ok { anyhow::bail!("git worktree add failed"); }
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
    let name = format!("_trek_session_{}", std::time::UNIX_EPOCH.elapsed()?.as_millis());
    let c = repo.find_commit(head)?;
    repo.branch(&name, &c, true)?;
    repo.set_head(&format!("refs/heads/{}", name))?;
    repo.reset(c.as_object(), ResetType::Hard, None)?;
    Ok(name)
}

fn load_commits(repo: &Repository, cli: &Cli, since_ts: Option<i64>) -> Result<Vec<Item>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL)?;
    revwalk.push_head()?;

    revwalk
        .take(cli.limit)
        .map(|id| -> Result<Option<Item>> {
            let oid = id?;
            let c = repo.find_commit(oid)?;
            if let Some(ts) = since_ts {
                if c.time().seconds() < ts {
                    return Ok(None);
                }
            }
            let summary = c.summary().unwrap_or("").trim().to_string();
            Ok(Some(Item { oid, summary }))
        })
        .filter_map(Result::transpose)
        .collect()
}