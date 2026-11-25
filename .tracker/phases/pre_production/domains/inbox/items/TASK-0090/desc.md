## Problem Description
In crates/scriptui-core/src/spell.rs:763-770, the test loops over diags with assert_eq!(diag.word, "utf8") without asserting diags.len() == 1 (or !diags.is_empty()), so it passes vacuously if check_document returns empty.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
