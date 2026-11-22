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

**Phase goal (Phase 3):** Implement the UI-agnostic editor state layer in `scriptui-core`: data-driven modeless CUA `Keymap` & `Action` dispatch, multi-buffer tab management (`BufferManager`) with dirty-exit guard and external reload, toggleable soft word-wrap layout with indent-guide metadata (`WrappedLayout`), and the `EditorState` coordinator uniting overlays, Find/Replace, fuzzy Command Palette, Spellcheck quick-fix, and Preview pane state.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 3A** | `/goal` | keymap | `crates/scriptui-core/src/keymap.rs`, `crates/scriptui-core/src/lib.rs` | — |
| 2 | **STEP 3B** | `/goal` | buffers | `crates/scriptui-core/src/buffers.rs`, `crates/scriptui-core/src/lib.rs` | — |
| 3 | **STEP 3C** | `/goal` | layout | `crates/scriptui-core/src/layout.rs`, `crates/scriptui-core/src/lib.rs` | — |
| 4 | **STEP 3D** | `/goal` | editor | `crates/scriptui-core/src/editor.rs`, `crates/scriptui-core/src/lib.rs` | STEP 3A, STEP 3B, STEP 3C |
| 5 | **STEP 3E** | `/goal` | editor | `crates/scriptui-core/tests/editor_state_pipeline.rs` | STEP 3D |

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

### Future phases

Outlines only (goal + step titles). Full step blocks are written at the phase review before each phase starts, followed by `neb plan import --write`, `scripts/phased-step.sh new-phase` (which regenerates `.orrery/phase.toml` for that phase), `orrery ingest STEPPED_IMPLEMENTATION_PLAN.md --marker "### STEP "`, and `orrery config set limits.max_deferred_steps <LIMIT>`.

- **Phase 4: Ratatui TUI Shell, Chrome, Editor Viewport & Mouse Routing (`scriptui-tui`)**
  - `crates/scriptui-tui` binary crate, `clap` CLI (`scriptui [FILES...]`, `--theme`, `--locale`, `scriptui dict import <locale> <dic> <aff>`), terminal lifecycle (raw mode, alt screen, mouse capture, OSC 52 / clipboard, panic hook)
  - Top-down container layout tree: buffer tab bar → main split (editor pane + right-hand preview edge-tab/pane) → optional inline Find/Replace bar → retro-futuristic telemetry status bar
  - Editor viewport widget: line-number gutter with current-line emphasis, multi-cursor & selection rendering, GFM color differentials, soft-wrap & indent guides
  - Full mouse event router (click cursor, drag selection, scroll wheel, tab clicks, preview edge-tab toggle & split divider drag, right-click spell quick-fix)
- **Phase 5: Expandable GFM Preview Pane, Command Palette & Interactive Overlays (`scriptui-tui`)**
  - Collapsible right-hand preview pane widget (collapsed vertical edge handle `◀ PREVIEW`, resizable split, full-screen toggle) rendering full GFM (box-drawn tables, task checkboxes, styled callouts, nested blockquotes, syntax-highlighted code blocks, footnotes, hard breaks) with synchronized scrolling
  - Fuzzy Command Palette overlay (`Ctrl+P` / `F1`) for live palette switching, spellcheck locale switching, wrap/tab-width toggles, and editor commands
  - Inline Find/Replace bar widget with plain/regex/case/whole-word toggles and `Alt+Enter` Select-All multi-cursor handoff
  - Spellcheck quick-fix popup (`Ctrl+.` / right-click) with ranked suggestions, Add to Personal Dictionary, and Ignore, plus Open/Save-As and Dirty-Exit modal dialogs
- **Phase 6: Integration Verification, Performance & Polish**
  - Ratatui `TestBackend` snapshot render tests across all 5 palettes (CRT Amber GFM differentials, collapsed/split/full preview, Find/Replace, Command Palette, Spellcheck popup)
  - End-to-end integration tests (multi-cursor editing, prose-safe smart indentation, GFM sync-scroll, custom Hunspell locale import)
  - Large-file incremental highlight/spellcheck responsiveness benchmark and `README.md` keybinding reference

### Open Questions

_None._

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.

---

## Phase 3 Step Blocks

