# git-trek Roadmap

Visual git time travel & file recovery

---

## Core Infrastructure ?? CURRENT

- [x] **main() entry point with error handling**
- [x] **setup_terminal() raw mode + mouse capture**
- [x] **restore_terminal() cleanup on exit**
- [x] **run_app() main event loop with 50ms poll**
- [x] **App struct (repo, data, view, commit_idx, selected_file, mouse, should_quit, message)**
- [x] **App::new() initialize from CLI args**
- [x] **Cli struct (limit, dry_run)**
- [x] **Cli::parse_checked() with validation**
- [x] **--limit flag for commit count**
- [x] **--dry-run flag for testing**
- [x] **Post-exit message display**
- [x] **App::current_commit_label() format hash**
- [x] **App::files_at_current_commit() sorted by size**
- [x] **App::file_health() get status for path**
- [x] **App::handle_click() dispatch File/ViewTab**
- [x] **App::scroll_timeline() with bounds checking**
- [x] **App::set_view()**
- [x] **App::next_view()**
- [x] **App::prev_view()**
- [x] **App::restore_selected() restore with message**

---

## Data Model ?? CURRENT

- [x] **HealthStatus enum (Stable, Grew, Shrank, MaybeFucked, New, Deleted)**
- [x] **HealthStatus::from_size_change() old/new comparison**
- [x] **HealthStatus::from_ratio() with 0.7/0.95/1.05 thresholds**
- [x] **FileSnapshot struct (lines, bytes)**
- [x] **TrackedFile struct (path, history HashMap)**
- [x] **TrackedFile::new()**
- [x] **TrackedFile::lines_at() get lines at commit index**
- [x] **TrackedFile::health_at() compare to previous commit**
- [x] **CommitInfo struct (oid, summary, author, timestamp, files_changed, insertions, deletions)**
- [x] **RepoData struct (commits Vec, files HashMap)**
- [x] **RepoData::new()**

---

## Git Integration ?? CURRENT

- [x] **load_repo_data() main loader function**
- [x] **collect_commit_oids() revwalk with TOPOLOGICAL|TIME sort**
- [x] **build_commit_info() extract commit metadata**
- [x] **get_diff_stats() insertions/deletions count**
- [x] **collect_file_snapshots() tree walk for files**
- [x] **process_blob_entry() count newlines in blob**
- [x] **format_oid() 8-char hash display**
- [x] **get_file_content() retrieve file at commit**
- [x] **restore_file() write file to disk**

---

## Input Handling ?? CURRENT

- [x] **HitBox struct (rect, id)**
- [x] **HitId enum (File, ViewTab, None)**
- [x] **MouseState struct (x, y, hover)**
- [x] **MouseState::update_position()**
- [x] **MouseState::set_hover()**
- [x] **hit_test() find element under cursor**
- [x] **handle_mouse() dispatch Moved/Down/Scroll**
- [x] **handle_key() keyboard dispatch**
- [x] **view_from_key() map 1-5 to ViewMode**
- [x] **handle_navigation() arrows/tab**
- [x] **Q key quits app**
- [x] **R key restores selected file**
- [x] **Esc key deselects file**
- [x] **Left/Right arrows navigate timeline**
- [x] **Tab/BackTab cycles views**
- [x] **ScrollUp moves timeline backward**
- [x] **ScrollDown moves timeline forward**
- [x] **Click file to select**
- [x] **Click tab to switch view**
- [x] **Hover highlights element**

---

## Treemap View ?? CURRENT

- [x] **ViewMode enum (Treemap, Heatmap, Minimap, River, Focus)**
- [x] **ViewMode::ALL const array**
- [x] **ViewMode::name() display string**
- [x] **ViewMode::index() numeric index**
- [x] **ViewMode::from_index() with default fallback**
- [x] **ViewMode::next() cycle forward**
- [x] **ViewMode::prev() cycle backward**
- [x] **RenderResult struct (hit_boxes Vec)**
- [x] **draw() dispatch to current view**
- [x] **CLR_STABLE constant (60,60,60 gray)**
- [x] **CLR_GREW constant (80,200,120 green)**
- [x] **CLR_SHRANK constant (200,200,80 yellow)**
- [x] **CLR_FUCKED constant (255,80,80 red)**
- [x] **CLR_NEW constant (80,180,255 blue)**
- [x] **CLR_HOVER constant (255,0,255 magenta)**
- [x] **CLR_SELECTED constant (0,255,255 cyan)**
- [x] **draw() main treemap render with 4-chunk layout**
- [x] **draw_header() title + view tabs**
- [x] **draw_timeline() slider with position marker**
- [x] **draw_treemap_area() file rectangles**
- [x] **draw_status() bottom hints bar**
- [x] **health_color() map HealthStatus to Color**
- [x] **truncate_path() shorten filename for display**
- [x] **compute_treemap_layout() sequential strip algorithm**
- [x] **File rectangle shows truncated name**
- [x] **File rectangle shows line count**
- [x] **Hit boxes registered for files**
- [x] **Hit boxes registered for view tabs**
- [x] **Active tab highlighted (black on cyan)**
- [x] **Tab hover highlight (cyan text)**
- [x] **Timeline shows ◉ at current position**
- [x] **Timeline shows N / total**
- [x] **Timeline shows commit summary (40 chars)**
- [x] **Files panel shows commit hash**
- [x] **Treemap limited to 20 files**
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

