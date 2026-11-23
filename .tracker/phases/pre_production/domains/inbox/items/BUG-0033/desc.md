## Problem Description
chrome.rs:39-64 truncates tabs without scrolling/indicators; modals.rs/find_bar.rs/palette.rs lack cursor/scroll editing; editor.rs:662-665 requires pre-populated Find query for Ctrl+Shift+L. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
