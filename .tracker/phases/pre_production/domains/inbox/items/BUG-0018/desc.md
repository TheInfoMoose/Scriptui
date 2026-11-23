## Problem Description
crates/scriptui-core/src/editor.rs:246-254 hardcodes resolve_theme(&config.theme, &[]) and SpellEngine::default_en_us().ok(); crates/scriptui-tui/src/app.rs:68-76 swallows config.toml errors and never resolves custom theme/dict dirs; palette.rs:77-83 omits custom themes. From Phase 11 audit (BLK-02).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
