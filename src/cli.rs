use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "git-trek", about = "Navigate git history like it's 1989", version)]
pub struct Cli {
    /// Walk all refs, not just HEAD ancestry
    #[arg(long)]
    pub all: bool,

    /// Max commits to load
    #[arg(long, default_value_t = 200)]
    pub limit: usize,

    /// YYYY-MM-DD (include commits since date)
    #[arg(long)]
    pub since: Option<String>,

    /// Filter by author substring
    #[arg(long)]
    pub author: Option<String>,

    /// Only commits that touch this path
    #[arg(long)]
    pub path: Option<String>,

    /// Run command at each selection (e.g. `cargo test -q`)
    #[arg(long)]
    pub cmd: Option<String>,

    /// Timeout for --cmd (seconds). 0 = unlimited
    #[arg(long, default_value_t = 0)]
    pub cmd_timeout: u64,

    /// Stash changes on start; pop on exit
    #[arg(long)]
    pub autostash: bool,

    /// Use a temporary git worktree for navigation
    #[arg(long)]
    pub worktree: bool,

    /// Test mode: initialize app and exit (hidden)
    #[arg(long, hide = true)]
    pub dry_run: bool,
}

impl Cli {
    pub fn parse_checked() -> Result<Self> {
        let me = <Cli as Parser>::parse();
        if me.limit == 0 { return Err(anyhow!("--limit must be > 0")); }
        if let Some(s) = &me.since {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| anyhow!("--since expects YYYY-MM-DD"))?;
        }
        Ok(me)
    }
}
