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

**Phase goal (Phase 5):** Fix Phase 4 review findings (dedent-in-whitespace `BUG-0002`, `BackTab+SHIFT` normalization, mouse-wheel scroll snap-back, and idle 20 Hz redraw storm), then build the styled GFM preview pane with synchronized scrolling, inline Find/Replace bar with `Alt+Enter` multi-cursor handoff, fuzzy Command Palette (live palette/locale/indent switching), Spellcheck quick-fix popup, Open/Save-As & Dirty-Exit modals, and full `TuiApp` overlay/mouse integration.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Phase 5 Top-Down UI Container Tree

```text
Root (Vertical)
├── [Length(1)]         TabBar          (open buffers, active highlight, dirty '*' marker)
├── [Min(1)]            MainSplit       (Horizontal, driven by PreviewMode)
│   ├── PreviewMode::Collapsed
│   │   ├── [Min(1)]    EditorPane      (gutter + soft-wrapped GFM text + indent guides + cursors)
│   │   └── [Length(3)] PreviewEdgeTab  (vertical '◀ PREVIEW' toggle handle)
│   ├── PreviewMode::Split(pct)
│   │   ├── [Percentage(100 - pct)] EditorPane
│   │   ├── [Length(1)]             SplitDivider (draggable vertical separator)
│   │   └── [Percentage(pct)]       PreviewPane  (styled GFM: tables, checkboxes, callouts, code, sync-scroll)
│   └── PreviewMode::Expanded
│       └── [Percentage(100)]       PreviewPane
├── [Length(0 | 1 | 2)] FindReplaceBar  (row 0: Find + match count + [Aa][W][.*]; row 1: Replace)
└── [Length(1)]         StatusBar       (telemetry: file/dirty, Ln/Col, cursors, wrap, indent, locale, theme)

Floating Modals (rendered over Root with Clear + centered_rect when OverlayMode != None/Find/Replace):
├── OverlayMode::CommandPalette         (max 60% width × 14 rows: query input + fuzzy filtered actions/themes/locales)
├── OverlayMode::SpellQuickFix          (max 46 cols × suggestions+4 rows: ranked suggestions + Add + Ignore)
├── OverlayMode::OpenFilePrompt / SaveAsPrompt (max 64 cols × 3 rows: single-line path input)
└── OverlayMode::ConfirmQuit / ConfirmCloseBuffer (max 56 cols × 5 rows: [Y] Discard / [N] Cancel)
```

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 5A** | `/goal` | core | `crates/scriptui-core/src/keymap.rs`, `crates/scriptui-core/src/editor.rs` | — |
| 2 | **STEP 5B** | `/goal` | tui | `crates/scriptui-tui/src/viewport.rs`, `crates/scriptui-tui/src/app.rs` | STEP 5A |
| 3 | **STEP 5C** | `/goal` | tui | `crates/scriptui-tui/src/preview_pane.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 5B |
| 4 | **STEP 5D** | `/goal` | tui | `crates/scriptui-tui/src/find_bar.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 5C |
| 5 | **STEP 5E** | `/goal` | tui | `crates/scriptui-tui/src/palette.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 5D |
| 6 | **STEP 5F** | `/goal` | tui | `crates/scriptui-tui/src/modals.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 5E |
| 7 | **STEP 5G** | `/goal` | tui | `crates/scriptui-tui/src/app.rs`, `crates/scriptui-tui/src/mouse.rs` | STEP 5F |

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

### Future phases

Outlines only (goal + step titles). Full step blocks are written at the phase review before each phase starts, followed by `neb plan import --write`, `scripts/phased-step.sh new-phase` (which regenerates `.orrery/phase.toml` for that phase), `orrery ingest STEPPED_IMPLEMENTATION_PLAN.md --marker "### STEP "`, and `orrery config set limits.max_deferred_steps <LIMIT>`.

- **Phase 6: Integration Verification, Performance & Polish**
  - Unicode display width & tab-stop expansion in `compute_wrapped_layout` (`BUG-0003`)
  - Cached GFM parse & spellcheck diagnostics per document revision in `EditorState` / viewport
  - Bracketed paste (`EnableBracketedPaste` + `Event::Paste`) & `Action::Paste` wiring
  - Ratatui `TestBackend` snapshot render tests across all 5 palettes (CRT Amber GFM differentials, collapsed/split/full preview, Find/Replace, Command Palette, Spellcheck popup)
  - End-to-end integration tests (multi-cursor editing, prose-safe smart indentation, GFM sync-scroll, custom Hunspell locale import)
  - Large-file incremental highlight/spellcheck responsiveness benchmark and `README.md` keybinding reference

### Open Questions

_None._

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.
- `BUG-0003`: `compute_wrapped_layout` measures `char` count rather than Unicode cell width and tab stops (scheduled for Phase 6 polish).

---

## Phase 5 Step Blocks

