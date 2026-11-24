# ScripTUI Whole-Codebase Production & Release Readiness Audit Report

**Repository:** `ScripTUI` (crate workspace `scriptui-core` + `scriptui-tui` `v0.1.0`)  
**Audit Date:** 2026-11-23  
**Audit Mode:** Read-only whole-codebase inspection and non-modifying static/test verification  

---

## 1. Overall Production & Release Readiness Assessment

### Executive Summary

ScripTUI has a well-structured two-crate Rust workspace (`scriptui-core` + `scriptui-tui`) with strong static baseline hygiene: **zero `unsafe` blocks**, **zero non-test `.unwrap()` / `.expect()` / `panic!` / `todo!()` / `unimplemented!()` calls**, **zero `#[allow(...)]` suppressions**, clean `0×0`–`2×2` terminal-geometry clamping, and **98/98 passing automated tests** under `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`.

Despite green automated gates, **the codebase is NOT ready for a `v0.1.0` production release (NO-GO)**. Direct source inspection uncovered a runtime UTF-8 slice-indexing panic on valid Markdown input, broken startup resolution for `--locale` / `--theme` / `config.toml` custom assets, an unimplemented `Ctrl+V` (`Action::Paste`) binding advertised in `README.md`, missing OS signal restoration (`SIGTERM` / `SIGHUP`), per-frame `syntect::parsing::SyntaxSet` reconstruction in the preview path, unversioned path-dependency metadata blocking `cargo publish` and `cargo-deny`, missing third-party licensing attribution for bundled Hunspell assets (`en_US.{aff,dic}`) and `MPL-2.0` (`spellbook`), and absent CI/release automation and real-terminal verification.

### Go / No-Go Verdict

**Verdict: NO-GO for `v0.1.0` production release** until the **Critical** and **High** release blockers below are resolved and verified.

### Release Blockers Summary Table

| ID | Severity | Dimension | Summary & Primary Location |
| :--- | :--- | :--- | :--- |
| **BLK-01** | **Critical** | Correctness / Security | Unchecked UTF-8 slice `&trimmed[1..6]` panics on `Enter` in `parse_task_list` (`crates/scriptui-core/src/indent.rs:83`). |
| **BLK-02** | **High** | Correctness / Scope Lapse | Startup ignores `config.spell_locale` (`--locale`) and custom theme search dirs (`--theme` / `config.toml`) (`crates/scriptui-core/src/editor.rs:245-254`, `crates/scriptui-tui/src/app.rs:66-79`). |
| **BLK-03** | **High** | Correctness / Scope Lapse | `Ctrl+V` (`Action::Paste`) is bound and documented (`README.md:137`) but executes a no-op (`crates/scriptui-core/src/editor.rs:732`). |
| **BLK-04** | **High** | Correctness | `DocumentBuffer::dirty` never resets on `Undo` back to saved state (`crates/scriptui-core/src/buffer.rs:279, 366`, `history.rs:52-65`). |
| **BLK-05** | **High** | Correctness | Opening a non-existent CLI file (`scriptui new.md`) fails to launch (`crates/scriptui-tui/src/app.rs:77-79`, `crates/scriptui-core/src/buffers.rs:121`, `buffer.rs:62-66`). |
| **BLK-06** | **High** | Correctness | `TuiApp::handle_key_event` drops `KeyEventKind::Repeat` events (`crates/scriptui-tui/src/app.rs:177-179`). |
| **BLK-07** | **High** | Architecture / Performance | Synchronous full-document preview rebuild including `SyntaxSet::load_defaults_newlines()` on every rendered frame (`crates/scriptui-tui/src/preview_pane.rs:56`, `crates/scriptui-core/src/preview.rs:58`). |
| **BLK-08** | **High** | Security / Safety | No OS signal handling (`SIGTERM`, `SIGHUP`, `SIGTSTP`/`SIGCONT`), leaving PTY in raw/mouse mode; predictable temp paths without `O_EXCL` in `save_atomic` / `save_to_file` (`crates/scriptui-tui/src/terminal.rs:19-75`, `crates/scriptui-core/src/buffer.rs:101-110`, `spell.rs:74-82`). |
| **BLK-09** | **High** | Packaging / Licensing | Unversioned `scriptui-core` path dependency blocks `cargo publish` & `cargo-deny` (`crates/scriptui-tui/Cargo.toml:16`); missing third-party license notice for `assets/dicts/en_US.{aff,dic}` and `spellbook` (`MPL-2.0`). |
| **BLK-10** | **High** | Tooling / Verification | Missing `deny.toml` (with 3 unmaintained transitive RustSec advisories), missing `.github/workflows/` CI/release pipelines, zero tests for `TerminalGuard`/`run_event_loop`, and 5 unchecked `[USER]` real-terminal checks (`docs/functional-verification-matrix.md:56-65`). |

### Prioritized Remediation Roadmap

#### Phase A — Critical & High Release Blockers (Must Fix Before `v0.1.0`)
1. **Fix UTF-8 task-list panic (`BLK-01`):** Replace byte slice `&trimmed[1..6]` in `crates/scriptui-core/src/indent.rs:83` with `trimmed.strip_prefix(...)` or `trimmed.is_char_boundary(6)` and add multi-byte regression tests.
2. **Wire startup theme, locale, and config error reporting (`BLK-02`):** Pass `config_dir.join("themes")` and `config_dir.join("dicts")` into `EditorState` startup or invoke `switch_theme`/`switch_spell_locale` in `TuiApp::from_cli`; surface `config.toml` parse failures and invalid `--theme`/`--locale` errors instead of silently falling back to `crt_amber`/`en_US`.
3. **Implement or explicitly reconcile `Ctrl+V` (`Action::Paste`) (`BLK-03`):** Wire an internal clipboard fallback and/or system clipboard integration for `Action::Paste` (or document/align `Keymap::default_cua()` and `README.md` if terminal bracketed paste only is intended).
4. **Fix `dirty` tracking across Undo (`BLK-04`):** Track `saved_revision` on `DocumentBuffer` (updated on load/save) and compute `is_dirty()` against `revision != saved_revision` (or track undo stack distance), and avoid setting `dirty = true` on no-op `apply_insert("")`.
5. **Fix CLI new-file launch & `KeyEventKind::Repeat` (`BLK-05`, `BLK-06`):** Allow `NotFound` paths passed on CLI to create an empty named buffer ready for first save; accept `KeyEventKind::Press | KeyEventKind::Repeat` in `TuiApp::handle_key_event`.
6. **Cache `SyntaxSet` and memoize preview rendering (`BLK-07`):** Move `SyntaxSet::load_defaults_newlines()` into a `OnceLock` static in `highlight.rs` and cache `PreviewDocument` by `(doc_id, buffer_index, doc_revision, theme_name, preview_width)`.
7. **Harden signal cleanup and atomic file saves (`BLK-08`):** Install `SIGTERM`/`SIGHUP`/`SIGTSTP` cleanup handling (e.g., `signal-hook`) and drop `TerminalGuard` before printing exit errors in `main.rs:47-50`; use `OpenOptions::create_new(true)` with randomized temp filenames for atomic saves and `import_locale_files`.
8. **Resolve Cargo metadata, third-party licenses, `deny.toml`, and CI/real-terminal gates (`BLK-09`, `BLK-10`):** Add `version = "0.1.0"` (or `publish = false`) to `crates/scriptui-tui/Cargo.toml:16`, complete `Cargo.toml` (`repository`, `rust-version`), add `THIRD_PARTY_LICENSES.md` / `NOTICE` for `en_US.{aff,dic}` and `spellbook` (`MPL-2.0`), add `deny.toml` with documented exceptions for transitive `bincode`/`paste`/`yaml-rust`, add `.github/workflows/{ci,release}.yml`, and execute `STEP 10D` real-terminal checks.

