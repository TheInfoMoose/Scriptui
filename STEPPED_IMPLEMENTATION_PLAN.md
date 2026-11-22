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

**Phase goal (Phase 1):** Establish the decoupled `scriptui-core` Cargo workspace with `ropey`-backed document buffer, multi-cursor selection model, transactional undo/redo, native indentation (with prose-safe smart GFM continuation), search with select-all/replace-all, and retro-futuristic TOML theme/config system defaulting to the CRT Amber palette.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 1A** | `/goal` | core | `Cargo.toml`, `Cargo.lock`, `crates/scriptui-core/Cargo.toml`, `crates/scriptui-core/src/lib.rs`, `crates/scriptui-core/src/error.rs`, `crates/scriptui-core/src/selection.rs` | — |
| 2 | **STEP 1B** | `/goal` | buffer | `crates/scriptui-core/src/buffer.rs`, `crates/scriptui-core/src/lib.rs` | STEP 1A |
| 3 | **STEP 1C** | `/goal` | history | `crates/scriptui-core/src/history.rs`, `crates/scriptui-core/src/lib.rs` | STEP 1B |
| 4 | **STEP 1D** | `/goal` | indent | `crates/scriptui-core/src/indent.rs`, `crates/scriptui-core/src/lib.rs` | STEP 1B |
| 5 | **STEP 1E** | `/goal` | search | `crates/scriptui-core/src/search.rs`, `crates/scriptui-core/src/lib.rs` | STEP 1B |
| 6 | **STEP 1F** | `/goal` | theme | `crates/scriptui-core/src/theme.rs`, `crates/scriptui-core/src/config.rs`, `crates/scriptui-core/src/lib.rs` | STEP 1A |

### Completed

| Step ID | Status | Command | Scope | Target Files |
| :--- | :---: | :---: | :--- | :--- |

### Future phases

Outlines only (goal + step titles). Full step blocks are written at the phase review before each phase starts, followed by `neb plan import --write` and `scripts/phased-step.sh new-phase` (which regenerates `.orrery/phase.toml` for that phase).

- **Phase 2: Full GFM Markdown AST, Live Highlighting, Preview IR & Spellcheck Engine (`scriptui-core`)**
  - `comrak`-backed GFM parser & `sourcepos` span mapper (headings H1–H6, emphasis/strong/strikethrough, links/autolinks, tables, task lists, footnotes, alerts/callouts, nested blockquotes, hard line breaks, horizontal rules, YAML frontmatter)
  - `syntect` fenced code-block syntax highlighter mapped onto `ThemePalette` semantic tokens
  - Live editor per-line styled span synthesizer (combining GFM tokens, search matches, multi-cursor selections, and spellcheck underlines)
  - Preview document IR builder & bidirectional source-line-to-preview-line scroll synchronization map
  - Pure-Rust Hunspell spellcheck engine (`spellbook`) with bundled `en_US` `.dic`/`.aff`, Markdown-aware AST skip spans, ranked suggestions, personal dictionary persistence, and XDG/CLI locale ingestion
- **Phase 3: Multi-Buffer State Machine, Keymap & Soft-Wrap Layout (`scriptui-core`)**
  - Data-driven `Keymap` & `Action` enum (modeless CUA defaults: `Ctrl+S`, `Ctrl+F`, `Ctrl+H`, `Ctrl+Shift+L`, `Ctrl+Z`/`Y`, `Ctrl+P`/`F1`, `Ctrl+.`, `Shift+Arrows`, `Tab`/`Shift+Tab`, preview/wrap toggles)
  - `BufferManager` (multi-buffer tabs, open/new/close/switch, dirty-buffer exit guard, external file-change reload state)
  - `EditorState` coordinator (active buffer, Find/Replace bar state, Command Palette fuzzy filter, Spellcheck quick-fix popup state, Preview pane state: `Collapsed` / `Split(u16)` / `Expanded`)
  - Toggleable soft word-wrap layout calculator with visual indent-guide metadata
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

## Phase 1 Step Blocks

### STEP 1A: Workspace Manifest, Error Types, and Multi-Cursor Selection Model

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 1A: Create workspace Cargo.toml and scriptui-core crate with ScriptuiError, Position, Range, Selection, and SelectionSet multi-cursor primitives plus unit tests.

[SCOPE_LOCK]
ALLOWED_FILES:
  - Cargo.toml (NEW)
  - Cargo.lock (NEW)
  - crates/scriptui-core/Cargo.toml (NEW)
  - crates/scriptui-core/src/lib.rs (NEW)
  - crates/scriptui-core/src/error.rs (NEW)
  - crates/scriptui-core/src/selection.rs (NEW)
