## Problem Description
Keymap::default_cua() binds Ctrl+V to Action::Paste (keymap.rs:356-358) and README.md:137 documents it, but EditorState::dispatch executes Action::Copy | Action::Paste => {} (editor.rs:732). From Phase 11 audit (BLK-03).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
