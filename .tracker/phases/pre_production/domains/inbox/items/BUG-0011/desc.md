## Problem Description
`STEP 6C` updated `compute_wrapped_layout` (`crates/scriptui-core/src/layout.rs`) to wrap using Unicode display cell widths and tab-stop expansion, but `render_editor_viewport` (`crates/scriptui-tui/src/viewport.rs:291-310`) still renders 1 `char` per terminal cell (without expanding `\t` to spaces or skipping trailing continuation cells for width-2 CJK/emoji chars), and `WrappedLayout::logical_to_visual` / `visual_to_logical` (`crates/scriptui-core/src/layout.rs:30-65`) + mouse/caret column mapping still use char offsets rather than display-cell columns. Lines containing tabs or wide Unicode characters therefore have misaligned rendering, cursor placement, and mouse click/drag targeting.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