READ_ONLY_REFS:
  - STEPPED_IMPLEMENTATION_PLAN.md:1-60 (phase roadmap and base gate)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No TUI crates (ratatui, crossterm, clap) in workspace or scriptui-core manifests yet.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
true

[MUTATION_SPEC]
1. Cargo.toml (NEW) — create workspace root manifest:
   - `[workspace]`: `resolver = "2"`, `members = ["crates/scriptui-core"]`.
   - `[workspace.package]`: `version = "0.1.0"`, `edition = "2021"`, `license = "MIT"`, `description = "Retro-futuristic Ratatui text editor and Markdown IDE baseline"`.
2. crates/scriptui-core/Cargo.toml (NEW) — create core library manifest:
   - `[package]`: `name = "scriptui-core"`, `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`, `description.workspace = true`.
   - `[dependencies]`: `ropey = "1"`, `regex = "1"`, `serde = { version = "1", features = ["derive"] }`, `toml = "0.8"`, `thiserror = "2"`.
   - `[dev-dependencies]`: `tempfile = "3"`.
3. crates/scriptui-core/src/lib.rs (NEW) — declare modules and re-exports:
   - `pub mod error;`
   - `pub mod selection;`
   - `pub use error::{ScriptuiError, ScriptuiResult};`
   - `pub use selection::{Position, Selection, SelectionSet};`
4. crates/scriptui-core/src/error.rs (NEW) — define `ScriptuiError` with `#[derive(Debug, thiserror::Error)]` and `pub type ScriptuiResult<T> = std::result::Result<T, ScriptuiError>;` containing variants:
   - `Io { path: std::path::PathBuf, #[source] source: std::io::Error }` (`"I/O error at {path}: {source}"`)
   - `TomlParse { path: std::path::PathBuf, #[source] source: toml::de::Error }` (`"TOML parse error in {path}: {source}"`)
   - `InvalidRegex { pattern: String, #[source] source: regex::Error }` (`"invalid regex '{pattern}': {source}"`)
   - `UnknownTheme(String)` (`"unknown theme: {0}"`)
   - `OutOfBounds { offset: usize, len: usize }` (`"char offset {offset} out of bounds for length {len}"`)
