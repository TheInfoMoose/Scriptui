## Problem Description
In `crates/scriptui-core/src/editor.rs:484-510`, `EditorState::jump_to_match` only calls `self.update_find_matches()` when `self.find_replace.matches.is_empty()`. If the user edits the buffer, switches tabs (`NextBuffer`/`PrevBuffer`/`CloseBuffer`), or updates `find_replace.query` without clearing `matches`, `Action::FindNext`/`Action::FindPrev` jumps using stale character offsets from the previous buffer/text. Fix by recomputing `self.update_find_matches()?` in `jump_to_match` and clearing `matches` on buffer switches.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