### STEP 3A: Data-Driven Keymap and Modeless CUA Action Enum

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3A: Implement UI-agnostic KeyCode, KeyModifiers, KeyChord, Action enum, and data-driven modeless CUA Keymap in crates/scriptui-core/src/keymap.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/keymap.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/lib.rs:1-34 (module declarations and re-exports)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct SpellEngine\b' crates/scriptui-core/src/spell.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod keymap;` and `pub use keymap::{Action, KeyChord, KeyCode, KeyModifiers, Keymap};`.
2. crates/scriptui-core/src/keymap.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)] pub enum KeyCode` with variants: `Char(char)`, `Enter`, `Tab`, `BackTab`, `Backspace`, `Delete`, `Escape`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`, `PageDown`, `F(u8)`.
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)] pub struct KeyModifiers { pub ctrl: bool, pub alt: bool, pub shift: bool }` with associated constants:
     - `pub const NONE: Self` (`false, false, false`)
     - `pub const CTRL: Self` (`true, false, false`)
     - `pub const ALT: Self` (`false, true, false`)
     - `pub const SHIFT: Self` (`false, false, true`)
     - `pub const CTRL_SHIFT: Self` (`true, false, true`)
     - `pub const ALT_SHIFT: Self` (`false, true, true`)
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)] pub struct KeyChord { pub code: KeyCode, pub modifiers: KeyModifiers }` with:
     - `pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self`
     - `pub fn normalized(self) -> Self` (if `self.code` is `KeyCode::Char(c)` and `(self.modifiers.ctrl || self.modifiers.alt)`, normalizes `c` via `c.to_ascii_lowercase()`; otherwise returns `self` unchanged).
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)] pub enum Action` with variants:
     - File / Buffer / App: `Save`, `SaveAs`, `OpenFile`, `NewBuffer`, `CloseBuffer`, `NextBuffer`, `PrevBuffer`, `Quit`
     - History: `Undo`, `Redo`
     - Search / Multi-cursor: `OpenFind`, `OpenReplace`, `FindNext`, `FindPrev`, `SelectAllOccurrences`
     - Overlays / View: `OpenCommandPalette`, `OpenSpellQuickFix`, `TogglePreview`, `ExpandPreview`, `ToggleSoftWrap`, `ToggleLineNumbers`
     - Navigation: `MoveLeft`, `MoveRight`, `MoveUp`, `MoveDown`, `MoveHome`, `MoveEnd`, `PageUp`, `PageDown`
     - Selection: `SelectLeft`, `SelectRight`, `SelectUp`, `SelectDown`, `SelectHome`, `SelectEnd`
     - Editing: `InsertNewline`, `Backspace`, `Delete`, `Indent`, `Dedent`, `Escape`
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct Keymap { bindings: std::collections::HashMap<KeyChord, Action> }` with:
     - `pub fn default_cua() -> Self` binding:
       - `Ctrl+s` → `Save`, `Ctrl+Shift+s` → `SaveAs`, `Ctrl+o` → `OpenFile`, `Ctrl+n` → `NewBuffer`, `Ctrl+w` → `CloseBuffer`, `Ctrl+PageDown` → `NextBuffer`, `Ctrl+PageUp` → `PrevBuffer`, `Ctrl+q` → `Quit`
       - `Ctrl+z` → `Undo`, `Ctrl+y` → `Redo`, `Ctrl+Shift+z` → `Redo`
       - `Ctrl+f` → `OpenFind`, `Ctrl+h` → `OpenReplace`, `F(3)` → `FindNext`, `Shift+F(3)` → `FindPrev`, `Ctrl+Shift+l` → `SelectAllOccurrences`
       - `Ctrl+p` → `OpenCommandPalette`, `F(1)` → `OpenCommandPalette`, `Ctrl+.` → `OpenSpellQuickFix`
       - `Ctrl+Shift+p` → `TogglePreview`, `Alt+p` → `ExpandPreview`, `Alt+z` → `ToggleSoftWrap`
       - `Left`/`Right`/`Up`/`Down`/`Home`/`End`/`PageUp`/`PageDown` (`NONE`) → `Move*`/`Page*`
       - `Shift+Left`/`Shift+Right`/`Shift+Up`/`Shift+Down`/`Shift+Home`/`Shift+End` → `Select*`
       - `Enter` → `InsertNewline`, `Backspace` → `Backspace`, `Delete` → `Delete`, `Tab` → `Indent`, `BackTab` (`NONE`) → `Dedent`, `Shift+Tab` → `Dedent`, `Escape` → `Escape`
     - `impl Default for Keymap` delegating to `Self::default_cua()`
     - `pub fn lookup(&self, chord: KeyChord) -> Option<Action>` (looks up `chord.normalized()`)
     - `pub fn bind(&mut self, chord: KeyChord, action: Action)` (inserts `chord.normalized()`)
     - `pub fn unbind(&mut self, chord: KeyChord) -> Option<Action>` (removes `chord.normalized()`)
     - `pub fn bindings_for(&self, action: Action) -> Vec<KeyChord>`