### STEP 5A: Core Phase 4 Review Fixes (`BUG-0002`, `BackTab+SHIFT`) & Overlay Action Helpers

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0002`

```text
[STEP_ID]
STEP 5A: Fix KeyChord::normalized for BackTab+SHIFT, fix handle_indent dedent offset clamping inside deleted whitespace (BUG-0002), and add Phase 5 overlay/modal helpers on EditorState in crates/scriptui-core.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/keymap.rs
  - crates/scriptui-core/src/editor.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/keymap.rs:65-87, 325-340 (KeyChord::normalized, BackTab/Tab bindings)
  - crates/scriptui-core/src/editor.rs:39-48, 213-347, 620-666 (OverlayMode, EditorState, spell/theme methods, handle_indent)
  - crates/scriptui-core/src/search.rs:113-150 (replace_all)
  - crates/scriptui-core/src/spell.rs:153-210 (SpellEngine::load_locale, personal_mut, ignore_word)
  - crates/scriptui-core/src/indent.rs:7-17 (IndentStyle, IndentConfig)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn selected_text\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub fn normalized\b' crates/scriptui-core/src/keymap.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/keymap.rs:
   - In `KeyChord::normalized(self)`:
     - When `self.code == KeyCode::BackTab` and `self.modifiers.shift && !self.modifiers.ctrl && !self.modifiers.alt`, return `Self { code: KeyCode::BackTab, modifiers: KeyModifiers::NONE }`.
   - Add unit test `backtab_with_shift_normalizes_to_dedent` verifying `Keymap::default_cua().lookup(KeyChord::new(KeyCode::BackTab, KeyModifiers::SHIFT)) == Some(Action::Dedent)`.
2. crates/scriptui-core/src/editor.rs:
   - Fix `BUG-0002` dedent edge case in `handle_indent`:
     - In `shift_offset(offset: usize) -> usize`: for each `edit` with `edit.start <= offset`, compute `inserted_len = edit.inserted.chars().count() as isize` and `deleted_len = (edit.end - edit.start) as isize`; if `offset < edit.end` (offset lies strictly inside a deleted range), contribute `inserted_len - (offset - edit.start) as isize` instead of `inserted_len - deleted_len`.
   - Extend `OverlayMode` enum with two new variants:
     - `OpenFilePrompt`
     - `SaveAsPrompt`
   - Add field `pub file_prompt_input: String` to `EditorState` (initialized to `String::new()` in `EditorState::new`).
   - Update `dispatch`:
     - `Action::OpenFile`: clear `self.file_prompt_input`, set `self.overlay = OverlayMode::OpenFilePrompt`.
     - `Action::SaveAs`: set `self.file_prompt_input` to the active buffer's existing path string (or empty), set `self.overlay = OverlayMode::SaveAsPrompt`.
     - `Action::Save`: if `self.buffers.active().document.path().is_none()`, set `self.file_prompt_input.clear()` and `self.overlay = OverlayMode::SaveAsPrompt` (instead of returning `MissingPath`), else save active buffer and set `self.status_message = Some("Saved".to_string())`.
   - Add methods on `EditorState`:
     - `pub fn switch_spell_locale(&mut self, locale: &str, search_dirs: &[PathBuf]) -> ScriptuiResult<()>`: loads `SpellEngine::load_locale(locale, search_dirs)?`, sets `self.spell_engine = Some(engine)` and `self.config.spell_locale = locale.to_string()`.
     - `pub fn add_spell_popup_word_to_personal_dict(&mut self) -> bool`: if `self.spell_popup` and `self.spell_engine` are `Some`, calls `engine.personal_mut().add(&popup.diagnostic.word)`, clears `self.spell_popup = None`, sets `self.overlay = OverlayMode::None`, returns `true`.
     - `pub fn ignore_spell_popup_word(&mut self) -> bool`: if `self.spell_popup` and `self.spell_engine` are `Some`, calls `engine.ignore_word(&popup.diagnostic.word)`, clears `self.spell_popup = None`, sets `self.overlay = OverlayMode::None`, returns `true`.
     - `pub fn replace_active_match(&mut self) -> ScriptuiResult<bool>`: refreshes `update_find_matches()?` if needed; if `active_match` points to a valid `SearchMatch` `m`, replaces `m.start..m.end` with `self.find_replace.replacement` via `apply_raw_edit`, records an `EditTransaction` in undo history, moves caret to end of replacement, refreshes `update_find_matches()?`, and returns `Ok(true)`; else `Ok(false)`.
     - `pub fn replace_all_matches(&mut self) -> ScriptuiResult<usize>`: calls `crate::search::replace_all(&mut active.document, &query, &replacement)?`; if non-empty, records an `EditTransaction`, refreshes `update_find_matches()?`, and returns `Ok(count)`.
     - `pub fn toggle_indent_style(&mut self)`: cycles `self.config.indent.style`: `Spaces(4) => Spaces(2)`, `Spaces(2) => Tabs`, `_ => Spaces(4)`.
     - `pub fn confirm_dirty_action(&mut self, force: bool)`: when `force` is `true`, if `self.overlay == OverlayMode::ConfirmQuit`, sets `self.should_quit = true` and `self.overlay = OverlayMode::None`; if `self.overlay == OverlayMode::ConfirmCloseBuffer`, calls `self.buffers.close_active(true)`, clears `find_replace.matches` & `active_match`, and sets `self.overlay = OverlayMode::None`. When `force` is `false`, sets `self.overlay = OverlayMode::None`.
