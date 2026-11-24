## Problem Description
In crates/scriptui-tui/src/find_bar.rs:117-239, replacing is_passthrough_shortcut with catch-all '_ => Ok(false)' causes TuiApp::handle_key_event (app.rs:270-364) to fall through and execute document-mutating keymap actions (Ctrl+Z Undo, Ctrl+Y/Ctrl+Shift+Z Redo, Ctrl+X Cut, Ctrl+C Copy, Ctrl+V Paste, Ctrl+A SelectAll, Ctrl+Shift+L SelectAllOccurrences, Ctrl+. OpenSpellQuickFix) while Find/Replace has focus. Regression from Phase 14 (STEP 14A, plan origin). Pass through only non-editing global/window/view/buffer/search shortcuts (or filter actions in app.rs) while trapping document-editing shortcuts.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
