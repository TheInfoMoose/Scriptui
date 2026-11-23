## Problem Description
editor.rs:793-966 collapses multi-cursors in handle_indent, move_primary_selection, and ClearCurrentLine; selection.rs:64-71,114-143 does not remap primary index on sort/merge. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
