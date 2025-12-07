//! Input handling tests
//! Anchors: input-hit-test, view-mode-*, mouse-state-*

use git_trek::mouse::{hit_test, HitBox, HitId, MouseState};
use git_trek::views::ViewMode;
use ratatui::layout::Rect;

// === hit_test tests ===

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
fn test_hit_test_boundary_inclusive() {
    let boxes = vec![
        HitBox { rect: Rect::new(10, 10, 5, 5), id: HitId::ViewTab(0) },
    ];
    // Top-left corner (inclusive)
    assert_eq!(hit_test(10, 10, &boxes), HitId::ViewTab(0));
    // Just inside bottom-right
    assert_eq!(hit_test(14, 14, &boxes), HitId::ViewTab(0));
}

#[test]
fn test_hit_test_boundary_exclusive() {
    let boxes = vec![
        HitBox { rect: Rect::new(10, 10, 5, 5), id: HitId::ViewTab(0) },
    ];
    // Right edge (exclusive)
    assert_eq!(hit_test(15, 10, &boxes), HitId::None);
    // Bottom edge (exclusive)
    assert_eq!(hit_test(10, 15, &boxes), HitId::None);
}

#[test]
fn test_hit_test_first_match_wins() {
    let boxes = vec![
        HitBox { rect: Rect::new(0, 0, 20, 20), id: HitId::File("first.rs".to_string()) },
        HitBox { rect: Rect::new(5, 5, 10, 10), id: HitId::File("second.rs".to_string()) },
    ];
    assert_eq!(hit_test(7, 7, &boxes), HitId::File("first.rs".to_string()));
}

#[test]
fn test_hit_test_zero_size_rect() {
    let boxes = vec![
        HitBox { rect: Rect::new(10, 10, 0, 0), id: HitId::File("zero.rs".to_string()) },
    ];
    assert_eq!(hit_test(10, 10, &boxes), HitId::None);
}

#[test]
fn test_hit_test_large_coordinates() {
    let boxes = vec![
        HitBox { rect: Rect::new(1000, 1000, 100, 100), id: HitId::File("far.rs".to_string()) },
    ];
    assert_eq!(hit_test(1050, 1050, &boxes), HitId::File("far.rs".to_string()));
    assert_eq!(hit_test(999, 1050, &boxes), HitId::None);
}

// === MouseState tests ===

#[test]
fn test_mouse_state_default() {
    let state = MouseState::default();
    assert_eq!(state.x, 0);
    assert_eq!(state.y, 0);
    assert_eq!(state.hover, HitId::None);
}

#[test]
fn test_mouse_state_update_position() {
    let mut state = MouseState::default();
    state.update_position(42, 99);
    assert_eq!(state.x, 42);
    assert_eq!(state.y, 99);
}

#[test]
fn test_mouse_state_update_position_max_values() {
    let mut state = MouseState::default();
    state.update_position(u16::MAX, u16::MAX);
    assert_eq!(state.x, u16::MAX);
    assert_eq!(state.y, u16::MAX);
}

#[test]
fn test_mouse_state_set_hover() {
    let mut state = MouseState::default();
    state.set_hover(HitId::File("test.rs".to_string()));
    assert_eq!(state.hover, HitId::File("test.rs".to_string()));
}

#[test]
fn test_mouse_state_set_hover_overwrites() {
    let mut state = MouseState::default();
    state.set_hover(HitId::File("first.rs".to_string()));
    state.set_hover(HitId::ViewTab(2));
    assert_eq!(state.hover, HitId::ViewTab(2));
}

#[test]
fn test_mouse_state_set_hover_none() {
    let mut state = MouseState::default();
    state.set_hover(HitId::File("test.rs".to_string()));
    state.set_hover(HitId::None);
    assert_eq!(state.hover, HitId::None);
}

// === ViewMode tests ===

#[test]
fn test_viewmode_name() {
    assert_eq!(ViewMode::Treemap.name(), "Treemap");
    assert_eq!(ViewMode::Heatmap.name(), "Heatmap");
    assert_eq!(ViewMode::Minimap.name(), "Minimap");
    assert_eq!(ViewMode::River.name(), "River");
    assert_eq!(ViewMode::Focus.name(), "Focus");
}

#[test]
fn test_viewmode_index() {
    assert_eq!(ViewMode::Treemap.index(), 0);
    assert_eq!(ViewMode::Heatmap.index(), 1);
    assert_eq!(ViewMode::Minimap.index(), 2);
    assert_eq!(ViewMode::River.index(), 3);
    assert_eq!(ViewMode::Focus.index(), 4);
}

#[test]
fn test_viewmode_from_index_valid() {
    assert_eq!(ViewMode::from_index(0), ViewMode::Treemap);
    assert_eq!(ViewMode::from_index(1), ViewMode::Heatmap);
    assert_eq!(ViewMode::from_index(2), ViewMode::Minimap);
    assert_eq!(ViewMode::from_index(3), ViewMode::River);
    assert_eq!(ViewMode::from_index(4), ViewMode::Focus);
}

#[test]
fn test_viewmode_from_index_invalid_defaults_to_treemap() {
    assert_eq!(ViewMode::from_index(5), ViewMode::Treemap);
    assert_eq!(ViewMode::from_index(99), ViewMode::Treemap);
    assert_eq!(ViewMode::from_index(usize::MAX), ViewMode::Treemap);
}

#[test]
fn test_viewmode_next_cycles() {
    assert_eq!(ViewMode::Treemap.next(), ViewMode::Heatmap);
    assert_eq!(ViewMode::Heatmap.next(), ViewMode::Minimap);
    assert_eq!(ViewMode::Minimap.next(), ViewMode::River);
    assert_eq!(ViewMode::River.next(), ViewMode::Focus);
    assert_eq!(ViewMode::Focus.next(), ViewMode::Treemap);
}

#[test]
fn test_viewmode_prev_cycles() {
    assert_eq!(ViewMode::Treemap.prev(), ViewMode::Focus);
    assert_eq!(ViewMode::Focus.prev(), ViewMode::River);
    assert_eq!(ViewMode::River.prev(), ViewMode::Minimap);
    assert_eq!(ViewMode::Minimap.prev(), ViewMode::Heatmap);
    assert_eq!(ViewMode::Heatmap.prev(), ViewMode::Treemap);
}

#[test]
fn test_viewmode_all_array() {
    assert_eq!(ViewMode::ALL.len(), 5);
    assert_eq!(ViewMode::ALL[0], ViewMode::Treemap);
    assert_eq!(ViewMode::ALL[4], ViewMode::Focus);
}

#[test]
fn test_viewmode_roundtrip() {
    for mode in ViewMode::ALL {
        let idx = mode.index();
        let recovered = ViewMode::from_index(idx);
        assert_eq!(mode, recovered);
    }
}

#[test]
fn test_viewmode_next_prev_inverse() {
    for mode in ViewMode::ALL {
        assert_eq!(mode.next().prev(), mode);
        assert_eq!(mode.prev().next(), mode);
    }
}