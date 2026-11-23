# ScripTUI Functional Verification Matrix (Phase 10)

Maps every user-facing behavior documented in `README.md` to automated test coverage (existing + Phase 10 steps `10A`–`10C`) and the `[USER]` real-terminal checklist for Linux desktop terminals (resolved OQ-3).

## 1. CLI Options, Subcommands & Configuration

| README Feature | Automated Test | Crate / File | Real-Terminal `[USER]` Check |
| :--- | :--- | :--- | :---: |
| `scriptui --version` (`CARGO_PKG_VERSION`) | `cli_version_matches_workspace_version`, `cli_binary_version_help_and_dict_import_with_xdg_config_home` (10C) | `scriptui-tui/tests/tui_integration.rs` | — |
| `scriptui --help` | `cli_binary_version_help_and_dict_import_with_xdg_config_home` (10C) | `scriptui-tui/tests/tui_integration.rs` | — |
| `--theme <NAME>` (overrides `config.toml`) | `cli_parses_files_theme_and_locale`, `all_five_builtin_palettes_render_with_distinct_colors_and_crt_amber_gfm_differentials`, `cli_startup_opens_multiple_files_and_overrides_config` (10C) | `scriptui-tui/src/cli.rs`, `tests/tui_integration.rs` | — |
| `--locale <LOCALE>` (overrides `config.toml`) | `cli_parses_files_theme_and_locale`, `cli_startup_opens_multiple_files_and_overrides_config` (10C) | `scriptui-tui/src/cli.rs`, `tests/tui_integration.rs` | — |
| `[FILE...]` multiple files opened in tabs | `cli_startup_opens_multiple_files_and_overrides_config` (10C) | `scriptui-tui/tests/tui_integration.rs` | — |
| `dict import <LOCALE> <DIC> <AFF>` (`XDG_CONFIG_HOME`) | `cli_parses_dict_import_subcommand_and_executes`, `cli_binary_version_help_and_dict_import_with_xdg_config_home` (10C) | `scriptui-tui/src/cli.rs`, `tests/tui_integration.rs` | — |
| `config.toml` defaults & partial override | `config_defaults_and_partial_toml_override`, `config_flags_control_spellcheck_preview_split_and_soft_wrap` (10A) | `scriptui-core/src/config.rs`, `tests/editor_state_pipeline.rs` | — |
| `config.toml` `spell_enabled = false` | `config_flags_control_spellcheck_preview_split_and_soft_wrap` (10A), `cli_startup_opens_multiple_files_and_overrides_config` (10C) | `scriptui-core/tests/editor_state_pipeline.rs`, `scriptui-tui/tests/tui_integration.rs` | — |
| `config.toml` `line_numbers = false` | `config_flags_control_spellcheck_preview_split_and_soft_wrap` (10A), `cli_startup_opens_multiple_files_and_overrides_config` (10C) | `scriptui-core/tests/editor_state_pipeline.rs`, `scriptui-tui/tests/tui_integration.rs` | — |
| `config.toml` `soft_wrap = false` | `no_wrap_mode_preserves_one_visual_line_per_logical_line`, `config_flags_control_spellcheck_preview_split_and_soft_wrap` (10A) | `scriptui-core/src/layout.rs`, `tests/editor_state_pipeline.rs` | — |
| `config.toml` `preview_split_percent` | `config_flags_control_spellcheck_preview_split_and_soft_wrap` (10A) | `scriptui-core/tests/editor_state_pipeline.rs` | — |

## 2. Themes & Hunspell Spellcheck

| README Feature | Automated Test | Crate / File | Real-Terminal `[USER]` Check |
| :--- | :--- | :--- | :---: |
| 5 built-in palettes (`crt_amber`, `phosphor_green`, `ice_cyan`, `vector_magenta`, `high_contrast_mono`) | `all_builtin_themes_parse_and_crt_amber_matches_suite`, `all_five_builtin_palettes_render_with_distinct_colors_and_crt_amber_gfm_differentials` | `scriptui-core/src/theme.rs`, `scriptui-tui/tests/tui_integration.rs` | Check #1 (visual) |
| Custom TOML theme overrides built-in & rejects traversal | `custom_toml_theme_overrides_builtin`, `resolve_theme_rejects_path_traversal`, `custom_toml_theme_overrides_builtin_and_validates_names` (10C) | `scriptui-core/src/theme.rs`, `scriptui-tui/tests/tui_integration.rs` | — |
| Bundled `en_US` spellcheck (contractions, curly apostrophe, suggestions) | `default_en_us_flags_misspelling_and_suggests`, `default_en_us_accepts_contractions_and_standard_english_words`, `default_en_us_suggests_for_curly_apostrophe_misspelling` (10A) | `scriptui-core/src/spell.rs` | — |
| Spellcheck skips code blocks, inline code, frontmatter, URLs | `spell_skips_code_ranges_and_respects_personal_and_ignore`, `spell_skip_ranges_cover_code_frontmatter_and_urls` | `scriptui-core/src/spell.rs`, `src/markdown.rs` | — |
| Custom locale import, discovery & Command Palette switch | `import_and_load_custom_locale_and_reject_traversal`, `custom_hunspell_locale_import_discovery_and_palette_switch` | `scriptui-core/src/spell.rs`, `scriptui-tui/tests/tui_integration.rs` | — |

## 3. Keybindings (CUA Default)