#### Phase B — Medium Priority Fixes (Pre-Release Polish / Quality)
- Guard Find/Replace unhandled navigation/editing keys (`Delete`, arrows, `Home`/`End`) from falling through to the editor (`find_bar.rs:117-208`), reset `find_bar.focus` when overlays change, and refresh stale find matches on buffer switch.
- Fix `PreviewMode::Expanded` synced scroll passing visual `scroll_line` as logical line (`app.rs:120-126`).
- Guard `Super`/`Meta` modifiers in `crossterm_key_to_chord` and `app.rs:258-260`.
- Preserve multi-cursors across `handle_indent`, `move_primary_selection`, and `ClearCurrentLine` (`editor.rs:793-966`), and remap `SelectionSet::primary` during `normalize()` (`selection.rs:107-144`).
- Switch `preview.rs` table/wrap width calculations from `.chars().count()` to `unicode_width` and handle narrow tables cleanly (`preview.rs:688-691`).
- Sanitize C0/C1 control characters before rendering untrusted strings in `chrome.rs`, `preview_pane.rs`, and `modals.rs`.
- Wire `PersonalDictionary::{load_from_file, save_to_file}` (`spell.rs:47-108`) and preserve personal/ignored words across `switch_spell_locale` (`editor.rs:427-437`).
- Add tab-bar overflow scrolling/indicators (`chrome.rs:39-64`), overlay cursor/scroll editing, `Ctrl+Shift+L` cursor-word seeding (`editor.rs:662-665`), and fix GFM autolink/unclosed-fence/CRLF-frontmatter edge cases (`markdown.rs:307-438`).

#### Phase C — Low Priority Improvements & Hygiene
- Deduplicate `push_coalesced`, atomic write-and-rename helpers, GFM alert matching, palette filtering (`editor.rs` vs `palette.rs`), and invalid-regex handling (`app.rs` vs `find_bar.rs`).
- Fix `BUG-0014` (hard/soft breaks in `preview.rs:859`), `BUG-0015` (Setext `source_line` in `preview.rs:374-389`), `BUG-0016` (curly apostrophes in `PersonalDictionary::add` / `ignore_word`), and `TASK-0049` (`CHANGELOG.md` link definitions).
- Decompose monolithic functions (`preview.rs`, `editor.rs`, `viewport.rs`, `mouse.rs`), add rustdoc comments on public invariants, and close unit-test edge-case gaps across `error.rs`, `config.rs`, `layout_tree.rs`, `main.rs`, and `highlight.rs`.

---

## 2. Audit Scope, Repository Inventory & Verification Commands

### Repository Inventory

| Area | Files Covered | Total Lines | Notes |
| :--- | :--- | ---: | :--- |
| **Root Manifests & Docs** | `Cargo.toml` (19), `Cargo.lock`, `LICENSE` (22), `.gitignore` (6), `README.md` (219), `CHANGELOG.md` (22), `STEPPED_IMPLEMENTATION_PLAN.md` (751), `docs/functional-verification-matrix.md` (66) | 1,105+ | `.github/` and `deny.toml` are absent |
| **`crates/scriptui-core/`** | `Cargo.toml` (24), 17 `src/*.rs` modules (`lib.rs`, `buffer.rs`, `buffers.rs`, `config.rs`, `editor.rs`, `error.rs`, `highlight.rs`, `history.rs`, `indent.rs`, `keymap.rs`, `layout.rs`, `markdown.rs`, `preview.rs`, `search.rs`, `selection.rs`, `spell.rs`, `theme.rs`) (7,544), 2 `tests/*.rs` suites (`editor_state_pipeline.rs`, `markdown_spell_pipeline.rs`) (657), `assets/dicts/en_US.{aff,dic}` (206 + 49,060) | 8,225 (Rust) | Pure-Rust headless engine; embedded ~549 KB Hunspell assets |
| **`crates/scriptui-tui/`** | `Cargo.toml` (23), 13 `src/*.rs` modules (`lib.rs`, `main.rs`, `app.rs`, `chrome.rs`, `cli.rs`, `find_bar.rs`, `layout_tree.rs`, `modals.rs`, `mouse.rs`, `palette.rs`, `preview_pane.rs`, `terminal.rs`, `viewport.rs`) (4,139), `tests/tui_integration.rs` (1,290) | 5,429 | Ratatui `0.29` + Crossterm `0.28` + Clap `4` frontend |
| **Scripts & Workflow Metadata** | `scripts/neon-dispatch` (52), `scripts/phased-step.sh` (617), `.orrery/` (`phase.toml`, `project.toml`), `.phased/deferred/11A.md`, `.tracker/` (77 inbox item directories, inspected read-only) | 669+ | Phased workflow tracking (`Phase 11`) |

### Non-Modifying Verification Commands & Observed Outputs

All commands were executed read-only from the repository root using toolchain `cargo 1.99.1` / `rustc 1.99.1`:

| Command | Exit Code | Observed Output Summary |
| :--- | :---: | :--- |
| `git status --porcelain` | `0` | Only `docs/codebase-audit-report.md` (this report) and `.agents/` (agent workspace); zero other tracked/untracked modifications. |
| `cargo fmt --all -- --check` | `0` | Zero formatting diffs. |
| `cargo check --workspace --all-targets --locked` | `0` | Compiled `scriptui-core v0.1.0` and `scriptui-tui v0.1.0` cleanly. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | `0` | Zero warnings across all targets (`lib`, `bin`, `tests`). |
| `cargo test --workspace --locked` | `0` | **98 passed, 0 failed, 0 ignored**: `scriptui-core` unit (`56`), `editor_state_pipeline` (`5`), `markdown_spell_pipeline` (`1`), `scriptui-tui` unit (`22`), `scriptui` bin (`0`), `tui_integration` (`14`), doc-tests (`0`). |
| `cargo tree -d --locked` | `0` | Duplicate crate versions across `darling`/`darling_core`/`darling_macro` (`0.20.11`/`0.24.1`), `foldhash` (`0.1.5`/`0.2.0`), `hashbrown` (`0.15.5`/`0.17.1`), `linux-raw-sys` (`0.4.15`/`0.12.1`), `rustix` (`0.38.44`/`1.1.6`), `syn` (`2.0.119`/`3.0.10`), and `unicode-width` (`0.1.14`/`0.2.0`). |
| `cargo deny --offline --locked check all` | `5` | Warned missing `deny.toml` (used default config). Failed `advisories` with 3 unmaintained transitive advisories (`RUSTSEC-2025-0141` `bincode 1.3.3`, `RUSTSEC-2024-0436` `paste 1.0.15`, `RUSTSEC-2024-0320` `yaml-rust 0.4.5`) and `licenses` (default config rejects all unspecified licenses; `spellbook@0.4.2` is `MPL-2.0`). |

