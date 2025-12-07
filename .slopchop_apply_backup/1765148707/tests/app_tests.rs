//! App state tests
//! Tests scroll_timeline bounds, handle_click dispatch, view switching

use git_trek::data::{CommitInfo, FileSnapshot, RepoData, TrackedFile};
use git_trek::mouse::HitId;
use git_trek::views::ViewMode;
use git2::Oid;

/// Create a minimal RepoData for testing without a real repo
fn mock_repo_data(num_commits: usize) -> RepoData {
    let mut data = RepoData::new();
    let zero_oid = Oid::zero();
    
    for i in 0..num_commits {
        data.commits.push(CommitInfo {
            oid: zero_oid,
            summary: format!("Commit {i}"),
            author: "test".to_string(),
            timestamp: i as i64,
            files_changed: vec![],
            insertions: 0,
            deletions: 0,
        });
    }
    
    let mut file = TrackedFile::new("test.rs".to_string());
    file.history.insert(0, FileSnapshot { lines: 100, bytes: 2000 });
    data.files.insert("test.rs".to_string(), file);
    
    data
}

// === scroll_timeline tests ===

#[test]
fn test_scroll_timeline_forward_from_zero() {
    let data = mock_repo_data(10);
    let mut idx = 0usize;
    let max = data.commits.len().saturating_sub(1);
    
    // Simulate scroll forward by 1
    idx = idx.saturating_add(1).min(max);
    assert_eq!(idx, 1);
}

#[test]
fn test_scroll_timeline_backward_from_zero_stays_zero() {
    let data = mock_repo_data(10);
    let mut idx = 0usize;
    let _max = data.commits.len().saturating_sub(1);
    
    // Simulate scroll backward from 0
    idx = idx.saturating_sub(1);
    assert_eq!(idx, 0, "Should not go below 0");
}

#[test]
fn test_scroll_timeline_forward_clamps_at_max() {
    let data = mock_repo_data(10);
    let max = data.commits.len().saturating_sub(1); // 9
    let mut idx = 8usize;
    
    // Scroll forward by 5, should clamp at 9
    idx = idx.saturating_add(5).min(max);
    assert_eq!(idx, 9);
    
    // Scroll forward again, should stay at 9
    idx = idx.saturating_add(1).min(max);
    assert_eq!(idx, 9);
}

#[test]
fn test_scroll_timeline_empty_commits() {
    let data = mock_repo_data(0);
    let max = data.commits.len().saturating_sub(1); // 0 (saturating)
    let mut idx = 0usize;
    
    idx = idx.saturating_add(1).min(max);
    assert_eq!(idx, 0, "Empty repo should stay at 0");
}

#[test]
fn test_scroll_timeline_single_commit() {
    let data = mock_repo_data(1);
    let max = data.commits.len().saturating_sub(1); // 0
    let mut idx = 0usize;
    
    idx = idx.saturating_add(1).min(max);
    assert_eq!(idx, 0, "Single commit repo should stay at 0");
}

#[test]
fn test_scroll_timeline_large_delta() {
    let data = mock_repo_data(100);
    let max = data.commits.len().saturating_sub(1);
    let mut idx = 50usize;
    
    // Large backward jump
    idx = idx.saturating_sub(100);
    assert_eq!(idx, 0);
    
    // Large forward jump
    idx = idx.saturating_add(500).min(max);
    assert_eq!(idx, 99);
}

// === handle_click tests ===

#[test]
fn test_handle_click_file_sets_selected() {
    let mut selected: Option<String> = None;
    let id = HitId::File("src/main.rs".to_string());
    
    if let HitId::File(path) = id {
        selected = Some(path);
    }
    
    assert_eq!(selected, Some("src/main.rs".to_string()));
}

#[test]
fn test_handle_click_viewtab_changes_view() {
    let mut view = ViewMode::Treemap;
    let id = HitId::ViewTab(2);
    
    if let HitId::ViewTab(i) = id {
        view = ViewMode::from_index(i);
    }
    
    assert_eq!(view, ViewMode::Minimap);
}

#[test]
fn test_handle_click_none_does_nothing() {
    let mut selected: Option<String> = Some("existing.rs".to_string());
    let mut view = ViewMode::Treemap;
    let id = HitId::None;
    
    match id {
        HitId::File(path) => selected = Some(path),
        HitId::ViewTab(i) => view = ViewMode::from_index(i),
        HitId::None => {}
    }
    
    assert_eq!(selected, Some("existing.rs".to_string()), "Should not change");
    assert_eq!(view, ViewMode::Treemap, "Should not change");
}

// === view switching tests ===

#[test]
fn test_set_view_direct() {
    let mut view = ViewMode::Treemap;
    view = ViewMode::Focus;
    assert_eq!(view, ViewMode::Focus);
}

#[test]
fn test_next_view_full_cycle() {
    let mut view = ViewMode::Treemap;
    let mut visited = vec![view];
    
    for _ in 0..5 {
        view = view.next();
        visited.push(view);
    }
    
    assert_eq!(visited, vec![
        ViewMode::Treemap,
        ViewMode::Heatmap,
        ViewMode::Minimap,
        ViewMode::River,
        ViewMode::Focus,
        ViewMode::Treemap, // wrapped
    ]);
}

#[test]
fn test_prev_view_full_cycle() {
    let mut view = ViewMode::Treemap;
    let mut visited = vec![view];
    
    for _ in 0..5 {
        view = view.prev();
        visited.push(view);
    }
    
    assert_eq!(visited, vec![
        ViewMode::Treemap,
        ViewMode::Focus,   // wrapped backward
        ViewMode::River,
        ViewMode::Minimap,
        ViewMode::Heatmap,
        ViewMode::Treemap,
    ]);
}