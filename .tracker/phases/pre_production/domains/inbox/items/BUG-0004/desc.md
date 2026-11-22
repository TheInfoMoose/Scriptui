## Problem Description
In `crates/scriptui-tui/src/app.rs:161-238`, overlay key handlers return `Ok(false)`/`false` for unrecognized keys, and `TuiApp::handle_key_event` only returns early on `true`. Pressing unhandled keys while a blocking floating modal (`ConfirmQuit`, `ConfirmCloseBuffer`, `SpellQuickFix`, `CommandPalette`, `OpenFilePrompt`, `SaveAsPrompt`) is open falls through to the editor and mutates the buffer behind the modal (and invalidates `spell_popup.diagnostic` offsets). Fix: unconditionally trap/return early after dispatching to a blocking floating modal handler.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
