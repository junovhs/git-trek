use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "git-trek", about = "Navigate git history visually", version)]
pub struct Cli {
    /// Maximum number of commits to load
    #[arg(long, short, default_value_t = 200)]
    pub limit: usize,

    /// Run initialization check without starting TUI
    #[arg(long, hide = true)]
    pub check: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Parser::parse()
    }
}
