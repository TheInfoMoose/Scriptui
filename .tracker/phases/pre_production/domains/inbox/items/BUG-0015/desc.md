## Problem Description
crates/scriptui-core/src/preview.rs:371-389: render_wrapped_tokens advances source_line per line break (base_source_line + break_idx), then line 377 emits the H1/H2 underline rule using the original source_line, producing non-monotonic PreviewLine.source_line values (e.g., 0, 1, 0) on multi-line Setext headings. Pre-existing, amplified by STEP 9A SoftBreak->HardBreak.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