5. crates/scriptui-core/src/selection.rs (NEW) — implement multi-cursor selection primitives with `#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]` on `Position` and `Selection`:
   - `pub struct Position { pub line: usize, pub col: usize }`
   - `pub struct Selection { pub anchor: usize, pub head: usize }` (char offsets into a buffer) with methods:
     - `pub const fn point(offset: usize) -> Self` (`anchor == offset`, `head == offset`)
     - `pub const fn new(anchor: usize, head: usize) -> Self`
     - `pub const fn start(self) -> usize` (`if self.anchor <= self.head { self.anchor } else { self.head }`)
     - `pub const fn end(self) -> usize` (`if self.anchor >= self.head { self.anchor } else { self.head }`)
     - `pub const fn is_empty(self) -> bool` (`self.anchor == self.head`)
     - `pub const fn len(self) -> usize` (`self.end() - self.start()`)
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct SelectionSet { ranges: Vec<Selection>, primary: usize }` with methods:
     - `pub fn point(offset: usize) -> Self` (single caret at `offset`, `primary = 0`)
     - `pub fn from_ranges(ranges: Vec<Selection>) -> Self` (if empty defaults to `point(0)`; otherwise sorts by `(start(), end())` and merges overlapping or identical ranges while preserving head direction of the merged span; `primary` clamped to valid index)
     - `pub fn ranges(&self) -> &[Selection]`
     - `pub fn primary(&self) -> Selection`
     - `pub fn len(&self) -> usize`
     - `pub fn is_empty(&self) -> bool` (always `false` since at least one caret exists)
     - `pub fn clamp(&mut self, max_char_len: usize)` (clamps every `anchor` and `head` to `<= max_char_len`, then re-normalizes/merges)
     - `pub fn collapse_to_primary(&mut self)` (keeps only `primary()` as a single caret at `primary().head`)
6. Tests in `crates/scriptui-core/src/selection.rs` (`#[cfg(test)] mod tests`):
   - `selection_start_end_and_empty`: tests `Selection::point(4)` (`is_empty() == true`, `len() == 0`), forward `Selection::new(2, 7)` (`start() == 2`, `end() == 7`, `len() == 5`), and backward `Selection::new(9, 3)` (`start() == 3`, `end() == 9`, `len() == 6`).
   - `selection_set_merges_overlaps_and_sorts`: constructs `SelectionSet::from_ranges(vec![Selection::new(10, 12), Selection::new(2, 6), Selection::new(4, 8)])`; asserts `set.len() == 2`, `set.ranges()[0] == Selection::new(2, 8)`, `set.ranges()[1] == Selection::new(10, 12)`.
   - `selection_set_clamp_and_collapse`: clamps `[Selection::new(5, 20), Selection::new(15, 25)]` to `10`; asserts `set.len() == 1` and `set.ranges()[0] == Selection::new(5, 10)`; calls `collapse_to_primary()` and asserts `set.primary() == Selection::point(10)`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub enum ScriptuiError\b' crates/scriptui-core/src/error.rs && grep -qE 'pub struct SelectionSet\b' crates/scriptui-core/src/selection.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 1B: Ropey-Backed DocumentBuffer with Atomic Save and Multi-Cursor Edits

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 1B: Implement ropey-backed DocumentBuffer in crates/scriptui-core/src/buffer.rs with atomic temp-file saving, external mtime detection, and multi-cursor insert/delete operations.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/buffer.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/error.rs:1-40 (ScriptuiError, ScriptuiResult)
  - crates/scriptui-core/src/selection.rs:1-120 (Position, Selection, SelectionSet)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No undo stack or search logic inside buffer.rs (those belong to STEP 1C and STEP 1E).
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct SelectionSet\b' crates/scriptui-core/src/selection.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod buffer;` and `pub use buffer::{BufferEdit, DocumentBuffer};`.
2. crates/scriptui-core/src/buffer.rs (NEW) — implement:
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct BufferEdit { pub start: usize, pub end: usize, pub inserted: String, pub removed: String }` (single atomic char-range replacement for history recording).
   - `#[derive(Debug, Clone)] pub struct DocumentBuffer` holding `rope: ropey::Rope`, `path: Option<std::path::PathBuf>`, `dirty: bool`, `last_disk_mtime: Option<std::time::SystemTime>`, and `selection: SelectionSet`.
   - Constructors & file I/O:
     - `pub fn empty() -> Self`
     - `pub fn from_str_content(text: &str) -> Self`
     - `pub fn from_file(path: &std::path::Path) -> ScriptuiResult<Self>` (reads UTF-8 text, records `metadata().modified().ok()`, sets `dirty = false`)
     - `pub fn save_atomic(&mut self) -> ScriptuiResult<()>` (requires `self.path`; writes full rope contents to a sibling temporary file `<filename>.scriptui.tmp.<pid>` in the same parent directory, flushes/syncs, renames atomically over `self.path`, updates `last_disk_mtime`, sets `dirty = false`; returns `ScriptuiError::Io` with `ErrorKind::InvalidInput` if `path` is `None`)
     - `pub fn save_as_atomic(&mut self, path: std::path::PathBuf) -> ScriptuiResult<()>` (sets `self.path = Some(path)` and delegates to `save_atomic`)
     - `pub fn has_external_modification(&self) -> ScriptuiResult<bool>` (returns `Ok(false)` if `path` or `last_disk_mtime` is `None`; otherwise compares current filesystem `mtime` against `last_disk_mtime`)
   - Inspection & coordinate conversion:
     - `pub fn path(&self) -> Option<&std::path::Path>`
     - `pub fn is_dirty(&self) -> bool`
     - `pub fn set_dirty(&mut self, dirty: bool)`
     - `pub fn len_chars(&self) -> usize`
     - `pub fn len_lines(&self) -> usize`
     - `pub fn text(&self) -> String`
     - `pub fn line_text(&self, line_idx: usize) -> Option<String>` (returns line content without trailing `\r\n` or `\n`, or `None` if out of bounds)
     - `pub fn selection(&self) -> &SelectionSet`
     - `pub fn set_selection(&mut self, mut selection: SelectionSet)` (clamps to `self.len_chars()`)
     - `pub fn char_to_position(&self, char_idx: usize) -> Position` (clamps `char_idx` to `self.len_chars()`)
     - `pub fn position_to_char(&self, pos: Position) -> usize` (clamps line to last line and col to line's non-newline char length)
   - Multi-cursor editing primitives (apply from highest offset to lowest so earlier offsets stay valid, update `self.selection` to post-edit caret positions, mark `dirty = true`, and return `Vec<BufferEdit>` in execution order):
     - `pub fn apply_insert(&mut self, text: &str) -> Vec<BufferEdit>` (replaces each selection range with `text` and collapses each caret to the end of its inserted text)
     - `pub fn apply_delete(&mut self, backward: bool) -> Vec<BufferEdit>` (for non-empty ranges deletes the selected span; for empty carets deletes 1 char before caret if `backward && caret > 0`, or 1 char after caret if `!backward && caret < len_chars()`)
     - `pub fn apply_raw_edit(&mut self, start: usize, end: usize, replacement: &str) -> ScriptuiResult<BufferEdit>` (validates `start <= end && end <= self.len_chars()`, otherwise `Err(ScriptuiError::OutOfBounds { .. })`; applies replacement, marks `dirty = true`, returns `BufferEdit`)
3. Tests in `crates/scriptui-core/src/buffer.rs` (`#[cfg(test)] mod tests`):
   - `multi_cursor_insert_replaces_all_selections`: starts with `"foo bar foo"`, sets `SelectionSet::from_ranges(vec![Selection::new(0, 3), Selection::new(8, 11)])`, calls `apply_insert("baz")`; asserts `buf.text() == "baz bar baz"`, `buf.is_dirty()`, and `buf.selection().ranges() == &[Selection::point(3), Selection::point(11)]`.
   - `multi_cursor_backspace_and_delete`: starts with `"ab cd"`, sets carets at `2` and `5`, calls `apply_delete(true)`; asserts `buf.text() == "a c"`; sets carets at `0` and `2`, calls `apply_delete(false)`; asserts `buf.text() == " "`.
   - `atomic_save_and_external_modification`: uses `tempfile::tempdir()`, saves `"hello\nworld\n"` via `save_as_atomic`, asserts `!buf.is_dirty()` and `std::fs::read_to_string(&p).unwrap() == "hello\nworld\n"`; verifies `has_external_modification().unwrap() == false`; rewrites file and verifies reload via `from_file` reads exact content.
   - `position_char_roundtrip`: starts with `"abc\ndef\nghi"`, asserts `char_to_position(5) == Position { line: 1, col: 1 }` and `position_to_char(Position { line: 2, col: 2 }) == 10`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct DocumentBuffer\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn save_atomic\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn apply_insert\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn apply_delete\b' crates/scriptui-core/src/buffer.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 1C: Transactional Multi-Cursor Undo/Redo History

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 1C: Implement transactional UndoStack in crates/scriptui-core/src/history.rs restoring multi-cursor edits and SelectionSets across undo and redo.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/history.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:1-140 (BufferEdit, DocumentBuffer, apply_raw_edit)
  - crates/scriptui-core/src/selection.rs:1-100 (SelectionSet)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No modifications to buffer.rs.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct DocumentBuffer\b' crates/scriptui-core/src/buffer.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod history;` and `pub use history::{EditTransaction, UndoStack};`.
2. crates/scriptui-core/src/history.rs (NEW) — implement:
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct EditTransaction { pub edits: Vec<BufferEdit>, pub selection_before: SelectionSet, pub selection_after: SelectionSet }`
   - `#[derive(Debug, Clone)] pub struct UndoStack { undo_entries: Vec<EditTransaction>, redo_entries: Vec<EditTransaction>, max_entries: usize }`
   - Methods on `UndoStack`:
     - `pub fn new(max_entries: usize) -> Self` (clamps `max_entries` to at least `1`)
     - `pub fn record(&mut self, tx: EditTransaction)` (if `tx.edits.is_empty()` does nothing; pushes onto `undo_entries`, clears `redo_entries`, drops oldest entry if `undo_entries.len() > self.max_entries`)
     - `pub fn can_undo(&self) -> bool`
     - `pub fn can_redo(&self) -> bool`
     - `pub fn clear(&mut self)`
     - `pub fn undo(&mut self, buf: &mut DocumentBuffer) -> ScriptuiResult<bool>` (pops top `EditTransaction` from `undo_entries`, replays each `edit` in reverse order by replacing `edit.start .. edit.start + edit.inserted.chars().count()` with `&edit.removed` via `buf.apply_raw_edit`, restores `buf.set_selection(tx.selection_before.clone())`, pushes `tx` onto `redo_entries`, returns `Ok(true)`; returns `Ok(false)` when empty)
     - `pub fn redo(&mut self, buf: &mut DocumentBuffer) -> ScriptuiResult<bool>` (pops top `EditTransaction` from `redo_entries`, replays each `edit` in forward order via `buf.apply_raw_edit(edit.start, edit.end, &edit.inserted)`, restores `buf.set_selection(tx.selection_after.clone())`, pushes `tx` onto `undo_entries`, returns `Ok(true)`; returns `Ok(false)` when empty)
3. Tests in `crates/scriptui-core/src/history.rs` (`#[cfg(test)] mod tests`):
   - `undo_redo_multi_cursor_transaction`: starts `DocumentBuffer::from_str_content("alpha beta alpha")`, selects both `"alpha"` spans, records `selection_before`, calls `apply_insert("gamma")`, records `EditTransaction` in `UndoStack::new(64)`; calls `undo(&mut buf)` and asserts `buf.text() == "alpha beta alpha"` and `buf.selection() == &selection_before`; calls `redo(&mut buf)` and asserts `buf.text() == "gamma beta gamma"`.
   - `new_record_clears_redo_stack`: records two transactions, undoes one, asserts `can_redo() == true`, records a third transaction, asserts `can_redo() == false`.
   - `max_entries_evicts_oldest`: creates `UndoStack::new(2)`, records 3 transactions, undoes twice (`Ok(true)` twice), third `undo` returns `Ok(false)`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct UndoStack\b' crates/scriptui-core/src/history.rs && grep -qE 'pub fn undo\b' crates/scriptui-core/src/history.rs && grep -qE 'pub fn redo\b' crates/scriptui-core/src/history.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 1D: Native Indentation Engine and Prose-Safe Smart GFM Continuation

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 1D: Implement native indentation engine in crates/scriptui-core/src/indent.rs supporting soft/hard tabs, block indent/dedent, and smart Enter continuation that never carries over first-line prose paragraph indentation.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/indent.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:1-140 (DocumentBuffer, BufferEdit, line_text)
  - crates/scriptui-core/src/selection.rs:1-100 (Selection, SelectionSet)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No modifications to buffer.rs.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct DocumentBuffer\b' crates/scriptui-core/src/buffer.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod indent;` and `pub use indent::{EnterOutcome, IndentConfig, IndentStyle, compute_enter_indent, dedent_lines, indent_lines};`.
2. crates/scriptui-core/src/indent.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)] pub enum IndentStyle { Spaces(u8), Tabs }`
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)] pub struct IndentConfig { pub style: IndentStyle, pub smart_gfm_continuation: bool }` with `Default` yielding `IndentStyle::Spaces(4)` and `smart_gfm_continuation: true`, plus `pub fn indent_unit(&self) -> String` (`"\t"` for `Tabs`, `" ".repeat(n as usize)` for `Spaces(n)` clamped to `>= 1`).
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub enum EnterOutcome { Insert(String), ClearCurrentLine }`
   - `pub fn compute_enter_indent(current_line: &str, in_fenced_code_block: bool, config: IndentConfig) -> EnterOutcome`:
     - Extracts leading whitespace (`leading`) and `trimmed = &current_line[leading.len()..]`.
     - If `in_fenced_code_block`: returns `EnterOutcome::Insert(format!("\n{leading}"))`.
     - If `config.smart_gfm_continuation`:
       - **Task list** (`- [ ] `, `- [x] `, `- [X] `, `* [ ] `, `+ [ ] `): if nothing follows the checkbox prefix (trimmed of spaces), returns `EnterOutcome::ClearCurrentLine`; otherwise returns `EnterOutcome::Insert(format!("\n{leading}{marker} [ ] "))` preserving the bullet marker (`-`, `*`, or `+`).
       - **Unordered bullet** (`- `, `* `, `+ `): if nothing follows the bullet prefix, returns `EnterOutcome::ClearCurrentLine`; otherwise returns `EnterOutcome::Insert(format!("\n{leading}{marker} "))`.
       - **Ordered list** (`<digits>. ` or `<digits>) `): if nothing follows the delimiter+space, returns `EnterOutcome::ClearCurrentLine`; otherwise parses `<digits>` as `usize`, increments by 1, and returns `EnterOutcome::Insert(format!("\n{leading}{next}{delim} "))`.
       - **Blockquote** (one or more `>` markers with optional single space, e.g. `> `, `> > `): if nothing follows the blockquote prefix, returns `EnterOutcome::ClearCurrentLine`; otherwise returns `EnterOutcome::Insert(format!("\n{leading}{quote_prefix}"))`.
     - **Prose / initial-line paragraph rule (CRITICAL):** For plain prose lines (even if `leading` is non-empty, such as a first-line paragraph indent like `"    Once upon a time"`), returns `EnterOutcome::Insert("\n".to_string())` — NEVER carries over first-line paragraph indentation.
   - `pub fn indent_lines(buf: &mut DocumentBuffer, start_line: usize, end_line: usize, config: IndentConfig) -> ScriptuiResult<Vec<BufferEdit>>` (inserts `config.indent_unit()` at column 0 of each line in `start_line..=end_line` clamped to `buf.len_lines() - 1`, iterating from `end_line` down to `start_line`).
   - `pub fn dedent_lines(buf: &mut DocumentBuffer, start_line: usize, end_line: usize, config: IndentConfig) -> ScriptuiResult<Vec<BufferEdit>>` (removes up to one `Tabs` char or up to `n` leading spaces from column 0 of each line in `start_line..=end_line`, iterating from `end_line` down to `start_line`).
3. Tests in `crates/scriptui-core/src/indent.rs` (`#[cfg(test)] mod tests`):
   - `prose_paragraph_indent_does_not_auto_indent`: calls `compute_enter_indent("    First line of paragraph.", false, IndentConfig::default())`; asserts `EnterOutcome::Insert("\n".to_string())`.
   - `gfm_list_and_blockquote_continuation`: asserts `compute_enter_indent("  - item", false, cfg) == EnterOutcome::Insert("\n  - ".to_string())`, `compute_enter_indent("1. first", false, cfg) == EnterOutcome::Insert("\n2. ".to_string())`, `compute_enter_indent("- [x] done", false, cfg) == EnterOutcome::Insert("\n- [ ] ".to_string())`, `compute_enter_indent("> quote", false, cfg) == EnterOutcome::Insert("\n> ".to_string())`, and `compute_enter_indent("  - ", false, cfg) == EnterOutcome::ClearCurrentLine`.
   - `fenced_code_block_preserves_indent`: calls `compute_enter_indent("    let x = 1;", true, IndentConfig::default())`; asserts `EnterOutcome::Insert("\n    ".to_string())`.
   - `block_indent_and_dedent`: starts `DocumentBuffer::from_str_content("a\nb\nc")`, runs `indent_lines(&mut buf, 0, 1, IndentConfig { style: IndentStyle::Spaces(2), smart_gfm_continuation: true })`; asserts `buf.text() == "  a\n  b\nc"`; runs `dedent_lines(&mut buf, 0, 1, cfg)`; asserts `buf.text() == "a\nb\nc"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct IndentConfig\b' crates/scriptui-core/src/indent.rs && grep -qE 'pub fn compute_enter_indent\b' crates/scriptui-core/src/indent.rs && grep -qE 'pub fn indent_lines\b' crates/scriptui-core/src/indent.rs && grep -qE 'pub fn dedent_lines\b' crates/scriptui-core/src/indent.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 1E: Search, Replace All, and Select-All Multi-Cursor Engine

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 1E: Implement search, replace, replace-all, and select-all-occurrences multi-cursor engine in crates/scriptui-core/src/search.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/search.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:1-140 (DocumentBuffer, BufferEdit, apply_raw_edit)
  - crates/scriptui-core/src/selection.rs:1-100 (Selection, SelectionSet)
  - crates/scriptui-core/src/error.rs:1-40 (ScriptuiError::InvalidRegex)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No modifications to buffer.rs.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct DocumentBuffer\b' crates/scriptui-core/src/buffer.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod search;` and `pub use search::{SearchMatch, SearchQuery, find_all, next_match, prev_match, replace_all, select_all_occurrences};`.
