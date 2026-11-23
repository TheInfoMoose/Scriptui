## Problem Description
theme.rs:315-320 and spell.rs:163 check is_file() without canonical containment; buffer.rs:63 and terminal.rs:137-146 accept unbounded file/OSC 52 sizes. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
