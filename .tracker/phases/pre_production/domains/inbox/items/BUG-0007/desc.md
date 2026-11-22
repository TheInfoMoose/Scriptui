## Problem Description
In `crates/scriptui-tui/src/find_bar.rs:132-136`, the `Alt+Enter` branch closes `state.overlay = OverlayMode::None` without resetting `bar_state.focus = FindFocus::Query` (unlike `Esc` at line 120). Re-opening `OverlayMode::Replace` later starts with focus stuck on `Replacement`. Fix: reset `bar_state.focus = FindFocus::Query` on `Alt+Enter`.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
