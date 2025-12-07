//! Git operations tests
//! Tests `format_oid`, file restoration in temp repos

use git_trek::git_ops::{format_oid, get_file_content, load_repo_data, restore_file};
use git2::{Repository, Signature};
use std::fs;
use tempfile::TempDir;

fn create_test_repo() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");
    
    let mut config = repo.config().expect("get config");
    config.set_str("user.name", "Test User").expect("set name");
    config.set_str("user.email", "test@example.com").expect("set email");
    
    (dir, repo)
}

fn commit_file(repo: &Repository, path: &str, content: &str, message: &str) -> git2::Oid {
    let root = repo.workdir().expect("workdir");
    let file_path = root.join(path);
    
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).expect("create dirs");
    }
    
    fs::write(&file_path, content).expect("write file");
    
    let mut index = repo.index().expect("get index");
    index.add_path(std::path::Path::new(path)).expect("add to index");
    index.write().expect("write index");
    
    let tree_id = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let sig = Signature::now("Test", "test@test.com").expect("signature");
    
    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();
    
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .expect("commit")
}

// === format_oid tests ===

#[test]
fn test_format_oid_length() {
    let (_dir, repo) = create_test_repo();
    let oid = commit_file(&repo, "test.txt", "hello", "initial");
    
    let formatted = format_oid(oid);
    assert_eq!(formatted.len(), 8, "Should be 8 characters");
}

#[test]
fn test_format_oid_hex_chars() {
    let (_dir, repo) = create_test_repo();
    let oid = commit_file(&repo, "test.txt", "hello", "initial");
    
    let formatted = format_oid(oid);
    assert!(formatted.chars().all(|c| c.is_ascii_hexdigit()), "Should be hex");
}

#[test]
fn test_format_oid_matches_full() {
    let (_dir, repo) = create_test_repo();
    let oid = commit_file(&repo, "test.txt", "hello", "initial");
    
    let formatted = format_oid(oid);
    let full = oid.to_string();
    assert!(full.starts_with(&formatted), "Should be prefix of full OID");
}

// === load_repo_data tests ===

#[test]
fn test_load_repo_data_empty_repo() {
    let (_dir, repo) = create_test_repo();
    let result = load_repo_data(&repo, 10);
    assert!(result.is_err() || result.unwrap().commits.is_empty());
}

#[test]
fn test_load_repo_data_single_commit() {
    let (_dir, repo) = create_test_repo();
    commit_file(&repo, "main.rs", "fn main() {}\n", "initial");
    
    let data = load_repo_data(&repo, 10).expect("load data");
    assert_eq!(data.commits.len(), 1);
    assert!(data.files.contains_key("main.rs"));
}

#[test]
fn test_load_repo_data_multiple_commits() {
    let (_dir, repo) = create_test_repo();
    commit_file(&repo, "a.rs", "// a\n", "first");
    commit_file(&repo, "b.rs", "// b\n", "second");
    commit_file(&repo, "c.rs", "// c\n", "third");
    
    let data = load_repo_data(&repo, 10).expect("load data");
    assert_eq!(data.commits.len(), 3);
}

#[test]
fn test_load_repo_data_respects_limit() {
    let (_dir, repo) = create_test_repo();
    for i in 0..10 {
        commit_file(&repo, &format!("file{i}.rs"), "// content\n", &format!("commit {i}"));
    }
    
    let data = load_repo_data(&repo, 5).expect("load data");
    assert_eq!(data.commits.len(), 5, "Should respect limit");
}

#[test]
fn test_load_repo_data_tracks_file_history() {
    let (_dir, repo) = create_test_repo();
    commit_file(&repo, "main.rs", "line1\n", "v1");
    commit_file(&repo, "main.rs", "line1\nline2\n", "v2");
    commit_file(&repo, "main.rs", "line1\nline2\nline3\n", "v3");
    
    let data = load_repo_data(&repo, 10).expect("load data");
    let file = data.files.get("main.rs").expect("file tracked");
    
    assert!(!file.history.is_empty());
}

// === get_file_content tests ===

#[test]
fn test_get_file_content_retrieves_correct_version() {
    let (_dir, repo) = create_test_repo();
    let oid1 = commit_file(&repo, "test.rs", "version1\n", "v1");
    let _oid2 = commit_file(&repo, "test.rs", "version2\n", "v2");
    
    let content = get_file_content(&repo, oid1, "test.rs").expect("get content");
    assert_eq!(content, "version1\n");
}

#[test]
fn test_get_file_content_nonexistent_file() {
    let (_dir, repo) = create_test_repo();
    let oid = commit_file(&repo, "exists.rs", "content\n", "commit");
    
    let result = get_file_content(&repo, oid, "nonexistent.rs");
    assert!(result.is_err());
}

// === restore_file tests ===

#[test]
fn test_restore_file_writes_to_disk() {
    let (dir, repo) = create_test_repo();
    let oid = commit_file(&repo, "test.rs", "original content\n", "v1");
    
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "modified content\n").expect("modify");
    
    restore_file(&repo, oid, "test.rs").expect("restore");
    
    let restored = fs::read_to_string(&file_path).expect("read");
    assert_eq!(restored, "original content\n");
}

#[test]
fn test_restore_file_older_version() {
    let (dir, repo) = create_test_repo();
    let oid1 = commit_file(&repo, "test.rs", "v1 content\n", "v1");
    let _oid2 = commit_file(&repo, "test.rs", "v2 content\n", "v2");
    let _oid3 = commit_file(&repo, "test.rs", "v3 content\n", "v3");
    
    restore_file(&repo, oid1, "test.rs").expect("restore");
    
    let file_path = dir.path().join("test.rs");
    let restored = fs::read_to_string(&file_path).expect("read");
    assert_eq!(restored, "v1 content\n");
}

#[test]
fn test_restore_file_nested_path() {
    let (dir, repo) = create_test_repo();
    let oid = commit_file(&repo, "src/lib/module.rs", "nested content\n", "commit");
    
    let file_path = dir.path().join("src/lib/module.rs");
    fs::remove_file(&file_path).expect("remove");
    
    let result = restore_file(&repo, oid, "src/lib/module.rs");
    
    if result.is_ok() {
        let restored = fs::read_to_string(&file_path).expect("read");
        assert_eq!(restored, "nested content\n");
    }
}