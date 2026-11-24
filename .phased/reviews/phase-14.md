# Phase 14 review

Range: `222f5af4dfe16472772a72f48640ccf76fc3fd5a..ededde6228ffa27e8d6bea48582c23b641db0957`

## Independent pass

1. **`crates/scriptui-tui/src/find_bar.rs:117-239`** (`regression`)
   - **Trigger:** While `OverlayMode::Find` or `OverlayMode::Replace` is active, press any document-editing or selection shortcut bound in the global keymap (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, `Ctrl+X`, `Ctrl+C`, `Ctrl+V`, `Ctrl+A`, `Ctrl+Shift+L`, `Ctrl+.`).
   - **Effect:** Deleting `is_passthrough_shortcut` and changing the catch-all arm from `_ => Ok(true)` to `_ => Ok(false)` causes `TuiApp::handle_key_event` (`crates/scriptui-tui/src/app.rs:270-364`) to fall through to `self.state.keymap.lookup` and mutate the underlying document behind the Find/Replace bar (e.g., `Ctrl+A` + `Ctrl+X` selects and cuts the entire document instead of being trapped; `Ctrl+Z` / `Ctrl+V` silently undo or paste into the buffer). At `<base>`, `_ => Ok(true)` trapped these keys and only passed through an explicit shortcut allowlist.

2. **`crates/scriptui-core/src/editor.rs:863-903`** (`new defect`)
   - **Trigger:** Multi-cursor `Action::InsertNewline` where one cursor's line returns `EnterOutcome::ClearCurrentLine` (e.g., an empty bullet `"- "`) and another non-overlapping selection in `SelectionSet` starts or ends inside that same line (e.g., `"hello\n- "` with `[Selection::new(7, 2), Selection::point(8)]`, or `"- \nhello"` with `[Selection::point(0), Selection::new(1, 5)]`).
   - **Effect:** `planned_edits` collects overlapping ranges (`(line_start, line_end, "")` and `(sel_start, sel_end, insert_text)`). Applying them in reverse via `apply_raw_edit` either returns `Err(ScriptuiError::OutOfBounds)` after the first edit has already mutated the buffer (leaving partial unrecorded state with no rollback or history entry), or silently corrupts surrounding text (deleting the newline just inserted or characters past the cleared line) and computes wrong caret positions.

### Checked
- Full diffs for `.github/workflows/ci.yml`, `crates/scriptui-core/src/buffer.rs`, `crates/scriptui-core/src/editor.rs`, `crates/scriptui-tui/src/find_bar.rs`, `crates/scriptui-tui/src/modals.rs`, and `crates/scriptui-tui/src/palette.rs`.
- Sibling `SelectionSet::from_ranges` call sites across `crates/`, `UndoStack` edit ordering in `crates/scriptui-core/src/history.rs`, keymap bindings in `crates/scriptui-core/src/keymap.rs`, and overlay dispatch in `crates/scriptui-tui/src/app.rs`.
- Ran `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (all passed).
- Checked all changed files for sensitive data (none found).

### Not checked
- Did not run `cargo deny check` or `cargo build --release --locked` locally, nor execute `.github/workflows/ci.yml` on GitHub Actions `ubuntu-latest` / `macos-14` runners.

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step-by-step check
- **STEP 14A (`TASK-0077`, `be6c288`) — Advances `BUG-0037`, `BUG-0040`:**
  - `modals.rs` and `palette.rs` modifier filtering (`SUPER | HYPER | META` on char input; strict modifier checks on confirm modal `y`/`n`/`Enter`/`Esc`) match contract and pass tests (`BUG-0040` resolved).
  - `find_bar.rs` followed the contract literally ("Return `Ok(false)` for unhandled keys (do not swallow unmatched keys with `_ => Ok(true)`)"), which passes through global shortcuts (`BUG-0037`) **but** caused Independent Finding #1 (regression: document-editing `Ctrl` shortcuts now leak through `TuiApp::handle_key_event` to mutate the editor buffer). Not withdrawn or downgraded.
- **STEP 14B (`TASK-0078`, `aed0790`) — Advances `BUG-0038`, `TASK-0076`:**
  - `DocumentBuffer::apply_insert` and `apply_delete` preserve `primary_index` via `SelectionSet::from_ranges_with_primary`; `from_file_rejects_files_exceeding_max_file_bytes` exercises the real `MAX_FILE_BYTES` guard. Contract fully met.
- **STEP 14C (`TASK-0079`, `206333d`) — Advances `BUG-0039`:**
  - Per-cursor `EnterOutcome` handling in `handle_insert_newline` satisfies the mixed point-cursor contract and undo/redo test (`BUG-0039`), but fails to guard or coalesce overlapping `planned_edits` when a non-empty selection starts/ends inside a `ClearCurrentLine` line (Independent Finding #2).
- **STEP 14D (`TASK-0080`, `5167e8d`):**
  - `.github/workflows/ci.yml` matches all contract requirements (`name: CI`, `master` push/PR, `contents: read`, `check` on `ubuntu-latest` with fmt/clippy/test/cargo-deny/release-locked, `macos-test` on `macos-14`).
- **Deferrals / Amendments / Unrequested changes:** None. All 4 steps landed inside `ALLOWED_FILES` with zero deferrals and zero amendments.

### Origins of independent findings
1. `crates/scriptui-tui/src/find_bar.rs:117-239` (`regression`) → **origin: `plan`** (STEP 14A contract explicitly instructed returning `Ok(false)` for all unmatched keys instead of restricting passthrough to non-editing global actions). Filed as **`BUG-0041`** (`pending`, `medium`).
2. `crates/scriptui-core/src/editor.rs:863-903` (`new defect`) → **origin: `coder`** (STEP 14C combined `line_start..line_end` clears with `sel_start..sel_end` inserts in `planned_edits` without resolving overlapping spans before reverse application). Filed as **`BUG-0042`** (`pending`, `medium`).

## Handoff

**Verdict:** Conditional pass — all 4 Phase 14 steps met their contracts with 0 deferrals and passing `fmt`/`clippy`/`test`, but 2 medium defects were found (1 plan-origin regression in `find_bar.rs`, 1 coder-origin overlap bug in `editor.rs`).

**New pending items (by priority):**
- `BUG-0041` (`medium`, `regression`, `plan`): `find_bar.rs:117-239` catch-all `_ => Ok(false)` leaks `Ctrl+Z/Y/X/C/V/A` and `Ctrl+Shift+L`/`Ctrl+.` into the editor behind Find/Replace. Fix by allowing only non-editing global shortcuts (or filtering actions in `app.rs`).
- `BUG-0042` (`medium`, `new defect`, `coder`): `editor.rs:863-903` `handle_insert_newline` can produce overlapping `planned_edits` when a non-empty selection crosses a `ClearCurrentLine` line, causing partial unrecorded `OutOfBounds` failure or silent text corruption.

**Deferred steps to re-split:** None.

**Advances items ready to close (pending user approval):**
- `BUG-0038`, `BUG-0039`, `BUG-0040`, `TASK-0076` (cleanly fixed; `BUG-0042` tracks the overlap edge case).
- `BUG-0037` (fixed for global shortcuts, regressed editing-shortcut isolation into `BUG-0041` — user to confirm close vs. hold open).

**User decisions for next phase (`phase-plan`):**
- Fix `BUG-0041` and `BUG-0042` before/alongside `.github/workflows/release.yml` + `v0.1.0` cut.
- Resolve **OQ-2** (`release.yml` dry-run: prerelease tag vs. `workflow_dispatch`).
