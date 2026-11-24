## Problem Description
crates/scriptui-core/src/buffer.rs:345 and 405 rebuild self.selection with SelectionSet::from_ranges(new_ranges), discarding self.selection.primary_index(). Typing or deleting with a multi-cursor SelectionSet whose primary_index > 0 silently resets primary to index 0.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