---

## 3. Dimension Assessments

### 1. Architecture

#### Verified-Clean Design Strengths
- **Strict Headless Core Isolation (`crates/scriptui-core/Cargo.toml:11-21`, `crates/scriptui-tui/Cargo.toml:15-19`):** `scriptui-core` has zero `ratatui` or `crossterm` dependencies and compiles `syntect` with `default-features = false, features = ["default-fancy"]` (pure-Rust `fancy-regex`, avoiding C `oniguruma` bindings).
- **Clean TUI Module Decomposition (`crates/scriptui-tui/src/lib.rs:1-36`):** Layout (`layout_tree.rs`), chrome (`chrome.rs`), viewport (`viewport.rs`), preview (`preview_pane.rs`), overlays (`find_bar.rs`, `palette.rs`, `modals.rs`), mouse routing (`mouse.rs`), and terminal guard (`terminal.rs`) are well-separated by responsibility.
- **Structured Core Error Enum (`crates/scriptui-core/src/error.rs:3-33`):** `ScriptuiError` (`thiserror`) cleanly models 7 error variants (`Io`, `TomlParse`, `InvalidRegex`, `UnknownTheme`, `OutOfBounds`, `InvalidLocale`, `SpellDict`).

#### Findings
1. **[High] Per-frame synchronous preview rebuild & `SyntaxSet` reconstruction (`crates/scriptui-tui/src/preview_pane.rs:56`, `crates/scriptui-core/src/preview.rs:58`, `crates/scriptui-core/src/highlight.rs:85-89`):**
   - **Evidence:** `TuiApp::render_frame` calls `render_preview_pane`, which unconditionally calls `state.build_active_preview(inner_width)` on every rendered frame (`app.rs:128-136`, `preview_pane.rs:56`). Inside `build_preview`, `CodeHighlighter::new()` invokes `SyntaxSet::load_defaults_newlines()` (`highlight.rs:87`) on every single call. Unlike `EditorState::active_analysis()` (`editor.rs:276-313`), preview output is uncached. Similarly, `render_editor_viewport` recomputes `compute_wrapped_layout` across all buffer lines on every frame (`viewport.rs:125-130`).
   - **Impact:** Every cursor movement, selection drag, or wheel scroll with preview open re-parses Markdown and decompresses syntect's default syntax bundle synchronously on the event-loop thread.
   - **Remediation:** Cache `SyntaxSet` via `std::sync::OnceLock` in `highlight.rs` and cache `PreviewDocument` keyed by `(doc_id, doc_revision, theme.name, inner_width)`.
2. **[High] Unwired startup asset resolution & silent error swallowing (`crates/scriptui-core/src/editor.rs:245-254`, `crates/scriptui-tui/src/app.rs:66-76`):**
   - **Evidence:** `TuiApp::from_cli` swallows config parse/I/O failures via `AppConfig::load_or_default(&config_path).unwrap_or_default()` (`app.rs:68`). `EditorState::new` ignores `config.spell_locale` (`editor.rs:250-254` calls `SpellEngine::default_en_us().ok()`) and passes `&[]` to `resolve_theme(&config.theme, &[])` with silent fallback `Err(_) => default_theme()?` (`editor.rs:246-249`).
   - **Impact:** Misconfigured `config.toml` files, invalid CLI `--theme`/`--locale` arguments, and custom user themes/dictionaries fail silently at startup.
   - **Remediation:** Propagate or surface `config.toml` errors in `status_message`, pass config theme/dict search directories during initialization, and surface invalid theme/locale errors cleanly.
3. **[Medium] Split/duplicated UI state between `EditorState` and `TuiApp` (`crates/scriptui-core/src/editor.rs:86-206, 225-242`, `crates/scriptui-tui/src/app.rs:55-63`, `crates/scriptui-tui/src/palette.rs:61-144`):**
   - **Evidence:** `EditorState` exposes all major fields as `pub` (`editor.rs:226-238`) alongside an internal `CommandPaletteState::default_commands()` and `filtered()`. Meanwhile `TuiApp` maintains a separate `palette_entries: Vec<PaletteEntry>` (`palette.rs:61-144`) that ignores `state.palette.commands`/`filtered()`, and holds global `PreviewPaneState` (`app.rs:60`) outside per-buffer `ManagedBuffer` state.
   - **Impact:** Direct field mutation (e.g., `mouse.rs:43-45`) can leave caches/overlays out of sync; `preview_pane.sync_enabled` is enabled on keyboard toggle (`app.rs:245-248`) but omitted on mouse edge-tab click (`mouse.rs:51`) and Command Palette dispatch (`palette.rs:294-296`); preview scroll offset bleeds across buffer switches.
   - **Remediation:** Consolidate palette command lists and per-buffer preview scroll state, and encapsulate `EditorState` invariants behind methods.
4. **[Medium] Redundant full-document clones and undocumented transaction ordering invariant (`crates/scriptui-core/src/editor.rs:796-798`, `search.rs:57`, `history.rs:6-10, 52-79`):**
   - **Evidence:** `handle_insert_newline` clones the full rope via `doc.text()` and runs `parse_gfm(&text)` on every Enter keystroke instead of reading `self.active_analysis()`. `search::find_all` clones `buf.text()` on every search. `UndoStack::undo`/`redo` implicitly require `EditTransaction::edits` (`pub` field at `history.rs:7`) to be non-overlapping and sorted in descending `start` order.
   - **Impact:** Quadratic allocation overhead on large files and fragile public invariants in `EditTransaction`.
   - **Remediation:** Reuse cached `GfmDocument` in `handle_insert_newline`, search rope chunks/slices without full `String` clones if needed, and document/validate `EditTransaction::edits` invariants.
5. **[Low] Duplicate transitive `unicode-width` & proc-macro versions (`Cargo.lock`, `cargo tree -d --locked`):**
   - **Evidence:** `ratatui 0.29 -> unicode-truncate 1.1.0` pulls `unicode-width 0.1.14`, while `ratatui` and `scriptui-core` use `unicode-width 0.2.0` (plus duplicate `darling`, `foldhash`, `hashbrown`, `linux-raw-sys`, `rustix`, `syn`).
   - **Impact:** Increased compile footprint and potential width-table divergence between `ratatui`'s `unicode-truncate` and `scriptui-core`'s `layout.rs`.

