## Problem Description
crates/scriptui-core/src/spell.rs:205-235: ignore_word and PersonalDictionary::add store raw strings while is_correct only normalizes U+2019 to ASCII apostrophe on lookup. Ignoring or adding a curly-apostrophe word (e.g. foo\u2019s) does not suppress the straight-apostrophe form (foo's).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
