## Problem Description
crates/scriptui-core/src/indent.rs:83 slices &trimmed[1..6] without checking is_char_boundary(6), panicking on Enter when a multi-byte UTF-8 char straddles byte index 6 (e.g. '-    é'). From Phase 11 audit (BLK-01).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