---

### 2. Correctness and Bugs

#### Critical
1. **[Critical] UTF-8 slice-indexing panic in `indent::parse_task_list` (`crates/scriptui-core/src/indent.rs:74-86`):**
   - **Evidence:** `parse_task_list` checks `bytes.len() < 6` (`line 76`) and `matches!(marker, b'-' | b'*' | b'+')` (`line 80`), then immediately executes `let checkbox = &trimmed[1..6];` (`line 83`) without checking `trimmed.is_char_boundary(6)`.
   - **Impact:** Pressing `Enter` on any line beginning with `-`, `*`, or `+` where a multi-byte UTF-8 character straddles byte index `6` (e.g., `"-    é"`, where `'é'` spans bytes `5..7`) panics with `byte index 6 is not a char boundary`.
   - **Remediation:** Check `trimmed.get(1..6)` or `trimmed.is_char_boundary(6)` (or use `strip_prefix` patterns) before slicing `&trimmed[6..]`.

#### High
2. **[High] Startup `--locale` / `config.spell_locale` and custom `--theme` / `config.theme` silently ignored (`crates/scriptui-core/src/editor.rs:245-254`, `crates/scriptui-tui/src/app.rs:66-76`, `palette.rs:77-83`):**
   - **Evidence:** `EditorState::new` hardcodes `SpellEngine::default_en_us().ok()` (`editor.rs:251`) and `resolve_theme(&config.theme, &[])` (`editor.rs:246`). `TuiApp::from_cli` populates `config.theme` and `config.spell_locale` (`app.rs:69-74`) but never calls `switch_theme` or `switch_spell_locale` against `~/.config/scriptui/{themes,dicts}`. `build_palette_entries` (`palette.rs:77-83`) also only enumerates `builtin_theme_names()`, omitting custom theme files.
   - **Impact:** Non-`en_US` locales and custom TOML themes cannot be activated at startup via CLI or `config.toml`; custom theme names cannot be selected from the Command Palette.
   - **Remediation:** Resolve themes and locales against `AppConfig::default_config_dir()` subdirectories at startup and discover custom `.toml` themes in `build_palette_entries`.

3. **[High] `Ctrl+V` (`Action::Paste`) is a runtime no-op (`crates/scriptui-core/src/keymap.rs:356-358`, `crates/scriptui-core/src/editor.rs:732`, `crates/scriptui-tui/src/app.rs:253-255`):**
   - **Evidence:** `Keymap::default_cua()` binds `Ctrl+V` to `Action::Paste` (`keymap.rs:356-358`). `TuiApp::handle_key_event` routes `Action::Paste` via `other => self.state.dispatch(other)?` (`app.rs:253-255`), and `EditorState::dispatch` executes `Action::Copy | Action::Paste => {}` (`editor.rs:732`). Only terminal bracketed paste (`Event::Paste` -> `handle_paste_event` at `app.rs:326`) inserts pasted text.
   - **Impact:** Pressing `Ctrl+V` inside ScripTUI silently does nothing.
   - **Remediation:** Store internal clipboard text on `Copy`/`Cut` (and/or query OS clipboard if supported) and insert it on `Action::Paste`.

4. **[High] `dirty` flag never clears when undoing back to saved state (`crates/scriptui-core/src/buffer.rs:279, 366`, `crates/scriptui-core/src/history.rs:52-65`):**
   - **Evidence:** `DocumentBuffer::apply_raw_edit` unconditionally sets `self.dirty = true` (`buffer.rs:366`), and `UndoStack::undo` applies reverse edits via `apply_raw_edit` (`history.rs:59`). Additionally, `apply_insert` sets `self.dirty = true` (`buffer.rs:279`) before checking if any edit is non-empty (`buffer.rs:280-286`).
   - **Impact:** Undoing all modifications back to the exact saved content still marks the buffer dirty (`*`), triggering false unsaved-changes confirmation prompts on close/quit.
   - **Remediation:** Track clean/saved state via `saved_revision` or undo-stack clean-point tracking (and guard `apply_insert` against empty no-op edits).

5. **[High] Opening a non-existent file path on CLI fails startup (`crates/scriptui-tui/src/app.rs:77-79`, `crates/scriptui-core/src/buffers.rs:121`, `buffer.rs:62-66`):**
   - **Evidence:** `TuiApp::from_cli` calls `state.buffers.open_file(path)?` (`app.rs:78`), which calls `DocumentBuffer::from_file(path)?` (`fs::read_to_string`).
   - **Impact:** Running `scriptui notes.md` when `notes.md` does not yet exist exits immediately with `ExitCode::FAILURE` (`main.rs:34-35`).
   - **Remediation:** Support opening a new empty buffer bound to a non-existent target path when `ErrorKind::NotFound` occurs (unless parent directory is invalid).

6. **[High] `TuiApp::handle_key_event` drops `KeyEventKind::Repeat` (`crates/scriptui-tui/src/app.rs:177-179`):**
   - **Evidence:** `if key.kind != KeyEventKind::Press { return Ok(()); }` discards `KeyEventKind::Repeat` events.
   - **Impact:** Terminals/platforms sending repeat events (`Windows`, Kitty/WezTerm keyboard protocols) drop held navigation, `Backspace`, `Delete`, and character-repeat inputs.
   - **Remediation:** Accept `KeyEventKind::Press | KeyEventKind::Repeat`.