3. Unit tests in `crates/scriptui-core/src/editor.rs`:
   - `dedent_inside_leading_whitespace_clamps_caret_to_line_start`: on `"one\n    two"`, places caret at `(1, 2)` (offset 6), dispatches `Action::Dedent`, and asserts caret is at `(1, 0)` (offset 4), not on line 0.
   - `replace_active_and_all_and_spell_dict_helpers`: verifies `replace_active_match`, `replace_all_matches`, `add_spell_popup_word_to_personal_dict`, `ignore_spell_popup_word`, `toggle_indent_style`, and `confirm_dirty_action`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn replace_active_match\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub fn confirm_dirty_action\b' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 5B: TUI Scroll Follow-Cursor Decoupling & Dirty-Redraw Event Loop

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5B: Decouple mouse-wheel scrolling from ensure_cursor_visible via follow_cursor flag and gate TuiApp::run_event_loop redraws on input events in crates/scriptui-tui.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/viewport.rs
  - crates/scriptui-tui/src/app.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/viewport.rs:96-155 (render_editor_viewport)
  - crates/scriptui-tui/src/app.rs:31-75, 116-208 (TuiApp, render_frame, handle_key_event, handle_mouse_event, run_event_loop)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn render_editor_viewport\b' crates/scriptui-tui/src/viewport.rs && grep -qE 'pub struct TuiApp\b' crates/scriptui-tui/src/app.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/viewport.rs:
   - Update signature of `render_editor_viewport` to:
     `pub fn render_editor_viewport(frame: &mut Frame, area: Rect, state: &mut EditorState, follow_cursor: bool) -> (ViewportMetrics, WrappedLayout)`
   - When `follow_cursor` is `true`, call `ensure_cursor_visible(...)` as before.
   - When `follow_cursor` is `false`, only clamp `active.scroll_line = active.scroll_line.min(layout.len_visual_lines().saturating_sub(1))` (and set `active.scroll_col = 0` if `soft_wrap`) without snapping `scroll_line` back to `cursor_visual`.
   - Update existing unit tests in `viewport.rs` to pass `true` for `follow_cursor`, and add unit test `viewport_preserves_manual_scroll_when_follow_cursor_is_false` verifying that with 20 lines, caret on line 0, `scroll_line = 10`, and `follow_cursor = false`, `active.scroll_line` stays `10` after rendering.
2. crates/scriptui-tui/src/app.rs:
   - Add `pub follow_cursor: bool` to `TuiApp` (initialized to `true` in `from_cli`).
   - In `render_frame`, pass `self.follow_cursor` to `render_editor_viewport(frame, editor_rect, &mut self.state, self.follow_cursor)`.
   - In `handle_key_event`, set `self.follow_cursor = true` when a key is processed (`KeyEventKind::Press`).
   - In `handle_mouse_event`, set `self.follow_cursor = false` when `mouse.kind` is `MouseEventKind::ScrollUp | MouseEventKind::ScrollDown`, and set `self.follow_cursor = true` on `MouseEventKind::Down(MouseButton::Left | MouseButton::Right)`.
   - In `run_event_loop`, add `let mut needs_redraw = true;` before the loop; only call `guard.terminal_mut().draw(...)` when `needs_redraw` is `true` (then reset `needs_redraw = false`), and set `needs_redraw = true` when `Event::Key`, `Event::Mouse`, or `Event::Resize` is read.
   - Add unit test `mouse_wheel_scroll_not_snapped_back_on_next_render` in `app.rs` verifying that after rendering a 20-line document, handling `ScrollDown`, and re-rendering, `scroll_line` remains `3` until a key event re-enables `follow_cursor`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'follow_cursor: bool' crates/scriptui-tui/src/viewport.rs && grep -qE 'needs_redraw' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 5C: Styled GFM Preview Pane Widget with Synchronized Scrolling

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5C: Implement styled GFM PreviewPane renderer with synchronized source-line scrolling in crates/scriptui-tui/src/preview_pane.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/preview_pane.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/preview.rs:8-51 (PreviewLine, PreviewDocument, source_line_to_preview_line, preview_line_to_source_line)
  - crates/scriptui-tui/src/layout_tree.rs:80-112 (rgb_to_color, segment_to_style)
  - crates/scriptui-tui/src/lib.rs:1-21 (module declarations & re-exports)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn source_line_to_preview_line\b' crates/scriptui-core/src/preview.rs && grep -qE 'pub fn segment_to_style\b' crates/scriptui-tui/src/layout_tree.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod preview_pane;` and `pub use preview_pane::{PreviewPaneState, compute_synced_preview_scroll, render_preview_pane};`.
