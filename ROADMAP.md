# git-trek Roadmap

Visual git time travel & file recovery

---

## Core Infrastructure ?? CURRENT

- [x] **main() entry point with error handling** <!-- test: [no-test] -->
- [x] **setup_terminal() raw mode + mouse capture** <!-- test: [no-test] -->
- [x] **restore_terminal() cleanup on exit** <!-- test: [no-test] -->
- [x] **run_app() main event loop with 50ms poll** <!-- test: [no-test] -->
- [x] **App struct (repo, data, view, commit_idx, selected_file, mouse, should_quit, message)** <!-- test: [no-test] -->
- [x] **App::new() initialize from CLI args** <!-- test: [no-test] -->
- [x] **Cli struct (limit, dry_run)** <!-- test: [no-test] -->
- [x] **Cli::parse_checked() with validation** <!-- test: [no-test] -->
- [x] **--limit flag for commit count** <!-- test: [no-test] -->
- [x] **--dry-run flag for testing** <!-- test: [no-test] -->
- [x] **Post-exit message display** <!-- test: [no-test] -->
- [x] **App::current_commit_label() format hash** <!-- test: [no-test] -->
- [x] **App::files_at_current_commit() sorted by size** <!-- test: [no-test] -->
- [x] **App::file_health() get status for path** <!-- test: [no-test] -->
- [x] **App::handle_click() dispatch File/ViewTab** <!-- test: tests/app_tests.rs::test_handle_click_file_sets_selected -->
- [x] **App::scroll_timeline() with bounds checking** <!-- test: tests/app_tests.rs::test_scroll_timeline_forward_from_zero -->
- [x] **App::set_view()** <!-- test: tests/app_tests.rs::test_set_view_direct -->
- [x] **App::next_view()** <!-- test: tests/app_tests.rs::test_next_view_full_cycle -->
- [x] **App::prev_view()** <!-- test: tests/app_tests.rs::test_prev_view_full_cycle -->
- [x] **App::restore_selected() restore with message** <!-- test: [no-test] -->

---

## Data Model ?? CURRENT

- [x] **HealthStatus enum (Stable, Grew, Shrank, MaybeFucked, New, Deleted)** <!-- test: [no-test] -->
- [x] **HealthStatus::from_size_change() old/new comparison** <!-- test: tests/data_tests.rs::test_health_none_to_some_is_new -->
- [x] **HealthStatus::from_ratio() with 0.7/0.95/1.05 thresholds** <!-- test: tests/data_tests.rs::test_health_ratio_under_70_is_fucked -->
- [x] **FileSnapshot struct (lines, bytes)** <!-- test: [no-test] -->
- [x] **TrackedFile struct (path, history HashMap)** <!-- test: [no-test] -->
- [x] **TrackedFile::new()** <!-- test: [no-test] -->
- [x] **TrackedFile::lines_at() get lines at commit index** <!-- test: tests/data_tests.rs::test_tracked_file_lines_at_present -->
- [x] **TrackedFile::health_at() compare to previous commit** <!-- test: tests/data_tests.rs::test_tracked_file_health_at_shrink -->
- [x] **CommitInfo struct (oid, summary, author, timestamp, files_changed, insertions, deletions)** <!-- test: [no-test] -->
- [x] **RepoData struct (commits Vec, files HashMap)** <!-- test: [no-test] -->
- [x] **RepoData::new()** <!-- test: [no-test] -->

---

## Git Integration ?? CURRENT

- [x] **load_repo_data() main loader function** <!-- test: tests/git_tests.rs::test_load_repo_data_single_commit -->
- [x] **collect_commit_oids() revwalk with TOPOLOGICAL|TIME sort** <!-- test: [no-test] -->
- [x] **build_commit_info() extract commit metadata** <!-- test: [no-test] -->
- [x] **get_diff_stats() insertions/deletions count** <!-- test: [no-test] -->
- [x] **collect_file_snapshots() tree walk for files** <!-- test: [no-test] -->
- [x] **process_blob_entry() count newlines in blob** <!-- test: [no-test] -->
- [x] **format_oid() 8-char hash display** <!-- test: tests/git_tests.rs::test_format_oid_length -->
- [x] **get_file_content() retrieve file at commit** <!-- test: tests/git_tests.rs::test_get_file_content_retrieves_correct_version -->
- [x] **restore_file() write file to disk** <!-- test: tests/git_tests.rs::test_restore_file_writes_to_disk -->