#### Medium
7. **[Medium] Find/Replace bar drops unhandled keys to the editor & leaks focus/match state (`crates/scriptui-tui/src/find_bar.rs:117-208`, `crates/scriptui-tui/src/app.rs:182-187`):**
   - **Evidence:** `handle_find_bar_key` returns `Ok(false)` for `Delete`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`, `PageDown`, and `Ctrl+Enter` in `Find` mode. `app.rs:184-186` then falls through to keymap/character dispatch at `app.rs:219`, mutating or scrolling the underlying document. Switching buffers (`mouse.rs:44-45`, `editor.rs:586-610`) clears `matches` while keeping `query.pattern`; `Alt+Enter` (`find_bar.rs:134-136`) can return `Err(InvalidRegex)` before closing the bar; switching to Command Palette via `Ctrl+P` leaves `find_bar.focus == FindFocus::Replacement`.
   - **Remediation:** Consume unhandled non-global keys inside `handle_find_bar_key` (or implement cursor editing), reset `find_bar.focus` on overlay close/switch, and recompute matches when reopening Find or switching tabs.

8. **[Medium] `PreviewMode::Expanded` passes visual `scroll_line` as logical source line (`crates/scriptui-tui/src/app.rs:120-126`, `layout_tree.rs:66`):**
   - **Evidence:** In `PreviewMode::Expanded`, `regions.editor` is `None`, so `wrapped_layout` is `None` (`app.rs:109`). `app.rs:124-126` then passes `active().scroll_line` (a visual wrapped-row offset) directly as `editor_top_source_line` to `compute_synced_preview_scroll`.
   - **Impact:** Synced preview scroll overshoots its position on documents with soft-wrapped lines.
   - **Remediation:** Compute or store the logical top line before entering `PreviewMode::Expanded` (or map via `compute_wrapped_layout` if needed).

9. **[Medium] `Super`/`Meta` modifiers leak into character insertion & navigation (`crates/scriptui-tui/src/terminal.rs:98-104`, `crates/scriptui-tui/src/app.rs:257-265`):**
   - **Evidence:** `crossterm_key_to_chord` drops `SUPER`, `HYPER`, and `META`, and `app.rs:258-260` only checks `!key.modifiers.intersects(CONTROL | ALT)`.
   - **Impact:** Pressing `Super+S` inserts `'s'` into the buffer and `Super+Left` moves the cursor.
   - **Remediation:** Reject or ignore keys carrying `SUPER`/`HYPER`/`META` before chord lookup or text insertion.

10. **[Medium] Multi-cursor collapse and `SelectionSet::primary` tracking bugs (`crates/scriptui-core/src/editor.rs:793-966`, `crates/scriptui-core/src/selection.rs:64-71, 92-144`):**
    - **Evidence:** `handle_indent` (`editor.rs:850-889`), `move_primary_selection` (`editor.rs:902-965`), and `handle_insert_newline` (`ClearCurrentLine`, `editor.rs:811-829`) act only on `primary` and collapse active multi-cursors (`SelectionSet::from_ranges(vec![...])`). Additionally, `SelectionSet::from_ranges` (`selection.rs:68`) sets `primary: 0` before `normalize()` sorts `ranges`, and `normalize()` (`selection.rs:114-143`) does not remap `self.primary` after sorting/merging.
    - **Remediation:** Operate on all selections in `SelectionSet` for movement/indent/newline, and track the original `primary` selection through `normalize()`.

11. **[Medium] Preview width uses `.chars().count()` instead of display width and overflows narrow tables (`crates/scriptui-core/src/preview.rs:163, 680, 688-704, 726, 1148-1177`):**
    - **Evidence:** `preview.rs` measures cell/prefix widths via `.chars().count()` rather than `unicode_width`, misaligning table borders and line wrapping for CJK/emoji text. In `render_table` (`preview.rs:688-691`), `if max_content >= col_count && total_content > max_content` skips shrinking columns completely whenever `avail < col_count * 4 + 1`.
    - **Remediation:** Measure preview widths via `unicode_width::UnicodeWidthStr`/`Char` and clamp/wrap tables even when `avail` is smaller than `col_count * 4 + 1`.

12. **[Medium] Viewport combining-char omission, straddling wide-char cursor, & control-char zero width (`crates/scriptui-tui/src/viewport.rs:295-374`, `crates/scriptui-core/src/layout.rs:147`):**
    - **Evidence:** `viewport.rs:295-297` skips all `w == 0` chars (`continue;`), dropping Unicode combining marks (e.g., `e\u{0301}`) instead of appending them to the preceding cell. When a 2-cell wide char straddles the right edge at `h_scroll + visible_width - 1`, `viewport.rs:321-322` skips rendering it while `366-373` still draws the cursor on that blank cell; wide-char cursors also only style `screen_x` (not `screen_x + 1`). In `layout.rs:147`, non-tab control characters (`\x00`–`\x1f`, `\x7f`) have `width == 0`, desynchronizing columns if present in a buffer.
    - **Remediation:** Group grapheme clusters or attach combining marks to the previous cell, style both cells of wide cursors, and map control chars to visible placeholders or width 1.

13. **[Medium] Char-based deletion splits CRLF & grapheme clusters (`crates/scriptui-core/src/buffer.rs:302-309`, `layout.rs:230-239`):**
    - **Evidence:** `apply_delete` removes a single `char` (`start - 1..start` or `start..start + 1`), splitting `\r\n` into orphaned `\r`/`\n` and breaking multi-codepoint grapheme clusters; `compute_wrapped_layout` wraps on `char` boundaries rather than grapheme clusters.
    - **Remediation:** Treat `\r\n` and grapheme clusters atomically during Backspace/Delete and soft-wrapping.

14. **[Medium] GFM parsing & spellcheck edge cases (`crates/scriptui-core/src/markdown.rs:307-324, 350-360, 422-438, 511-525`, `crates/scriptui-core/src/spell.rs:289-301, 410-412`, `editor.rs:427-437`):**
    - **Evidence:**
      - Duplicate autolinks in a paragraph match only the first occurrence via `slice.find(&needle)` (`markdown.rs:311`).
      - Reference link definitions (`[ref]: https://...`) have no `comrak` AST node and are not skipped from spellcheck.
      - Unclosed fenced code blocks at EOF miscompute `CodeBlockRegion` bounds because `markdown.rs:350-355` assumes `span.end_line` is a closing fence.
      - CRLF front matter compares normalized `raw.len()` against original `\r\n` byte indices (`markdown.rs:426`).
      - `is_word_char` (`spell.rs:410-412`) excludes ASCII digits, splitting `"utf8"`/`"1st"` despite `en_US.aff:14-17` defining `WORDCHARS 0123456789` and `COMPOUNDRULE` ordinals.
      - `EditorState::switch_spell_locale` (`editor.rs:427-437`) discards in-memory `PersonalDictionary` and `ignored` words.
    - **Remediation:** Advance autolink search offsets, strip reference definitions or scan source lines, inspect closing fence line before decrementing `end_line`, map front matter via `\n` line count/closing delimiter, align `is_word_char` with Hunspell `WORDCHARS`, and carry over `PersonalDictionary`/`ignored` across locale switches.

15. **[Medium] Tab bar silent truncation, un-cleared drag state, & non-vertical cursor movement (`crates/scriptui-tui/src/chrome.rs:39-64`, `mouse.rs:24-27`, `editor.rs:910-957`, `highlight.rs:290-296`):**
    - **Evidence:** `render_tab_bar` stops emitting tabs once `cursor_x >= area_right`, hiding the active tab when many buffers are open. `MouseRouter` does not reset `dragging_selection_anchor` / `dragging_split_divider` if a modal opens or focus is lost before `MouseUp`. Vertical cursor moves (`Up`/`Down`/`PageUp`/`PageDown` at `editor.rs:910-957`) reuse logical `char` column instead of display column + sticky target column. Nested inline highlights in `highlight_editor_line` (`highlight.rs:290-296`) overwrite `flags = token_flags` (e.g., `Strong` inside `Link` loses `underline`).

