## Problem Description
buffer.rs:101-110 and spell.rs:74-82 use File::create on predictable .scriptui.tmp.<pid> paths (no O_EXCL); spell.rs:386-393 writes locale files non-atomically. From Phase 11 audit (BLK-08b).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
