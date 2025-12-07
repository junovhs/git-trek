use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "git-trek", about = "Navigate git history like it's 1989", version)]
struct CliRaw {
    #[arg(long, default_value_t = 200)]
    pub limit: usize,
    #[arg(long, hide = true)]
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct Cli {
    pub limit: usize,
    pub dry_run: bool,
}

impl Cli {
    pub fn parse_checked() -> Result<Self> {
        let raw = <CliRaw as Parser>::parse();
        if raw.limit == 0 {
            return Err(anyhow!("--limit must be > 0"));
        }
        Ok(Self {
            limit: raw.limit,
            dry_run: raw.dry_run,
        })
    }
}