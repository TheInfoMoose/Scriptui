## Problem Description
In crates/scriptui-core/src/editor.rs:844-905, handle_insert_newline branches solely on the primary cursor line's EnterOutcome. With a primary cursor on an empty list marker ('- ') and a secondary cursor on normal text ('hello'), Enter clears '- ' but inserts no newline at 'hello'; in the reverse order, Enter inserts '\n' everywhere without clearing '- '.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