#### Low
16. **[Low] Open tracker bugs confirmed still present in `HEAD` (`preview.rs:374-389, 859`, `spell.rs:110-115, 205-210`):**
    - **`BUG-0014`:** `preview.rs:859` maps `NodeValue::SoftBreak | NodeValue::LineBreak => InlineToken::HardBreak`, breaking reflow on hard-wrapped Markdown.
    - **`BUG-0015`:** `preview.rs:374-389` emits the Setext heading underline rule with `source_line` after `render_wrapped_tokens` advances lines, producing non-monotonic `source_line` entries.
    - **`BUG-0016`:** `PersonalDictionary::add` (`spell.rs:110-115`) and `SpellEngine::ignore_word` (`spell.rs:205-210`) do not normalize curly apostrophe `\u{2019}` to `'`.

---

### 3. Security and Safety

#### Verified-Clean Safety Controls
- **Zero `unsafe` blocks:** Confirmed `0` matches across `crates/scriptui-core/src/` and `crates/scriptui-tui/src/` (only dictionary word `unsafe/YTR` in `en_US.dic:46857`).
- **Zero production `.unwrap()` / `.expect()` / `panic!`:** Python AST/regex scan across all non-test code in `crates/*/src/*.rs` confirmed `0` occurrences (all existing `.unwrap()`/`.expect()` calls are inside `#[cfg(test)]` modules or integration tests).
- **Safe `0×0`–`2×2` terminal rendering:** Verified `layout_tree.rs`, `chrome.rs`, `viewport.rs`, `preview_pane.rs`, `find_bar.rs`, `modals.rs`, and `palette.rs` all guard `width == 0 || height == 0` (or `< 2`) and use saturating arithmetic.
- **Regex ReDoS protection:** `search.rs:49-55` uses Rust's linear-time `regex` crate (no backtracking ReDoS; invalid/oversized patterns return `ScriptuiError::InvalidRegex`).
- **Path-traversal guards on theme/locale identifiers:** `resolve_theme` (`theme.rs:307-313`) and `validate_locale_name` (`spell.rs:15-29`) strictly allow only `[A-Za-z0-9_-]+` (and `len <= 32` for locales).
- **OSC 52 delimiter safety & sensitive data check:** `encode_osc52_clipboard` (`terminal.rs:137-140`) base64-encodes payloads, preventing `\x07`/`\x1b\` breakout; zero hardcoded IPs, credentials, tokens, or real user paths exist in source or fixtures (fixtures use `testuser` and `example.com`).

#### Findings
1. **[Critical] Input-triggered runtime panic on UTF-8 task-list lines (`crates/scriptui-core/src/indent.rs:83`):**
   - As documented in §2.1, `&trimmed[1..6]` can panic on untrusted Markdown input containing multi-byte UTF-8 characters spanning byte index 6 when `Enter` is pressed.
2. **[High] Missing OS signal handling leaves terminal corrupted & exit errors printed on alt screen (`crates/scriptui-tui/src/terminal.rs:19-75`, `crates/scriptui-tui/src/main.rs:39-53`):**
   - **Evidence:** `TerminalGuard` installs a `panic::set_hook` wrapper (`terminal.rs:25-29`) and restores terminal state in `Drop` (`terminal.rs:60-64`), but neither `terminal.rs` nor `main.rs` registers handlers for `SIGTERM`, `SIGHUP`, or `SIGTSTP`/`SIGCONT`. Furthermore, on `Err(err)` from `app.run_event_loop(&mut guard)` (`main.rs:47-50`), `eprintln!("ScripTUI exited with error: {err}");` executes *before* `guard` is dropped at the end of `main()`, writing the error onto the alternate screen immediately before `LeaveAlternateScreen` hides it.
   - **Remediation:** Register signal cleanup (`SIGTERM`/`SIGHUP`/`SIGTSTP`/`SIGCONT`), restore/drop `TerminalGuard` before printing exit errors to `stderr`, and restore previous panic hook if needed.
3. **[High] Predictable temp paths without `O_EXCL` in atomic saves & non-atomic `import_locale_files` (`crates/scriptui-core/src/buffer.rs:101-110`, `crates/scriptui-core/src/spell.rs:74-82, 386-393`):**
   - **Evidence:** `DocumentBuffer::save_atomic` (`buffer.rs:101-109`) and `PersonalDictionary::save_to_file` (`spell.rs:74-82`) open `{file_name}.scriptui.tmp.<pid>` using `File::create(&tmp_path)` (which omits `O_EXCL`). In shared/world-writable directories (e.g., `/tmp`), a pre-existing symlink at that predictable name can be followed to clobber arbitrary files. Separately, `import_locale_files` (`spell.rs:386-393`) writes `.dic` and `.aff` via non-atomic `fs::write`.
   - **Remediation:** Create temp files using randomized names with `OpenOptions::new().write(true).create_new(true)` (or `tempfile::NamedTempFile::new_in`), and use atomic write-then-rename in `import_locale_files`.
4. **[High] Missing third-party licensing / provenance notice & `cargo-deny` advisory/wildcard failures (`crates/scriptui-core/assets/dicts/en_US.{aff,dic}`, `LICENSE:1-22`, `crates/scriptui-core/Cargo.toml:19`, `crates/scriptui-tui/Cargo.toml:16`):**
   - **Evidence:** `spell.rs:12-13` embeds `en_US.aff` (3,090 bytes) and `en_US.dic` (546,109 bytes, 49,058 entries) derived from system Hunspell wordlists (`STEPPED_IMPLEMENTATION_PLAN.md:492`), and `scriptui-core` depends on `spellbook@0.4.2` (`MPL-2.0`). Root `LICENSE` contains only `MIT License / Copyright (c) 2026 InfoMoose` with no `NOTICE` or `THIRD_PARTY_LICENSES` file. Meanwhile, `cargo deny --offline --locked check all` reports 3 unmaintained transitive advisories (`RUSTSEC-2025-0141` `bincode 1.3.3` via `syntect`, `RUSTSEC-2024-0436` `paste 1.0.15` via `ratatui`, `RUSTSEC-2024-0320` `yaml-rust 0.4.5` via `syntect`), and `crates/scriptui-tui/Cargo.toml:16` lacks `version` on `scriptui-core` (causing `cargo-deny` wildcard rejection, `TASK-0061`).
   - **Remediation:** Add `THIRD_PARTY_LICENSES.md` / `NOTICE` covering `en_US.{aff,dic}` upstream licensing and `spellbook` (`MPL-2.0`), add `version = "0.1.0"` to `scriptui-tui/Cargo.toml:16`, and commit `deny.toml` explicitly documenting approved licenses and the 3 upstream transitive advisories.
5. **[Medium] Missing symlink containment checks on theme/locale loading & unbounded file/OSC 52 payloads (`crates/scriptui-core/src/theme.rs:315-320`, `spell.rs:163`, `buffer.rs:63`, `search.rs:57`, `terminal.rs:137-146`):**
   - **Evidence:** `resolve_theme` and `load_locale` check `candidate.is_file()` without checking `symlink_metadata` or canonical containment inside `search_dirs`. `DocumentBuffer::from_file`, `import_locale_files`, and `build_preview` load unbounded files into memory (`spell::overlaps_any` at `spell.rs:427-431` also performs an `O(words * skip_ranges)` linear scan over sorted ranges instead of binary search). `write_osc52_clipboard` writes unbounded OSC 52 sequences synchronously to `stdout`.
   - **Remediation:** Canonicalize and verify directory containment (or reject symlinks) on theme/locale loads, add configurable max file-size / OSC 52 payload bounds, and use `partition_point` in `overlaps_any`.
6. **[Medium] Unsanitized C0/C1 control characters in non-viewport UI surfaces (`crates/scriptui-tui/src/chrome.rs:45, 58, 183, 189`, `preview_pane.rs:88`, `modals.rs:30-35, 95, 253`):**
   - **Evidence:** Tab labels (`display_name()`), `status_message`, `preview_pane` `seg.text` (from untrusted Markdown), and modal text (`diagnostic.word`, `file_prompt_input`) are passed directly into `Span::styled(...)` without stripping/replacing C0/C1 control bytes (`\x1b`, `\r`, `\x07`, `\x08`).
   - **Remediation:** Strip or render control characters safely (e.g., `escape_default` or `?`) before constructing `Span::styled`.

---

### 4. Test Coverage

All **98** automated tests currently pass (`56` core unit + `6` core integration + `22` TUI unit + `14` TUI integration, `0` ignored). However, critical gaps exist and explain why multiple High/Critical bugs escaped detection:

| Area / Gap | Severity | Evidence & Missing Coverage |
| :--- | :--- | :--- |
| **TerminalGuard & event loop** | **High** | `TerminalGuard::enter` / `Drop` / panic hook (`terminal.rs:19-75`), `TuiApp::run_event_loop` (`app.rs:380-418`), and `main.rs` exit paths (`0` tests in `src/main.rs`) have **0 automated tests**; all 5 `[USER]` checks in `docs/functional-verification-matrix.md:56-65` remain unchecked (`[ ]`). |
| **Startup CLI `--theme` / `--locale` / `config.toml` / new file** | **High** | `tui_integration.rs:1202` tested `--locale en_US` (the default, `TASK-0058`), and `tui_integration.rs:1278` tested `resolve_theme` directly instead of `TuiApp::from_cli`. Zero tests verify `TuiApp::from_cli` with custom themes, non-`en_US` locales, malformed `config.toml`, or non-existent CLI file arguments. |
| **UTF-8 & Undo/Dirty edge cases** | **High** | `indent.rs` has no tests with multi-byte UTF-8 chars in `parse_task_list` (masked the `indent.rs:83` panic), `IndentStyle::Tabs`, `1) ` lists, or `>>` blockquotes. `buffer.rs`/`history.rs` lack tests for `dirty` after full undo, CRLF round-trips, combining marks, `save_atomic` on untitled buffer, and `UndoStack::new(0)`. |
| **Key Repeat, `Super`/`Meta`, & Find Bar fallback** | **Medium** | `app.rs:177-179, 258-264` and `find_bar.rs:117-208` have **0 tests** for `KeyEventKind::Repeat`, `Super` modifiers, `Delete`/arrow keys inside Find/Replace, regex metacharacters after `Alt+R` (`TASK-0059`), and `Action::Paste`. |
| **Degenerate rects, tab overflow, & expanded preview sync** | **Medium** | Zero tests cover `0×0`/`1×1`/`2×2` frames, `layout_tree.rs` directly, tab-bar overflow (`chrome.rs:39-64`), `PreviewMode::Expanded` scroll sync (`app.rs:120-126`), or horizontal scroll (`scroll_col > 0` + mouse click). |
| **Core module edge cases** | **Low–Med** | `error.rs` has `0` unit tests; `highlight.rs` tests only 1 of 22 `MdTokenKind` variants (`Heading(1)`); `markdown.rs`/`preview.rs` lack tests for duplicate autolinks, unclosed fences, CRLF front matter, narrow tables, and CJK in preview; `spell.rs` lacks `PersonalDictionary::{load_from_file, save_to_file}` round-trip tests and alphanumeric tokens (`"1st"`, `"utf8"`). Narrow-viewport `CommandPalette` / `SpellQuickFix` assertions are tracked in `TASK-0057`. |

---

### 5. Scope Lapse

Cross-referencing `README.md`, `CHANGELOG.md`, `STEPPED_IMPLEMENTATION_PLAN.md`, `docs/functional-verification-matrix.md`, `.tracker/`, and `.phased/` against live source code reveals both **unimplemented documented promises** and **workflow/tracker drift**:

| ID | Severity | Documented / Planned Promise | Actual Codebase State |
| :--- | :--- | :--- | :--- |
| **SCP-01** | **High** | `README.md:56, 66, 109` & `functional-verification-matrix.md:12`: `--locale` and `config.toml` `spell_locale` load imported locales at startup. | Ignored at startup (`editor.rs:250-254`, `app.rs:72-76`); masked by `tui_integration.rs:1202` testing `"en_US"` (`TASK-0058`). |
| **SCP-02** | **High** | `README.md:55, 65, 89-99` & `functional-verification-matrix.md:11, 26`: `--theme` / `config.toml` load custom TOML themes from `~/.config/scriptui/themes/`. | `EditorState::new` searches `&[]` (`editor.rs:246-249`), `TuiApp::from_cli` never calls `switch_theme`, and `palette.rs:77-83` omits custom theme files. |
| **SCP-03** | **High** | `README.md:137`: `Ctrl+V` pastes from clipboard. | `keymap.rs:356-358` binds `Ctrl+V` -> `Action::Paste`, but `editor.rs:732` is `{}` (no-op). `functional-verification-matrix.md:44` added a workaround note pointing to bracketed `Event::Paste`. |
| **SCP-04** | **Medium** | `README.md:151`: `Ctrl+Shift+L` selects all occurrences of current selection/word. | `editor.rs:662-665` searches `self.find_replace.query` only; does nothing unless Find was previously populated. |
| **SCP-05** | **Medium** | `crates/scriptui-core/src/spell.rs:47-108` & `buffers.rs:205-229`: Persistent `PersonalDictionary` and external disk-modification polling/reload. | Both APIs are implemented and tested in core (`buffers.rs:281-303`) but have **zero runtime callers** in `editor.rs` or `scriptui-tui`. |
| **SCP-06** | **Medium** | `STEPPED_IMPLEMENTATION_PLAN.md:135` (`### Parked`) & `functional-verification-matrix.md:56-65` (`STEP 10D`). | Plan line 135 falsely states `_None — all tracked tasks and bugs are closed._`, whereas `.tracker/` contains **13 non-closed items** (`BUG-0014`, `BUG-0015`, `BUG-0016`, `TASK-0042`–`0044`, `TASK-0049`–`0050`, `TASK-0057`–`0061`). All 5 `[USER]` checks in `STEP 10D` remain unchecked. |
| **SCP-07** | **Low** | `STEPPED_IMPLEMENTATION_PLAN.md:23, 108, 181-186` & `editor.rs:86-158`. | Plan summary inverted `11A`/`11B` vs active Up Next (`STEP 11A` is deferred at `.phased/deferred/11A.md`); `repository.workspace = true` from STEP 8A is missing from `Cargo.toml`; Command Palette omits `OpenFile`, `SaveAs`, `NextBuffer`, `PrevBuffer`, `FindNext`, `FindPrev`. (`Action::ToggleLineNumbers` exists in `editor.rs:144-147, 691-693` with no default keybinding and is unlisted in `README.md`.) |

