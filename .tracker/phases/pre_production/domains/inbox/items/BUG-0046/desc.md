## Problem Description
In crates/scriptui-tui/src/find_bar.rs:77,98 and crates/scriptui-tui/src/palette.rs:256,270, find/replace pattern and replacement strings, palette query, and dynamic palette entry labels (theme/locale names from disk) are passed directly to Span::styled without sanitize_control_chars, allowing raw C0/C1 control sequences into terminal cells.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