2. crates/scriptui-tui/src/preview_pane.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct PreviewPaneState { pub scroll_line: usize, pub total_lines: usize, pub sync_enabled: bool }` with `impl Default` setting `scroll_line: 0, total_lines: 0, sync_enabled: true`.
   - `pub fn compute_synced_preview_scroll(preview_doc: &PreviewDocument, editor_top_source_line: usize, viewport_height: usize) -> usize`:
     - Calls `let target = preview_doc.source_line_to_preview_line(editor_top_source_line);`
     - Clamps `target` to `0..=preview_doc.lines.len().saturating_sub(viewport_height.max(1))`.
   - `pub fn render_preview_pane(frame: &mut Frame, area: Rect, state: &EditorState, pane_state: &mut PreviewPaneState, editor_top_source_line: usize)`:
     - Early-returns if `area.width < 2 || area.height < 2`.
     - Computes `inner_width = area.width.saturating_sub(2).max(1) as usize` and `inner_height = area.height.saturating_sub(2).max(1) as usize`.
     - Builds `preview_doc = state.build_active_preview(inner_width)`.
     - Updates `pane_state.total_lines = preview_doc.lines.len()`.
     - If `pane_state.sync_enabled`, sets `pane_state.scroll_line = compute_synced_preview_scroll(&preview_doc, editor_top_source_line, inner_height)`; else clamps `pane_state.scroll_line` to `0..=pane_state.total_lines.saturating_sub(1)`.
     - Converts each visible `PreviewLine` in `pane_state.scroll_line..(pane_state.scroll_line + inner_height)` to a Ratatui `Line` by mapping each `StyledSegment` to `Span::styled(seg.text.clone(), style)` via `segment_to_style(seg)` (filling `bg` with `theme.palette.background` when `seg.bg.is_none()`).
     - Renders inside a `Block::default().borders(Borders::ALL)` titled `" Preview [SYNC] "` when `sync_enabled` else `" Preview "`, styled with `theme.palette.border_default`, `theme.palette.foreground`, and `theme.palette.background`.
3. Tests in `crates/scriptui-tui/src/preview_pane.rs` (`#[cfg(test)] mod tests`):
   - `preview_pane_renders_styled_gfm_and_syncs_scroll`: populates an `EditorState` with a GFM heading, task list (`- [x] Done`), alert (`> [!WARNING]\n> Watch out`), and pipe table; renders on `TestBackend::new(60, 14)`, asserts heading cell fg equals `rgb_to_color(theme.palette.md_h1)`, rendered text contains `"✓"` / `"[WARNING]"` / box-drawing `'┌'` or `'│'`, and verifies `compute_synced_preview_scroll` advances when `editor_top_source_line` increases.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn render_preview_pane\b' crates/scriptui-tui/src/preview_pane.rs && grep -qE 'pub fn compute_synced_preview_scroll\b' crates/scriptui-tui/src/preview_pane.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 5D: Inline Find/Replace Bar Widget & Interactive Key Router

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5D: Implement inline Find/Replace bar widget and keyboard router (plain/regex/case/whole-word toggles and Alt+Enter multi-cursor handoff) in crates/scriptui-tui/src/find_bar.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/find_bar.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/editor.rs:39-67, 268-274 (OverlayMode, FindReplaceState, update_find_matches, replace_active_match, replace_all_matches)
  - crates/scriptui-tui/src/layout_tree.rs:16-45, 80-83 (find_bar region, rgb_to_color)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn replace_active_match\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub fn render_preview_pane\b' crates/scriptui-tui/src/preview_pane.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod find_bar;` and `pub use find_bar::{FindBarState, FindFocus, handle_find_bar_key, render_find_bar};`.
