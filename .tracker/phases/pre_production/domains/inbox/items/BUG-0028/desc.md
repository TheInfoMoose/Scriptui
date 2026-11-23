## Problem Description
terminal.rs:98-104 drops SUPER/HYPER/META and app.rs:258-260 only checks CONTROL|ALT, so Super+S inserts 's' and Super+Left moves cursor. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
