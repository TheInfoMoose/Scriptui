## Problem Description
crates/scriptui-tui/src/modals.rs:277-280 (handle_file_prompt_key) and crates/scriptui-tui/src/palette.rs:320-323 (handle_palette_key) only exclude CONTROL | ALT, so Super/Hyper/Meta char keys type into filenames/queries. crates/scriptui-tui/src/modals.rs:207-214 (handle_confirm_modal_key) checks no modifiers at all, so Super+Y/Ctrl+Y confirms quit/close.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