---

## Input Handling ?? CURRENT

- [x] **HitBox struct (rect, id)** <!-- test: [no-test] -->
- [x] **HitId enum (File, ViewTab, None)** <!-- test: [no-test] -->
- [x] **MouseState struct (x, y, hover)** <!-- test: tests/input_tests.rs::test_mouse_state_default -->
- [x] **MouseState::update_position()** <!-- test: tests/input_tests.rs::test_mouse_state_update_position -->
- [x] **MouseState::set_hover()** <!-- test: tests/input_tests.rs::test_mouse_state_set_hover -->
- [x] **hit_test() find element under cursor** <!-- test: tests/input_tests.rs::test_hit_test_hit_returns_id -->
- [x] **handle_mouse() dispatch Moved/Down/Scroll** <!-- test: [no-test] -->
- [x] **handle_key() keyboard dispatch** <!-- test: [no-test] -->
- [x] **view_from_key() map 1-5 to ViewMode** <!-- test: [no-test] -->
- [x] **handle_navigation() arrows/tab** <!-- test: [no-test] -->
- [x] **Q key quits app** <!-- test: [no-test] -->
- [x] **R key restores selected file** <!-- test: [no-test] -->
- [x] **Esc key deselects file** <!-- test: [no-test] -->
- [x] **Left/Right arrows navigate timeline** <!-- test: [no-test] -->
- [x] **Tab/BackTab cycles views** <!-- test: [no-test] -->
- [x] **ScrollUp moves timeline backward** <!-- test: [no-test] -->
- [x] **ScrollDown moves timeline forward** <!-- test: [no-test] -->
- [x] **Click file to select** <!-- test: [no-test] -->
- [x] **Click tab to switch view** <!-- test: [no-test] -->
- [x] **Hover highlights element** <!-- test: [no-test] -->

---

## Treemap View ?? CURRENT

- [x] **ViewMode enum (Treemap, Heatmap, Minimap, River, Focus)** <!-- test: [no-test] -->
- [x] **ViewMode::ALL const array** <!-- test: tests/input_tests.rs::test_viewmode_all_array -->
- [x] **ViewMode::name() display string** <!-- test: tests/input_tests.rs::test_viewmode_name -->
- [x] **ViewMode::index() numeric index** <!-- test: tests/input_tests.rs::test_viewmode_index -->
- [x] **ViewMode::from_index() with default fallback** <!-- test: tests/input_tests.rs::test_viewmode_from_index_valid -->
- [x] **ViewMode::next() cycle forward** <!-- test: tests/input_tests.rs::test_viewmode_next_cycles -->
- [x] **ViewMode::prev() cycle backward** <!-- test: tests/input_tests.rs::test_viewmode_prev_cycles -->
- [x] **RenderResult struct (hit_boxes Vec)** <!-- test: [no-test] -->
- [x] **draw() dispatch to current view** <!-- test: [no-test] -->
- [x] **CLR_STABLE constant (60,60,60 gray)** <!-- test: [no-test] -->
- [x] **CLR_GREW constant (80,200,120 green)** <!-- test: [no-test] -->
- [x] **CLR_SHRANK constant (200,200,80 yellow)** <!-- test: [no-test] -->
- [x] **CLR_FUCKED constant (255,80,80 red)** <!-- test: [no-test] -->
- [x] **CLR_NEW constant (80,180,255 blue)** <!-- test: [no-test] -->
- [x] **CLR_HOVER constant (255,0,255 magenta)** <!-- test: [no-test] -->
- [x] **CLR_SELECTED constant (0,255,255 cyan)** <!-- test: [no-test] -->
- [x] **draw() main treemap render with 4-chunk layout** <!-- test: [no-test] -->
- [x] **draw_header() title + view tabs** <!-- test: [no-test] -->
- [x] **draw_timeline() slider with position marker** <!-- test: [no-test] -->
- [x] **draw_treemap_area() file rectangles** <!-- test: [no-test] -->
- [x] **draw_status() bottom hints bar** <!-- test: [no-test] -->
- [x] **health_color() map HealthStatus to Color** <!-- test: [no-test] -->
- [x] **truncate_path() shorten filename for display** <!-- test: tests/treemap_tests.rs::test_truncate_path_short_unchanged -->
- [x] **compute_treemap_layout() sequential strip algorithm** <!-- test: tests/treemap_tests.rs::test_treemap_layout_rects_within_bounds -->
- [x] **File rectangle shows truncated name** <!-- test: [no-test] -->
- [x] **File rectangle shows line count** <!-- test: [no-test] -->
- [x] **Hit boxes registered for files** <!-- test: [no-test] -->
- [x] **Hit boxes registered for view tabs** <!-- test: [no-test] -->
- [x] **Active tab highlighted (black on cyan)** <!-- test: [no-test] -->
- [x] **Tab hover highlight (cyan text)** <!-- test: [no-test] -->
- [x] **Timeline shows ◉ at current position** <!-- test: [no-test] -->
- [x] **Timeline shows N / total** <!-- test: [no-test] -->
- [x] **Timeline shows commit summary (40 chars)** <!-- test: [no-test] -->
- [x] **Files panel shows commit hash** <!-- test: [no-test] -->
- [x] **Treemap limited to 20 files** <!-- test: tests/treemap_tests.rs::test_treemap_layout_respects_20_file_limit -->
- [ ] **Squarified layout algorithm (better aspect ratios)**
- [ ] **Filter binary files (gif/png/jpg/etc)**
- [ ] **Color legend for health status**
- [ ] **Directory grouping/nesting**
- [ ] **Hover tooltip with full path + stats**
- [ ] **Remove 20 file limit (overflow handling)**

