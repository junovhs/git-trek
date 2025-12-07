//! Input handling tests
//! Anchors: input-hit-test, view-mode-name, view-mode-index, view-mode-from-index, view-mode-next, view-mode-prev

use git_trek::mouse::{hit_test, HitBox, HitId};
use git_trek::views::ViewMode;
use ratatui::layout::Rect;

#[test]
fn test_hit_test_no_boxes_returns_none() {
    assert_eq!(hit_test(10, 10, &[]), HitId::None);
}

#[test]
fn test_hit_test_miss_returns_none() {
    let boxes = vec![
        HitBox { rect: Rect::new(0, 0, 10, 10), id: HitId::File("test.rs".to_string()) },
    ];
    assert_eq!(hit_test(15, 15, &boxes), HitId::None);
}

#[test]
fn test_hit_test_hit_returns_id() {
    let boxes = vec![
        HitBox { rect: Rect::new(0, 0, 10, 10), id: HitId::File("test.rs".to_string()) },
    ];
    assert_eq!(hit_test(5, 5, &boxes), HitId::File("test.rs".to_string()));
}

#[test]
fn test_hit_test_boundary() {
    let boxes = vec![
        HitBox { rect: Rect::new(10, 10, 5, 5), id: HitId::ViewTab(0) },
    ];
    assert_eq!(hit_test(10, 10, &boxes), HitId::ViewTab(0));
    assert_eq!(hit_test(14, 14, &boxes), HitId::ViewTab(0));
    assert_eq!(hit_test(15, 10, &boxes), HitId::None);
}

#[test]
fn test_viewmode_name() {
    assert_eq!(ViewMode::Treemap.name(), "Treemap");
    assert_eq!(ViewMode::Heatmap.name(), "Heatmap");
    assert_eq!(ViewMode::Focus.name(), "Focus");
}

#[test]
fn test_viewmode_index() {
    assert_eq!(ViewMode::Treemap.index(), 0);
    assert_eq!(ViewMode::Focus.index(), 4);
}

#[test]
fn test_viewmode_from_index_valid() {
    assert_eq!(ViewMode::from_index(0), ViewMode::Treemap);
    assert_eq!(ViewMode::from_index(4), ViewMode::Focus);
}

#[test]
fn test_viewmode_from_index_invalid() {
    assert_eq!(ViewMode::from_index(99), ViewMode::Treemap);
}

#[test]
fn test_viewmode_next_cycles() {
    assert_eq!(ViewMode::Treemap.next(), ViewMode::Heatmap);
    assert_eq!(ViewMode::Focus.next(), ViewMode::Treemap);
}

#[test]
fn test_viewmode_prev_cycles() {
    assert_eq!(ViewMode::Treemap.prev(), ViewMode::Focus);
    assert_eq!(ViewMode::Heatmap.prev(), ViewMode::Treemap);
}

#[test]
fn test_viewmode_all_array() {
    assert_eq!(ViewMode::ALL.len(), 5);
    assert_eq!(ViewMode::ALL[0], ViewMode::Treemap);
}