---

### 6. Incomplete Implementations

#### Verified-Clean Checks
- **Zero explicit code-stub markers:** `grep_search` across `crates/` confirmed `0` occurrences of `todo!()`, `unimplemented!()`, `TODO`, `FIXME`, `XXX`, or `HACK`.

#### Findings
1. **[High] `Action::Paste` empty match arm (`crates/scriptui-core/src/editor.rs:732`, `crates/scriptui-tui/src/app.rs:253-255`):** `Action::Copy | Action::Paste => {}` leaves `Ctrl+V` unimplemented while `Copy`/`Cut` are handled externally via OSC 52 in `app.rs:222-244`.
2. **[High] Deferred `deny.toml` & absent `.github/workflows/` (`STEP 11A`, `.phased/deferred/11A.md`, `TASK-0044`, `TASK-0061`):** Automated dependency/license enforcement and CI/release workflows are not yet implemented.
3. **[Medium] Partially wired `AppConfig` & state fields (`crates/scriptui-core/src/config.rs:29, 39`, `buffers.rs:23`, `editor.rs:237, 672, 675`):**
   - `config.spell_locale` is ignored by `EditorState::new`.
   - `config.preview_split_percent` is only read when toggling/expanding preview, not on initial state creation (and lacks load-time validation).
   - `ManagedBuffer::external_change_pending` is never polled or surfaced to users.
   - `EditorState::status_message` is populated on `Action::Save` (`editor.rs:625`), but remains silent on failed `open_spell_quick_fix_at_cursor` (`editor.rs:672`) and other core actions.
