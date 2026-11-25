## Problem Description
In crates/scriptui-core/src/markdown.rs:541-561, collect_reference_link_def_skip_ranges skips paragraph-continuation lines matching [label]: dest even though CommonMark forbids link ref defs from interrupting paragraphs. In markdown.rs:307-349, NodeValue::Image does not advance autolink_cursor, so a subsequent autolink with the same URL in the same paragraph resolves to the image URL byte range.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
