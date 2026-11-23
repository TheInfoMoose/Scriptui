## Problem Description
app.rs:120-126 passes visual scroll_line as logical line in Expanded mode; preview.rs:163,680,688-704 uses chars().count() instead of unicode_width and skips shrinking narrow tables. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
