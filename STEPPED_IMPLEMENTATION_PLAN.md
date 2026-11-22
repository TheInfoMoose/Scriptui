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

**Phase goal (Phase 7):** Resolve both Phase 6 review findings (`BUG-0010` and `BUG-0011`): key `AnalysisCache` on a monotonic `DocumentBuffer` identity and route `OpenFilePrompt` through `EditorState::open_file` so opening or reloading a file into buffer 0 never serves stale GFM/spell diagnostics (`BUG-0010`), and align `WrappedLayout` cursor/mouse column mapping and `render_editor_viewport` cell rendering with Unicode display widths and tab-stop expansion (`BUG-0011`).

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 7A** | `/goal` | core | `crates/scriptui-core/src/buffer.rs`, `crates/scriptui-core/src/editor.rs`, `crates/scriptui-tui/src/modals.rs` | — |
| 2 | **STEP 7B** | `/goal` | core | `crates/scriptui-core/src/layout.rs`, `crates/scriptui-core/src/lib.rs` | STEP 7A |
| 3 | **STEP 7C** | `/goal` | tui | `crates/scriptui-tui/src/viewport.rs`, `crates/scriptui-tui/src/mouse.rs` | STEP 7B |

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
| **STEP 6A** | Done (`835176c`) | `/goal` | core | `crates/scriptui-core/src/buffer.rs`, `crates/scriptui-core/src/editor.rs` |
| **STEP 6B** | Done (`a3c05b0`) | `/goal` | tui | `crates/scriptui-tui/src/find_bar.rs`, `crates/scriptui-tui/src/preview_pane.rs`, `crates/scriptui-tui/src/app.rs` |
| **STEP 6C** | Done (`1096f90`) | `/goal` | core | `crates/scriptui-core/Cargo.toml`, `Cargo.lock`, `crates/scriptui-core/src/layout.rs` |
| **STEP 6D** | Done (`3845e5b`) | `/goal` | core | `crates/scriptui-core/src/editor.rs`, `crates/scriptui-tui/src/viewport.rs` |
| **STEP 6E** | Done (`1141a77`) | `/goal` | tui | `crates/scriptui-tui/src/terminal.rs`, `crates/scriptui-tui/src/app.rs` |
| **STEP 6F** | Done (`176be56`) | `/goal` | tui | `crates/scriptui-tui/tests/tui_integration.rs` |
| **STEP 6G** | Done (`872842e`) | `/goal` | docs | `README.md` |

### Future phases

_None — Phase 7 resolves all remaining Phase 6 review findings._

### Open Questions

_None._

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.
_None — both open bugs (`BUG-0010`, `BUG-0011`) are scheduled in Phase 7._

---

## Phase 7 Step Blocks

### STEP 7A: Monotonic `DocumentBuffer` ID & `AnalysisCache` Invalidation on File Open/Reload (`BUG-0010`)

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0010`

```text
[STEP_ID]
STEP 7A: Add a monotonic doc_id to DocumentBuffer, key EditorState::AnalysisCache on doc_id, add EditorState::open_file and invalidate_analysis_cache, and route OpenFilePrompt through EditorState::open_file (BUG-0010).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/buffer.rs
  - crates/scriptui-core/src/editor.rs
  - crates/scriptui-tui/src/modals.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:19-70, 150-170 (DocumentBuffer fields, constructors, revision)
  - crates/scriptui-core/src/buffers.rs:106-152, 218-235 (BufferManager::open_file replace_initial branch, reload_active_from_disk)
  - crates/scriptui-core/src/editor.rs:215-315 (AnalysisCache, EditorState::active_analysis)
  - crates/scriptui-tui/src/modals.rs:285-310 (handle_file_prompt_key OpenFilePrompt Enter branch)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn revision\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn active_analysis\b' crates/scriptui-core/src/editor.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/buffer.rs:
   - Add `use std::sync::atomic::{AtomicU64, Ordering};` and a private helper:
     `static NEXT_DOCUMENT_ID: AtomicU64 = AtomicU64::new(1);`
     `fn next_document_id() -> u64 { NEXT_DOCUMENT_ID.fetch_add(1, Ordering::Relaxed) }`
   - Add `doc_id: u64` field to `DocumentBuffer`, initialized via `next_document_id()` in `empty`, `from_str_content`, and `from_file`.
   - Expose `pub fn doc_id(&self) -> u64 { self.doc_id }`.
