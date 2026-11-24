## Problem Description
DocumentBuffer::apply_insert("") on a point selection returns a non-empty Vec containing a no-op BufferEdit { start, end, inserted: "", removed: "" }. EditorState::insert_text passes this to UndoStack::record, which only checks tx.edits.is_empty(), pushing a no-op transaction and desyncing UndoStack::is_at_saved_state() / dirty tracking after subsequent edits and Undo.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