3. Tests in `crates/scriptui-core/src/keymap.rs` (`#[cfg(test)] mod tests`):
   - `default_cua_bindings_cover_core_actions`: creates `Keymap::default_cua()`, asserts lookups for `Ctrl+s` (`Save`), `Ctrl+f` (`OpenFind`), `Ctrl+h` (`OpenReplace`), `Ctrl+Shift+l` (`SelectAllOccurrences`), `Ctrl+z` (`Undo`), `Ctrl+y` (`Redo`), `Ctrl+p` and `F(1)` (`OpenCommandPalette`), `Ctrl+.` (`OpenSpellQuickFix`), `Shift+Right` (`SelectRight`), `Tab` (`Indent`), `BackTab` (`Dedent`), `Ctrl+Shift+p` (`TogglePreview`), and `Alt+z` (`ToggleSoftWrap`).
   - `lookup_normalizes_ctrl_char_case_and_supports_override`: verifies `lookup(KeyChord::new(KeyCode::Char('S'), KeyModifiers::CTRL)) == Some(Action::Save)`, binds `KeyChord::new(KeyCode::Char('k'), KeyModifiers::CTRL)` to `Action::CloseBuffer`, asserts it resolves, then `unbind` returns `Some(Action::CloseBuffer)` and subsequent `lookup` returns `None`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub enum Action\b' crates/scriptui-core/src/keymap.rs && grep -qE 'pub struct Keymap\b' crates/scriptui-core/src/keymap.rs && grep -qE 'pub fn default_cua\b' crates/scriptui-core/src/keymap.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 3B: Multi-Buffer Tab Manager with Dirty-Exit Guard and External Reload

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3B: Implement BufferManager and ManagedBuffer in crates/scriptui-core/src/buffers.rs for multi-buffer tab lifecycle, dirty-close confirmation guard, and external modification reload.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/buffers.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:19-165 (DocumentBuffer, from_file, save_atomic, has_external_modification, path, is_dirty, len_chars)
  - crates/scriptui-core/src/history.rs:12-55 (UndoStack::new, clear)
  - crates/scriptui-core/src/lib.rs:1-36 (module declarations)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No modifications to buffer.rs or history.rs.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct DocumentBuffer\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub struct UndoStack\b' crates/scriptui-core/src/history.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod buffers;` and `pub use buffers::{BufferId, BufferManager, CloseResult, ManagedBuffer};`.
2. crates/scriptui-core/src/buffers.rs (NEW) — implement:
   - `pub type BufferId = usize;`
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum CloseResult { Closed, NeedsConfirmation }`
   - `#[derive(Debug, Clone)] pub struct ManagedBuffer` with fields:
     - `pub id: BufferId`
     - `pub document: DocumentBuffer`
     - `pub history: UndoStack`
     - `pub scroll_line: usize`
     - `pub scroll_col: usize`
     - `pub external_change_pending: bool`
     and methods:
     - `pub fn display_name(&self) -> String` (returns the UTF-8 file name from `self.document.path()` if present, otherwise `format!("Untitled-{}", self.id)`)
     - `pub fn is_dirty(&self) -> bool` (`self.document.is_dirty()`)
   - `#[derive(Debug, Clone)] pub struct BufferManager { buffers: Vec<ManagedBuffer>, active_index: usize, next_id: BufferId, undo_limit: usize }` with:
     - `pub const DEFAULT_UNDO_LIMIT: usize = 500;`
     - `pub fn new() -> Self` (creates 1 empty `ManagedBuffer` with `id = 1`, `next_id = 2`, `active_index = 0`, `undo_limit = DEFAULT_UNDO_LIMIT`) and `impl Default for BufferManager`
     - `pub fn len(&self) -> usize` and `pub fn is_empty(&self) -> bool`
     - `pub fn active_index(&self) -> usize`
     - `pub fn buffers(&self) -> &[ManagedBuffer]`
     - `pub fn active(&self) -> &ManagedBuffer` (returns `&self.buffers[self.active_index]`, guaranteed valid because `buffers` is never empty)
     - `pub fn active_mut(&mut self) -> &mut ManagedBuffer`
     - `pub fn new_buffer(&mut self) -> BufferId` (allocates `id = self.next_id`, increments `next_id`, pushes a new empty `ManagedBuffer`, sets `active_index` to the new buffer, returns `id`)
     - `pub fn open_file(&mut self, path: &std::path::Path) -> ScriptuiResult<BufferId>`:
       - Deduplication: compares `std::fs::canonicalize(path).ok()` (or raw `path`) against existing open buffers; if already open, switches `active_index` to that buffer and returns `Ok(existing.id)`.
       - Loads `DocumentBuffer::from_file(path)?`.
       - If `self.buffers.len() == 1` and `self.buffers[0]` is untouched (`!is_dirty() && path().is_none() && len_chars() == 0`), replaces `self.buffers[0]` in place (preserving its `id` or assigning a fresh `id`, resetting `history`, `scroll_line`, `scroll_col`, `external_change_pending = false`) and returns its `id`.
       - Otherwise pushes a new `ManagedBuffer`, sets `active_index` to it, and returns its `id`.
     - `pub fn switch_to(&mut self, index: usize) -> bool` (if `index < self.buffers.len()`, updates `active_index` and returns `true`, else `false`)
     - `pub fn next_buffer(&mut self)` (`self.active_index = (self.active_index + 1) % self.buffers.len()`)
     - `pub fn prev_buffer(&mut self)` (`self.active_index = if self.active_index == 0 { self.buffers.len() - 1 } else { self.active_index - 1 }`)
     - `pub fn close_active(&mut self, force: bool) -> CloseResult`:
       - If `!force && self.active().is_dirty()`, returns `CloseResult::NeedsConfirmation`.
       - Removes `self.active_index`; if `self.buffers.is_empty()`, pushes a fresh empty `ManagedBuffer` (using `next_id`) and sets `active_index = 0`; else clamps `active_index = self.active_index.min(self.buffers.len() - 1)`. Returns `CloseResult::Closed`.
     - `pub fn dirty_buffer_ids(&self) -> Vec<BufferId>`
     - `pub fn has_dirty_buffers(&self) -> bool`
     - `pub fn poll_external_modifications(&mut self) -> Vec<BufferId>` (for each buffer, if `buf.document.has_external_modification().unwrap_or(false)` sets `buf.external_change_pending = true`; returns all buffer IDs where `external_change_pending` is `true`)
     - `pub fn reload_active_from_disk(&mut self) -> ScriptuiResult<bool>` (if active buffer has a `path`, reloads `DocumentBuffer::from_file(&path)?`, clears `history`, sets `external_change_pending = false`, returns `Ok(true)`; if no path, returns `Ok(false)`)
