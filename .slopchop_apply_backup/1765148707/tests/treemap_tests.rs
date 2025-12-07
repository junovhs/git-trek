//! Treemap layout and utility tests
//! Tests truncate_path edge cases, compute_treemap_layout validity

use git_trek::views::treemap::{compute_treemap_layout, truncate_path};
use ratatui::layout::Rect;

// === truncate_path tests ===

#[test]
fn test_truncate_path_short_unchanged() {
    assert_eq!(truncate_path("main.rs", 20), "main.rs");
}

#[test]
fn test_truncate_path_exact_length() {
    assert_eq!(truncate_path("main.rs", 7), "main.rs");
}

#[test]
fn test_truncate_path_extracts_filename() {
    let result = truncate_path("src/views/treemap.rs", 10);
    assert!(result.len() <= 10);
    assert!(result.starts_with("treemap"));
}

#[test]
fn test_truncate_path_adds_ellipsis() {
    let result = truncate_path("src/very_long_filename.rs", 8);
    assert!(result.ends_with(".."));
    assert!(result.len() <= 8);
}

#[test]
fn test_truncate_path_empty() {
    assert_eq!(truncate_path("", 10), "");
}

#[test]
fn test_truncate_path_just_filename() {
    assert_eq!(truncate_path("x.rs", 10), "x.rs");
}

#[test]
fn test_truncate_path_max_zero() {
    // Edge case: max width 0
    let result = truncate_path("test.rs", 0);
    assert!(result.is_empty() || result.len() <= 2);
}

#[test]
fn test_truncate_path_max_one() {
    let result = truncate_path("test.rs", 1);
    assert!(result.len() <= 1);
}

#[test]
fn test_truncate_path_unicode() {
    let result = truncate_path("src/日本語.rs", 8);
    assert!(result.chars().count() <= 8);
}

#[test]
fn test_truncate_path_deeply_nested() {
    let result = truncate_path("a/b/c/d/e/f/g/file.rs", 10);
    assert!(result.len() <= 10);
    assert!(result.contains("file") || result.ends_with(".."));
}

// === compute_treemap_layout tests ===

#[test]
fn test_treemap_layout_empty_files() {
    let files: Vec<(String, usize)> = vec![];
    let area = Rect::new(0, 0, 100, 50);
    let result = compute_treemap_layout(&files, area);
    assert!(result.is_empty());
}

#[test]
fn test_treemap_layout_single_file() {
    let files = vec![("main.rs".to_string(), 100)];
    let area = Rect::new(0, 0, 100, 50);
    let result = compute_treemap_layout(&files, area);
    
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "main.rs");
    assert_eq!(result[0].1, 100);
}

#[test]
fn test_treemap_layout_rects_within_bounds() {
    let files = vec![
        ("a.rs".to_string(), 100),
        ("b.rs".to_string(), 50),
        ("c.rs".to_string(), 25),
    ];
    let area = Rect::new(10, 20, 80, 40);
    let result = compute_treemap_layout(&files, area);
    
    for (path, _lines, rect) in &result {
        assert!(rect.x >= area.x, "{path} x out of bounds");
        assert!(rect.y >= area.y, "{path} y out of bounds");
        assert!(rect.x + rect.width <= area.x + area.width, "{path} right edge out of bounds");
        assert!(rect.y + rect.height <= area.y + area.height, "{path} bottom edge out of bounds");
    }
}

#[test]
fn test_treemap_layout_all_files_present() {
    let files = vec![
        ("a.rs".to_string(), 100),
        ("b.rs".to_string(), 50),
        ("c.rs".to_string(), 25),
    ];
    let area = Rect::new(0, 0, 100, 50);
    let result = compute_treemap_layout(&files, area);
    
    let paths: Vec<_> = result.iter().map(|(p, _, _)| p.as_str()).collect();
    assert!(paths.contains(&"a.rs"));
    assert!(paths.contains(&"b.rs"));
    assert!(paths.contains(&"c.rs"));
}

#[test]
fn test_treemap_layout_zero_area_width() {
    let files = vec![("a.rs".to_string(), 100)];
    let area = Rect::new(0, 0, 0, 50);
    let result = compute_treemap_layout(&files, area);
    assert!(result.is_empty());
}

#[test]
fn test_treemap_layout_zero_area_height() {
    let files = vec![("a.rs".to_string(), 100)];
    let area = Rect::new(0, 0, 100, 0);
    let result = compute_treemap_layout(&files, area);
    assert!(result.is_empty());
}

#[test]
fn test_treemap_layout_tiny_area() {
    let files = vec![("a.rs".to_string(), 100)];
    let area = Rect::new(0, 0, 2, 2);
    let result = compute_treemap_layout(&files, area);
    // Should either be empty or produce valid (small) rect
    for (_, _, rect) in &result {
        assert!(rect.width >= 1);
        assert!(rect.height >= 1);
    }
}

#[test]
fn test_treemap_layout_respects_20_file_limit() {
    let files: Vec<_> = (0..30)
        .map(|i| (format!("file{i}.rs"), 10))
        .collect();
    let area = Rect::new(0, 0, 200, 100);
    let result = compute_treemap_layout(&files, area);
    
    assert!(result.len() <= 20, "Should limit to 20 files, got {}", result.len());
}

#[test]
fn test_treemap_layout_zero_line_files() {
    let files = vec![
        ("empty.rs".to_string(), 0),
        ("also_empty.rs".to_string(), 0),
    ];
    let area = Rect::new(0, 0, 100, 50);
    let result = compute_treemap_layout(&files, area);
    // Should handle gracefully (either empty or minimal rects)
    assert!(result.is_empty() || result.iter().all(|(_, _, r)| r.width > 0 && r.height > 0));
}

#[test]
fn test_treemap_layout_mixed_sizes() {
    let files = vec![
        ("huge.rs".to_string(), 10000),
        ("tiny.rs".to_string(), 1),
    ];
    let area = Rect::new(0, 0, 100, 50);
    let result = compute_treemap_layout(&files, area);
    
    // Huge file should get more space
    if result.len() == 2 {
        let huge_area = result[0].2.width as u32 * result[0].2.height as u32;
        let tiny_area = result[1].2.width as u32 * result[1].2.height as u32;
        assert!(huge_area > tiny_area, "Larger file should get more area");
    }
}