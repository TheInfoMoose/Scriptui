## Problem Description
STEP 4E spec gap: MUTATION_SPEC mandates an 8-parameter signature `pub fn ensure_cursor_visible(scroll_line: &mut usize, scroll_col: &mut usize, cursor_visual_row: usize, cursor_visual_col: usize, viewport_height: usize, viewport_width: usize, total_visual_lines: usize, soft_wrap: bool)` vs POST_GATE running `cargo clippy --workspace --all-targets -- -D warnings`, which rejects >7 arguments (`clippy::too_many_arguments` 8/7) unless the signature is refactored into a parameter struct/tuple or `#[allow(clippy::too_many_arguments)]` is explicitly permitted. Location: `crates/scriptui-tui/src/viewport.rs:28`.
Lapse: L4 other
Kind: scope-lapse
Found during: STEP 4E (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
