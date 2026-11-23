## Problem Description
chrome.rs/preview_pane.rs/modals.rs pass untrusted strings directly to Span::styled; viewport.rs:295-374 drops combining marks and mis-renders straddling wide cursors; buffer.rs:302-309 splits CRLF on single-char delete. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