2. crates/scriptui-core/src/search.rs (NEW) — implement:
   - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct SearchQuery { pub pattern: String, pub case_sensitive: bool, pub whole_word: bool, pub use_regex: bool }` with `pub fn plain(pattern: impl Into<String>) -> Self` (`case_sensitive: false`, `whole_word: false`, `use_regex: false`).
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct SearchMatch { pub start: usize, pub end: usize }` (char offsets).
   - `pub fn find_all(buf: &DocumentBuffer, query: &SearchQuery) -> ScriptuiResult<Vec<SearchMatch>>`:
     - Returns `Ok(Vec::new())` if `query.pattern.is_empty()`.
     - Compiles a `regex::Regex` using `regex::RegexBuilder` (escaping `query.pattern` with `regex::escape` when `!query.use_regex`; wrapping with `\b(?:...)\b` when `query.whole_word`; setting `.case_insensitive(!query.case_sensitive)`). Maps compilation errors to `ScriptuiError::InvalidRegex`.
     - Converts every non-empty byte match span from `buf.text()` into char offsets (`start`, `end`) accurately for UTF-8 multibyte text.
   - `pub fn next_match(matches: &[SearchMatch], after_char: usize) -> Option<SearchMatch>` (returns first match with `m.start >= after_char`, wrapping around to `matches.first().copied()`).
   - `pub fn prev_match(matches: &[SearchMatch], before_char: usize) -> Option<SearchMatch>` (returns last match with `m.start < before_char`, wrapping around to `matches.last().copied()`).
   - `pub fn select_all_occurrences(buf: &mut DocumentBuffer, query: &SearchQuery) -> ScriptuiResult<usize>` (calls `find_all`; if non-empty, sets `buf.set_selection(SelectionSet::from_ranges(...))` covering every match and returns `Ok(count)`; if empty, leaves selection unchanged and returns `Ok(0)`).
   - `pub fn replace_all(buf: &mut DocumentBuffer, query: &SearchQuery, replacement: &str) -> ScriptuiResult<Vec<BufferEdit>>` (calls `find_all`, iterates matches in reverse order applying `buf.apply_raw_edit(m.start, m.end, replacement)`, updates selection to the end of the last replaced match if any, returns collected `Vec<BufferEdit>`).
