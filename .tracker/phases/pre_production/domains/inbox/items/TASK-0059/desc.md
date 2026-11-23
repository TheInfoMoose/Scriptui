## Problem Description
In crates/scriptui-tui/tests/tui_integration.rs:629-632, Alt+R toggles use_regex on literal query "foo" so match count stays 2 before and after. Use a regex pattern with metacharacters to verify regex matching runs through the find bar.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
