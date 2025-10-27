use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{ // Corrected spelling
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::{build::CheckoutBuilder, Repository, Oid, ResetType, StatusOptions};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{collections::HashMap, io, time::Duration};

//=============================================================================
// App: State Management
//=============================================================================
mod app {
    use super::*;
    use anyhow::Result;

    #[derive(PartialEq, Eq)] // <-- FIX: Added this to allow comparison
    pub enum AppState {
        Navigating,
        DetailView,
        ConfirmingCheckout,
    }

    #[derive(Clone, Default)]
    pub struct DetailedCommitInfo {
        pub full_hash: String,
        pub author: String,
        pub date: String,
        pub full_message: String,
        pub files_changed: usize,
        pub insertions: usize,
        pub deletions: usize,
    }

    #[derive(Clone)]
    pub struct CommitInfo {
        pub oid: Oid,
        pub summary: String,
    }

    pub struct App {
        pub repo: Repository,
        pub commits: Vec<CommitInfo>,
        pub tags: HashMap<Oid, String>,
        pub current_index: usize,
        pub anchor_index: usize,
        pub scroll_offset: usize,
        pub original_branch: String,
        pub state: AppState,
        pub detailed_info: DetailedCommitInfo,
        pub should_quit: bool,
    }

    impl App {
        pub fn new() -> Result<Self> {
            let repo = Repository::open_from_env().context("Not in a git repository")?;
            Self::validate_repo_state(&repo)?;

            let (head_oid, original_branch) = Self::get_head_info(&repo)?;

            {
                let nav_branch_name = format!("_trek_session_{}", std::time::UNIX_EPOCH.elapsed()?.as_millis());
                let head_commit = repo.find_commit(head_oid)?;
                repo.branch(&nav_branch_name, &head_commit, true)?;
                repo.set_head(&format!("refs/heads/{}", nav_branch_name))?;
            }

            let commits = Self::load_commits(&repo, head_oid)?;
            let tags = Self::load_tags(&repo)?;
            let current_index = commits.iter().position(|c| c.oid == head_oid).unwrap_or(0);

            Ok(App {
                repo,
                commits,
                tags,
                current_index,
                anchor_index: current_index,
                scroll_offset: 0,
                original_branch,
                state: AppState::Navigating,
                detailed_info: DetailedCommitInfo::default(),
                should_quit: false,
            })
        }

        fn validate_repo_state(repo: &Repository) -> Result<()> {
            let mut opts = StatusOptions::new();
            opts.include_untracked(true).recurse_untracked_dirs(true);
            let statuses = repo.statuses(Some(&mut opts))?;
            if !statuses.is_empty() {
                anyhow::bail!("🚫 Working tree is not clean. Stash or commit your changes first!");
            }
            Ok(())
        }

        fn get_head_info(repo: &Repository) -> Result<(Oid, String)> {
            let head = repo.head()?;
            let oid = head.target().context("Could not get OID from HEAD")?;
            let branch_name = head.shorthand().unwrap_or("HEAD").to_string();
            Ok((oid, branch_name))
        }

        fn load_commits(repo: &Repository, start_oid: Oid) -> Result<Vec<CommitInfo>> {
            let mut revwalk = repo.revwalk()?;
            revwalk.push(start_oid)?;
            revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;

            revwalk
                .take(50)
                .map(|oid_result| {
                    let oid = oid_result?;
                    let commit = repo.find_commit(oid)?;
                    Ok(CommitInfo {
                        oid,
                        summary: commit.summary().unwrap_or("").chars().take(70).collect(),
                    })
                })
                .collect()
        }

        fn load_tags(repo: &Repository) -> Result<HashMap<Oid, String>> {
            let mut tags_map = HashMap::new();
            if let Ok(tag_names) = repo.tag_names(None) {
                for tag_name in tag_names.iter().flatten() {
                    if let Ok(obj) = repo.revparse_single(tag_name) {
                        if let Ok(commit) = obj.peel_to_commit() {
                            tags_map.insert(commit.id(), tag_name.to_string());
                        }
                    }
                }
            }
            Ok(tags_map)
        }

        fn load_detailed_info(&mut self) -> Result<()> {
            let commit_oid = self.commits[self.current_index].oid;
            let commit = self.repo.find_commit(commit_oid)?;

            let stats = if commit.parent_count() > 0 {
                let parent = commit.parent(0)?;
                self.repo.diff_tree_to_tree(Some(&parent.tree()?), Some(&commit.tree()?), None)?
            } else {
                self.repo.diff_tree_to_tree(None, Some(&commit.tree()?), None)?
            }
            .stats()?;
            
            let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                .context("Failed to parse commit timestamp")?;

            self.detailed_info = DetailedCommitInfo {
                full_hash: commit.id().to_string(),
                author: commit.author().to_string(),
                date: timestamp.to_rfc2822(),
                full_message: commit.message().unwrap_or("").to_string(),
                files_changed: stats.files_changed(),
                insertions: stats.insertions(),
                deletions: stats.deletions(),
            };
            Ok(())
        }

        pub fn move_selection(&mut self, delta: isize) -> Result<()> {
            let new_index = self.current_index as isize + delta;
            if new_index >= 0 && new_index < self.commits.len() as isize {
                self.current_index = new_index as usize;
                self.update_working_directory()?;
                self.adjust_scroll();
            }
            Ok(())
        }
        
        pub fn jump_to_letter(&mut self, letter: char) -> Result<()> {
            if let Some(target_index) = letter.to_ascii_uppercase().to_digit(36).and_then(|d| (d as usize).checked_sub(10)) {
                let final_index = self.scroll_offset + target_index;
                if final_index < self.commits.len() {
                    self.current_index = final_index;
                    self.update_working_directory()?;
                    self.adjust_scroll();
                }
            }
            Ok(())
        }
        
        fn update_working_directory(&self) -> Result<()> {
            let target_oid = self.commits[self.current_index].oid;
            let commit = self.repo.find_commit(target_oid)?;
            self.repo.reset(commit.as_object(), ResetType::Hard, None)?;
            Ok(())
        }
        
        fn adjust_scroll(&mut self) {
            if self.current_index < self.scroll_offset {
                self.scroll_offset = self.current_index;
            } else if self.current_index >= self.scroll_offset + 10 {
                self.scroll_offset = self.current_index - 9;
            }
        }

        fn cleanup(&self) -> Result<()> {
            self.repo.set_head(&format!("refs/heads/{}", self.original_branch))?;
            self.repo.checkout_head(Some(CheckoutBuilder::new().force()))?;
            Ok(())
        }

        pub fn stop(&mut self) -> Result<String> {
            self.cleanup()?;
            self.should_quit = true;
            Ok("👋 Transmission ended. Returned to original timeline.".to_string())
        }
        
        pub fn checkout_commit(&mut self) -> Result<String> {
            let oid_to_checkout = self.commits[self.current_index].oid;
            self.cleanup()?;
            
            self.repo.set_head_detached(oid_to_checkout)?;
            self.repo.checkout_head(Some(CheckoutBuilder::new().force()))?;
            
            self.should_quit = true;
            Ok(format!(
                "✅ Mission successful. You are now at commit {}.\nTo return to your previous assignment, run: git switch {}",
                &oid_to_checkout.to_string()[..8], self.original_branch
            ))
        }

        pub fn enter_detail_view(&mut self) -> Result<()> {
            self.load_detailed_info()?;
            self.state = AppState::DetailView;
            Ok(())
        }

        pub fn exit_detail_view(&mut self) {
            self.state = AppState::Navigating;
        }

        pub fn enter_confirm_checkout(&mut self) {
            self.state = AppState::ConfirmingCheckout;
        }

        pub fn exit_confirm_checkout(&mut self) {
            self.state = AppState::DetailView;
        }
    }
}

mod ui {
    use super::*;

    pub fn draw(f: &mut Frame, app: &app::App) {
        match app.state {
            app::AppState::Navigating => draw_timeline_view(f, app),
            app::AppState::DetailView | app::AppState::ConfirmingCheckout => draw_detail_view(f, app),
        }
    }

    fn draw_timeline_view(f: &mut Frame, app: &app::App) {
        let chunks = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)
        ]).split(f.area());

        f.render_widget(header_block(), chunks[0]);
        draw_timeline_list(f, app, chunks[1]);
        
        let controls = Paragraph::new("↑↓: Navigate | A-J: Jump | Enter: Details | Q: Quit")
            .style(Style::default().fg(Color::DarkGray)).alignment(Alignment::Center);
        f.render_widget(controls, chunks[2]);
    }

    fn draw_timeline_list(f: &mut Frame, app: &app::App, area: Rect) {
        let list_block = titled_block("TEMPORAL LOG", Color::Magenta);
        let list_area = list_block.inner(area);
        f.render_widget(list_block, area);

        let visible_range = app.scroll_offset..std::cmp::min(app.scroll_offset + 10, app.commits.len());
        let mut lines = vec![];
        for (i, commit_idx) in visible_range.enumerate() {
            let commit = &app.commits[commit_idx];
            let letter = std::char::from_u32('A' as u32 + i as u32).unwrap_or('?');
            let (marker, marker_color) = if commit_idx == app.current_index {
                ("◉", Color::Cyan)
            } else if commit_idx == app.anchor_index {
                ("◎", Color::Green)
            } else {
                ("○", Color::DarkGray)
            };

            let tag_badge = app.tags.get(&commit.oid)
                .map_or(Span::raw(""), |tag| Span::styled(format!(" [{}]", tag), Style::default().yellow()));

            let line = Line::from(vec![
                Span::styled(marker, Style::default().fg(marker_color)),
                Span::raw(format!(" [{}] ", letter)).yellow(),
                Span::styled(&commit.summary, Style::default().fg(if commit_idx == app.current_index { Color::White } else { Color::Gray })),
                tag_badge,
            ]);
            lines.push(line);
            if i < 9 && commit_idx < app.commits.len() - 1 {
                lines.push(Line::from(Span::styled("   ·", Style::default().fg(Color::DarkGray))));
            }
        }
        f.render_widget(Paragraph::new(lines), list_area);
    }
    
    fn draw_detail_view(f: &mut Frame, app: &app::App) {
        let chunks = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Percentage(60), Constraint::Percentage(40)
        ]).split(f.area());

        let info = &app.detailed_info;
        f.render_widget(
            Paragraph::new(Text::from(info.full_message.as_str()))
                .block(titled_block("LOG ENTRY", Color::Cyan))
                .wrap(Wrap { trim: false }),
            chunks[0],
        );

        let bottom_chunks = Layout::default().direction(Direction::Horizontal).constraints([
            Constraint::Percentage(50), Constraint::Percentage(50)
        ]).split(chunks[1]);

        let meta_text = vec![
            styled_kv("Hash", &info.full_hash),
            styled_kv("Author", &info.author),
            styled_kv("Date", &info.date),
        ];
        f.render_widget(Paragraph::new(meta_text).block(titled_block("DATABANK RECORD", Color::Green)), bottom_chunks[0]);

        let controls_text = if app.state == app::AppState::ConfirmingCheckout {
            Line::from("Confirm temporal shift? [Y/N]".bold().red())
        } else {
            Line::from(vec!["[Enter]".bold(), " to Engage | ".into(), "[Esc]".bold(), " to Return".into()])
        };
        
        let files_str = info.files_changed.to_string();
        let insertions_str = format!("+{}", info.insertions);
        let deletions_str = format!("-{}", info.deletions);

        let stats_text = vec![
            styled_kv("Files", &files_str),
            Line::from(vec!["Insertions: ".yellow(), insertions_str.green()]),
            Line::from(vec!["Deletions:  ".yellow(), deletions_str.red()]),
            Line::from(""),
            controls_text,
        ];
        
        f.render_widget(Paragraph::new(stats_text).block(titled_block("SENSOR READINGS", Color::Magenta)), bottom_chunks[1]);
    }

    fn header_block<'a>() -> Paragraph<'a> {
        let title = " 🚀 GIT TREK ";
        let title_span = Span::styled(title, Style::default().fg(Color::Cyan).bold());
        Paragraph::new(Line::from(vec![" ".into(), title_span, " ".into()]))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Double),
            )
            .alignment(Alignment::Center)
    }
    
    fn titled_block<'a>(title: &'a str, color: Color) -> Block<'a> {
        Block::default().borders(Borders::ALL).border_type(BorderType::Double)
            .border_style(Style::default().fg(color)).title(format!(" {} ", title))
    }

    fn styled_kv<'a>(key: &'a str, value: &'a str) -> Line<'a> {
        Line::from(vec![
            Span::styled(format!("{:<8}: ", key), Style::default().yellow()),
            Span::styled(value, Style::default().white()),
        ])
    }
}

