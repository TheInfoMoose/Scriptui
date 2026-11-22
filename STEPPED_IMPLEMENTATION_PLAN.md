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

**Phase goal (Phase 4):** Roll in Phase 3 editor-state fixes (`BUG-0001`, `BUG-0002`) and clipboard/select-all actions in `scriptui-core`, then build the `scriptui-tui` Ratatui shell: `clap` CLI (including `dict import`), terminal lifecycle with panic hook & OSC 52 clipboard, top-down container layout chrome (tab bar, split editor/preview regions, telemetry status bar), editor viewport widget (gutter, GFM syntax differentials, multi-cursor, soft-wrap & indent guides), full mouse event router, and interactive `main.rs` event loop.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Phase 4 Top-Down UI Container Tree

```text
Root (Vertical)
├── [Length(1)] TabBar                 (open buffers, active highlight, dirty '*' marker)
├── [Min(1)]    MainSplit              (Horizontal, driven by PreviewMode)
│   ├── PreviewMode::Collapsed
│   │   ├── [Min(1)]    EditorPane     (gutter + soft-wrapped GFM text + indent guides + cursors)
│   │   └── [Length(3)] PreviewEdgeTab (vertical '◀ PREVIEW' toggle handle)
│   ├── PreviewMode::Split(pct)
│   │   ├── [Percentage(100 - pct)] EditorPane
│   │   ├── [Length(1)]             SplitDivider (draggable vertical separator)
│   │   └── [Percentage(pct)]       PreviewPane  (shell frame in Phase 4; full GFM in Phase 5)
│   └── PreviewMode::Expanded
│       └── [Percentage(100)]       PreviewPane
├── [Length(0 | 1 | 2)] FindReplaceBar (0 normally, 1 for Find, 2 for Replace)
└── [Length(1)] StatusBar              (telemetry: file/dirty, Ln/Col, cursors, wrap, indent, locale, theme, status)
```

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 4A** | `/goal` | core | `crates/scriptui-core/src/keymap.rs`, `crates/scriptui-core/src/editor.rs` | — |
| 2 | **STEP 4B** | `/goal` | tui | `Cargo.toml`, `Cargo.lock`, `crates/scriptui-tui/Cargo.toml`, `crates/scriptui-tui/src/cli.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 4A |
| 3 | **STEP 4C** | `/goal` | tui | `crates/scriptui-tui/src/terminal.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 4B |
| 4 | **STEP 4D** | `/goal` | tui | `crates/scriptui-tui/src/layout_tree.rs`, `crates/scriptui-tui/src/chrome.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 4C |
| 5 | **STEP 4E** | `/goal` | tui | `crates/scriptui-tui/src/viewport.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 4D |
| 6 | **STEP 4F** | `/goal` | tui | `crates/scriptui-tui/src/mouse.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 4E |
| 7 | **STEP 4G** | `/goal` | tui | `crates/scriptui-tui/src/app.rs`, `crates/scriptui-tui/src/main.rs`, `crates/scriptui-tui/src/lib.rs` | STEP 4F |

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

### Future phases

Outlines only (goal + step titles). Full step blocks are written at the phase review before each phase starts, followed by `neb plan import --write`, `scripts/phased-step.sh new-phase` (which regenerates `.orrery/phase.toml` for that phase), `orrery ingest STEPPED_IMPLEMENTATION_PLAN.md --marker "### STEP "`, and `orrery config set limits.max_deferred_steps <LIMIT>`.

- **Phase 5: Expandable GFM Preview Pane, Command Palette & Interactive Overlays (`scriptui-tui`)**
  - Collapsible right-hand preview pane widget rendering full GFM (box-drawn tables, task checkboxes, styled callouts, nested blockquotes, syntax-highlighted code blocks, footnotes, hard breaks) with synchronized scrolling
  - Fuzzy Command Palette overlay (`Ctrl+P` / `F1`) for live palette switching, spellcheck locale switching, wrap/tab-width toggles, and editor commands
  - Inline Find/Replace bar widget with plain/regex/case/whole-word toggles and `Alt+Enter` Select-All multi-cursor handoff
  - Spellcheck quick-fix popup (`Ctrl+.` / right-click) with ranked suggestions, Add to Personal Dictionary, and Ignore, plus Open/Save-As and Dirty-Exit modal dialogs
- **Phase 6: Integration Verification, Performance & Polish**
  - Unicode display width & tab-stop expansion in `compute_wrapped_layout` (`BUG-0003`)
  - Ratatui `TestBackend` snapshot render tests across all 5 palettes (CRT Amber GFM differentials, collapsed/split/full preview, Find/Replace, Command Palette, Spellcheck popup)
  - End-to-end integration tests (multi-cursor editing, prose-safe smart indentation, GFM sync-scroll, custom Hunspell locale import)
  - Large-file incremental highlight/spellcheck responsiveness benchmark and `README.md` keybinding reference

### Open Questions

_None._

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.
- `BUG-0003`: `compute_wrapped_layout` measures `char` count rather than Unicode cell width and tab stops (scheduled for Phase 6 polish).

---

## Phase 4 Step Blocks

### STEP 4A: Core Phase 3 Fixes (`BUG-0001`, `BUG-0002`) and Clipboard/SelectAll Actions

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0001`, `BUG-0002`