| README Shortcut | Action | Automated Test | Real-Terminal `[USER]` Check |
| :--- | :--- | :--- | :---: |
| `Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S`, `Ctrl+W`, `Ctrl+PageDown`, `Ctrl+PageUp`, `Ctrl+Q` | File, Buffer & App lifecycle | `readme_default_cua_keymap_covers_every_documented_shortcut` (10A), `tui_file_buffer_lifecycle_and_dirty_confirm_modals` (10B) | Check #1 |
| `Enter`, `Backspace`, `Delete`, `Tab`, `Shift+Tab` | Editing & prose-safe indentation | `multicursor_find_handoff_and_prose_safe_indentation_e2e`, `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste` (10B) | — |
| `Ctrl+C`, `Ctrl+X`, `Ctrl+V`, `Ctrl+A` | Copy (OSC 52), Cut (OSC 52), Paste (bracketed paste), Select All | `tui_app_renders_full_shell_and_handles_typing_and_osc52_copy`, `bracketed_paste_routes_to_editor_find_bar_and_ignores_confirm_modal`, `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste` (10B) | Checks #2, #3 |
| `Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, `Esc` | Undo, Redo, Escape overlay / collapse multi-cursor | `readme_default_cua_keymap_covers_every_documented_shortcut` (10A), `editor_navigation_selection_cut_find_prev_and_escape_collapse` (10A), `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste` (10B) | — |
| `Ctrl+F`, `Ctrl+H`, `F3`, `Shift+F3`, `Ctrl+Shift+L` | Find, Replace, Next/Prev, Select All Occurrences | `tui_find_replace_bar_inline_shortcuts_and_f3_navigation` (10B) | — |
| Find/Replace inline: `Alt+C`, `Alt+W`, `Alt+R`, `Alt+Enter`, `Enter`, `Shift+Enter`, `Ctrl+Enter`, `Tab`/`Shift+Tab`, `Esc` | Inline Find/Replace bar controls | `find_bar_typing_toggles_replace_and_alt_enter_multicursor`, `tui_find_replace_bar_inline_shortcuts_and_f3_navigation` (10B) | — |
| `Ctrl+P` / `F1`, `Ctrl+.`, `Ctrl+Shift+P`, `Alt+P`, `Alt+Z` | Command Palette, Spell Quick-Fix, Preview modes, Soft Wrap | `tui_app_overlays_and_styled_preview_integration`, `preview_modes_sync_scroll_and_independent_wheel_scroll`, `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste` (10B) | Check #5 |
| `Left`/`Right`/`Up`/`Down`, `Home`/`End`, `PageUp`/`PageDown`, `Shift+Arrows`/`Shift+Home`/`Shift+End` | Navigation & Selection | `editor_navigation_selection_cut_find_prev_and_escape_collapse` (10A), `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste` (10B) | — |

> **Note on `Ctrl+V`:** `Keymap::default_cua()` maps `Ctrl+V` to `Action::Paste`, while runtime paste enters through terminal bracketed paste (`Event::Paste` -> `TuiApp::handle_paste_event`). Real-terminal paste is verified in Check #2 below.

## 4. Mouse Interactions

| README Mouse Action | Automated Test | Real-Terminal `[USER]` Check |
| :--- | :--- | :---: |
| Left-click & drag in editor | `mouse_click_drag_scroll_tab_and_divider_routing`, `tui_mouse_click_drag_tab_right_click_spell_and_split_divider` (10B) | Check #4 |
| Scroll wheel over editor & preview | `mouse_wheel_scroll_not_snapped_back_on_next_render`, `preview_modes_sync_scroll_and_independent_wheel_scroll` | Check #4 |
| Left-click on tab bar | `mouse_click_drag_scroll_tab_and_divider_routing`, `tui_mouse_click_drag_tab_right_click_spell_and_split_divider` (10B) | Check #4 |
| Right-click in editor (Spellcheck Quick-Fix) | `tui_mouse_click_drag_tab_right_click_spell_and_split_divider` (10B) | Check #4 |
| Left-click on preview edge tab & drag split divider | `mouse_click_drag_scroll_tab_and_divider_routing`, `tui_mouse_click_drag_tab_right_click_spell_and_split_divider` (10B) | Check #4 |

## 5. `[USER]` Real-Terminal Checklist (Linux Desktop Terminal)

Run after Orrery finishes `STEP 10A`–`STEP 10C`, before `/phase-review 10`. Record terminal emulator name and pass/fail (file any failure as a pending `neb` bug).

- **Environment:** Linux — Terminal emulator: `____________________`
- [ ] **Check #1 — Startup, Palette & Raw Mode Restore:** Launch `cargo run -p scriptui-tui -- README.md`. Verify tab bar, gutter, GFM colors, and status bar render cleanly. Press `Ctrl+Q` to exit and confirm shell echo, cursor visibility, and normal terminal mode are restored cleanly.
- [ ] **Check #2 — Bracketed Paste:** Copy multi-line text from another window, paste into the editor (e.g., `Ctrl+Shift+V`), and confirm lines insert verbatim without triggering auto-indent runaway or shortcut collisions. Open Find (`Ctrl+F`) and paste into the Find input.
- [ ] **Check #3 — OSC 52 Clipboard Sync:** Select text (`Shift+Right` or mouse drag), press `Ctrl+C`, and paste into another application (note: requires a terminal with OSC 52 write enabled, e.g., Kitty/Alacritty/WezTerm; record N/A if disabled in terminal settings).
- [ ] **Check #4 — Mouse Interactions:** Left-click to place caret, click-and-drag to select, scroll wheel over editor and over split preview (`Ctrl+Shift+P`), click preview edge tab `◀`, drag the `│` split divider, right-click a misspelled word (`wrld`) to open Spellcheck Quick-Fix.
- [ ] **Check #5 — Live Terminal Resize:** Open split preview (`Ctrl+Shift+P`), resize the terminal window down to ~40×12 and back to wide; confirm no panic, crash, or garbled layout.