3. Tests in `crates/scriptui-core/src/search.rs` (`#[cfg(test)] mod tests`):
   - `find_all_case_and_whole_word`: searches `"Cat cat caterpillar"` for `"cat"`: default finds 3 matches; `case_sensitive = true` finds 2; `whole_word = true` (case-insensitive) finds 2 (`0..3`, `4..7`).
   - `find_all_regex_and_invalid_regex`: regex `r"v\d+\.\d+"` on `"v1.0 and v2.15"` finds 2 matches; invalid regex `"[unclosed"` returns `Err(ScriptuiError::InvalidRegex { .. })`.
   - `select_all_and_live_multi_cursor_change`: on `"amber CRT amber"`, calls `select_all_occurrences(&mut buf, &SearchQuery::plain("amber"))`, asserts count `2`, then calls `buf.apply_insert("gold")` and asserts `buf.text() == "gold CRT gold"`.
   - `replace_all_utf8_safe`: on `"café test café"`, replaces `"café"` with `"bistro"`, asserts `buf.text() == "bistro test bistro"` and `edits.len() == 2`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct SearchQuery\b' crates/scriptui-core/src/search.rs && grep -qE 'pub fn find_all\b' crates/scriptui-core/src/search.rs && grep -qE 'pub fn select_all_occurrences\b' crates/scriptui-core/src/search.rs && grep -qE 'pub fn replace_all\b' crates/scriptui-core/src/search.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 1F: Retro-Futuristic Theme Palettes (CRT Amber Default) and XDG Config Loader

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 1F: Implement TOML-driven ThemePalette with 5 built-in retro-futuristic presets (CRT Amber default matching Nebulous/Orrery/Rocket-Factory) and AppConfig loader in crates/scriptui-core/src/{theme,config}.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/theme.rs (NEW)
  - crates/scriptui-core/src/config.rs (NEW)
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/error.rs:1-40 (ScriptuiError::TomlParse, UnknownTheme)
  - crates/scriptui-core/src/indent.rs:1-40 (IndentConfig, IndentStyle)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui Color imports in scriptui-core (keep theme RGB purely toolkit-agnostic).
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub enum ScriptuiError\b' crates/scriptui-core/src/error.rs && grep -qE 'pub struct IndentConfig\b' crates/scriptui-core/src/indent.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/lib.rs — add `pub mod config;`, `pub mod theme;`, `pub use config::AppConfig;`, and `pub use theme::{RgbColor, Theme, ThemePalette, builtin_theme_names, default_theme, resolve_theme};`.
2. crates/scriptui-core/src/theme.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)] pub struct RgbColor { pub r: u8, pub g: u8, pub b: u8 }`
   - `#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)] pub struct ThemePalette` with fields:
     - Chrome & editor base: `background`, `surface`, `foreground`, `muted`, `accent`, `border_default`, `border_focused`, `gutter_fg`, `gutter_active_fg`, `selection_bg`, `cursor_bg`, `cursor_fg`, `search_match_bg`, `search_active_bg`
     - GFM semantic differentials: `md_h1`, `md_h2`, `md_h3`, `md_emphasis`, `md_strong`, `md_strikethrough`, `md_link`, `md_code_inline`, `md_code_block`, `md_blockquote`, `md_list_marker`, `md_table_border`, `md_hr`, `md_alert_note`, `md_alert_tip`, `md_alert_warning`, `md_alert_caution`
     - Diagnostics: `spell_error`, `status_ok`, `status_warn`, `status_error`
   - `#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)] pub struct Theme { pub name: String, pub palette: ThemePalette }`
   - `pub const DEFAULT_CRT_AMBER_TOML: &str` defining `name = "crt_amber"` with exact Nebulous/Orrery/Rocket-Factory CRT Amber anchor RGB values:
     - `background = { r = 10, g = 10, b = 8 }`, `surface = { r = 20, g = 18, b = 14 }`, `foreground = { r = 255, g = 176, b = 0 }`, `muted = { r = 153, g = 107, b = 20 }`, `accent = { r = 255, g = 196, b = 61 }`, `border_default = { r = 120, g = 82, b = 16 }`, `border_focused = { r = 255, g = 209, b = 102 }`, `md_h1 = { r = 255, g = 218, b = 121 }`, `md_code_inline = { r = 94, g = 212, b = 212 }`, `md_alert_tip = { r = 64, g = 224, b = 112 }`, `md_alert_warning = { r = 255, g = 159, b = 28 }`, `md_alert_caution = { r = 255, g = 82, b = 82 }`, `spell_error = { r = 255, g = 82, b = 82 }`, plus coherent CRT amber ramp & signal values for all remaining fields.
   - Four additional built-in TOML theme constants: `PHOSPHOR_GREEN_TOML` (`"phosphor_green"`), `ICE_CYAN_TOML` (`"ice_cyan"`), `VECTOR_MAGENTA_TOML` (`"vector_magenta"`), `HIGH_CONTRAST_MONO_TOML` (`"high_contrast_mono"`).
   - Functions:
     - `pub fn builtin_theme_names() -> &'static [&'static str]` returning `&["crt_amber", "phosphor_green", "ice_cyan", "vector_magenta", "high_contrast_mono"]`.
     - `pub fn default_theme() -> ScriptuiResult<Theme>` (parses `DEFAULT_CRT_AMBER_TOML`).
     - `pub fn load_theme_file(path: &std::path::Path) -> ScriptuiResult<Theme>`
     - `pub fn resolve_theme(name: &str, search_dirs: &[std::path::PathBuf]) -> ScriptuiResult<Theme>` (checks `<dir>/<name>.toml` in `search_dirs` first, then built-in presets, else `Err(ScriptuiError::UnknownTheme(name.to_string()))`).