```text
[STEP_ID]
STEP 4A: Fix stale search matches in EditorState::jump_to_match (BUG-0001), fix selection offset shift & trailing col-0 line in EditorState::handle_indent (BUG-0002), and add Copy/Cut/Paste/SelectAll actions in crates/scriptui-core.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/keymap.rs
  - crates/scriptui-core/src/editor.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/keymap.rs:89-340 (Action enum and default_cua bindings)
  - crates/scriptui-core/src/editor.rs:350-586 (dispatch, jump_to_match, handle_indent)
  - crates/scriptui-core/src/buffer.rs:1-140 (DocumentBuffer::text, selection, set_selection, apply_delete)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct EditorState\b' crates/scriptui-core/src/editor.rs && grep -qE 'pub enum Action\b' crates/scriptui-core/src/keymap.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/keymap.rs:
   - Add variants `Copy`, `Cut`, `Paste`, `SelectAll` to `pub enum Action` under `// Editing`.
   - In `Keymap::default_cua()`, bind:
     - `Ctrl+c` → `Action::Copy`
     - `Ctrl+x` → `Action::Cut`
     - `Ctrl+v` → `Action::Paste`
     - `Ctrl+a` → `Action::SelectAll`
2. crates/scriptui-core/src/editor.rs:
   - Fix `BUG-0001` (stale search matches):
     - In `jump_to_match(&mut self, forward: bool)`: always refresh matches via `if self.find_replace.query.pattern.is_empty() { self.find_replace.matches.clear(); self.find_replace.active_match = None; return Ok(()); } else { self.update_find_matches()?; }` before calling `next_match`/`prev_match`.
     - In `dispatch`: when handling `Action::NewBuffer`, `Action::CloseBuffer` (when closed), `Action::NextBuffer`, or `Action::PrevBuffer`, clear `self.find_replace.matches.clear()` and set `self.find_replace.active_match = None`.
   - Fix `BUG-0002` (indent selection tracking & trailing col-0 line):
     - In `handle_indent(&mut self, indent: bool)`:
       - Compute `start_pos = active.document.char_to_position(primary.start())` and `end_pos = active.document.char_to_position(primary.end())`.
       - Set `start_line = start_pos.line` and `end_line = if !primary.is_empty() && end_pos.col == 0 && end_pos.line > start_line { end_pos.line - 1 } else { end_pos.line }`.
       - After applying `indent_lines`/`dedent_lines`, adjust `primary.anchor` and `primary.head` by summing the signed character delta (`inserted.chars().count() as isize - (edit.end - edit.start) as isize`) for every `edit` whose `edit.start <= offset` (clamping to `0..=active.document.len_chars()`), and call `active.document.set_selection(SelectionSet::from_ranges(vec![Selection::new(new_anchor, new_head)]))` before capturing `selection_after`.
   - Add `pub fn selected_text(&self) -> String` on `EditorState`:
     - Collects the text slice for each non-empty `Selection` in `self.buffers.active().document.selection().ranges()` (using `text.chars().skip(s.start()).take(s.end() - s.start()).collect::<String>()`), joined by `"\n"`.
   - In `dispatch`:
     - `Action::SelectAll` → sets selection to `SelectionSet::from_ranges(vec![Selection::new(0, len_chars)])` (or `SelectionSet::point(0)` if empty).
     - `Action::Cut` → if any selection is non-empty, deletes selected ranges (`handle_delete(true)`).
     - `Action::Copy` | `Action::Paste` → no-op in pure core (handled by TUI clipboard layer).
3. Unit tests in `crates/scriptui-core/src/editor.rs`:
   - `find_next_refreshes_stale_matches_after_edit_and_buffer_switch`: populates search matches for `"alpha"`, edits text or switches to a second buffer containing `"alpha"` at a different offset, dispatches `Action::FindNext`, and asserts the selection matches `"alpha"` in the active buffer without stale offsets.
   - `indent_shifts_selection_and_excludes_trailing_col_zero_line`: on `"one\ntwo\nthree"`, selects from `(0, 1)` to `(2, 0)` (start of line 2), dispatches `Action::Indent`, asserts lines 0 and 1 are indented while line 2 (`"three"`) is untouched, and asserts primary selection shifted accordingly; also verifies `Action::SelectAll` and `selected_text()`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'Action::SelectAll' crates/scriptui-core/src/editor.rs && grep -qE 'pub fn selected_text\b' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 4B: `scriptui-tui` Crate Scaffold and `clap` CLI

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4B: Add crates/scriptui-tui workspace member with ratatui, crossterm, clap dependencies and CLI parser supporting files, --theme, --locale, and `scriptui dict import <locale> <dic> <aff>`.

