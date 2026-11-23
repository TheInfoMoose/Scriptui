## Problem Description
crates/scriptui-tui/src/app.rs:177-179 returns early when key.kind != KeyEventKind::Press, dropping held key repeats on terminals emitting Repeat events. From Phase 11 audit (BLK-06).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
