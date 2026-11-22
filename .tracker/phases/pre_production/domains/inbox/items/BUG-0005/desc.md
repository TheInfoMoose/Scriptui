## Problem Description
In `crates/scriptui-tui/src/app.rs:242-289`, `TuiApp::handle_mouse_event` does not check `self.state.overlay` before routing events to `MouseRouter::handle_mouse`. Clicking tabs or text while a floating modal (`SpellQuickFix`, `ConfirmQuit`, `ConfirmCloseBuffer`, `CommandPalette`, `OpenFilePrompt`, `SaveAsPrompt`) is open switches buffers or moves the cursor behind the modal; switching tabs while `SpellQuickFix` is open causes `Enter` to apply the spell fix to the wrong buffer at stale offsets. Fix: ignore mouse events in `TuiApp::handle_mouse_event` when a floating modal is active.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
