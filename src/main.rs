mod app;
mod cli;
mod data;
mod error;
mod git_ops;
mod mouse;
mod views;

mod event;
mod terminal;

use anyhow::Result;

use crate::app::App;
use crate::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    if cli.check {
        App::new(&cli)?;
        println!("git-trek initialized successfully");
        return Ok(());
    }

    let mut terminal = terminal::setup()?;
    let mut app = App::new(&cli)?;

    let result = event::run(&mut terminal, &mut app);

    terminal::restore(&mut terminal)?;

    if let Some(msg) = app.message() {
        println!("{msg}");
    }

    result
}
