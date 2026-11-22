## Problem Description
In `crates/scriptui-tui/src/preview_pane.rs:60-64` and `crates/scriptui-tui/src/app.rs:253`, unsynced preview scroll clamps `scroll_line` to `total_lines.saturating_sub(1)` instead of `total_lines.saturating_sub(viewport_height.max(1))`, allowing the user to scroll down until only 1 line is visible at the top and the rest of the preview pane is blank. Fix: store `viewport_height: usize` on `PreviewPaneState` and clamp unsynced scroll to `total_lines.saturating_sub(viewport_height.max(1))`.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