3. Tests in `crates/scriptui-core/src/buffers.rs` (`#[cfg(test)] mod tests`):
   - `open_file_reuses_initial_empty_tab_and_deduplicates_path`: writes two temp files `a.md` and `b.md`, opens `a.md` on a fresh `BufferManager` (asserts `len() == 1` and `active().display_name() == "a.md"`), opens `b.md` (asserts `len() == 2` and `active_index() == 1`), re-opens `a.md` (asserts `len() == 2` and `active_index() == 0`).
   - `close_active_guards_dirty_buffer_and_allows_force_close`: inserts text into active buffer, asserts `has_dirty_buffers() == true` and `close_active(false) == CloseResult::NeedsConfirmation`; calls `close_active(true)` and asserts `CloseResult::Closed`, `len() == 1`, and `!active().is_dirty()`.
   - `external_modification_detection_and_reload`: writes a temp file, opens it, rewrites the file on disk (bumping mtime via sleeping or rewriting with different content and checking reload), verifies `reload_active_from_disk().unwrap() == true` loads the updated text and clears `external_change_pending`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct BufferManager\b' crates/scriptui-core/src/buffers.rs && grep -qE 'pub enum CloseResult\b' crates/scriptui-core/src/buffers.rs && grep -qE 'pub fn reload_active_from_disk\b' crates/scriptui-core/src/buffers.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 3C: Toggleable Soft Word-Wrap Layout Calculator and Indent Guides

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3C: Implement toggleable soft word-wrap layout calculator, logical/visual coordinate mapping, and indent-guide metadata in crates/scriptui-core/src/layout.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/layout.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:155-215 (DocumentBuffer::len_lines, line_text)
  - crates/scriptui-core/src/selection.rs:1-15 (Position)
  - crates/scriptui-core/src/lib.rs:1-38 (module declarations)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct DocumentBuffer\b' crates/scriptui-core/src/buffer.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod layout;` and `pub use layout::{VisualLine, WrappedLayout, compute_indent_guides, compute_wrapped_layout};`.