[SCOPE_LOCK]
ALLOWED_FILES:
  - Cargo.toml
  - Cargo.lock
  - crates/scriptui-tui/Cargo.toml (NEW)
  - crates/scriptui-tui/src/cli.rs (NEW)
  - crates/scriptui-tui/src/lib.rs (NEW)
READ_ONLY_REFS:
  - Cargo.toml:1-9 (workspace members and package metadata)
  - crates/scriptui-core/Cargo.toml:1-19 (core crate metadata)
  - crates/scriptui-core/src/spell.rs:1-45 (import_locale_files, validate_locale_name)
  - crates/scriptui-core/src/config.rs:56-84 (AppConfig::default_config_dir)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No modifications to crates/scriptui-core/.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn import_locale_files\b' crates/scriptui-core/src/spell.rs

[MUTATION_SPEC]
1. Cargo.toml — add `"crates/scriptui-tui"` to `workspace.members`.
2. crates/scriptui-tui/Cargo.toml (NEW) — define package `scriptui-tui` inheriting workspace `version`, `edition`, `license`, `description`, with `[[bin]] name = "scriptui" path = "src/main.rs"` (wait: do NOT add `[[bin]]` yet until STEP 4G creates `src/main.rs`, or keep default `[lib]` only so `cargo test --workspace` compiles cleanly now!).
   - Dependencies:
     - `scriptui-core = { path = "../scriptui-core" }`
     - `ratatui = "0.29"`
     - `crossterm = "0.28"`
     - `clap = { version = "4", features = ["derive"] }`
   - Dev-dependencies:
     - `tempfile = "3"`
3. crates/scriptui-tui/src/lib.rs (NEW) — expose `pub mod cli;` and `pub use cli::{Cli, Command, DictCommand, execute_dict_import};`.
4. crates/scriptui-tui/src/cli.rs (NEW) — implement:
   - `#[derive(Debug, Clone, PartialEq, Eq, clap::Parser)] #[command(name = "scriptui", version, about = "Retro-futuristic Ratatui text editor and Markdown IDE")] pub struct Cli`:
     - `#[command(subcommand)] pub command: Option<Command>`
     - `#[arg(long)] pub theme: Option<String>`
     - `#[arg(long)] pub locale: Option<String>`
     - `#[arg(value_name = "FILE")] pub files: Vec<std::path::PathBuf>`
   - `#[derive(Debug, Clone, PartialEq, Eq, clap::Subcommand)] pub enum Command { Dict { #[command(subcommand)] action: DictCommand } }`
   - `#[derive(Debug, Clone, PartialEq, Eq, clap::Subcommand)] pub enum DictCommand { Import { locale: String, dic: std::path::PathBuf, aff: std::path::PathBuf } }`
   - `pub fn execute_dict_import(locale: &str, dic: &std::path::Path, aff: &std::path::Path, dict_dir: &std::path::Path) -> scriptui_core::ScriptuiResult<(std::path::PathBuf, std::path::PathBuf)>` delegating directly to `scriptui_core::import_locale_files(locale, dic, aff, dict_dir)`.
5. Tests in `crates/scriptui-tui/src/cli.rs` (`#[cfg(test)] mod tests`):
   - `cli_parses_files_theme_and_locale`: parses `["scriptui", "--theme", "phosphor_green", "--locale", "en_GB", "notes.md", "todo.md"]` via `Cli::try_parse_from` and asserts fields.
   - `cli_parses_dict_import_subcommand_and_executes`: parses `["scriptui", "dict", "import", "fr_FR", "/tmp/fr.dic", "/tmp/fr.aff"]`, then runs `execute_dict_import` with temp files into a `tempfile::tempdir()` and asserts the imported `.dic`/`.aff` files exist.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct Cli\b' crates/scriptui-tui/src/cli.rs && grep -qE 'pub fn execute_dict_import\b' crates/scriptui-tui/src/cli.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 4C: Terminal Lifecycle, Key Event Translation & OSC 52 Clipboard

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4C: Implement TerminalGuard (raw mode, alt screen, mouse capture, panic hook), crossterm-to-KeyChord translation, and OSC 52 clipboard encoding in crates/scriptui-tui/src/terminal.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/terminal.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/keymap.rs:5-87 (KeyCode, KeyModifiers, KeyChord)
  - crates/scriptui-tui/src/lib.rs:1-10 (module declarations)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct Cli\b' crates/scriptui-tui/src/cli.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod terminal;` and `pub use terminal::{TerminalGuard, crossterm_key_to_chord, encode_osc52_clipboard, write_osc52_clipboard};`.