mod tui {
    use super::*;

    pub fn run(mut app: app::App) -> Result<String> {
        let mut terminal = setup_terminal()?;
        let mut final_message = String::new();

        while !app.should_quit {
            terminal.draw(|f| ui::draw(f, &app))?;
            
            if let Some(msg) = handle_events(&mut app)? {
                final_message = msg;
            }
        }
        Ok(final_message)
    }

    fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout(); // Corrected: io::stdout()
        execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend).map_err(Into::into)
    }

    pub fn cleanup_terminal() -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, event::DisableMouseCapture)?;
        Ok(())
    }

    fn handle_events(app: &mut app::App) -> Result<Option<String>> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    return match app.state {
                        app::AppState::Navigating => handle_navigation_keys(app, key.code),
                        app::AppState::DetailView => handle_detail_view_keys(app, key.code),
                        app::AppState::ConfirmingCheckout => handle_confirmation_keys(app, key.code),
                    };
                }
            }
        }
        Ok(None)
    }

    fn handle_navigation_keys(app: &mut app::App, key_code: KeyCode) -> Result<Option<String>> {
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => app.stop().map(Some),
            KeyCode::Up | KeyCode::Char('w') => app.move_selection(-1).map(|_| None),
            KeyCode::Down | KeyCode::Char('s') => app.move_selection(1).map(|_| None),
            KeyCode::Enter => app.enter_detail_view().map(|_| None),
            KeyCode::Char(c) if c.is_alphabetic() => app.jump_to_letter(c).map(|_| None),
            _ => Ok(None),
        }
    }

    fn handle_detail_view_keys(app: &mut app::App, key_code: KeyCode) -> Result<Option<String>> {
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => app.exit_detail_view(),
            KeyCode::Enter | KeyCode::Char('c') => app.enter_confirm_checkout(),
            _ => {}
        }
        Ok(None)
    }

    fn handle_confirmation_keys(app: &mut app::App, key_code: KeyCode) -> Result<Option<String>> {
        match key_code {
            KeyCode::Char('y') | KeyCode::Char('Y') => app.checkout_commit().map(Some),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Backspace => {
                app.exit_confirm_checkout();
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

#[derive(Parser)]
#[command(name = "git-trek", about = "🚀 Navigate git history like it's 1989!")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start your journey through time (default)
    Start,
}

fn main() -> Result<()> {
    let _cli = Cli::parse(); 
    
    let app = app::App::new()?;
    let final_message = match tui::run(app) {
        Ok(message) => message,
        Err(e) => {
            tui::cleanup_terminal()?;
            return Err(e);
        }
    };
    tui::cleanup_terminal()?;
    if !final_message.is_empty() {
        println!("{}", final_message);
    }

    Ok(())
}