---

## Heatmap View

- [ ] **Time (x) vs File (y) grid layout**
- [ ] **Cell intensity = lines changed**
- [ ] **Render intensity as ░▒▓█ characters**
- [ ] **File path labels on y-axis**
- [ ] **Date labels on x-axis**
- [ ] **YOU ARE HERE marker at current commit**
- [ ] **Click row to select file**
- [ ] **Click cell to jump to commit**
- [ ] **Scroll to pan time axis**
- [ ] **Hit boxes for rows and cells**

---

## Minimap View

- [ ] **Three panel layout (before/after/diff)**
- [ ] **BEFORE panel code silhouette**
- [ ] **AFTER panel code silhouette**
- [ ] **DIFF panel with +/- regions**
- [ ] **Pixel density = code density**
- [ ] **Show line counts and delta**
- [ ] **SIGNIFICANT SHRINKAGE warning**
- [ ] **Drag to select BEFORE commit**
- [ ] **Drag to select AFTER commit**
- [ ] **R key restores BEFORE version**

---

## River View

- [ ] **Stacked stream chart layout**
- [ ] **Each file as colored stream area**
- [ ] **Y-axis shows total lines**
- [ ] **X-axis shows time/commits**
- [ ] **Stream appears when file created**
- [ ] **Stream ends when file deleted**
- [ ] **Visualize file splits**
- [ ] **Hover shows file name at point**
- [ ] **Click jumps to commit**
- [ ] **Scroll to zoom time axis**
- [ ] **YOU ARE HERE vertical line**

---

## Focus View

- [ ] **Deep dive on selected file**
- [ ] **Sparkline showing size over N commits**
- [ ] **List of commits touching this file**
- [ ] **Up/Down to select commit in list**
- [ ] **Preview file content at selected commit**
- [ ] **Minimap thumbnail for each commit**
- [ ] **R key restores from selected commit**
- [ ] **Warning flags on suspicious commits**
- [ ] **Enter key opens full preview**

---

## Polish & QoL

- [ ] **Filter files by path pattern**
- [ ] **Filter files by extension**
- [ ] **Search commits by message**
- [ ] **Branch selection**
- [ ] **? key shows help overlay**
- [ ] **Config file support**

---

