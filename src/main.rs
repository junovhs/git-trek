use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::{Repository, Oid, StatusOptions};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    time::Duration,
};

#[derive(Parser)]
#[command(name = "git-trek")]
#[command(about = "🚀 Navigate git history like it's 1989!", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start your journey through time
    Start,
    /// Return to where you began (exit without changes)
    Stop,
    /// Apply this point in history to your branch
    Restore,
}

struct App {
    repo: Repository,
    commits: Vec<CommitInfo>,
    current_index: usize,
    anchor_index: usize,
    scroll_offset: usize,
    nav_branch: String,
    original_branch: String,
    anchor_ref: String,
    message: Option<String>,
}

#[derive(Clone)]
struct CommitInfo {
    oid: Oid,
    summary: String,
    author: String,
    time_ago: String,
    stats: String,
}

impl App {
    fn new() -> Result<Self> {
        let repo = Repository::open_from_env()
            .context("Not in a git repository")?;
        
        {
            let mut opts = StatusOptions::new();
            opts.include_untracked(true)
                .recurse_untracked_dirs(true);

            let statuses = repo.statuses(Some(&mut opts))?;
            
            if !statuses.is_empty() {
                anyhow::bail!("🚫 Working tree is not clean. Stash or commit your changes first!");
            }
        }
        
        let (head_oid, original_branch) = {
            let head = repo.head()?;
            let oid = head.target().context("Could not get OID from HEAD")?;
            let branch = head.shorthand().unwrap_or("HEAD").to_string();
            (oid, branch)
        };
        
        let nav_branch = "_trek".to_string();
        let anchor_ref = "refs/trek/anchor".to_string();
        
        repo.reference(&anchor_ref, head_oid, true, "git-trek: anchor")?;
        
        if let Ok(mut branch_ref) = repo.find_reference(&format!("refs/heads/{}", nav_branch)) {
            branch_ref.set_target(head_oid, "git-trek: reset nav branch")?;
            repo.set_head(&format!("refs/heads/{}", nav_branch))?;
        } else {
            let commit = repo.find_commit(head_oid)?;
            repo.branch(&nav_branch, &commit, false)?;
            repo.set_head(&format!("refs/heads/{}", nav_branch))?;
        }
        repo.checkout_head(None)?;
        
        let commits = Self::load_commits(&repo, head_oid)?;
        let current_index = commits.iter().position(|c| c.oid == head_oid).unwrap_or(0);
        
        Ok(App {
            repo,
            commits,
            current_index,
            anchor_index: current_index,
            scroll_offset: 0,
            nav_branch,
            original_branch,
            anchor_ref,
            message: Some("🚀 Trek initiated! Use arrow keys or WASD to navigate, Q to quit, R to restore".into()),
        })
    }
    
    fn load_commits(repo: &Repository, start_oid: Oid) -> Result<Vec<CommitInfo>> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push(start_oid)?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;
        
        let mut commits = Vec::new();
        
        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            
            let summary = commit.summary().unwrap_or("").to_string();
            let author = commit.author().name().unwrap_or("").to_string();
            
            let time = commit.time();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;
            let diff = now - time.seconds();
            
            let time_ago = match diff {
                0..=60 => "just now".to_string(),
                61..=3600 => format!("{} minutes ago", diff / 60),
                3601..=86400 => format!("{} hours ago", diff / 3600),
                86401..=604800 => format!("{} days ago", diff / 86400),
                _ => format!("{} weeks ago", diff / 604800),
            };
            
            let stats = if commit.parent_count() > 0 {
                let parent = commit.parent(0)?;
                let diff = repo.diff_tree_to_tree(
                    Some(&parent.tree()?),
                    Some(&commit.tree()?),
                    None,
                )?;
                let stats = diff.stats()?;
                format!("+{} -{}", stats.insertions(), stats.deletions())
            } else {
                "initial".to_string()
            };
            
            commits.push(CommitInfo {
                oid,
                summary: summary.chars().take(50).collect(),
                author: author.chars().take(20).collect(),
                time_ago,
                stats,
            });
            
            if commits.len() >= 50 {
                break;
            }
        }
        
        Ok(commits)
    }
    
    fn move_to(&mut self, index: usize) -> Result<()> {
        if index >= self.commits.len() {
            return Ok(());
        }
        
        self.current_index = index;
        let target_oid = self.commits[index].oid;
        
        let commit = self.repo.find_commit(target_oid)?;
        self.repo.reset(commit.as_object(), git2::ResetType::Hard, None)?;
        
        if self.current_index < self.scroll_offset {
            self.scroll_offset = self.current_index;
        } else if self.current_index >= self.scroll_offset + 10 {
            self.scroll_offset = self.current_index - 9;
        }
        
        Ok(())
    }
    
    fn jump_to_letter(&mut self, letter: char) -> Result<()> {
        let letter_upper = letter.to_ascii_uppercase();
        if ('A'..='J').contains(&letter_upper) {
            let target_index = self.scroll_offset + (letter_upper as usize - 'A' as usize);
            self.move_to(target_index)?;
        }
        Ok(())
    }
    
    fn restore(&mut self) -> Result<String> {
        let current_oid = self.commits[self.current_index].oid;
        
        self.repo.set_head(&format!("refs/heads/{}", self.original_branch))?;
        
        let commit = self.repo.find_commit(current_oid)?;
        self.repo.reset(commit.as_object(), git2::ResetType::Hard, None)?;
        
        self.cleanup()?;
        
        Ok(format!("✅ Restored to {}", &current_oid.to_string()[..8]))
    }
    
    fn cleanup(&self) -> Result<()> {
        if let Ok(mut branch) = self.repo.find_branch(&self.nav_branch, git2::BranchType::Local) {
            branch.delete()?;
        }
        
        if let Ok(mut anchor) = self.repo.find_reference(&self.anchor_ref) {
            anchor.delete()?;
        }
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<String> {
        self.repo.set_head(&format!("refs/heads/{}", self.original_branch))?;
        let anchor = self.repo.find_reference(&self.anchor_ref)?;
        let commit = self.repo.find_commit(anchor.target().context("Could not get OID from anchor")?)?;
        self.repo.reset(commit.as_object(), git2::ResetType::Hard, None)?;
        
        self.cleanup()?;
        Ok("👋 Trek ended. Back to where you started!".to_string())
    }
}

fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(f.area());
    
    let header = Paragraph::new(vec![
        Line::from(vec![Span::raw("╔══════════════════════════════════════════════════════════════╗")]),
        Line::from(vec![
            Span::raw("║ "),
            Span::styled("🚀 GIT TREK", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled("STARDATE 2024", Style::default().fg(Color::Yellow)),
            Span::raw("                            ║"),
        ]),
        Line::from(vec![Span::raw("╚══════════════════════════════════════════════════════════════╝")]),
    ])
    .style(Style::default().fg(Color::Green));
    f.render_widget(header, chunks[0]);
    
    let visible_range = app.scroll_offset..std::cmp::min(app.scroll_offset + 10, app.commits.len());
    let mut timeline_lines = vec![];
    
    for (i, commit_idx) in visible_range.enumerate() {
        let commit = &app.commits[commit_idx];
        let letter = (b'A' + i as u8) as char;
        
        let (marker, marker_color) = if commit_idx == app.current_index {
            ("◉", Color::Green)
        } else if commit_idx == app.anchor_index {
            ("◎", Color::Cyan)
        } else {
            ("○", Color::Gray)
        };
        
        let line = Line::from(vec![
            Span::raw("  "),
            Span::styled(marker, Style::default().fg(marker_color)),
            Span::raw(" "),
            Span::styled(format!("[{}]", letter), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(&commit.summary, Style::default().fg(if commit_idx == app.current_index { Color::White } else { Color::Gray })),
        ]);
        
        timeline_lines.push(line);
        
        if i < 9 && commit_idx < app.commits.len() - 1 {
            timeline_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("│", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }
    
    let timeline = Paragraph::new(timeline_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" TEMPORAL FLUX NAVIGATOR "));
    f.render_widget(timeline, chunks[1]);
    
    let current = &app.commits[app.current_index];
    let hash_str = current.oid.to_string();
    let info_text = vec![
        Line::from(vec![
            Span::styled("Hash: ", Style::default().fg(Color::Cyan)),
            Span::raw(&hash_str[..8]),
            Span::raw("  "),
            Span::styled("Author: ", Style::default().fg(Color::Cyan)),
            Span::raw(&current.author),
        ]),
        Line::from(vec![
            Span::styled("Time: ", Style::default().fg(Color::Cyan)),
            Span::raw(&current.time_ago),
            Span::raw("  "),
            Span::styled("Changes: ", Style::default().fg(Color::Cyan)),
            Span::raw(&current.stats),
        ]),
    ];
    
    let info = Paragraph::new(info_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(" SCAN RESULTS "));
    f.render_widget(info, chunks[2]);
    
    let controls = if let Some(msg) = &app.message {
        Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
    } else {
        Paragraph::new("↑↓/WS: Navigate | A-J: Jump | R: Restore | Q: Quit")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
    };
    
    f.render_widget(controls, chunks[3]);
}

// The function that takes over the terminal MUST be responsible for cleaning it up.
// It now returns a Result<String> with the message to be printed by main.
fn run_interactive(mut app: App) -> Result<String> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw_ui(f, &app))?;
        
        if app.message.is_some() {
            app.message = None;
        }
        
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            let msg = app.stop()?;
                            cleanup_terminal()?; // Clean up BEFORE returning
                            return Ok(msg);
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            let msg = app.restore()?;
                            cleanup_terminal()?; // Clean up BEFORE returning
                            return Ok(msg);
                        }
                        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                            if app.current_index > 0 {
                                app.move_to(app.current_index - 1)?;
                            } else {
                                app.message = Some("🛑 Beginning of history!".into());
                            }
                        }
                        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                            if app.current_index < app.commits.len() - 1 {
                                app.move_to(app.current_index + 1)?;
                            } else {
                                app.message = Some("🛑 End of history!".into());
                            }
                        }
                        KeyCode::Char(c) if ('a'..='j').contains(&c.to_ascii_lowercase()) => {
                            app.jump_to_letter(c)?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// This helper function remains the same.
fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

// The main function is now simpler and more robust.
fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Start) | None => {
            // 1. Attempt to create the app. This can fail if the tree is dirty.
            let app = App::new()?;
            
            // 2. Run the interactive session. This can fail if the terminal has issues.
            //    We handle cleanup inside run_interactive.
            let final_message = match run_interactive(app) {
                Ok(message) => message,
                Err(e) => {
                    // If an error happens *during* the interactive session,
                    // we must still try to clean up the terminal.
                    cleanup_terminal()?;
                    return Err(e);
                }
            };

            // 3. Print the final message AFTER the terminal has been restored.
            println!("{}", final_message);
        }
        Some(Commands::Stop) => {
            let mut app = App::new()?;
            let msg = app.stop()?;
            println!("{}", msg);
        }
        Some(Commands::Restore) => {
            println!("⚠️  Restore must be done from interactive mode (just run 'git-trek')");
        }
    }
    
    Ok(())
}