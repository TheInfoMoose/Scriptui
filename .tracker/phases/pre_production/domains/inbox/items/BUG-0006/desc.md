## Problem Description
In `crates/scriptui-core/src/editor.rs:383-420`, `EditorState::replace_active_match` calls `self.update_find_matches()?`, which unconditionally sets `active_match = Some(0)`. Replacing match 2/3 jumps back to 1/2, and when the replacement contains the search pattern (e.g., replacing "foo" with "foobar"), repeated Enter presses infinitely re-replace match 0. Fix: after refreshing matches, set `active_match` to the first match with `m.start >= new_caret` (wrapping to `0` if none).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