2. crates/scriptui-tui/src/terminal.rs (NEW) — implement:
   - `pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;`
   - `pub struct TerminalGuard { terminal: CrosstermTerminal }` with:
     - `pub fn enter() -> std::io::Result<Self>`: installs a panic hook (calling `restore_terminal_best_effort()` before delegating to the previous hook), enables raw mode, executes `EnterAlternateScreen` and `EnableMouseCapture` on stdout, and constructs `Terminal::new(CrosstermBackend::new(stdout))`.
     - `pub fn terminal_mut(&mut self) -> &mut CrosstermTerminal`
     - `impl Drop for TerminalGuard` calling `restore_terminal_best_effort()`.
   - `fn restore_terminal_best_effort()`: ignores errors while calling `disable_raw_mode()`, `execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)`, and showing cursor.
   - `pub fn crossterm_key_to_chord(event: crossterm::event::KeyEvent) -> Option<scriptui_core::KeyChord>`:
     - Maps `crossterm::event::KeyCode` (`Char`, `Enter`, `Tab`, `BackTab`, `Backspace`, `Delete`, `Esc`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`, `PageDown`, `F(n)`) to `scriptui_core::KeyCode`.
     - Maps `CONTROL`, `ALT`, `SHIFT` flags to `scriptui_core::KeyModifiers` and returns `Some(KeyChord::new(code, modifiers).normalized())`; returns `None` for unmapped keys.
   - `pub fn encode_osc52_clipboard(text: &str) -> String`:
     - Pure-Rust RFC 4648 base64 encoding of `text.as_bytes()` wrapped in `\x1b]52;c;<base64>\x07`.
   - `pub fn write_osc52_clipboard<W: std::io::Write>(writer: &mut W, text: &str) -> std::io::Result<()>`:
     - Writes `encode_osc52_clipboard(text)` and flushes `writer`.
3. Tests in `crates/scriptui-tui/src/terminal.rs` (`#[cfg(test)] mod tests`):
   - `crossterm_key_event_maps_to_normalized_key_chord`: verifies `Ctrl+S` maps to `KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL)`, `Shift+F3` maps to `KeyChord::new(KeyCode::F(3), KeyModifiers::SHIFT)`, and `BackTab` maps to `KeyCode::BackTab`.
   - `osc52_clipboard_encodes_rfc4648_base64_sequence`: verifies `encode_osc52_clipboard("hello")` equals `"\x1b]52;c;aGVsbG8=\x07"` and `encode_osc52_clipboard("a")` equals `"\x1b]52;c;YQ==\x07"`, and verifies `write_osc52_clipboard` writes to a `Vec<u8>` buffer.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct TerminalGuard\b' crates/scriptui-tui/src/terminal.rs && grep -qE 'pub fn encode_osc52_clipboard\b' crates/scriptui-tui/src/terminal.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 4D: Top-Down Container Layout Tree, Theme Bridge & Chrome Widgets

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4D: Implement top-down ScreenRegions container layout calculator, RgbColor/StyledSegment-to-Ratatui style bridge, and chrome widgets (TabBar, PreviewEdgeTab, SplitDivider, StatusBar) in crates/scriptui-tui.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/layout_tree.rs (NEW)
  - crates/scriptui-tui/src/chrome.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/editor.rs:32-50, 213-226 (PreviewMode, OverlayMode, EditorState)
  - crates/scriptui-core/src/theme.rs:7-62 (RgbColor, ThemePalette, Theme)
  - crates/scriptui-core/src/highlight.rs:8-24 (StyleFlags, StyledSegment)
  - crates/scriptui-core/src/buffers.rs:16-50 (ManagedBuffer::display_name, is_dirty)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct TerminalGuard\b' crates/scriptui-tui/src/terminal.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod layout_tree;`, `pub mod chrome;`, and re-export `ScreenRegions`, `compute_screen_regions`, `rgb_to_color`, `segment_to_style`, `TabHitbox`, `render_tab_bar`, `render_preview_edge_tab`, `render_split_divider`, `render_status_bar`.
