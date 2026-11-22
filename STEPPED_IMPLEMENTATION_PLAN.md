# ScripTUI — Stepped Implementation Plan

## Step Execution Roadmap

**Workflow:** phased

**Base gate:** `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

**Forbidden paths:** `STEPPED_IMPLEMENTATION_PLAN.md:docs/*:.phased/*:.orrery/*:scripts/*`

**Planning files:** `docs/*`

**How to read this:** work goes strictly top to bottom through **Up Next**, one step at a time. The **#** column is the only order. Step IDs are kept because commits and notes cite them. A step whose "Waits on" names an Open Question or a **[USER]** action doesn't start until that's resolved.

**Who runs what (phased workflow + Orrery `gemini-phased`):**
- **Planner / Reviewer (Claude or review subagent)** writes the active phase (full step blocks to the zero-judgment bar), outlines future phases, and performs one phase review at the end of each phase using snapshot-backed step commits (`git show <sha>`).
- **Orchestrator / Validator (Hydrogen / Orrery `agy-hydrogen-orch` + `agy-hydrogen-val`)** runs the phase. At the start of each step, `phased:step_start` (`scripts/phased-step.sh pregate` + `neb start` + `scripts/phased-step.sh start`) takes a repository snapshot. After Neon finishes, `phased:step_check` (`scripts/phased-step.sh check`) validates scope and `POST_GATE` strictly against that snapshot diff, the validator reads the diff, and `phased:step_commit` commits the step and closes its Nebulous task.
- **Coder (Neon / Orrery `agy-neon-impl`)** executes one step inside `ALLOWED_FILES`. It never commits, never edits this plan or forbidden paths, and files bugs or scope lapses as **pending** Nebulous items.
- **Two code-fault failures** on one step, or an unamendable scope lapse → the step is deferred (`phased:step_defer`: saves attempt patch, restores step files, parks the task). A step whose `PRE_GATE` fails is deferred without dispatching an agent. Deferrals exceeding `max(2, ⌈steps/3⌉)` stop the phase early.

**Phase goal (Phase 6):** Resolve all Phase 5 review findings (`BUG-0004`–`BUG-0009`: modal key/mouse trapping, `replace_active_match` advancement, Find focus reset, cached Command Palette entries, unsynced preview scroll clamp), implement Unicode display width & tab-stop layout (`BUG-0003`), add revision-keyed GFM/spellcheck analysis caching, wire bracketed paste (`EnableBracketedPaste` + `Event::Paste`), verify all 5 palettes & end-to-end workflows with Ratatui `TestBackend` integration tests and a large-file responsiveness check, and publish `README.md`.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 6A** | `/goal` | core | `crates/scriptui-core/src/buffer.rs`, `crates/scriptui-core/src/editor.rs` | — |
| 2 | **STEP 6B** | `/goal` | tui | `crates/scriptui-tui/src/find_bar.rs`, `crates/scriptui-tui/src/preview_pane.rs`, `crates/scriptui-tui/src/app.rs` | STEP 6A |
| 3 | **STEP 6C** | `/goal` | core | `crates/scriptui-core/Cargo.toml`, `Cargo.lock`, `crates/scriptui-core/src/layout.rs` | STEP 6B |
| 4 | **STEP 6D** | `/goal` | core | `crates/scriptui-core/src/editor.rs`, `crates/scriptui-tui/src/viewport.rs` | STEP 6C |
| 5 | **STEP 6E** | `/goal` | tui | `crates/scriptui-tui/src/terminal.rs`, `crates/scriptui-tui/src/app.rs` | STEP 6D |
| 6 | **STEP 6F** | `/goal` | tui | `crates/scriptui-tui/tests/tui_integration.rs` | STEP 6E |
| 7 | **STEP 6G** | `/goal` | docs | `README.md` | STEP 6F |

### Completed

| Step ID | Status | Command | Scope | Target Files |
| :--- | :---: | :---: | :--- | :--- |
| **STEP 1A** | Done (`5997d28`) | `/goal` | core | `Cargo.toml`, `Cargo.lock`, `crates/scriptui-core/Cargo.toml`, `lib.rs`, `error.rs`, `selection.rs` |
| **STEP 1B** | Done (`be17f82`) | `/goal` | buffer | `crates/scriptui-core/src/buffer.rs`, `lib.rs` |
| **STEP 1C** | Done (`e47c1dc`) | `/goal` | history | `crates/scriptui-core/src/history.rs`, `lib.rs` |
| **STEP 1D** | Done (`38ecf17`) | `/goal` | indent | `crates/scriptui-core/src/indent.rs`, `lib.rs` |
| **STEP 1E** | Done (`24ccfa5`) | `/goal` | search | `crates/scriptui-core/src/search.rs`, `lib.rs` |
| **STEP 1F** | Done (`e63dd3b`) | `/goal` | theme | `crates/scriptui-core/src/theme.rs`, `config.rs`, `lib.rs` |
| **STEP 2A** | Done (`ba7a4eb`) | `/goal` | markdown | `crates/scriptui-core/Cargo.toml`, `Cargo.lock`, `error.rs`, `markdown.rs`, `lib.rs` |
| **STEP 2B** | Done (`1cfaac5`) | `/goal` | highlight | `crates/scriptui-core/src/highlight.rs`, `lib.rs` |
| **STEP 2C** | Done (`6df87bc`) | `/goal` | preview | `crates/scriptui-core/src/preview.rs`, `lib.rs` |
| **STEP 2D** | Done (`636d07e`) | `/goal` | spell | `crates/scriptui-core/assets/dicts/en_US.{aff,dic}`, `spell.rs`, `lib.rs` |
| **STEP 2E** | Done (`4893402`) | `/goal` | theme | `crates/scriptui-core/src/theme.rs`, `tests/markdown_spell_pipeline.rs` |
| **STEP 3A** | Done (`981129a`) | `/goal` | keymap | `crates/scriptui-core/src/keymap.rs`, `crates/scriptui-core/src/lib.rs` |
| **STEP 3B** | Done (`dcfbfec`) | `/goal` | buffers | `crates/scriptui-core/src/buffers.rs`, `crates/scriptui-core/src/lib.rs` |
| **STEP 3C** | Done (`435c363`) | `/goal` | layout | `crates/scriptui-core/src/layout.rs`, `crates/scriptui-core/src/lib.rs` |
| **STEP 3D** | Done (`f550a6e`) | `/goal` | editor | `crates/scriptui-core/src/editor.rs`, `crates/scriptui-core/src/lib.rs` |
| **STEP 3E** | Done (`55ff020`) | `/goal` | editor | `crates/scriptui-core/tests/editor_state_pipeline.rs` |
| **STEP 4A** | Done (`cb65eb7`) | `/goal` | core | `crates/scriptui-core/src/keymap.rs`, `crates/scriptui-core/src/editor.rs` |
| **STEP 4B** | Done (`ce7fdec`) | `/goal` | tui | `Cargo.toml`, `Cargo.lock`, `crates/scriptui-tui/Cargo.toml`, `cli.rs`, `lib.rs` |
| **STEP 4C** | Done (`3d1268a`) | `/goal` | tui | `crates/scriptui-tui/src/terminal.rs`, `crates/scriptui-tui/src/lib.rs` |
| **STEP 4D** | Done (`717bb4e`) | `/goal` | tui | `crates/scriptui-tui/src/layout_tree.rs`, `chrome.rs`, `lib.rs` |
| **STEP 4E** | Done (`0bdaf71`) | `/goal` | tui | `crates/scriptui-tui/src/viewport.rs`, `crates/scriptui-tui/src/lib.rs` |
| **STEP 4F** | Done (`af440cb`) | `/goal` | tui | `crates/scriptui-tui/src/mouse.rs`, `crates/scriptui-tui/src/lib.rs` |
| **STEP 4G** | Done (`192900d`) | `/goal` | tui | `crates/scriptui-tui/src/app.rs`, `main.rs`, `Cargo.toml`, `lib.rs` |
| **STEP 5A** | Done (`e9d0ac6`) | `/goal` | core | `crates/scriptui-core/src/keymap.rs`, `crates/scriptui-core/src/editor.rs` |
| **STEP 5B** | Done (`83b727e`) | `/goal` | tui | `crates/scriptui-tui/src/viewport.rs`, `crates/scriptui-tui/src/app.rs` |
| **STEP 5C** | Done (`6cd4249`) | `/goal` | tui | `crates/scriptui-tui/src/preview_pane.rs`, `crates/scriptui-tui/src/lib.rs` |
| **STEP 5D** | Done (`0b651e9`) | `/goal` | tui | `crates/scriptui-tui/src/find_bar.rs`, `crates/scriptui-tui/src/lib.rs` |
| **STEP 5E** | Done (`c2140a5`) | `/goal` | tui | `crates/scriptui-tui/src/palette.rs`, `crates/scriptui-tui/src/lib.rs` |
| **STEP 5F** | Done (`d096a7f`) | `/goal` | tui | `crates/scriptui-tui/src/modals.rs`, `crates/scriptui-tui/src/lib.rs` |
| **STEP 5G** | Done (`7199130`) | `/goal` | tui | `crates/scriptui-tui/src/app.rs`, `crates/scriptui-tui/src/mouse.rs` |

### Future phases

_None — Phase 6 completes the ScripTUI baseline roadmap._

### Open Questions

_None._

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.
_None — all open bugs (`BUG-0003` through `BUG-0009`) are scheduled in Phase 6._

---

## Phase 6 Step Blocks

### STEP 6A: Document Revision Counter & `replace_active_match` Advancement Fix (`BUG-0006`)

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0006`

```text
[STEP_ID]
STEP 6A: Add a monotonic revision counter to DocumentBuffer and fix EditorState::replace_active_match to advance active_match past new_caret instead of resetting to 0 (BUG-0006).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/buffer.rs
  - crates/scriptui-core/src/editor.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:19-65, 146-160, 217-340 (DocumentBuffer struct, constructors, apply_insert, apply_delete, apply_raw_edit)
  - crates/scriptui-core/src/editor.rs:272-278, 383-420 (update_find_matches, replace_active_match)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct DocumentBuffer\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn replace_active_match\b' crates/scriptui-core/src/editor.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/buffer.rs:
   - Add `revision: u64` field to `DocumentBuffer`, initialized to `0` in `empty`, `from_str_content`, and `from_file`.
   - Expose `pub fn revision(&self) -> u64 { self.revision }`.
   - Increment `self.revision = self.revision.wrapping_add(1)` whenever content changes:
     - in `apply_insert` (when `!edits.is_empty()` and any edit had `start < end || !text.is_empty()`),
     - in `apply_delete` (when `!edits.is_empty()`),
     - in `apply_raw_edit` (after applying the edit to `self.rope`).
   - Add unit test `document_revision_increments_on_content_mutations` verifying `revision()` starts at `0`, increments on `apply_insert`, `apply_delete`, and `apply_raw_edit`, and does not change on `set_selection`.
2. crates/scriptui-core/src/editor.rs:
   - Fix `BUG-0006` in `EditorState::replace_active_match`:
     - After `self.update_find_matches()?`, if `!self.find_replace.matches.is_empty()`, set `self.find_replace.active_match` to `self.find_replace.matches.iter().position(|next| next.start >= new_caret).or(Some(0))`.
   - Add unit test `replace_active_match_advances_past_overlapping_replacement`:
     - On `"foo bar foo baz foo"`, search `"foo"`, replace with `"foobar"` via `replace_active_match()` twice; assert the first replacement turns match 0 into `"foobar"` and advances `active_match` to the second `"foo"` (`Some(2)` or the match at `"foo baz"`), and the second replacement turns the second `"foo"` into `"foobar"` rather than re-replacing match 0 into `"foobarbar"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn revision\b' crates/scriptui-core/src/buffer.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 6B: TUI Modal Key/Mouse Trapping, Cached Palette Entries, Find Focus Reset & Preview Scroll Clamp (`BUG-0004`, `BUG-0005`, `BUG-0007`, `BUG-0008`, `BUG-0009`)

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0004`, `BUG-0005`, `BUG-0007`, `BUG-0008`, `BUG-0009`

```text
[STEP_ID]
STEP 6B: Fix Phase 5 TUI review bugs: trap unhandled keys and block mouse events behind floating modals (BUG-0004, BUG-0005), reset FindBarState.focus on Alt+Enter (BUG-0007), cache Command Palette entries on TuiApp (BUG-0008), and clamp unsynced preview scroll to viewport height (BUG-0009).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/find_bar.rs
  - crates/scriptui-tui/src/preview_pane.rs
  - crates/scriptui-tui/src/app.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/find_bar.rs:117-158 (handle_find_bar_key Esc and Alt+Enter branches)
  - crates/scriptui-tui/src/preview_pane.rs:10-65 (PreviewPaneState, render_preview_pane)
  - crates/scriptui-tui/src/app.rs:42-75, 123-289 (TuiApp, render_frame, handle_key_event, handle_mouse_event)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn revision\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub struct PreviewPaneState\b' crates/scriptui-tui/src/preview_pane.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/find_bar.rs:
   - Fix `BUG-0007`: in `handle_find_bar_key`, inside the `KeyCode::Enter` `ALT` branch, also reset `bar_state.focus = FindFocus::Query;`.
2. crates/scriptui-tui/src/preview_pane.rs:
   - Fix `BUG-0009`: add `pub viewport_height: usize` to `PreviewPaneState` (initialized to `0` in `Default`).
   - In `render_preview_pane`, set `pane_state.viewport_height = inner_height;` and when `!pane_state.sync_enabled`, clamp `pane_state.scroll_line = pane_state.scroll_line.min(pane_state.total_lines.saturating_sub(inner_height.max(1)));`.
3. crates/scriptui-tui/src/app.rs:
   - Add helper `fn is_blocking_modal(overlay: OverlayMode) -> bool` returning `true` for `OverlayMode::CommandPalette | OverlayMode::SpellQuickFix | OverlayMode::ConfirmQuit | OverlayMode::ConfirmCloseBuffer | OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt`.
   - Fix `BUG-0008`: add `pub palette_entries: Vec<PaletteEntry>` to `TuiApp` (initialized to `Vec::new()` in `from_cli`), plus `pub fn refresh_palette_entries(&mut self)` that scans `[AppConfig::default_config_dir().join("dicts")]` via `discover_installed_locales` and stores `build_palette_entries(&locales)`.
     - Call `self.refresh_palette_entries()` when `Action::OpenCommandPalette` is dispatched, and lazily if `self.palette_entries.is_empty()` inside `render_frame` / `handle_key_event` when `self.state.overlay == OverlayMode::CommandPalette`.
     - Reuse `&self.palette_entries` in both `render_frame` and `handle_key_event` (no filesystem scan on every keypress or redraw).
   - Fix `BUG-0004`: in `handle_key_event`, for every blocking modal arm (`CommandPalette`, `SpellQuickFix`, `ConfirmQuit | ConfirmCloseBuffer`, `OpenFilePrompt | SaveAsPrompt`), call its handler and unconditionally `return Ok(())` so unhandled keys never fall through to mutate the editor.
   - Fix `BUG-0005`: at the top of `handle_mouse_event`, if `is_blocking_modal(self.state.overlay)`, return `Ok(false)` immediately.
   - Fix `BUG-0009`: in `handle_mouse_event`'s preview wheel scroll branch, compute `max_scroll = self.preview_pane.total_lines.saturating_sub(self.preview_pane.viewport_height.max(1))`.
4. Unit tests in `crates/scriptui-tui/src/app.rs`:
   - `blocking_modals_trap_unhandled_keys_and_ignore_mouse`: open `ConfirmQuit` on dirty `"hello"`, send `'x'`, `Backspace`, and a left-click in the editor; assert document text is still `"hello"` and `overlay` remains `ConfirmQuit`.
   - `alt_enter_resets_find_focus_and_palette_entries_cached`: verify `FindFocus::Query` after `Alt+Enter` from `Replacement` focus, and verify `app.palette_entries` is populated once on `Ctrl+P`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'is_blocking_modal' crates/scriptui-tui/src/app.rs && grep -qE 'palette_entries' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 6C: Unicode Display Width & Tab-Stop Expansion in `compute_wrapped_layout` (`BUG-0003`)

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0003`

```text
[STEP_ID]
STEP 6C: Add unicode-width to scriptui-core and update compute_wrapped_layout in layout.rs to measure Unicode display cell width and tab-stop expansion (BUG-0003).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/Cargo.toml
  - Cargo.lock
  - crates/scriptui-core/src/layout.rs
READ_ONLY_REFS:
  - crates/scriptui-core/Cargo.toml:1-19 (dependencies)
  - crates/scriptui-core/src/layout.rs:1-145 (VisualLine, WrappedLayout, compute_indent_guides, compute_wrapped_layout)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn compute_wrapped_layout\b' crates/scriptui-core/src/layout.rs

[MUTATION_SPEC]
1. crates/scriptui-core/Cargo.toml:
   - Add `unicode-width = "0.2"` under `[dependencies]` (already present in `Cargo.lock` as `0.2.0`, so no network fetch is required).
2. crates/scriptui-core/src/layout.rs:
   - Import `unicode_width::UnicodeWidthChar`.
   - Add `pub fn char_display_width(ch: char, col_cells: usize, tab_width: usize) -> usize`:
     - When `ch == '\t'`, return `let step = tab_width.max(1); step - (col_cells % step)`.
     - Otherwise return `UnicodeWidthChar::width(ch).unwrap_or(0)`.
   - Add `pub fn line_display_width(line: &str, tab_width: usize) -> usize` accumulating `char_display_width(ch, col, tab_width)`.
   - Update `compute_wrapped_layout`:
     - Use `line_display_width(&line, tab_width) <= width` for the fast single-line path.
     - In the wrapping loop (`while start < chars.len()`):
       - Walk `idx` from `start` accumulating `cell_width += char_display_width(chars[idx], cell_width, tab_width)` while `cell_width <= width` (always advancing at least 1 char if the very first char exceeds `width`, so the loop always makes progress).
       - Let `max_end = idx`. If `max_end == chars.len()`, `end = max_end`; else if there is a whitespace char (`' '` or `'\t'`) in `start..max_end`, break at `ws + 1`; else hard-break at `max_end`.
3. Unit tests in `crates/scriptui-core/src/layout.rs`:
   - `soft_wrap_accounts_for_cjk_wide_chars_and_tab_stops`:
     - Verify `"日本語テスト"` (6 chars, 12 display cells) wraps into 2 visual lines at `viewport_width = 6` (`char_start..char_end` = `0..3` and `3..6`).
     - Verify `"\tabcdef"` with `tab_width = 4` (display width 4 + 6 = 10) wraps at `viewport_width = 8` (`0..5` or `0..1` at tab boundary, with every wrapped segment having display width `<= 8`).
     - Verify a single wide char `"日"` at `viewport_width = 1` still makes progress and produces 1 visual line `(0..1)` without infinite looping.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn char_display_width\b' crates/scriptui-core/src/layout.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 6D: Cached GFM Parse & Spellcheck Diagnostics per Document Revision

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6D: Cache parsed GfmDocument and spellcheck Vec<SpellDiagnostic> per (buffer_index, document revision, spell_revision) on EditorState and consume the cache in render_editor_viewport and open_spell_quick_fix_at_cursor.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/editor.rs
  - crates/scriptui-tui/src/viewport.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/editor.rs:213-385 (EditorState fields, new, open_spell_quick_fix_at_cursor, switch_spell_locale, add_spell_popup_word_to_personal_dict, ignore_spell_popup_word)
  - crates/scriptui-tui/src/viewport.rs:165-190 (render_editor_viewport GFM parse and spellcheck calls)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn revision\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn render_editor_viewport\b' crates/scriptui-tui/src/viewport.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/editor.rs:
   - Define private struct `AnalysisCache`:
     - `buffer_index: usize`
     - `doc_revision: u64`
     - `spell_revision: u64`
     - `gfm: crate::markdown::GfmDocument`
     - `spell_diagnostics: Vec<crate::spell::SpellDiagnostic>`
   - Add private fields to `EditorState`:
     - `spell_revision: u64` (initialized to `0`)
     - `analysis_cache: Option<AnalysisCache>` (initialized to `None`)
     - `analysis_recomputes: u64` (initialized to `0`)
   - Increment `self.spell_revision = self.spell_revision.wrapping_add(1)` in `switch_spell_locale`, `add_spell_popup_word_to_personal_dict`, and `ignore_spell_popup_word`.
   - Invalidate `self.analysis_cache = None` whenever the active buffer's document is replaced or buffers are closed/switched (`NewBuffer`, `CloseBuffer`, `NextBuffer`, `PrevBuffer`, `confirm_dirty_action` close).
   - Add `pub fn active_analysis(&mut self) -> (&crate::markdown::GfmDocument, &[crate::spell::SpellDiagnostic])`:
     - Checks if `self.analysis_cache` matches `(self.buffers.active_index(), self.buffers.active().document.revision(), self.spell_revision)`.
     - If not, computes `let text = self.buffers.active().document.text(); let gfm = parse_gfm(&text); let spell_diagnostics = self.spell_engine.as_ref().map(|e| e.check_document(&text, &gfm.spell_skip_byte_ranges)).unwrap_or_default();`, stores `AnalysisCache`, and increments `self.analysis_recomputes += 1`.
     - Returns references to `(&cache.gfm, &cache.spell_diagnostics)`.
   - Add `pub fn analysis_recompute_count(&self) -> u64 { self.analysis_recomputes }`.
   - Update `open_spell_quick_fix_at_cursor(&mut self)` to read diagnostics via `self.active_analysis()` instead of reparsing GFM and re-running spellcheck unconditionally.
   - Add unit test `active_analysis_caches_across_unchanged_revisions_and_invalidates_on_edit_or_spell_change` verifying `analysis_recompute_count()` stays `1` on repeated calls without edits, increments to `2` after `insert_text`, and increments to `3` after `ignore_spell_popup_word`.
2. crates/scriptui-tui/src/viewport.rs:
   - In `render_editor_viewport`, replace the direct `parse_gfm(&text)` and `state.spell_engine.as_ref().map(...)` calls with `let (gfm_doc, spell_diagnostics) = { let (g, s) = state.active_analysis(); (g.clone(), s.to_vec()) };` (or borrow directly while rendering so `parse_gfm` and `check_document` are not re-executed on scroll/cursor moves).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn active_analysis\b' crates/scriptui-core/src/editor.rs && grep -qE 'active_analysis' crates/scriptui-tui/src/viewport.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 6E: Bracketed Paste (`EnableBracketedPaste` + `Event::Paste`) Wiring

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6E: Enable bracketed paste in TerminalGuard and wire Event::Paste through TuiApp::handle_paste_event in crates/scriptui-tui.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/terminal.rs
  - crates/scriptui-tui/src/app.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/terminal.rs:1-70 (TerminalGuard::enter, restore_terminal_best_effort)
  - crates/scriptui-tui/src/app.rs:151-324 (handle_key_event, run_event_loop)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct TerminalGuard\b' crates/scriptui-tui/src/terminal.rs && grep -qE 'pub fn run_event_loop\b' crates/scriptui-tui/src/app.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/terminal.rs:
   - Import `DisableBracketedPaste` and `EnableBracketedPaste` from `crossterm::event`.
   - In `TerminalGuard::enter()`, execute `EnableBracketedPaste` alongside `EnterAlternateScreen` and `EnableMouseCapture`.
   - In `restore_terminal_best_effort()`, execute `DisableBracketedPaste` alongside `LeaveAlternateScreen`, `DisableMouseCapture`, and `Show`.
2. crates/scriptui-tui/src/app.rs:
   - Add `pub fn handle_paste_event(&mut self, text: &str) -> ScriptuiResult<()>`:
     - Normalize line endings: `let normalized = text.replace("\r\n", "\n").replace('\r', "\n");`
     - `let single_line = normalized.replace('\n', " ");`
     - Match `self.state.overlay`:
       - `OverlayMode::None` → `self.state.insert_text(&normalized); self.follow_cursor = true;`
       - `OverlayMode::Find` → `self.state.find_replace.query.pattern.push_str(&single_line);` and refresh `update_find_matches()` (capturing `InvalidRegex` into `status_message` without failing).
       - `OverlayMode::Replace` → if `self.find_bar.focus == FindFocus::Replacement`, push `&single_line` to `self.state.find_replace.replacement`; else push `&single_line` to `self.state.find_replace.query.pattern` and refresh `update_find_matches()`.
       - `OverlayMode::CommandPalette` → push `&single_line` to `self.state.palette.query`, reset `self.state.palette.selected_index = 0`.
       - `OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt` → push `&single_line` to `self.state.file_prompt_input`.
       - `OverlayMode::SpellQuickFix | OverlayMode::ConfirmQuit | OverlayMode::ConfirmCloseBuffer` → ignore paste.
     - Return `Ok(())`.
   - In `run_event_loop`: handle `Event::Paste(text)` by calling `if let Err(err) = self.handle_paste_event(&text) { self.state.status_message = Some(err.to_string()); }` and setting `needs_redraw = true;`.
3. Unit tests in `crates/scriptui-tui/src/app.rs`:
   - `bracketed_paste_routes_to_editor_find_bar_and_ignores_confirm_modal`:
     - Pastes `"line1\r\nline2"` in `OverlayMode::None` and asserts buffer text is `"line1\nline2"`.
     - Opens `OverlayMode::Find`, pastes `"line2"`, and asserts `find_replace.query.pattern == "line2"` and `find_replace.matches.len() == 1`.
     - Opens `OverlayMode::ConfirmQuit`, pastes `"ignored"`, and asserts buffer text is still `"line1\nline2"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'EnableBracketedPaste' crates/scriptui-tui/src/terminal.rs && grep -qE 'pub fn handle_paste_event\b' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 6F: Multi-Palette `TestBackend` Snapshot Suite, E2E Integration Tests & Large-File Responsiveness Check

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6F: Create comprehensive Ratatui TestBackend integration test suite in crates/scriptui-tui/tests/tui_integration.rs covering all 5 palettes, preview modes, multi-cursor editing, prose-safe indentation, custom Hunspell locale import, and large-file cached render responsiveness.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/tests/tui_integration.rs (NEW)
READ_ONLY_REFS:
  - crates/scriptui-core/src/theme.rs:1-310 (builtin_theme_names, resolve_theme, CRT Amber palette tokens)
  - crates/scriptui-tui/src/app.rs:42-324 (TuiApp, render_frame, handle_key_event, handle_mouse_event, handle_paste_event)
  - crates/scriptui-tui/src/cli.rs:1-120 (Cli, execute_dict_import)
  - crates/scriptui-tui/src/palette.rs:32-105 (discover_installed_locales, build_palette_entries, handle_palette_key)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No production crate modifications.

[PRE_GATE]
grep -qE 'pub fn active_analysis\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub fn handle_paste_event\b' crates/scriptui-tui/src/app.rs

[MUTATION_SPEC]
1. Create `crates/scriptui-tui/tests/tui_integration.rs` with 5 integration tests:
   - `all_five_builtin_palettes_render_with_distinct_colors_and_crt_amber_gfm_differentials`:
     - Iterates over every name in `scriptui_core::builtin_theme_names()` (`crt_amber`, `phosphor_green`, `ice_cyan`, `monochrome_amber_paper`, `obsidian_copper`), initializes `TuiApp::from_cli`, inserts GFM sample (`"# H1 Title\n## H2 Sub\n**bold** *italic* `code`\n- [x] task\n"`), renders on `TestBackend::new(100, 24)`, and asserts status bar shows the theme name and background cells match `rgb_to_color(theme.palette.background)`.
     - For `crt_amber`, asserts rendered H1 cell fg equals `rgb_to_color(palette.md_h1)` and differs from H2 cell fg (`md_h2`) and inline code cell fg (`md_code_fg`).
   - `preview_modes_sync_scroll_and_independent_wheel_scroll`:
     - Tests `PreviewMode::Collapsed` (edge tab `'◀'` visible), `PreviewMode::Split` (split divider + `"Preview [SYNC]"` visible), and `PreviewMode::Expanded` (full-width preview).
     - Verifies wheel scroll over the preview region disables sync (`!app.preview_pane.sync_enabled`), clamps scroll within `total_lines.saturating_sub(viewport_height.max(1))`, and toggling preview re-enables sync.
   - `multicursor_find_handoff_and_prose_safe_indentation_e2e`:
     - Tests `Ctrl+F` → `"item"` → `Alt+Enter` multi-cursor handoff → typing replacement → `Ctrl+Z` atomic undo restoring original text.
     - Tests prose-safe indentation: `"- first"` + `Enter` continues `"- "`, second `Enter` clears empty list marker, `Tab` indents, and `BackTab+SHIFT` dedents cleanly.
   - `custom_hunspell_locale_import_discovery_and_palette_switch`:
     - Creates temp source `.aff` + `.dic` files for a custom locale `"xx_TEST"` containing `"zorkle"`, imports via `execute_dict_import` into a temp `dicts` dir, verifies `discover_installed_locales(&[dicts_dir])` includes `"xx_TEST"`, builds palette entries, opens `OverlayMode::CommandPalette`, types `"xx_TEST"`, presses `Enter` via `handle_palette_key`, and asserts `state.config.spell_locale == "xx_TEST"` and `"zorkle"` produces zero spell diagnostics.
   - `large_file_cached_analysis_and_viewport_render_responsiveness`:
     - Builds a 5,000-line GFM document (`# Section N\nParagraph with **bold** and `code`.\n\n`), renders 10 consecutive frames while scrolling down via `MouseEventKind::ScrollDown` on `TestBackend::new(100, 30)`, asserts `app.state.analysis_recompute_count() == 1` (zero redundant GFM/spell recomputes during scrolling), and asserts average frame render completes well within a generous CI-safe bound (`< 500ms` total for all 10 scroll frames).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && test -f crates/scriptui-tui/tests/tui_integration.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 6G: `README.md` Keybinding Reference, Theme/Locale Guide & Usage Documentation

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6G: Author README.md documenting ScripTUI architecture, build/run/CLI commands, all 5 built-in retro-futuristic palettes, custom theme and Hunspell dictionary installation, and full CUA keybinding & mouse reference.

[SCOPE_LOCK]
ALLOWED_FILES:
  - README.md (NEW)
READ_ONLY_REFS:
  - crates/scriptui-core/src/keymap.rs:280-360 (Keymap::default_cua bindings)
  - crates/scriptui-core/src/theme.rs:270-305 (builtin_theme_names)
  - crates/scriptui-tui/src/cli.rs:1-80 (Cli flags and dict import subcommand)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
test -f crates/scriptui-tui/tests/tui_integration.rs

[MUTATION_SPEC]
1. Create `README.md` at the repository root covering:
   - Overview of ScripTUI (workspace crates `scriptui-core` and `scriptui-tui`, Ratatui + Crossterm TUI, Ropey rope buffer, GFM syntax highlighting & live styled preview, embedded Hunspell `en_US` spellcheck).
   - Quick Start: `cargo build --release`, `cargo run -p scriptui-tui -- [FILE...]`, CLI flags (`--theme <NAME>`, `--locale <LOCALE>`), and `scriptui dict import --locale <LOCALE> --aff <PATH> --dic <PATH>`.
   - Built-in Palettes table listing all 5 palettes (`crt_amber`, `phosphor_green`, `ice_cyan`, `monochrome_amber_paper`, `obsidian_copper`) and custom TOML theme path (`~/.config/scriptui/themes/`).
   - Complete CUA Keybindings reference table matching `Keymap::default_cua()` (File/Buffer, Editing & Indentation, Find/Replace & Multi-Cursor, Overlays & Preview, Navigation) plus Find/Replace inline shortcuts (`Alt+C`, `Alt+W`, `Alt+R`, `Alt+Enter`, `Ctrl+Enter`, `Tab`).
   - Mouse interactions (left-click caret, drag selection, wheel scroll on editor vs. preview, tab switching, right-click spellcheck quick-fix, preview edge-tab click, split divider drag).
   - Verification commands (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && test -f README.md && grep -qE 'crt_amber' README.md && grep -qE 'Ctrl\+Shift\+P' README.md

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```
