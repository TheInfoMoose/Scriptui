## Problem Description
preview_pane.rs:56 calls build_active_preview every frame and highlight.rs:87 calls SyntaxSet::load_defaults_newlines() on every build. Cache SyntaxSet via OnceLock and memoize PreviewDocument by (doc_id, revision, theme, width). From Phase 11 audit (BLK-07).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