2. crates/scriptui-tui/src/layout_tree.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct ScreenRegions`:
     - `pub tab_bar: Rect`
     - `pub editor: Option<Rect>`
     - `pub preview_edge_tab: Option<Rect>`
     - `pub split_divider: Option<Rect>`
     - `pub preview: Option<Rect>`
     - `pub find_bar: Option<Rect>`
     - `pub status_bar: Rect`
   - `pub fn compute_screen_regions(area: Rect, preview_mode: PreviewMode, overlay: OverlayMode) -> ScreenRegions`:
     - Strictly follows the Phase 4 Top-Down UI Container Tree:
       - Vertical split of `area`: `[Constraint::Length(1), Constraint::Min(1), Constraint::Length(find_height), Constraint::Length(1)]` where `find_height = match overlay { OverlayMode::Find => 1, OverlayMode::Replace => 2, _ => 0 }`.
       - Horizontal split of `main_area`:
         - `PreviewMode::Collapsed`: `[Constraint::Min(1), Constraint::Length(3)]` → `editor = Some(chunks[0])`, `preview_edge_tab = Some(chunks[1])`, `split_divider = None`, `preview = None`.
         - `PreviewMode::Split(pct)`: `let p = pct.clamp(20, 80);` → `[Constraint::Percentage(100 - p), Constraint::Length(1), Constraint::Percentage(p)]` → `editor = Some(chunks[0])`, `split_divider = Some(chunks[1])`, `preview = Some(chunks[2])`, `preview_edge_tab = None`.
         - `PreviewMode::Expanded`: `editor = None`, `preview_edge_tab = None`, `split_divider = None`, `preview = Some(main_area)`.
   - `pub fn rgb_to_color(c: RgbColor) -> ratatui::style::Color` (`Color::Rgb(c.r, c.g, c.b)`).
   - `pub fn segment_to_style(seg: &StyledSegment) -> ratatui::style::Style` (maps `fg`, optional `bg`, and `StyleFlags` `bold`, `italic`, `underline`, `dim`, `strikethrough` to Ratatui `Modifier`).
3. crates/scriptui-tui/src/chrome.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct TabHitbox { pub rect: Rect, pub buffer_index: usize }`
   - `pub fn render_tab_bar(frame: &mut ratatui::Frame, area: Rect, state: &EditorState) -> Vec<TabHitbox>`:
     - Renders each open buffer as `" <name>[*] "` on `theme.palette.surface`, highlighting the active buffer with `accent`/`background` (bold), returning each rendered tab's `TabHitbox` clamped to `area`.
   - `pub fn render_preview_edge_tab(frame: &mut ratatui::Frame, area: Rect, theme: &Theme)`:
     - Renders vertical collapsed edge handle (`◀` and vertical `"PREVIEW"` letters centered in `area`) using `surface`, `border_default`, and `accent`.
   - `pub fn render_split_divider(frame: &mut ratatui::Frame, area: Rect, theme: &Theme)`:
     - Renders vertical `│` column in `border_focused`.
   - `pub fn render_status_bar(frame: &mut ratatui::Frame, area: Rect, state: &EditorState)`:
     - Renders retro-futuristic single-line telemetry bar on `surface` with `foreground`/`accent`/`muted`: left side shows active `display_name()` + dirty `*` + optional `status_message`; right side shows `Ln {line+1}, Col {col+1}`, optional `{n} cursors` when `selection().len() > 1`, `WRAP`/`NOWRAP`, indent mode (`SPC {w}` / `TAB {w}`), `config.spell_locale`, and `theme.name`.
4. Tests in `crates/scriptui-tui/src/chrome.rs` (`#[cfg(test)] mod tests`):
   - `screen_regions_match_all_three_preview_modes_and_find_bar`: verifies `Collapsed` (`preview_edge_tab` width 3, `preview.is_none()`), `Split(50)` (`editor`, `split_divider` width 1, `preview`), `Expanded` (`editor.is_none()`, `preview.is_some()`), and `OverlayMode::Replace` (`find_bar` height 2).
   - `chrome_renders_tabs_edge_handle_and_telemetry_status_bar`: renders via `ratatui::Terminal::new(TestBackend::new(80, 10))`, asserts tab hitboxes count equals buffer count, edge handle contains `'◀'`, and status bar contains `"Ln 1, Col 1"` and `"crt_amber"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct ScreenRegions\b' crates/scriptui-tui/src/layout_tree.rs && grep -qE 'pub fn render_status_bar\b' crates/scriptui-tui/src/chrome.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 4E: Editor Viewport Widget (Gutter, GFM Differentials, Multi-Cursor, Soft Wrap & Indent Guides)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4E: Implement EditorViewport renderer and scroll-clamping helpers in crates/scriptui-tui/src/viewport.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/viewport.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/layout.rs:4-65, 81-143 (VisualLine, WrappedLayout, compute_wrapped_layout)
  - crates/scriptui-core/src/highlight.rs:25-32, 210-250 (LineOverlay, highlight_editor_line)
  - crates/scriptui-core/src/markdown.rs:1-90 (parse_gfm, GfmDocument)
  - crates/scriptui-tui/src/layout_tree.rs:1-80 (rgb_to_color, segment_to_style)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct ScreenRegions\b' crates/scriptui-tui/src/layout_tree.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod viewport;` and `pub use viewport::{ViewportMetrics, compute_gutter_width, ensure_cursor_visible, render_editor_viewport};`.
