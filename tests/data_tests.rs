//! Data model tests
//! Anchors: data-health-from-change, data-health-from-ratio, data-tracked-lines-at, data-tracked-health-at

use git_trek::data::{FileSnapshot, HealthStatus, TrackedFile};

#[test]
fn test_health_none_to_none_is_stable() {
    assert_eq!(HealthStatus::from_size_change(None, None), HealthStatus::Stable);
}

#[test]
fn test_health_none_to_some_is_new() {
    assert_eq!(HealthStatus::from_size_change(None, Some(100)), HealthStatus::New);
}

#[test]
fn test_health_some_to_none_is_deleted() {
    assert_eq!(HealthStatus::from_size_change(Some(100), None), HealthStatus::Deleted);
}

#[test]
fn test_health_ratio_under_70_is_fucked() {
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(69)), HealthStatus::MaybeFucked);
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(50)), HealthStatus::MaybeFucked);
}

#[test]
fn test_health_ratio_70_to_95_is_shrank() {
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(70)), HealthStatus::Shrank);
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(94)), HealthStatus::Shrank);
}

#[test]
fn test_health_ratio_95_to_105_is_stable() {
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(95)), HealthStatus::Stable);
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(100)), HealthStatus::Stable);
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(105)), HealthStatus::Stable);
}

#[test]
fn test_health_ratio_over_105_is_grew() {
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(106)), HealthStatus::Grew);
    assert_eq!(HealthStatus::from_size_change(Some(100), Some(200)), HealthStatus::Grew);
}

#[test]
fn test_health_zero_old_is_new() {
    assert_eq!(HealthStatus::from_size_change(Some(0), Some(100)), HealthStatus::New);
}

#[test]
fn test_tracked_file_lines_at_missing() {
    let tf = TrackedFile::new("test.rs".to_string());
    assert_eq!(tf.lines_at(0), None);
}

#[test]
fn test_tracked_file_lines_at_present() {
    let mut tf = TrackedFile::new("test.rs".to_string());
    tf.history.insert(0, FileSnapshot { lines: 100, bytes: 2000 });
    assert_eq!(tf.lines_at(0), Some(100));
}

#[test]
fn test_tracked_file_health_at_no_prev() {
    let mut tf = TrackedFile::new("test.rs".to_string());
    tf.history.insert(0, FileSnapshot { lines: 100, bytes: 2000 });
    assert_eq!(tf.health_at(0, None), HealthStatus::New);
}

#[test]
fn test_tracked_file_health_at_shrink() {
    let mut tf = TrackedFile::new("test.rs".to_string());
    tf.history.insert(0, FileSnapshot { lines: 100, bytes: 2000 });
    tf.history.insert(1, FileSnapshot { lines: 200, bytes: 4000 });
    assert_eq!(tf.health_at(0, Some(1)), HealthStatus::MaybeFucked);
}

#[test]
fn test_tracked_file_health_at_grow() {
    let mut tf = TrackedFile::new("test.rs".to_string());
    tf.history.insert(0, FileSnapshot { lines: 100, bytes: 2000 });
    tf.history.insert(1, FileSnapshot { lines: 200, bytes: 4000 });
    assert_eq!(tf.health_at(1, Some(0)), HealthStatus::Grew);
}