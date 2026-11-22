## Problem Description
In `crates/scriptui-core/src/layout.rs:81-143`, `compute_wrapped_layout` counts `chars.len()` instead of terminal display width (`unicode-width`) and does not expand hard tabs (`\t`) to `tab_width` columns during wrapping. ASCII/space-indented prose is exact, but CJK/emoji wide glyphs and tab-indented lines will drift from terminal cell columns.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
