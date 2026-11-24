## Problem Description
In crates/scriptui-tui/src/find_bar.rs:108-116, 204-212, 232, handle_find_bar_key only passes through Ctrl+F/H/Q/S and F3/Shift+F3 and swallows all other keys (including Ctrl+PageDown/PageUp, Ctrl+W, Ctrl+N, Ctrl+O, Ctrl+Shift+S, Ctrl+Shift+P, F1). This regresses global shortcuts that worked at aedcc05 while Find/Replace was open and makes the keyboard NextBuffer/PrevBuffer/CloseBuffer/NewBuffer find-bar reset branch in app.rs:348-360 unreachable.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