4. **[Medium] Partial overlay input & tab-bar UX (`crates/scriptui-tui/src/modals.rs:219-310`, `find_bar.rs:75-208`, `palette.rs:146-320`, `chrome.rs:39-64`):**
   - Text inputs in `FindBar`, `CommandPalette`, and `OpenFilePrompt`/`SaveAsPrompt` support only append (`push`) and tail-delete (`pop`), lacking cursor rendering, horizontal scrolling (clipping paths >58 chars in `modals.rs:230`), and `Left`/`Right`/`Home`/`End`/`Delete` editing. Empty `Enter` in file prompts (`modals.rs:287`) gives no feedback.
   - Tab bar (`chrome.rs:39-64`) has no scrolling or overflow indicator beyond `area.width`.
   - `Keymap::{bind, unbind, bindings_for}` (`keymap.rs:375-388`) and `Keymap` serde support are unused outside tests.

---

### 7. Code Hygiene

#### Verified-Clean Hygiene Baseline
- **`cargo fmt --all -- --check`:** Passes with exit code `0` (zero formatting diffs).
- **`cargo clippy --workspace --all-targets --locked -- -D warnings`:** Passes with exit code `0` (zero warnings).
- **`#[allow(...)]` & `#[ignore]`:** Zero `#[allow(...)]` suppressions and zero `#[ignore]` tests across both crates.
- **Scripts & `.gitignore`:** `.gitignore` properly excludes `/target/` and `.git/phased/`; `scripts/neon-dispatch` and `scripts/phased-step.sh` use `set -euo pipefail` and isolated `python3 -I` (note: `phased-step.sh` relies on GNU `sha256sum`/`find -printf` for Linux local orchestration, and `neon-dispatch:44` defaults `NEON_FLAGS` to `--dangerously-skip-permissions` relying on snapshot diff verification).

#### Findings
1. **[Medium] Duplicated logic across and within crates:**
   - **Palette filtering & subsequence matching:** `crates/scriptui-tui/src/palette.rs:107-144` duplicates `crates/scriptui-core/src/editor.rs:161-206`.
   - **Invalid-regex find error handling:** `crates/scriptui-tui/src/app.rs:337-346, 353-362` duplicates `crates/scriptui-tui/src/find_bar.rs:211-222`.
   - **`push_coalesced`:** Identical implementation in `crates/scriptui-core/src/highlight.rs:345-367` and `crates/scriptui-core/src/preview.rs:1194-1216`.
   - **Atomic temp-file write-and-rename:** Duplicated in `crates/scriptui-core/src/buffer.rs:96-130` and `crates/scriptui-core/src/spell.rs:69-105`.
   - **GFM alert header detection:** Duplicated in `crates/scriptui-core/src/markdown.rs:480-497` and `crates/scriptui-core/src/preview.rs:1007-1047`.
   - **Config directory path construction:** `main.rs:14` and `app.rs:67, 93, 192-194` independently construct `default_config_dir().join(...)`.
2. **[Medium] Incomplete `Cargo.toml` release metadata & missing `CHANGELOG.md` link definitions:**
   - Root `Cargo.toml:5-12` omits `repository`, `homepage`/`documentation`, `authors`, and `rust-version` (MSRV).
   - `CHANGELOG.md:7, 9` uses `[Unreleased]` and `[0.1.0]` Keep a Changelog reference links without link definitions at EOF (`TASK-0049`), and `2026-11-22` is a placeholder date (`STEPPED_IMPLEMENTATION_PLAN.md:92, 124`).
3. **[Low] Monolithic files/functions & sparse rustdoc documentation:**
   - Large modules/functions reduce maintainability: `preview.rs` (1,389 lines; `render_table` 174 lines, `collect_inline_node` 135 lines), `editor.rs` (1,357 lines; `dispatch` 178 lines), `markdown.rs` (647 lines; `CollectCtx::collect` 221 lines), `viewport.rs` (`render_editor_viewport` 282 lines at `96-377`), `mouse.rs` (`handle_mouse` 152 lines at `30-181`).
   - Public types/functions across `scriptui-core` largely lack `///` rustdoc comments (`doc-tests` = `0`), leaving invariants (`EditTransaction::edits` descending offset ordering, `char` vs `byte` column units) undocumented.
   - Minor inefficiencies/ergonomics: `UndoStack::record` (`history.rs:34-36`) uses `Vec::remove(0)` (`O(N)`) instead of `VecDeque::pop_front()`, `BufferManager::is_empty` (`buffers.rs:77-79`) always returns `false`, `theme::default_theme`/`resolve_theme` (`theme.rs:291-337`) re-parse embedded TOML strings on every call, and `lib.rs` omits `theme::load_theme_file` and `markdown::gfm_options` from top-level re-exports.
