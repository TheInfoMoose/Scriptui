## Problem Description
In crates/scriptui-core/src/markdown.rs:376-393 and 525-539, is_closing_fence rejects lines with >3 leading spaces or blockquote '>' prefixes. Closed fenced code blocks inside blockquotes or >=4-space list items therefore set CodeBlockRegion.end_line = span.end_line instead of span.end_line - 1, regressing closed fence exclusion.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
