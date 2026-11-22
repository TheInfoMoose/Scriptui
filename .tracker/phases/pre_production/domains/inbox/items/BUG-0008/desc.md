## Problem Description
In `crates/scriptui-tui/src/app.rs:124-128` and `168-173`, both `handle_key_event` and `render_frame` invoke `discover_installed_locales` (`std::fs::read_dir` + `is_file()`) and rebuild `Vec<PaletteEntry>` on every single keystroke while `CommandPalette` is open (twice per keystroke). Fix: cache `palette_entries: Vec<PaletteEntry>` on `TuiApp` and refresh only when opening `OverlayMode::CommandPalette`.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