2. crates/scriptui-core/src/layout.rs (NEW) — implement:
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct VisualLine`:
     - `pub logical_line: usize`
     - `pub wrap_index: usize`
     - `pub char_start: usize`
     - `pub char_end: usize`
     - `pub indent_guides: Vec<usize>`
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct WrappedLayout`:
     - `pub visual_lines: Vec<VisualLine>`
     - `pub viewport_width: usize`
     - `pub soft_wrap: bool`
     and methods:
     - `pub fn len_visual_lines(&self) -> usize` (`self.visual_lines.len()`)
     - `pub fn is_empty(&self) -> bool` (`self.visual_lines.is_empty()`)
     - `pub fn logical_to_visual(&self, pos: Position) -> (usize, usize)`:
       - Finds all `VisualLine`s with `vl.logical_line == pos.line`.
       - Picks the visual row where `pos.col >= vl.char_start && pos.col < vl.char_end` (or the last visual row of `pos.line` when `pos.col >= last.char_end`, clamping `visual_col` to `pos.col.saturating_sub(vl.char_start)`).
       - If `pos.line` is beyond the last logical line, returns the end of the last visual line (or `(0, 0)` if empty).
     - `pub fn visual_to_logical(&self, visual_row: usize, visual_col: usize) -> Position`:
       - If `self.visual_lines.is_empty()`, returns `Position { line: 0, col: 0 }`.
       - Clamps `row = visual_row.min(self.visual_lines.len() - 1)`, gets `vl = &self.visual_lines[row]`, and returns `Position { line: vl.logical_line, col: (vl.char_start + visual_col).min(vl.char_end) }`.
   - `pub fn compute_indent_guides(line: &str, tab_width: usize) -> Vec<usize>`:
     - Computes `step = tab_width.max(1)`.
     - Accumulates leading whitespace width across leading `' '` (+1) and `'\t'` (+`step`), stopping at the first non-whitespace char.
     - Computes `levels = leading_width / step` and returns `(0..levels).map(|level| level * step).collect()`.
   - `pub fn compute_wrapped_layout(buf: &DocumentBuffer, viewport_width: usize, soft_wrap: bool, tab_width: usize) -> WrappedLayout`:
     - Clamps `width = viewport_width.max(1)`.
     - For each `line_idx` in `0..buf.len_lines()`:
       - Reads `line = buf.line_text(line_idx).unwrap_or_default()` and collects `chars: Vec<char> = line.chars().collect()`.
       - Computes `guides = compute_indent_guides(&line, tab_width)`.
       - If `!soft_wrap || chars.len() <= width`: pushes one `VisualLine { logical_line: line_idx, wrap_index: 0, char_start: 0, char_end: chars.len(), indent_guides: guides }`.
       - Otherwise loops from `start = 0` while `start < chars.len()`:
         - `max_end = (start + width).min(chars.len())`.
         - If `max_end == chars.len()`, `end = max_end`; else finds the last index `ws` in `start..max_end` where `chars[ws] == ' ' || chars[ws] == '\t'`; if found, `end = ws + 1`, else hard-breaks at `end = max_end`.
         - Emits `VisualLine { logical_line: line_idx, wrap_index, char_start: start, char_end: end, indent_guides: if wrap_index == 0 { guides.clone() } else { Vec::new() } }`.
         - Advances `start = end`, `wrap_index += 1`.
3. Tests in `crates/scriptui-core/src/layout.rs` (`#[cfg(test)] mod tests`):
   - `soft_wrap_breaks_on_word_boundaries_and_hard_breaks_long_words`: tests `"alpha beta gamma"` at `width = 10` (asserts 2 visual lines `"alpha "`/`"beta "` or `"alpha beta"` boundary without splitting `"beta"`, covering full char range `0..16`), and `"abcdefghijkl"` at `width = 5` (asserts 3 visual lines: `0..5`, `5..10`, `10..12`).
   - `no_wrap_mode_preserves_one_visual_line_per_logical_line`: runs `compute_wrapped_layout` with `soft_wrap = false` and `width = 5` on a 2-line buffer `"long line here\nsecond"`; asserts `len_visual_lines() == 2` and `visual_lines[0].char_end == 14`.
   - `logical_visual_roundtrip_and_indent_guides`: tests `"        indented text here"` with `tab_width = 4`, `width = 14`, `soft_wrap = true`; asserts `visual_lines[0].indent_guides == vec![0, 4]`, `visual_lines[1].indent_guides.is_empty()`, and `visual_to_logical(vr, vc) == pos` for positions on both wrapped rows.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct WrappedLayout\b' crates/scriptui-core/src/layout.rs && grep -qE 'pub fn compute_wrapped_layout\b' crates/scriptui-core/src/layout.rs && grep -qE 'pub fn compute_indent_guides\b' crates/scriptui-core/src/layout.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 3D: EditorState Coordinator (Overlays, Find/Replace, Command Palette, Spell Popup, Preview State)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3D: Implement EditorState coordinator in crates/scriptui-core/src/editor.rs uniting BufferManager, Keymap, Find/Replace, fuzzy Command Palette, Spellcheck quick-fix popup, and PreviewMode.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/editor.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/keymap.rs:1-140 (Action, Keymap)
  - crates/scriptui-core/src/buffers.rs:1-160 (BufferManager, CloseResult, ManagedBuffer)
  - crates/scriptui-core/src/layout.rs:1-120 (WrappedLayout, compute_wrapped_layout)
  - crates/scriptui-core/src/config.rs:1-60 (AppConfig)
  - crates/scriptui-core/src/theme.rs:1-75 (Theme, default_theme, resolve_theme)
  - crates/scriptui-core/src/search.rs:1-125 (SearchQuery, SearchMatch, find_all, next_match, prev_match, select_all_occurrences, replace_all)
  - crates/scriptui-core/src/indent.rs:1-90 (compute_enter_indent, indent_lines, dedent_lines, EnterOutcome)
  - crates/scriptui-core/src/spell.rs:1-180 (SpellEngine, SpellDiagnostic)
  - crates/scriptui-core/src/preview.rs:1-70 (PreviewDocument, build_preview)
  - crates/scriptui-core/src/markdown.rs:1-90 (parse_gfm)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub enum Action\b' crates/scriptui-core/src/keymap.rs && grep -qE 'pub struct BufferManager\b' crates/scriptui-core/src/buffers.rs && grep -qE 'pub struct WrappedLayout\b' crates/scriptui-core/src/layout.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod editor;` and `pub use editor::{CommandEntry, CommandPaletteState, EditorState, FindReplaceState, OverlayMode, PreviewMode, SpellQuickFixState};`.
