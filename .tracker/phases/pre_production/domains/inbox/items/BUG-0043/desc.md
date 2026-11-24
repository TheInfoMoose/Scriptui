## Problem Description
In crates/scriptui-core/src/editor.rs:877-898, overlapped_by_other checks other_start < line_end && other_end > line_start without requiring !other.is_empty(). When two or more point cursors sit on the same empty continuation line and at least one is strictly inside (line_start, line_end), it falsely triggers overlap: boundary+interior cursors asymmetrically insert a newline and clear the line (inverting cursor order when one is at line_start), and two interior cursors both insert newlines and split the marker instead of clearing the line. Guard with !other.is_empty() and add a regression test.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