3. crates/scriptui-core/src/config.rs (NEW) — implement:
   - `#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)] pub struct AppConfig`:
     - `#[serde(default = "default_theme_name")] pub theme: String` (`"crt_amber"`)
     - `#[serde(default = "default_locale")] pub spell_locale: String` (`"en_US"`)
     - `#[serde(default = "default_true")] pub spell_enabled: bool` (`true`)
     - `#[serde(default = "default_true")] pub line_numbers: bool` (`true`)
     - `#[serde(default = "default_true")] pub soft_wrap: bool` (`true`)
     - `#[serde(default)] pub indent: IndentConfig`
     - `#[serde(default = "default_preview_split")] pub preview_split_percent: u16` (`50`)
   - `impl Default for AppConfig` matching the serde defaults.
   - `impl AppConfig`:
     - `pub fn load_from_file(path: &std::path::Path) -> ScriptuiResult<Self>`
     - `pub fn load_or_default(path: &std::path::Path) -> ScriptuiResult<Self>` (returns `Ok(Self::default())` if `!path.exists()`)
     - `pub fn default_config_dir() -> std::path::PathBuf` (respects `XDG_CONFIG_HOME`, falling back to `$HOME/.config/scriptui` or `.config/scriptui`)
4. Tests in `theme.rs` and `config.rs`:
   - `all_builtin_themes_parse_and_crt_amber_matches_suite`: iterates `builtin_theme_names()`, resolves each with empty `search_dirs`, asserts `default_theme().unwrap().palette.foreground == RgbColor { r: 255, g: 176, b: 0 }` and `background == RgbColor { r: 10, g: 10, b: 8 }`.
   - `custom_toml_theme_overrides_builtin`: writes a custom `crt_amber.toml` in a `tempfile::tempdir()`, resolves `"crt_amber"` with that dir, asserts custom foreground RGB is returned.
   - `config_defaults_and_partial_toml_override`: parses `"theme = \"ice_cyan\"\nspell_locale = \"en_GB\"\n"` via `toml::from_str::<AppConfig>`, asserts `theme == "ice_cyan"`, `spell_locale == "en_GB"`, `line_numbers == true`, `preview_split_percent == 50`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct ThemePalette\b' crates/scriptui-core/src/theme.rs && grep -qE 'pub const DEFAULT_CRT_AMBER_TOML\b' crates/scriptui-core/src/theme.rs && grep -qE 'pub struct AppConfig\b' crates/scriptui-core/src/config.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```