2. crates/scriptui-tui/src/viewport.rs (NEW) — implement:
   - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct ViewportMetrics { pub area: Rect, pub gutter_width: u16, pub text_area: Rect }`
   - `pub fn compute_gutter_width(total_lines: usize, show_line_numbers: bool) -> u16`:
     - If `!show_line_numbers`, returns `0`; else returns `(total_lines.max(1).to_string().len() as u16) + 2` (digits + 1 space + `│` or padding).
   - `pub fn ensure_cursor_visible(scroll_line: &mut usize, scroll_col: &mut usize, cursor_visual_row: usize, cursor_visual_col: usize, viewport_height: usize, viewport_width: usize, total_visual_lines: usize, soft_wrap: bool)`:
     - Clamps `*scroll_line` so it never exceeds `total_visual_lines.saturating_sub(1)`, then adjusts `*scroll_line` so `cursor_visual_row` is within `[*scroll_line, *scroll_line + viewport_height.max(1) - 1]`.
     - If `soft_wrap`, sets `*scroll_col = 0`; else adjusts `*scroll_col` so `cursor_visual_col` is within `[*scroll_col, *scroll_col + viewport_width.max(1) - 1]`.
   - `pub fn render_editor_viewport(frame: &mut ratatui::Frame, area: Rect, state: &mut EditorState) -> (ViewportMetrics, WrappedLayout)`:
     - Computes `gutter_width = compute_gutter_width(doc.len_lines(), state.config.line_numbers).min(area.width.saturating_sub(1))` and `text_area = Rect { x: area.x + gutter_width, y: area.y, width: area.width.saturating_sub(gutter_width), height: area.height }`.
     - Computes `layout = compute_wrapped_layout(&active.document, text_area.width.max(1) as usize, state.config.soft_wrap, state.config.indent.tab_width)`.
     - Computes primary cursor visual position via `layout.logical_to_visual(primary_pos)` and calls `ensure_cursor_visible(...)` on `active.scroll_line` / `active.scroll_col`.
     - Parses GFM once (`parse_gfm(&text)`) and optionally runs spell diagnostics if `state.spell_engine` is `Some`.
     - Fills `area` background with `theme.palette.background`.
     - For each visible `VisualLine` in `scroll_line..(scroll_line + text_area.height as usize)`:
       - Renders gutter cell: logical line number (`wrap_index == 0`) or blank continuation (`wrap_index > 0`), colored with `gutter_active_fg` when `vl.logical_line == primary_pos.line` else `gutter_fg`.
       - Builds `LineOverlay` for `vl.logical_line` (selection ranges, search matches, active search match, spell errors) and calls `highlight_editor_line`.
       - Slices highlighted character styles to `vl.char_start..vl.char_end` (and horizontal `scroll_col` when `!soft_wrap`), draws indent guide `│` in `muted` on leading space columns listed in `vl.indent_guides`, and paints every cursor head in `active.document.selection().ranges()` with `cursor_bg`/`cursor_fg` (including end-of-line cursor cells).
     - Returns `(ViewportMetrics { area, gutter_width, text_area }, layout)`.
3. Tests in `crates/scriptui-tui/src/viewport.rs` (`#[cfg(test)] mod tests`):
   - `viewport_renders_gutter_gfm_heading_and_indent_guides`: uses `TestBackend::new(40, 6)` with `"# Title\n        indented line"` and asserts line number `"1"` in gutter, heading cell foreground equals `rgb_to_color(theme.palette.md_h1)`, and indent guide `'│'` appears on row 1.
   - `viewport_renders_multi_cursor_and_clamps_scroll`: selects two occurrences of `"ab"` in `"ab cd ab"`, renders viewport with `scroll_line = 999`, asserts `scroll_line` clamped to `0` and both cursor positions have `cursor_bg` background.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn render_editor_viewport\b' crates/scriptui-tui/src/viewport.rs && grep -qE 'pub fn ensure_cursor_visible\b' crates/scriptui-tui/src/viewport.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 4F: Full Mouse Event Router

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4F: Implement full MouseRouter in crates/scriptui-tui/src/mouse.rs handling cursor click, drag selection, scroll wheel, tab clicks, preview edge-tab toggle, split divider drag, and right-click spell quick-fix.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/mouse.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/layout_tree.rs:1-75 (ScreenRegions)
  - crates/scriptui-tui/src/chrome.rs:1-40 (TabHitbox)
  - crates/scriptui-tui/src/viewport.rs:1-40 (ViewportMetrics)
  - crates/scriptui-core/src/editor.rs:275-305, 352-445 (EditorState::open_spell_quick_fix_at_cursor, dispatch, PreviewMode)
  - crates/scriptui-core/src/layout.rs:54-65 (WrappedLayout::visual_to_logical)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn render_editor_viewport\b' crates/scriptui-tui/src/viewport.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/lib.rs — add `pub mod mouse;` and `pub use mouse::{MouseContext, MouseRouter};`.