2. crates/scriptui-core/src/editor.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum PreviewMode { Collapsed, Split(u16), Expanded }`
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum OverlayMode { None, Find, Replace, CommandPalette, SpellQuickFix, ConfirmQuit, ConfirmCloseBuffer }`
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct FindReplaceState { pub query: SearchQuery, pub replacement: String, pub matches: Vec<SearchMatch>, pub active_match: Option<usize> }` with `impl Default` (using `SearchQuery::plain("")`).
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct CommandEntry { pub id: &'static str, pub label: &'static str, pub action: Action }`
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct CommandPaletteState { pub query: String, pub selected_index: usize, pub commands: Vec<CommandEntry> }` with:
     - `pub fn default_commands() -> Vec<CommandEntry>` (at least 12 entries covering Save, New Buffer, Close Buffer, Find, Replace, Select All Occurrences, Undo, Redo, Toggle Preview, Expand Preview, Toggle Soft Wrap, Toggle Line Numbers, Spellcheck Quick Fix, Quit)
     - `impl Default for CommandPaletteState`
     - `pub fn filtered(&self) -> Vec<&CommandEntry>` (if `query.trim().is_empty()` returns all commands; otherwise performs case-insensitive fuzzy matching against `label` and `id`, ranking exact substring matches ahead of ordered-subsequence matches, preserving relative order within each tier).
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct SpellQuickFixState { pub diagnostic: SpellDiagnostic, pub suggestions: Vec<String>, pub selected_index: usize }`
   - `pub struct EditorState` with public fields:
     - `pub config: AppConfig`
     - `pub theme: Theme`
     - `pub keymap: Keymap`
     - `pub buffers: BufferManager`
     - `pub preview_mode: PreviewMode`
     - `pub overlay: OverlayMode`
     - `pub find_replace: FindReplaceState`
     - `pub palette: CommandPaletteState`
     - `pub spell_popup: Option<SpellQuickFixState>`
     - `pub spell_engine: Option<SpellEngine>`
     - `pub status_message: Option<String>`
     - `pub should_quit: bool`
   - Methods on `EditorState`:
     - `pub fn new(config: AppConfig) -> ScriptuiResult<Self>` (resolves `config.theme` via `resolve_theme(&config.theme, &[])` falling back to `default_theme()?`, initializes `spell_engine` with `if config.spell_enabled { SpellEngine::default_en_us().ok() } else { None }`, `keymap = Keymap::default_cua()`, `buffers = BufferManager::new()`, `preview_mode = PreviewMode::Collapsed`, `overlay = OverlayMode::None`, `should_quit = false`)
     - `pub fn insert_text(&mut self, text: &str)` (captures `selection_before`, runs `apply_insert(text)`, records `EditTransaction` on `active_mut().history`)
     - `pub fn update_find_matches(&mut self) -> ScriptuiResult<usize>` (calls `find_all(&self.buffers.active().document, &self.find_replace.query)?`, stores `matches`, updates `active_match` to `if matches.is_empty() { None } else { Some(0) }`, returns `Ok(matches.len())`)
     - `pub fn open_spell_quick_fix_at_cursor(&mut self) -> bool` (if `spell_engine` is `Some`, parses active buffer text with `parse_gfm`, runs `check_document(&text, &doc.spell_skip_byte_ranges)`, finds the first diagnostic where `diag.line == pos.line && pos.col >= diag.col_start && pos.col <= diag.col_end` for primary caret `pos`; if found, populates `self.spell_popup = Some(SpellQuickFixState { suggestions: engine.suggest(&diag.word, 5), diagnostic: diag, selected_index: 0 })`, sets `self.overlay = OverlayMode::SpellQuickFix`, returns `true`; else returns `false`)
     - `pub fn apply_selected_spell_fix(&mut self) -> ScriptuiResult<bool>` (if `self.spell_popup` has a valid `suggestions.get(selected_index)` entry, computes char start/end from `diagnostic.line` + `col_start`/`col_end` via `DocumentBuffer::position_to_char`, applies `apply_raw_edit`, moves primary caret to end of replacement, records `EditTransaction`, clears `spell_popup`, sets `overlay = OverlayMode::None`, returns `Ok(true)`; else returns `Ok(false)`)
     - `pub fn switch_theme(&mut self, name: &str, search_dirs: &[std::path::PathBuf]) -> ScriptuiResult<()>` (resolves via `resolve_theme`, updates `self.theme` and `self.config.theme`)
     - `pub fn build_active_preview(&self, width: usize) -> PreviewDocument` (`build_preview(&self.buffers.active().document.text(), &self.theme, width)`)
     - `pub fn dispatch(&mut self, action: Action) -> ScriptuiResult<()>`:
       - `NewBuffer` → `self.buffers.new_buffer()`
       - `CloseBuffer` → if `self.buffers.close_active(false) == CloseResult::NeedsConfirmation`, sets `self.overlay = OverlayMode::ConfirmCloseBuffer`
       - `NextBuffer` / `PrevBuffer` → `buffers.next_buffer()` / `buffers.prev_buffer()`
       - `Quit` → if `self.buffers.has_dirty_buffers()` sets `self.overlay = OverlayMode::ConfirmQuit`, else `self.should_quit = true`
       - `Save` → if active buffer has a path, calls `save_atomic()?` and sets `status_message = Some("Saved".to_string())`; else sets `status_message = Some("No file path (use Save As)".to_string())`
       - `Undo` / `Redo` → delegates to `active_mut().history.undo/redo(&mut active_mut().document)?`
       - `OpenFind` → `self.overlay = OverlayMode::Find`
       - `OpenReplace` → `self.overlay = OverlayMode::Replace`
       - `FindNext` / `FindPrev` → updates matches if needed, jumps primary selection to `next_match`/`prev_match` relative to current caret
       - `SelectAllOccurrences` → calls `select_all_occurrences(&mut active_mut().document, &self.find_replace.query)?`
       - `OpenCommandPalette` → resets `palette.query.clear()`, `palette.selected_index = 0`, sets `self.overlay = OverlayMode::CommandPalette`
       - `OpenSpellQuickFix` → calls `self.open_spell_quick_fix_at_cursor()`
       - `TogglePreview` → `PreviewMode::Collapsed` ↔ `PreviewMode::Split(self.config.preview_split_percent.clamp(20, 80))` (and `Expanded` → `Collapsed`)
       - `ExpandPreview` → `PreviewMode::Expanded` → `PreviewMode::Split(self.config.preview_split_percent.clamp(20, 80))`, other modes → `PreviewMode::Expanded`
       - `ToggleSoftWrap` → toggles `self.config.soft_wrap`
       - `ToggleLineNumbers` → toggles `self.config.line_numbers`
       - `InsertNewline` → parses active buffer with `parse_gfm`, checks `is_line_in_code_block(pos.line).is_some()`, calls `compute_enter_indent`, applies `EnterOutcome::Insert(s)` via `insert_text(&s)` or `EnterOutcome::ClearCurrentLine` (clears current line via `apply_raw_edit` and records `EditTransaction`)
       - `Backspace` / `Delete` → captures `selection_before`, calls `apply_delete(true/false)`, records `EditTransaction`
       - `Indent` / `Dedent` → computes selected line range, captures `selection_before`, calls `indent_lines`/`dedent_lines`, records `EditTransaction`
       - `MoveLeft` / `MoveRight` / `MoveUp` / `MoveDown` / `MoveHome` / `MoveEnd` / `PageUp` / `PageDown` and `SelectLeft` / `SelectRight` / `SelectUp` / `SelectDown` / `SelectHome` / `SelectEnd` → updates active buffer's primary selection cleanly (moving collapses anchor to head; selecting preserves anchor)
       - `Escape` → if `self.overlay != OverlayMode::None`, clears `self.overlay = OverlayMode::None` and `self.spell_popup = None`; else calls `active_mut().document.selection()` collapse to primary
       - `SaveAs` / `OpenFile` → sets `status_message` noting UI dialog trigger (no panic or error)