2. crates/scriptui-core/src/editor.rs:
   - Update `AnalysisCache` to store `doc_id: u64` alongside `buffer_index: usize`, `doc_revision: u64`, and `spell_revision: u64`.
   - In `active_analysis(&mut self)`:
     - Read `let doc_id = self.buffers.active().document.doc_id();`.
     - Check `cache.doc_id == doc_id && cache.buffer_index == buffer_index && cache.doc_revision == doc_revision && cache.spell_revision == spell_revision`.
     - Populate `doc_id` when constructing `AnalysisCache`.
   - Add `pub fn invalidate_analysis_cache(&mut self) { self.analysis_cache = None; }`.
   - Add `pub fn open_file(&mut self, path: &Path) -> ScriptuiResult<crate::buffers::BufferId>` that calls `let id = self.buffers.open_file(path)?;`, clears `self.find_replace.matches`, resets `self.find_replace.active_match = None`, clears `self.analysis_cache = None`, and returns `Ok(id)`.
   - Add unit test `active_analysis_recomputes_when_open_file_replaces_initial_empty_buffer`:
     - Construct `EditorState::new(AppConfig::default())`.
     - Call `editor.active_analysis()` on the initial empty buffer (`analysis_recompute_count() == 1`, zero diagnostics).
     - Write a temp file containing `"# Heading\nhello wrld\n"`.
     - Open it via `editor.open_file(&temp_path)` (which replaces `buffers[0]` at `active_index == 0`, `revision == 0`).
     - Call `editor.active_analysis()` and assert `analysis_recompute_count() == 2`, `gfm.spans` is non-empty, and `spell_diagnostics` contains `"wrld"`.
     - Overwrite the temp file with `"clean text\n"`, call `editor.buffers.reload_active_from_disk()`, call `editor.active_analysis()`, and assert `analysis_recompute_count() == 3` and `spell_diagnostics` is empty.
3. crates/scriptui-tui/src/modals.rs:
   - In `handle_file_prompt_key`, inside `OverlayMode::OpenFilePrompt` `Enter` branch, replace `state.buffers.open_file(&PathBuf::from(&path))?;` and the manual `find_replace` clearing with `state.open_file(&PathBuf::from(&path))?;`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn doc_id\b' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn open_file\b' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 7B: Display-Cell Column Mapping Helpers in `scriptui-core/src/layout.rs` (`BUG-0011` Part 1)

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0011`

```text
[STEP_ID]
STEP 7B: Add Unicode display-cell and tab-stop column mapping helpers to WrappedLayout in crates/scriptui-core/src/layout.rs and re-export them from scriptui-core (BUG-0011).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/layout.rs
  - crates/scriptui-core/src/lib.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/layout.rs:21-98 (WrappedLayout, char_display_width, line_display_width)
  - crates/scriptui-core/src/lib.rs:35 (layout re-exports)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No ratatui or crossterm imports in scriptui-core.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn char_display_width\b' crates/scriptui-core/src/layout.rs