2. crates/scriptui-tui/src/mouse.rs (NEW) — implement:
   - `#[derive(Debug, Clone)] pub struct MouseContext<'a> { pub regions: ScreenRegions, pub tab_hitboxes: &'a [TabHitbox], pub viewport: Option<ViewportMetrics>, pub wrapped_layout: Option<&'a WrappedLayout>, pub total_area: Rect }`
   - `#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)] pub struct MouseRouter { dragging_selection_anchor: Option<usize>, dragging_split_divider: bool }` with:
     - `pub fn handle_mouse(&mut self, event: crossterm::event::MouseEvent, state: &mut EditorState, ctx: &MouseContext<'_>) -> scriptui_core::ScriptuiResult<bool>`:
       - Helper `rect_contains(rect: Rect, col: u16, row: u16) -> bool`.
       - `MouseEventKind::Down(MouseButton::Left)`:
         - If click hits a `TabHitbox` in `ctx.tab_hitboxes`: calls `state.buffers.switch_to(hb.buffer_index)`, clears `find_replace.matches` & `active_match`, returns `Ok(true)`.
         - If click hits `ctx.regions.preview_edge_tab`: dispatches `Action::TogglePreview`, returns `Ok(true)`.
         - If click hits `ctx.regions.split_divider`: sets `self.dragging_split_divider = true`, returns `Ok(true)`.
         - If click hits `viewport.text_area`: maps `(col, row)` → `(visual_row, visual_col)` using `active.scroll_line` / `active.scroll_col`, resolves logical `Position` via `wrapped_layout.visual_to_logical`, converts to char offset via `position_to_char`, sets `SelectionSet::point(offset)`, stores `self.dragging_selection_anchor = Some(offset)`, returns `Ok(true)`.
       - `MouseEventKind::Drag(MouseButton::Left)`:
         - If `self.dragging_split_divider` and `ctx.total_area.width > 0`: computes `preview_pct = (((ctx.total_area.x + ctx.total_area.width).saturating_sub(event.column)) as u32 * 100 / ctx.total_area.width as u32).clamp(20, 80) as u16`, updates `state.config.preview_split_percent = preview_pct` and `state.preview_mode = PreviewMode::Split(preview_pct)`, returns `Ok(true)`.
         - If `let Some(anchor) = self.dragging_selection_anchor` and `viewport` + `wrapped_layout` are present: clamps `(col, row)` into `text_area`, resolves `head` char offset, sets `SelectionSet::from_ranges(vec![Selection::new(anchor, head)])`, returns `Ok(true)`.
       - `MouseEventKind::Up(MouseButton::Left)`: clears `dragging_selection_anchor = None` and `dragging_split_divider = false`, returns `Ok(false)`.
       - `MouseEventKind::Down(MouseButton::Right)`:
         - If click hits `viewport.text_area`: moves primary caret to clicked position (same mapping as left click) and calls `state.open_spell_quick_fix_at_cursor()`, returns `Ok(true)`.
       - `MouseEventKind::ScrollUp` / `MouseEventKind::ScrollDown`:
         - Scrolls `state.buffers.active_mut().scroll_line` by 3 lines (clamped to `0..=max_visual_line`), returns `Ok(true)`.
       - Other events return `Ok(false)`.
3. Tests in `crates/scriptui-tui/src/mouse.rs` (`#[cfg(test)] mod tests`):
   - `mouse_click_drag_scroll_tab_and_divider_routing`: tests left click cursor placement, left drag selection range creation, scroll wheel update, tab hitbox switch, preview edge-tab toggle (`Collapsed` → `Split`), divider drag updating `PreviewMode::Split(pct)`, and right-click on `"hello wrld"` opening `OverlayMode::SpellQuickFix`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct MouseRouter\b' crates/scriptui-tui/src/mouse.rs && grep -qE 'pub fn handle_mouse\b' crates/scriptui-tui/src/mouse.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 4G: Interactive TUI App Loop, Frame Renderer & `main.rs` Entrypoint

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4G: Wire TuiApp frame renderer, keyboard/mouse event loop with OSC 52 clipboard, and `scriptui` binary entrypoint in crates/scriptui-tui.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/Cargo.toml
  - crates/scriptui-tui/src/app.rs (NEW)
  - crates/scriptui-tui/src/main.rs (NEW)
  - crates/scriptui-tui/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/cli.rs:1-60 (Cli, Command, DictCommand, execute_dict_import)
  - crates/scriptui-tui/src/terminal.rs:1-80 (TerminalGuard, crossterm_key_to_chord, write_osc52_clipboard)
  - crates/scriptui-tui/src/layout_tree.rs:1-60 (ScreenRegions, compute_screen_regions)
  - crates/scriptui-tui/src/chrome.rs:1-80 (render_tab_bar, render_preview_edge_tab, render_split_divider, render_status_bar)
  - crates/scriptui-tui/src/viewport.rs:1-80 (ViewportMetrics, render_editor_viewport)
  - crates/scriptui-tui/src/mouse.rs:1-80 (MouseContext, MouseRouter)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub struct MouseRouter\b' crates/scriptui-tui/src/mouse.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/Cargo.toml — add `[[bin]]` entry: `name = "scriptui"`, `path = "src/main.rs"`.