3. Tests in `crates/scriptui-core/src/editor.rs` (`#[cfg(test)] mod tests`):
   - `editor_state_dispatch_edit_undo_redo_and_smart_enter`: creates `EditorState::new(AppConfig::default()).unwrap()`, inserts `"- item"`, dispatches `Action::InsertNewline`, asserts text is `"- item\n- "`; dispatches `Action::Undo` and asserts `"- item"`; dispatches `Action::Redo` and asserts `"- item\n- "`.
   - `editor_state_find_select_all_and_command_palette_fuzzy_filter`: inserts `"foo bar foo"`, sets `find_replace.query = SearchQuery::plain("foo")`, dispatches `Action::SelectAllOccurrences`, asserts `selection().len() == 2`; sets `palette.query = "prv".to_string()` and asserts `palette.filtered()` contains `"Toggle Preview"` and `"Expand Preview"`.
   - `editor_state_preview_toggle_dirty_quit_guard_and_spell_quick_fix`: tests `TogglePreview` (`Collapsed` → `Split(50)` → `Collapsed`), `ExpandPreview` → `Expanded`, `Quit` on dirty buffer → `OverlayMode::ConfirmQuit` with `should_quit == false`, and `open_spell_quick_fix_at_cursor` + `apply_selected_spell_fix` on `"hello wrld"` with cursor at col 7 replacing `"wrld"` with a valid suggestion.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct EditorState\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub enum PreviewMode\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub enum OverlayMode\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub fn dispatch\b' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 3E: Phase 3 End-to-End Editor State Pipeline Integration Tests

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3E: Add end-to-end integration tests in crates/scriptui-core/tests/editor_state_pipeline.rs exercising Keymap, BufferManager, WrappedLayout, EditorState, GFM preview, and Spellcheck quick-fix together.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/tests/editor_state_pipeline.rs (NEW)
READ_ONLY_REFS:
  - crates/scriptui-core/src/editor.rs:1-140 (EditorState, PreviewMode, OverlayMode, dispatch)
  - crates/scriptui-core/src/keymap.rs:1-100 (Keymap, KeyChord, KeyCode, KeyModifiers, Action)
  - crates/scriptui-core/src/layout.rs:1-90 (compute_wrapped_layout, WrappedLayout)
  - crates/scriptui-core/src/buffers.rs:1-120 (BufferManager)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No modifications to src/*.rs files.

[PRE_GATE]
grep -qE 'pub struct EditorState\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub struct WrappedLayout\b' crates/scriptui-core/src/layout.rs

[MUTATION_SPEC]
1. crates/scriptui-core/tests/editor_state_pipeline.rs (NEW) — add two integration tests:
   - `multi_buffer_keymap_find_multicursor_and_undo_workflow`:
     - Creates a `tempfile::tempdir()` with `first.md` (`"alpha beta alpha\n"`) and `second.md` (`"# Second\n"`).
     - Initializes `EditorState::new(AppConfig::default()).unwrap()` and opens both files via `state.buffers.open_file(...)`.
     - Uses `state.keymap.lookup(...)` to resolve `Ctrl+PageUp` → `Action::PrevBuffer`, dispatches it, asserts active buffer is `"first.md"`.
     - Sets `state.find_replace.query = SearchQuery::plain("alpha")`, looks up `Ctrl+Shift+L` → `Action::SelectAllOccurrences`, dispatches it, inserts `"gamma"` via `state.insert_text("gamma")`, asserts text is `"gamma beta gamma\n"`.
     - Dispatches `Ctrl+Z` (`Undo`) → asserts `"alpha beta alpha\n"`; dispatches `Ctrl+Y` (`Redo`) → asserts `"gamma beta gamma\n"`.
     - Dispatches `Ctrl+Q` (`Quit`) → asserts `state.overlay == OverlayMode::ConfirmQuit` and `!state.should_quit`.
     - Dispatches `Ctrl+S` (`Save`), clears overlay with `Escape`, then dispatches `Ctrl+Q` → asserts `state.should_quit == true`.
   - `soft_wrap_layout_spell_quick_fix_and_preview_sync`:
     - Creates `EditorState::new(AppConfig::default()).unwrap()`.
     - Inserts `"# Heading\n\nCheck this documnt in the editor with soft wrap enabled."`.
     - Computes `layout = compute_wrapped_layout(&state.buffers.active().document, 24, true, 4)` and asserts `layout.len_visual_lines() > 3` and that `logical_to_visual` / `visual_to_logical` roundtrips for the typo position `Position { line: 2, col: 12 }`.
     - Moves cursor to `Position { line: 2, col: 12 }`, dispatches `Action::OpenSpellQuickFix`, asserts `state.overlay == OverlayMode::SpellQuickFix` and `state.spell_popup` suggestions contain `"document"`.
     - Selects the index of `"document"` in `state.spell_popup`, calls `state.apply_selected_spell_fix().unwrap()`, asserts `true` and that active buffer text now contains `"document"` and not `"documnt"`.
     - Calls `state.build_active_preview(60)` and asserts `plain_lines().join("\n")` contains `"document"` and `source_line_to_preview_line(2) >= source_line_to_preview_line(0)`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn multi_buffer_keymap_find_multicursor_and_undo_workflow\b' crates/scriptui-core/tests/editor_state_pipeline.rs && grep -qE 'fn soft_wrap_layout_spell_quick_fix_and_preview_sync\b' crates/scriptui-core/tests/editor_state_pipeline.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```