2. crates/scriptui-tui/src/find_bar.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub enum FindFocus { #[default] Query, Replacement }`
   - `#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)] pub struct FindBarState { pub focus: FindFocus }`
   - `pub fn render_find_bar(frame: &mut Frame, area: Rect, state: &EditorState, bar_state: FindBarState)`:
     - Early-returns if `area.width == 0 || area.height == 0`.
     - Computes match counter string: if `state.find_replace.matches.is_empty()` then `"0/0"`, else `format!("{}/{}", state.find_replace.active_match.unwrap_or(0) + 1, state.find_replace.matches.len())`.
     - Renders row 0 on `theme.palette.surface`: `" Find: "` + `state.find_replace.query.pattern` (highlighted with `accent` when `focus == FindFocus::Query`) + match counter + toggle pills `[Aa]` (accent when `case_sensitive`), `[W]` (accent when `whole_word`), `[.*]` (accent when `use_regex`), and `"[Alt+Enter: All]"`.
     - When `state.overlay == OverlayMode::Replace` and `area.height >= 2`, renders row 1: `" Replace: "` + `state.find_replace.replacement` (highlighted with `accent` when `focus == FindFocus::Replacement`) + `" [Enter: Replace] [Ctrl+Enter: All]"`.
   - `pub fn handle_find_bar_key(key: KeyEvent, state: &mut EditorState, bar_state: &mut FindBarState) -> ScriptuiResult<bool>`:
     - Returns `Ok(false)` if `state.overlay` is neither `OverlayMode::Find` nor `OverlayMode::Replace`.
     - `KeyCode::Esc` → sets `state.overlay = OverlayMode::None`, resets `bar_state.focus = FindFocus::Query`, returns `Ok(true)`.
     - `KeyCode::Tab` / `KeyCode::BackTab` → when `OverlayMode::Replace`, toggles `bar_state.focus` between `Query` and `Replacement`, returns `Ok(true)`.
     - `KeyCode::Enter` with `ALT` → dispatches `Action::SelectAllOccurrences`, sets `state.overlay = OverlayMode::None`, returns `Ok(true)`.
     - `KeyCode::Enter` with `CONTROL` (in `OverlayMode::Replace`) → calls `state.replace_all_matches()?`, sets `status_message`, returns `Ok(true)`.
     - `KeyCode::Enter` with `SHIFT` → dispatches `Action::FindPrev`, returns `Ok(true)`.
     - `KeyCode::Enter` (no modifiers) → if `OverlayMode::Replace && bar_state.focus == FindFocus::Replacement`, calls `state.replace_active_match()?`; else dispatches `Action::FindNext`; returns `Ok(true)`.
     - `KeyCode::Char('c')` with `ALT` → toggles `query.case_sensitive`, refreshes `update_find_matches()?`, returns `Ok(true)`.
     - `KeyCode::Char('w')` with `ALT` → toggles `query.whole_word`, refreshes `update_find_matches()?`, returns `Ok(true)`.
     - `KeyCode::Char('r')` with `ALT` → toggles `query.use_regex`, refreshes `update_find_matches` (recording `InvalidRegex` into `status_message` without crashing), returns `Ok(true)`.
     - `KeyCode::Backspace` → pops last char from focused field (`query.pattern` or `replacement`); refreshes matches if editing `Query`; returns `Ok(true)`.
     - `KeyCode::Char(c)` (without `CONTROL | ALT`) → pushes `c` to focused field; refreshes matches if editing `Query` (capturing regex errors into `status_message`); returns `Ok(true)`.
     - Any other key returns `Ok(false)`.
3. Tests in `crates/scriptui-tui/src/find_bar.rs` (`#[cfg(test)] mod tests`):
   - `find_bar_typing_toggles_replace_and_alt_enter_multicursor`: opens `OverlayMode::Replace` on `"foo bar foo"`, types `"foo"`, asserts rendered bar shows `"1/2"` and `[Aa]`, toggles `Alt+c`, tabs to replacement, types `"baz"`, triggers `Alt+Enter` on `"foo"` and verifies 2 multi-cursor selections are active and overlay closes.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn render_find_bar\b' crates/scriptui-tui/src/find_bar.rs && grep -qE 'pub fn handle_find_bar_key\b' crates/scriptui-tui/src/find_bar.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 5E: Fuzzy Command Palette Overlay (Themes, Locales, Toggles & Actions)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5E: Implement fuzzy Command Palette overlay widget and key handler supporting editor commands, live theme switching, spellcheck locale switching, and indent-style cycling in crates/scriptui-tui/src/palette.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/palette.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/editor.rs:69-205, 341-346 (CommandEntry, CommandPaletteState, switch_theme, switch_spell_locale, toggle_indent_style)
  - crates/scriptui-core/src/theme.rs:279-290 (builtin_theme_names)
  - crates/scriptui-tui/src/layout_tree.rs:80-83 (rgb_to_color)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn handle_find_bar_key\b' crates/scriptui-tui/src/find_bar.rs && grep -qE 'pub fn builtin_theme_names\b' crates/scriptui-core/src/theme.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod palette;` and `pub use palette::{PaletteEntry, PaletteItemKind, build_palette_entries, discover_installed_locales, filter_palette_entries, handle_palette_key, render_command_palette};`.
2. crates/scriptui-tui/src/palette.rs (NEW) — implement:
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub enum PaletteItemKind { Action(Action), Theme(String), Locale(String), ToggleIndentStyle }`
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct PaletteEntry { pub id: String, pub label: String, pub kind: PaletteItemKind }`
   - `pub fn discover_installed_locales(dict_dirs: &[PathBuf]) -> Vec<String>`:
     - Always includes `"en_US"`; scans each dir in `dict_dirs` for `<locale>.dic` files that also have a sibling `<locale>.aff`, returning a sorted deduplicated `Vec<String>`.
   - `pub fn build_palette_entries(extra_locales: &[String]) -> Vec<PaletteEntry>`:
     - Starts with `CommandPaletteState::default_commands()` mapped to `PaletteItemKind::Action(cmd.action)`.
     - Appends `"indent.cycle_style"` / `"Indent: Cycle Style (Spaces 4 / Spaces 2 / Tabs)"` (`PaletteItemKind::ToggleIndentStyle`).
     - Appends one entry per `&name` in `builtin_theme_names()`: `id: format!("theme.{name}")`, `label: format!("Theme: {name}")`, `kind: PaletteItemKind::Theme(name.to_string())`.
     - Appends one entry per locale in `extra_locales` (deduplicated, always ensuring `"en_US"` is included): `id: format!("locale.{loc}")`, `label: format!("Spell Locale: {loc}")`, `kind: PaletteItemKind::Locale(loc.clone())`.
   - `pub fn filter_palette_entries<'a>(entries: &'a [PaletteEntry], query: &str) -> Vec<&'a PaletteEntry>`:
     - Substring matches on `label`/`id` (case-insensitive) first, followed by ordered-subsequence fuzzy matches.
   - `pub fn render_command_palette(frame: &mut Frame, area: Rect, state: &EditorState, entries: &[PaletteEntry])`:
     - Computes centered modal rect (`width = area.width.min(60).max(20).min(area.width)`, `height = area.height.min(14).max(5).min(area.height)`).
     - Clears region with `ratatui::widgets::Clear`, renders bordered `Block` titled `" Command Palette "` with `border_focused`.
     - Row 0 inside block: `"> "` + `state.palette.query`.
     - Remaining rows: filtered entries, highlighting `state.palette.selected_index` with `accent` bg / `background` fg (bold).
   - `pub fn handle_palette_key(key: KeyEvent, state: &mut EditorState, entries: &[PaletteEntry], theme_dirs: &[PathBuf], dict_dirs: &[PathBuf]) -> ScriptuiResult<bool>`:
     - Returns `Ok(false)` if `state.overlay != OverlayMode::CommandPalette`.
     - `KeyCode::Esc` → closes overlay (`OverlayMode::None`), clears `state.palette.query`, returns `Ok(true)`.
     - `KeyCode::Up` / `KeyCode::Down` → moves `state.palette.selected_index` within filtered list bounds, returns `Ok(true)`.
     - `KeyCode::Backspace` → pops char from `state.palette.query`, resets `selected_index = 0`, returns `Ok(true)`.
     - `KeyCode::Char(c)` (without `CONTROL | ALT`) → pushes `c` to `state.palette.query`, resets `selected_index = 0`, returns `Ok(true)`.
     - `KeyCode::Enter` → executes selected `PaletteEntry`:
       - closes `OverlayMode::None` and clears `state.palette.query` first (so actions like `OpenFind` can set their own overlay),
       - `Action(a)` → `state.dispatch(a)?`,
       - `Theme(name)` → `state.switch_theme(&name, theme_dirs)?` and sets `status_message`,
       - `Locale(loc)` → `state.switch_spell_locale(&loc, dict_dirs)?` and sets `status_message`,
       - `ToggleIndentStyle` → `state.toggle_indent_style()` and sets `status_message`,
       - returns `Ok(true)`.
3. Tests in `crates/scriptui-tui/src/palette.rs` (`#[cfg(test)] mod tests`):
   - `command_palette_fuzzy_filters_and_switches_theme_and_renders`: opens `OverlayMode::CommandPalette`, types `"phgr"`, verifies `"Theme: phosphor_green"` is top fuzzy match, renders on `TestBackend::new(80, 20)` asserting `"Command Palette"` and `"phosphor_green"`, sends `Enter`, and asserts `state.theme.name == "phosphor_green"` and `state.overlay == OverlayMode::None`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn render_command_palette\b' crates/scriptui-tui/src/palette.rs && grep -qE 'pub fn handle_palette_key\b' crates/scriptui-tui/src/palette.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 5F: Spellcheck Quick-Fix Popup, Open/Save-As Prompt & Dirty-Exit Modals

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5F: Implement Spellcheck quick-fix popup, Open/Save-As file path prompt, and Dirty-Exit/Close confirmation modal widgets and key routers in crates/scriptui-tui/src/modals.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/modals.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/editor.rs:39-48, 206-211, 276-340 (OverlayMode, SpellQuickFixState, apply_selected_spell_fix, add_spell_popup_word_to_personal_dict, ignore_spell_popup_word, confirm_dirty_action)
  - crates/scriptui-tui/src/layout_tree.rs:80-83 (rgb_to_color)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn handle_palette_key\b' crates/scriptui-tui/src/palette.rs && grep -qE 'pub fn confirm_dirty_action\b' crates/scriptui-core/src/editor.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod modals;` and `pub use modals::{centered_rect, handle_confirm_modal_key, handle_file_prompt_key, handle_spell_quick_fix_key, render_confirm_modal, render_file_prompt, render_spell_quick_fix};`.
2. crates/scriptui-tui/src/modals.rs (NEW) — implement:
   - `pub fn centered_rect(area: Rect, max_width: u16, max_height: u16) -> Rect`: returns a centered `Rect` clamped inside `area`.
   - `pub fn render_spell_quick_fix(frame: &mut Frame, area: Rect, state: &EditorState)`:
     - When `let Some(popup) = &state.spell_popup`, renders a `Clear` + bordered modal (`" Spellcheck: <word> "`) listing each suggestion in `popup.suggestions`, followed by `"Add '<word>' to Personal Dictionary"` and `"Ignore '<word>' (Session)"`, highlighting `popup.selected_index` with `accent`/`background` (bold).
   - `pub fn handle_spell_quick_fix_key(key: KeyEvent, state: &mut EditorState) -> ScriptuiResult<bool>`:
     - Returns `Ok(false)` if `state.overlay != OverlayMode::SpellQuickFix`.
     - `Esc` → clears `spell_popup = None`, `overlay = OverlayMode::None`, returns `Ok(true)`.
     - `Up` / `Down` → moves `popup.selected_index` across `0..(suggestions.len() + 2)`, returns `Ok(true)`.
     - `Enter` → if `selected_index < suggestions.len()` calls `state.apply_selected_spell_fix()?`; else if `selected_index == suggestions.len()` calls `state.add_spell_popup_word_to_personal_dict()`; else calls `state.ignore_spell_popup_word()`; returns `Ok(true)`.
   - `pub fn render_confirm_modal(frame: &mut Frame, area: Rect, state: &EditorState)`:
     - Renders `Clear` + bordered modal for `OverlayMode::ConfirmQuit` (`" Unsaved Changes — Quit? "` / `"Discard unsaved changes and quit? [Y]es / [N]o"`) or `OverlayMode::ConfirmCloseBuffer` (`" Unsaved Changes — Close Buffer? "` / `"Discard unsaved changes and close? [Y]es / [N]o"`).
   - `pub fn handle_confirm_modal_key(key: KeyEvent, state: &mut EditorState) -> bool`:
     - Returns `false` if `state.overlay` is neither `ConfirmQuit` nor `ConfirmCloseBuffer`.
     - `Char('y') | Char('Y') | Enter` → `state.confirm_dirty_action(true)`, returns `true`.
     - `Char('n') | Char('N') | Esc` → `state.confirm_dirty_action(false)`, returns `true`.
   - `pub fn render_file_prompt(frame: &mut Frame, area: Rect, state: &EditorState)`:
     - Renders `Clear` + 3-row centered bordered modal titled `" Open File "` (`OpenFilePrompt`) or `" Save As "` (`SaveAsPrompt`) showing `state.file_prompt_input`.
   - `pub fn handle_file_prompt_key(key: KeyEvent, state: &mut EditorState) -> ScriptuiResult<bool>`:
     - Returns `Ok(false)` if `state.overlay` is neither `OpenFilePrompt` nor `SaveAsPrompt`.
     - `Esc` → sets `overlay = OverlayMode::None`, returns `Ok(true)`.
     - `Backspace` → pops char from `state.file_prompt_input`, returns `Ok(true)`.
     - `Char(c)` (without `CONTROL | ALT`) → pushes `c` to `state.file_prompt_input`, returns `Ok(true)`.
     - `Enter` → trims `state.file_prompt_input`; if non-empty:
       - `OpenFilePrompt` → `state.buffers.open_file(&PathBuf::from(&path))?`, clears `find_replace.matches` & `active_match`, sets `status_message`, closes overlay;
       - `SaveAsPrompt` → `state.buffers.active_mut().document.save_as(&PathBuf::from(&path))?`, sets `status_message`, closes overlay;
       - returns `Ok(true)`.
3. Tests in `crates/scriptui-tui/src/modals.rs` (`#[cfg(test)] mod tests`):
   - `spell_popup_confirm_modal_and_save_as_prompt_roundtrip`:
     - Opens `SpellQuickFix` on `"hello wrld"`, renders on `TestBackend::new(70, 16)`, asserts `"Add 'wrld' to Personal Dictionary"` appears, selects `"Ignore"` and verifies `"wrld"` is no longer flagged.
     - Marks buffer dirty, triggers `Action::Quit` → `ConfirmQuit`, renders modal, sends `'y'` and asserts `state.should_quit == true`.
     - Opens `SaveAsPrompt`, types a temp file path, sends `Enter`, and asserts the file exists on disk with expected contents.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn render_spell_quick_fix\b' crates/scriptui-tui/src/modals.rs && grep -qE 'pub fn handle_file_prompt_key\b' crates/scriptui-tui/src/modals.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 5G: Full Overlay, Styled Preview & Preview-Scroll Integration in `TuiApp` and `MouseRouter`

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5G: Wire styled GFM PreviewPane, Find/Replace bar, Command Palette, Spellcheck popup, File/Confirm modals, and preview-region mouse wheel scrolling into TuiApp and MouseRouter in crates/scriptui-tui.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/app.rs
  - crates/scriptui-tui/src/mouse.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/preview_pane.rs:1-80 (PreviewPaneState, render_preview_pane)
  - crates/scriptui-tui/src/find_bar.rs:1-90 (FindBarState, render_find_bar, handle_find_bar_key)
  - crates/scriptui-tui/src/palette.rs:1-110 (discover_installed_locales, build_palette_entries, render_command_palette, handle_palette_key)
  - crates/scriptui-tui/src/modals.rs:1-120 (render_spell_quick_fix, handle_spell_quick_fix_key, render_confirm_modal, handle_confirm_modal_key, render_file_prompt, handle_file_prompt_key)
  - crates/scriptui-tui/src/mouse.rs:13-26, 154-175 (MouseContext, MouseRouter, ScrollUp/ScrollDown)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn render_spell_quick_fix\b' crates/scriptui-tui/src/modals.rs && grep -qE 'pub fn render_preview_pane\b' crates/scriptui-tui/src/preview_pane.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/mouse.rs:
   - Add `pub fn is_over_preview(regions: ScreenRegions, col: u16, row: u16) -> bool` checking `regions.preview.is_some_and(|r| rect_contains(r, col, row))`.
   - In `MouseRouter::handle_mouse`: keep editor scroll handling on `ScrollUp`/`ScrollDown` when NOT over preview (caller `TuiApp::handle_mouse_event` checks `is_over_preview` first to scroll `preview_pane` independently).
2. crates/scriptui-tui/src/app.rs:
   - Add fields to `TuiApp`:
     - `pub preview_pane: PreviewPaneState`
     - `pub find_bar: FindBarState`
   - Initialize both with `Default::default()` in `TuiApp::from_cli`.
   - In `render_frame`:
     - Determine `editor_top_source_line`: if `wrapped_layout` is `Some(wl)`, use `wl.visual_lines.get(self.state.buffers.active().scroll_line).map_or(0, |vl| vl.logical_line)`; else `self.state.buffers.active().scroll_line`.
     - Replace the Phase 4 plain-text preview block with `render_preview_pane(frame, preview_rect, &self.state, &mut self.preview_pane, editor_top_source_line)`.
     - If `let Some(find_rect) = regions.find_bar`: call `render_find_bar(frame, find_rect, &self.state, self.find_bar)`.
     - After `render_status_bar`, render the active floating modal based on `self.state.overlay`:
       - `OverlayMode::CommandPalette` → build entries with `discover_installed_locales` on `[AppConfig::default_config_dir().join("dicts")]` and call `render_command_palette(...)`,
       - `OverlayMode::SpellQuickFix` → `render_spell_quick_fix(...)`,
       - `OverlayMode::ConfirmQuit | OverlayMode::ConfirmCloseBuffer` → `render_confirm_modal(...)`,
       - `OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt` → `render_file_prompt(...)`,
       - other variants → no floating modal.
   - In `handle_key_event`:
     - After ignoring non-`Press` events, dispatch to the active overlay handler first:
       - `OverlayMode::Find | OverlayMode::Replace` → if `handle_find_bar_key(key, &mut self.state, &mut self.find_bar)?` returns `true`, return `Ok(())`.
       - `OverlayMode::CommandPalette` → build entries + theme/dict dirs, if `handle_palette_key(...)?` returns `true`, return `Ok(())`.
       - `OverlayMode::SpellQuickFix` → if `handle_spell_quick_fix_key(key, &mut self.state)?` returns `true`, return `Ok(())`.
       - `OverlayMode::ConfirmQuit | OverlayMode::ConfirmCloseBuffer` → if `handle_confirm_modal_key(key, &mut self.state)` returns `true`, return `Ok(())`.
       - `OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt` → if `handle_file_prompt_key(key, &mut self.state)?` returns `true`, return `Ok(())`.
     - Otherwise run the existing keymap/typing dispatch (and re-enable `self.preview_pane.sync_enabled = true` on `Action::TogglePreview` / `Action::ExpandPreview`).
   - In `handle_mouse_event`:
     - If `mouse.kind` is `ScrollUp` or `ScrollDown` and `is_over_preview(frame.regions, mouse.column, mouse.row)`:
       - Set `self.preview_pane.sync_enabled = false`, adjust `self.preview_pane.scroll_line` by 3 lines (clamped to `0..=self.preview_pane.total_lines.saturating_sub(1)`), and return `Ok(true)`.
3. End-to-end `TestBackend` tests in `crates/scriptui-tui/src/app.rs`:
   - `tui_app_overlays_and_styled_preview_integration`:
     - Opens split preview (`Ctrl+Shift+P` / `Action::TogglePreview`) on `"# Heading\n\n- [x] Done\n\nfoo bar foo"`, renders, and asserts styled preview title `"Preview [SYNC]"` and `"Heading"` appear.
     - Sends `Ctrl+P`, types `"ice_cyan"`, sends `Enter`, and asserts `app.state.theme.name == "ice_cyan"`.
     - Sends `Ctrl+F`, types `"foo"`, renders, asserts `"1/2"` appears in find bar, sends `Alt+Enter`, and asserts 2 multi-cursor selections exist and overlay is `OverlayMode::None`.
     - Sends `Ctrl+Q` on dirty buffer, asserts `OverlayMode::ConfirmQuit`, renders modal, sends `'y'`, and asserts `app.state.should_quit == true`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'render_preview_pane' crates/scriptui-tui/src/app.rs && grep -qE 'handle_palette_key' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```
