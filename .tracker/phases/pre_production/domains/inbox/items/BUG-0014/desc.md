## Problem Description
crates/scriptui-core/src/preview.rs:859 emits InlineToken::HardBreak for NodeValue::SoftBreak unconditionally (per STEP 9A). Hard-wrapped Markdown paragraphs no longer reflow when preview width differs from source wrap width (ragged wrapping in narrow split panes). Consider a config option or preserving CommonMark reflow for plain paragraphs.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