[MUTATION_SPEC]
1. crates/scriptui-core/src/layout.rs:
   - Add `pub fn logical_col_to_display_col(line: &str, char_start: usize, logical_col: usize, tab_width: usize) -> usize`:
     - Iterates over `line.chars().skip(char_start).take(logical_col.saturating_sub(char_start))` accumulating `col += char_display_width(ch, col, tab_width)` and returns `col`.
   - Add `pub fn display_col_to_logical_col(line: &str, char_start: usize, char_end: usize, target_cell_col: usize, tab_width: usize) -> usize`:
     - Walks characters in `char_start..char_end` (via `line.chars().skip(char_start).take(char_end.saturating_sub(char_start)).enumerate()`), tracking `cell_col`.
     - For each character `(offset, ch)`, compute `let w = char_display_width(ch, cell_col, tab_width);`.
     - If `target_cell_col < cell_col + w`, return `char_start + offset` (clicking anywhere within a tab stop or wide character maps to that character's logical column).
     - Advance `cell_col += w`.
     - If `target_cell_col >= cell_col` after the loop, return `char_end`.
   - Add methods on `WrappedLayout`:
     - `pub fn logical_to_visual_cells(&self, buf: &DocumentBuffer, pos: Position, tab_width: usize) -> (usize, usize)`:
       - Finds the visual line for `pos` using the same wrap-segment selection rule as `logical_to_visual` (matching `vl.logical_line == pos.line && pos.col >= vl.char_start && pos.col < vl.char_end`, else `last_for_line` clamped to `vl.char_end`, else last visual line end).
       - Reads `let line = buf.line_text(vl.logical_line).unwrap_or_default();` and computes the display column via `logical_col_to_display_col(&line, vl.char_start, clamped_col, tab_width)`.
     - `pub fn visual_cells_to_logical(&self, buf: &DocumentBuffer, visual_row: usize, cell_col: usize, tab_width: usize) -> Position`:
       - Clamps `visual_row` to `self.visual_lines.len() - 1` (returning `Position { line: 0, col: 0 }` if empty).
       - Reads `let line = buf.line_text(vl.logical_line).unwrap_or_default();` and returns `Position { line: vl.logical_line, col: display_col_to_logical_col(&line, vl.char_start, vl.char_end, cell_col, tab_width) }`.
   - Preserve existing `logical_to_visual` and `visual_to_logical` unchanged for backward compatibility.
   - Add unit tests in `crates/scriptui-core/src/layout.rs`:
     - `display_cell_mapping_handles_tabs_and_cjk_wide_chars`:
       - On `"\tabc"` with `tab_width = 4`: `logical_to_visual_cells` maps col `0 -> 0`, col `1 -> 4`, col `2 -> 5`; `visual_cells_to_logical` maps cells `0..=3 -> col 0`, cell `4 -> col 1`, cell `5 -> col 2`, cell `99 -> col 4`.
       - On `"日本語a"` with `viewport_width = 20`: `logical_to_visual_cells` maps col `0 -> 0`, col `1 -> 2`, col `2 -> 4`, col `3 -> 6`; `visual_cells_to_logical` maps cells `0,1 -> col 0`, `2,3 -> col 1`, `4,5 -> col 2`, `6 -> col 3`.
       - On wrapped `"日本語テスト"` at `viewport_width = 6`: `logical_to_visual_cells` maps `(0, 4) -> (1, 2)` and `visual_cells_to_logical(1, 2) -> Position { line: 0, col: 4 }`.
2. crates/scriptui-core/src/lib.rs:
   - Re-export `char_display_width`, `display_col_to_logical_col`, `line_display_width`, and `logical_col_to_display_col` from `layout`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn logical_to_visual_cells\b' crates/scriptui-core/src/layout.rs && grep -qE 'pub fn visual_cells_to_logical\b' crates/scriptui-core/src/layout.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 7C: Display-Cell Viewport Rendering, Caret Placement & Mouse Hit-Testing (`BUG-0011` Part 2)

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0011`

```text
[STEP_ID]
STEP 7C: Update render_editor_viewport to expand tabs and render wide Unicode cells accurately, and switch viewport caret placement and MouseRouter click targeting to display-cell coordinates (BUG-0011).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/viewport.rs
  - crates/scriptui-tui/src/mouse.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/viewport.rs:117-160, 285-336 (render_editor_viewport layout, line cell loop, cursor overlay)
  - crates/scriptui-tui/src/mouse.rs:194-211 (resolve_char_offset)
  - crates/scriptui-core/src/layout.rs:82-105 (char_display_width, logical_to_visual_cells, visual_cells_to_logical)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - No .unwrap() or .expect() in non-test code.

[PRE_GATE]
grep -qE 'pub fn logical_to_visual_cells\b' crates/scriptui-core/src/layout.rs && grep -qE 'pub fn visual_cells_to_logical\b' crates/scriptui-core/src/layout.rs

[MUTATION_SPEC]
1. crates/scriptui-tui/src/viewport.rs:
   - Import `char_display_width` from `scriptui_core`.
   - Replace `layout.logical_to_visual(primary_pos)` with `layout.logical_to_visual_cells(&state.buffers.active().document, primary_pos, tab_width)` before `ensure_cursor_visible`.
   - Replace the 1-char-per-cell rendering loop (`for col_offset in 0..visible_width`) with a display-cell loop over `visual_slice`:
     - Track `display_col = 0usize` from the start of the visual slice.
     - For each `(offset, &(ch, style))` in `visual_slice.iter().enumerate()`:
       - Compute `let w = char_display_width(ch, display_col, tab_width);`.
       - If `w == 0`, continue.
       - If `ch == '\t'`:
         - For `step in 0..w`:
           - Let `cell_col = display_col + step;`
           - If `cell_col >= h_scroll && cell_col < h_scroll + visible_width`:
             - Let `x = text_area.x.saturating_add((cell_col - h_scroll) as u16);`
             - If `step == 0 && vl.indent_guides.contains(&cell_col)` (on `vl.wrap_index == 0`), draw `"│"` with `muted_fg` + `style.bg.unwrap_or(bg_color)`; else draw `" "` with `style`.
       - Else (normal or wide char):
         - Let `logical_col = vl.char_start.saturating_add(offset);`
         - If `display_col >= h_scroll && display_col + w <= h_scroll + visible_width`:
           - Let `x = text_area.x.saturating_add((display_col - h_scroll) as u16);`
           - If `ch == ' ' && vl.wrap_index == 0 && vl.indent_guides.contains(&display_col)`:
             - Draw `"│"` with `muted_fg` + `style.bg.unwrap_or(bg_color)`.
           - Else:
             - Write `ch` at `(x, y)` with `style`.
             - For `cont in 1..w`: set `(x.saturating_add(cont as u16), y)` symbol to `""` and style to `style` (preventing wide-char continuation cell artifacts).
       - Advance `display_col += w`.
       - If `display_col >= h_scroll.saturating_add(visible_width)`, break.
   - In the cursor overlay loop, replace `layout.logical_to_visual(head_pos)` with `layout.logical_to_visual_cells(&active.document, head_pos, tab_width)`.
   - Add unit test `viewport_expands_tabs_and_aligns_cjk_wide_chars_and_cursor`:
     - Render `"\tabc\n日本語X"` with `IndentStyle::Tabs` (tab width 4) and cursor on `'a'` (line 0, col 1): assert cells `0..4` in `text_area` are spaces (`"│"` at col 0 if indent guide applies, spaces at `1..4`), cell `4` is `"a"` with cursor bg `rgb_to_color(palette.cursor_bg)`.
     - Move cursor to `'X'` (line 1, col 3) and re-render: assert `'日'` is at cell 0, `'本'` at cell 2, `'語'` at cell 4, and `'X'` is at cell 6 with cursor bg `rgb_to_color(palette.cursor_bg)`.
2. crates/scriptui-tui/src/mouse.rs:
   - Import `IndentStyle` from `scriptui_core`.
   - In `resolve_char_offset`, compute:
     ```rust
     let tab_width = match state.config.indent.style {
         IndentStyle::Spaces(w) => w.max(1) as usize,
         IndentStyle::Tabs => 4,
     };
     ```
     and replace `wrapped_layout.visual_to_logical(visual_row, visual_col)` with `wrapped_layout.visual_cells_to_logical(&active.document, visual_row, visual_col, tab_width)`.
   - Add unit test `mouse_click_maps_display_cells_for_tabs_and_cjk_wide_chars`:
     - On `"\tabc\n日本語X"` rendered in a `TestBackend`, left-clicking visual cell 4 on row 0 places caret at char offset 1 (`'a'`), left-clicking visual cell 2 on row 0 places caret at char offset 0 (`'\t'`), and left-clicking visual cell 6 on row 1 places caret on `'X'` (line 1, col 3).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'logical_to_visual_cells' crates/scriptui-tui/src/viewport.rs && grep -qE 'visual_cells_to_logical' crates/scriptui-tui/src/mouse.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```
