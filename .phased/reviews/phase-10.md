# Phase 10 review

Range: `cf1a3e0e0803c823d1cb154d955959282cabe18c..bd7fa5c430958524b1cb24c5e77d8017eab1bc7d`

## Independent pass

### Scope traced
All code changes in this range are test-only across 3 files:
- `crates/scriptui-core/src/spell.rs` (1 new unit test in `mod tests`)
- `crates/scriptui-core/tests/editor_state_pipeline.rs` (3 new integration tests)
- `crates/scriptui-tui/tests/tui_integration.rs` (1 modified test assertion, 8 new integration tests)

Verification run: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all passed (98 tests total, 0 failures). No production code changed; no regressions, no loosened existing assertions (`cli_version_matches_workspace_version` now asserts `format!("scriptui {}", env!("CARGO_PKG_VERSION"))` instead of hardcoded `"scriptui 0.1.0"`), and no sensitive data found.

### Findings

1. **`crates/scriptui-tui/tests/tui_integration.rs:1051-1081`** (`tui_narrow_viewport_and_long_lines_render_without_panic`) — **test defect**
   - **Trigger:** Rendering `OverlayMode::CommandPalette` and `OverlayMode::SpellQuickFix` on a `40x12` viewport containing `"a"*97 + " wrld " + "b"*97`.
   - **Effect:** Both overlay assertions can pass even if the overlays render nothing:
     - For `CommandPalette` (lines 1055-1056), `assert!(rect_has_non_empty_cell(buf, frame_state.total_area))` is tautologically satisfied by `assert!(rect_has_non_empty_cell(buf, text_area))` because `text_area` is contained inside `total_area`. It never checks for palette content (e.g., `"Command Palette"` or the modal region).
     - For `SpellQuickFix` (lines 1076-1080), `assert!(rendered.contains("wrld"))` searches the entire frame buffer. Because the document itself contains `"wrld"` and the cursor is placed at char 99 (inside `"wrld"`), `"wrld"` is already rendered in the editor viewport in both `soft_wrap = true` and `soft_wrap = false`; asserting `"Spellcheck: wrld"` (the modal title rendered by `render_spell_quick_fix` in `crates/scriptui-tui/src/modals.rs:31`) would actually verify the popup rendered.

2. **`crates/scriptui-tui/tests/tui_integration.rs:1199-1208`** (`cli_startup_opens_multiple_files_and_overrides_config`) — **test defect**
   - **Trigger:** Passing `locale: Some("en_US".to_string())` to `TuiApp::from_cli(&cli)` and asserting `app.state.config.spell_locale == "en_US"`.
   - **Effect:** `"en_US"` is already `AppConfig::default().spell_locale`. The test would still pass if `TuiApp::from_cli` ignored `cli.locale` completely (whereas `theme: Some("ice_cyan")` genuinely overrides the default `"crt_amber"`).

3. **`crates/scriptui-tui/tests/tui_integration.rs:629-632`** (`tui_find_replace_bar_inline_shortcuts_and_f3_navigation`) — **test defect**
   - **Trigger:** Pressing `Alt+R` while the find query is literal `"foo"` (no regex metacharacters) on `"foo Foo foobar foo"`.
   - **Effect:** `matches.len()` remains `2` before and after `Alt+R`; the test verifies that `query.use_regex` flipped to `true`, but does not verify that regex matching actually runs through the find bar (a regex query such as `f..` would distinguish regex mode from literal mode).

### Not checked
- Interactive live TTY / terminal emulator rendering (all TUI tests run against `ratatui::backend::TestBackend`).
- Non-Linux / Windows path handling for `XDG_CONFIG_HOME` and CLI binary invocation.

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step-by-step contract check
- **STEP 10A (`TASK-0054`, advances `TASK-0053`):** Met. All 4 required tests (`default_en_us_suggests_for_curly_apostrophe_misspelling`, `readme_default_cua_keymap_covers_every_documented_shortcut`, `config_flags_control_spellcheck_preview_split_and_soft_wrap`, `editor_navigation_selection_cut_find_prev_and_escape_collapse`) are present, pass, and respect `ALLOWED_FILES` with no production changes. `TASK-0053` is ready to close.
- **STEP 10B (`TASK-0055`):** Partially met (validator let through 1 contract deviation and 1 minor test weakness).
  - `tui_file_buffer_lifecycle_and_dirty_confirm_modals`, `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste`, and `tui_mouse_click_drag_tab_right_click_spell_and_split_divider` meet contract.
  - `tui_narrow_viewport_and_long_lines_render_without_panic` deviates from contract item 1e ("non-empty rendered cells in the viewport and overlay regions"): `CommandPalette` asserts `total_area` (tautological with `text_area`) and `SpellQuickFix` asserts `"wrld"` across the full buffer (already visible in the editor viewport) rather than overlay-specific cells/title (`TASK-0057`).
  - `tui_find_replace_bar_inline_shortcuts_and_f3_navigation` toggles `Alt+R` on literal `"foo"`, verifying the flag but not regex metacharacter matching (`TASK-0059`).
- **STEP 10C (`TASK-0056`, advances `TASK-0048`):** Met as contracted. `cli_version_matches_workspace_version` now uses `env!("CARGO_PKG_VERSION")` (`TASK-0048` ready to close), `cli_binary_version_help_and_dict_import_with_xdg_config_home` uses `Command::env` (no `set_var`), `custom_toml_theme_overrides_builtin_and_validates_names` passes, and `cli_startup_opens_multiple_files_and_overrides_config` matches the step contract (though the contract itself specified default `"en_US"` for `locale`, see Finding 2 / `TASK-0058`).
- **Deferrals / Amendments:** None. All 3 steps completed without deferral or amendment.

### Finding origins & filed items
| # | Location | Class | Origin | Filed |
| :-: | :--- | :--- | :--- | :--- |
| 1 | `crates/scriptui-tui/tests/tui_integration.rs:1051-1081` | test defect | `coder` | `TASK-0057` (`pending`, low) |
| 2 | `crates/scriptui-tui/tests/tui_integration.rs:1199-1208` | test defect | `plan` | `TASK-0058` (`pending`, low) |
| 3 | `crates/scriptui-tui/tests/tui_integration.rs:629-632` | test defect | `coder` | `TASK-0059` (`pending`, low) |

## Handoff

- **Verdict:** PASS WITH NOTES — all 3 steps (`10A`, `10B`, `10C`) landed test-only with zero regressions and `fmt`/`clippy`/`test` green (98 tests). Three low-severity test-assertion weaknesses were filed as pending tasks.
- **Pending items filed (by priority):**
  - Low: `TASK-0057` (narrow-viewport `CommandPalette` / `SpellQuickFix` overlay assertions), `TASK-0058` (non-default `--locale` override in CLI startup test), `TASK-0059` (regex metacharacters after `Alt+R` in find bar test).
- **Advances items ready to close (pending user approval):** `TASK-0048` (dynamic `CARGO_PKG_VERSION` in version test), `TASK-0053` (curly-apostrophe `SpellEngine::suggest` unit test).
- **Deferred steps to re-split:** None.
- **User decisions / follow-up:**
  - Confirm closing `TASK-0048` and `TASK-0053`.
  - Complete the `[USER]` Linux desktop terminal checklist in `docs/functional-verification-matrix.md` (`STEP 10D`) before or alongside Phase 11 (multiagent audit); `TASK-0057`..`TASK-0059` can be picked in Phase 12 with other audit fixes.
