# project -- Semantic Map

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

## Layer 0 -- Config

`Cargo.toml`
Rust package manifest and dependencies. Centralizes project configuration.

`slopchop.toml`
Configuration for slopchop. Centralizes project configuration.

## Layer 1 -- Core

`src/cli.rs`
Maximum number of commits to load. Defines command-line interface.
→ Exports: Cli, parse_args

`src/lib.rs`
Library root and public exports. Provides application entry point.

`src/main.rs`
Orchestrates `anyhow`, `crate`, `crossterm`. Provides application entry point.

`src/views/mod.rs`
Result of rendering a view, containing hit boxes for mouse interaction. Supports application functionality.
→ Exports: Render, ViewMode, draw, from_index, index, name, new, next, prev

## Layer 2 -- Domain

`src/app.rs`
Module providing `App`, `clear_selection`, `commit_count`. Supports application functionality.
→ Exports: App, clear_selection, commit_count, commit_idx, commit_label, current_commit, file_health, files_at_current, handle_click, message, mouse, mouse_mut, new, next_view, prev_view, quit, restore_selected, scroll_timeline, selected_file, set_view, should_quit, view

`src/data.rs`
Health status of a file based on change magnitude. Supports application functionality.
→ Exports: Commit, FileHistory, Health, History, Snapshot, files_at_commit, from_change, health_at, lines_at, new

`src/error.rs`
Module providing `TrekError`. Defines error types and handling.
→ Exports: TrekError

`src/git_ops.rs`
Find and open the git repository. Supports application functionality.
→ Exports: find_repository, get_file_content, load_history, restore_file

`src/mouse.rs`
Identifies a clickable element. Supports application functionality.
→ Exports: HitBox, HitTarget, MouseState, contains, hit_test, new, update_hover, update_position

`src/views/constellation.rs`
Placeholder file. Supports application functionality.

`src/views/flow.rs`
Placeholder file. Supports application functionality.

`src/views/seismic.rs`
Placeholder file. Supports application functionality.

`src/views/strata.rs`
Placeholder file. Supports application functionality.

`src/views/surgery.rs`
Placeholder file. Supports application functionality.

`src/views/terrain.rs`
Module providing `draw`. Supports application functionality.
→ Exports: draw

`src/views/terrain/layout.rs`
Compute treemap layout for files in a given area. Supports application functionality.
→ Exports: compute

