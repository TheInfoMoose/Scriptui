## Problem Description
In crates/scriptui-core/src/editor.rs:863-903, multi-cursor Action::InsertNewline collects (line_start, line_end, '') for ClearCurrentLine cursors and (sel_start, sel_end, insert_text) for Insert cursors without checking for overlaps. When a valid non-overlapping SelectionSet has one cursor on an empty bullet/quote line and another selection starting or ending on that same line (e.g. 'hello\n- ' with [Selection::new(7, 2), Selection::point(8)] or '- \nhello' with [Selection::point(0), Selection::new(1, 5)]), applying planned_edits in reverse via apply_raw_edit either returns Err(OutOfBounds) after a partial unrecorded buffer mutation or silently corrupts adjacent text and caret offsets. New defect from Phase 14 (STEP 14C, coder origin).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
