## Problem Description
buffer.rs:366 unconditionally sets dirty = true in apply_raw_edit (called by UndoStack::undo at history.rs:59), and buffer.rs:279 sets dirty = true before checking empty inserts. Track saved_revision or clean-point distance. From Phase 11 audit (BLK-04).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
