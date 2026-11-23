## Problem Description
crates/scriptui-core/src/spell.rs:237-243,500-517: suggest() normalizes U+2019 to ASCII apostrophe before querying spellbook, but no unit test exercises suggest() with a curly-apostrophe misspelling.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