2. crates/scriptui-tui/src/lib.rs — add `pub mod app;` and `pub use app::{FrameState, TuiApp};`.
3. crates/scriptui-tui/src/app.rs (NEW) — implement:
   - `#[derive(Debug, Clone)] pub struct FrameState { pub regions: ScreenRegions, pub tab_hitboxes: Vec<TabHitbox>, pub viewport: Option<ViewportMetrics>, pub wrapped_layout: Option<WrappedLayout>, pub total_area: Rect }`
   - `pub struct TuiApp { pub state: EditorState, pub mouse: MouseRouter, pub last_frame: Option<FrameState> }` with:
     - `pub fn from_cli(cli: &Cli) -> scriptui_core::ScriptuiResult<Self>`:
       - Loads `AppConfig::load_or_default(&AppConfig::default_config_dir().join("config.toml"))` (falling back to `AppConfig::default()` on error).
       - Applies `cli.theme` and `cli.locale` overrides onto `config`.
       - Creates `EditorState::new(config)?` and opens each file in `&cli.files` via `state.buffers.open_file(path)?`.
     - `pub fn render_frame(&mut self, frame: &mut ratatui::Frame)`:
       - Computes `regions = compute_screen_regions(frame.area(), self.state.preview_mode, self.state.overlay)`.
       - Renders tab bar (`tab_hitboxes = render_tab_bar(...)`).
       - If `let Some(editor_rect) = regions.editor`: renders viewport via `render_editor_viewport(...)` and captures `viewport` + `wrapped_layout`.
       - If `let Some(edge_rect) = regions.preview_edge_tab`: calls `render_preview_edge_tab(...)`.
       - If `let Some(div_rect) = regions.split_divider`: calls `render_split_divider(...)`.
       - If `let Some(preview_rect) = regions.preview`: renders bordered preview shell placeholder showing `self.state.build_active_preview(preview_rect.width.saturating_sub(2).max(1) as usize).plain_lines()` (Phase 5 upgrades this to full styled GFM rendering + sync-scroll).
       - Renders `render_status_bar(...)`.
       - Stores `self.last_frame = Some(FrameState { ... })`.
     - `pub fn handle_key_event<W: std::io::Write>(&mut self, key: crossterm::event::KeyEvent, clipboard_out: &mut W) -> scriptui_core::ScriptuiResult<()>`:
       - Ignores non-`KeyEventKind::Press` events.
       - Maps `crossterm_key_to_chord(key)`:
         - If `state.keymap.lookup(chord)` returns `Some(action)`:
           - For `Action::Copy`: writes `self.state.selected_text()` via `write_osc52_clipboard(clipboard_out, ...)` if non-empty.
           - For `Action::Cut`: writes `self.state.selected_text()` via `write_osc52_clipboard(clipboard_out, ...)` if non-empty, then dispatches `Action::Cut`.
           - Otherwise dispatches `self.state.dispatch(action)?`.
         - Else if `key.code == KeyCode::Char(c)` and `!key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT)`: inserts `c.to_string()` via `self.state.insert_text(&s)`.
     - `pub fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) -> scriptui_core::ScriptuiResult<bool>` delegating to `self.mouse.handle_mouse` using `self.last_frame`.
     - `pub fn run_event_loop(&mut self, guard: &mut TerminalGuard) -> std::io::Result<()>`:
       - Loops while `!self.state.should_quit`: draws via `guard.terminal_mut().draw(|f| self.render_frame(f))?`, polls `crossterm::event::poll(Duration::from_millis(50))?`, and routes `Event::Key` and `Event::Mouse` (recording any `ScriptuiError` into `self.state.status_message`).
4. crates/scriptui-tui/src/main.rs (NEW):
   - Parses `Cli::parse()`.
   - If `Some(Command::Dict { action: DictCommand::Import { locale, dic, aff } })`: calls `execute_dict_import(&locale, &dic, &aff, &AppConfig::default_config_dir().join("dicts"))`, prints confirmation, exits 0 (or prints error to stderr and returns `ExitCode::FAILURE`).
   - Otherwise constructs `TuiApp::from_cli(&cli)`, enters `TerminalGuard::enter()`, and runs `app.run_event_loop(&mut guard)`.
5. Tests in `crates/scriptui-tui/src/app.rs` (`#[cfg(test)] mod tests`):
   - `tui_app_renders_full_shell_and_handles_typing_and_osc52_copy`: constructs `TuiApp` on a `TestBackend::new(80, 24)`, sends `'H'`, `'i'` key events, `Ctrl+A`, `Ctrl+C` (asserting OSC 52 buffer contains base64 `"SGk="`), renders a frame, and asserts `last_frame.is_some()` and rendered buffer contains `"Hi"` and `"crt_amber"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub struct TuiApp\b' crates/scriptui-tui/src/app.rs && grep -qE 'fn main\b' crates/scriptui-tui/src/main.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```
