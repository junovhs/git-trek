use anyhow::{anyhow, Result};
use clap::Parser;

/// Bundled boolean flags to avoid `clippy::struct_excessive_bools`
#[derive(Debug, Clone, Copy, Default)]
pub struct CliFlags {
    bits: u8,
}

impl CliFlags {
    const ALL: u8 = 0b0001;
    const AUTOSTASH: u8 = 0b0010;
    const WORKTREE: u8 = 0b0100;
    const DRY_RUN: u8 = 0b1000;

    pub const fn all(self) -> bool { self.bits & Self::ALL != 0 }
    pub const fn autostash(self) -> bool { self.bits & Self::AUTOSTASH != 0 }
    pub const fn worktree(self) -> bool { self.bits & Self::WORKTREE != 0 }
    pub const fn dry_run(self) -> bool { self.bits & Self::DRY_RUN != 0 }

    fn set(&mut self, flag: u8, value: bool) {
        if value { self.bits |= flag; } else { self.bits &= !flag; }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug, Clone)]
#[command(name = "git-trek", about = "Navigate git history like it's 1989", version)]
pub struct CliRaw {
    #[arg(long)]
    pub all: bool,
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub author: Option<String>,
    #[arg(long)]
    pub path: Option<String>,
    #[arg(long)]
    pub cmd: Option<String>,
    #[arg(long, default_value_t = 0)]
    pub cmd_timeout: u64,
    #[arg(long)]
    pub autostash: bool,
    #[arg(long)]
    pub worktree: bool,
    #[arg(long, hide = true)]
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct Cli {
    pub flags: CliFlags,
    pub limit: usize,
    pub since: Option<String>,
    pub author: Option<String>,
    pub path: Option<String>,
    pub cmd: Option<String>,
    pub cmd_timeout: u64,
}

impl Cli {
    pub fn parse_checked() -> Result<Self> {
        let raw = <CliRaw as Parser>::parse();
        if raw.limit == 0 {
            return Err(anyhow!("--limit must be > 0"));
        }
        if let Some(s) = &raw.since {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|_| anyhow!("--since expects YYYY-MM-DD"))?;
        }
        let mut flags = CliFlags::default();
        flags.set(CliFlags::ALL, raw.all);
        flags.set(CliFlags::AUTOSTASH, raw.autostash);
        flags.set(CliFlags::WORKTREE, raw.worktree);
        flags.set(CliFlags::DRY_RUN, raw.dry_run);

        Ok(Self {
            flags,
            limit: raw.limit,
            since: raw.since,
            author: raw.author,
            path: raw.path,
            cmd: raw.cmd,
            cmd_timeout: raw.cmd_timeout,
        })
    }
}