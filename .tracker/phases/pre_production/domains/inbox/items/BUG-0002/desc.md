## Problem Description
In `crates/scriptui-core/src/editor.rs:565-585`, `EditorState::handle_indent` mutates line prefixes via `indent_lines`/`dedent_lines` without shifting `active.document.selection()`, leaving `selection_after` at pre-edit offsets. Additionally, when a multi-line selection ends at column 0 of a trailing line (`Shift+Down`), `end_line` erroneously includes that trailing line.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
