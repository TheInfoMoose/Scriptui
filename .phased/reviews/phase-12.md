# Phase 12 review

Range: `a8b26742af4d15e3596e07d77bda5c083a80f5ea..615cc6d0f770ec54bd56a2438fd884cd4a7bf043`

## Independent pass

### Findings

1. **`crates/scriptui-core/src/buffer.rs:301-336` / `crates/scriptui-core/src/editor.rs:374-384` / `crates/scriptui-core/src/history.rs:38-55` — No-op empty insert records an undo entry and desyncs `UndoStack::is_at_saved_state()`**
   - **Class:** `new defect`
   - **Trigger:** Calling `EditorState::insert_text("")` (for example, an empty `Event::Paste("")`) on a clean buffer with a point selection.
   - **Effect:** `DocumentBuffer::apply_insert("")` still pushes a no-op `BufferEdit { start, end, inserted: "", removed: "" }` into `edits`. Although `buffer.rs:328-335` avoids setting `dirty = true`, `EditorState::insert_text` passes the non-empty `edits` vector to `UndoStack::record`, which only checks `tx.edits.is_empty()`. This pushes a no-op transaction onto `undo_entries`, making `is_at_saved_state()` false; after a subsequent real edit and one `Undo`, `dirty` remains `true` even though the document text is back at the saved state.

2. **`crates/scriptui-core/src/spell.rs:452-467` — Partial locale write if second `fs::rename` fails in `import_locale_files`**
   - **Class:** `old bug`
   - **Trigger:** `fs::rename(&dic_tmp, &dic_dest)` succeeds and `fs::rename(&aff_tmp, &aff_dest)` subsequently fails.
   - **Effect:** `dic_dest` is created or overwritten on disk while `aff_dest` is not updated, leaving an inconsistent `.dic`/`.aff` pair without rolling back `dic_dest`.

3. **`crates/scriptui-tui/src/app.rs:415-416` — `run_event_loop` does not ignore `ErrorKind::Interrupted` from `event::poll` / `event::read`**
   - **Class:** `new defect`
   - **Trigger:** A Unix signal (`SIGTERM`, `SIGHUP`, `SIGTSTP`) arrives while `crossterm::event::poll` or `event::read` is blocked, if the underlying OS call surfaces `io::ErrorKind::Interrupted` (`EINTR`).
   - **Effect:** `run_event_loop` returns `Err(Interrupted)` and `main` exits with `ExitCode::FAILURE` instead of continuing to check `guard.should_terminate()` or `guard.handle_suspend_if_requested()`.

### What was traced
- All 13 modified Rust source files (`buffer.rs`, `buffers.rs`, `editor.rs`, `highlight.rs`, `history.rs`, `indent.rs`, `spell.rs`, `app.rs`, `main.rs`, `modals.rs`, `palette.rs`, `preview_pane.rs`, `terminal.rs`), `Cargo.toml` manifests, `deny.toml`, and `THIRD_PARTY_LICENSES.md` were diffed in full against `a8b26742af4d15e3596e07d77bda5c083a80f5ea`.
- Verified `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all pass (65 core unit tests, 5 `editor_state_pipeline` tests, 1 `markdown_spell_pipeline` test, 26 tui unit tests, 14 `tui_integration` tests).

### What was not checked
- Live interactive terminal execution of Unix signal delivery (`SIGTERM`, `SIGHUP`, `SIGTSTP`/`SIGCONT` suspend and resume) and real terminal OSC 52 / raw mode restoration.
- `cargo deny check` execution (not run in this review).
- `Cargo.lock` diff details.

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step Compliance
- **STEP 12A (`TASK-0060`, `TASK-0061`, `TASK-0062`):** Met. `deny.toml`, `THIRD_PARTY_LICENSES.md`, workspace `repository`/`rust-version`, and pinned `scriptui-core` version all match contract; `cargo deny --offline check` exits `0` (`advisories ok, bans ok, licenses ok, sources ok`).
- **STEP 12B (`BUG-0017`, `BUG-0020` part 1, `BUG-0025`):** Met. UTF-8 safe `parse_task_list`, `DocumentBuffer::empty_with_path`, `apply_insert` dirty/revision guard, and `O_EXCL` atomic writes in `buffer.rs`/`spell.rs` implemented and tested.
- **STEP 12C (`BUG-0019`, `BUG-0020` part 2, `BUG-0018` core):** Met. `UndoStack` save-depth tracking, `EditorState::new_with_dirs`, and internal clipboard `Copy`/`Cut`/`Paste` implemented and tested; `mark_saved()` wired in `editor.rs` and `modals.rs`.
- **STEP 12D (`BUG-0018`, `BUG-0019` TUI, `BUG-0021`, `BUG-0022`):** Met. `open_or_create_file`, `discover_custom_themes`, `build_palette_entries_with_themes`, `TuiApp::from_cli` config warning + custom dirs, `KeyEventKind::Repeat`, and `Action::Copy` forwarding implemented and tested.
- **STEP 12E (`BUG-0023`):** Met. `DEFAULT_SYNTAX_SET` (`LazyLock`) and `PreviewPaneState` `CachedPreview` implemented and tested. Drops (`Copy`, `PartialEq`, `Eq` removed from `PreviewPaneState`) match contract.
- **STEP 12F (`BUG-0024`):** Met. `signal-hook = "0.3"` under `[target.'cfg(unix)'.dependencies]`, `TerminalGuard` `SIGTERM`/`SIGHUP`/`SIGTSTP` handling, `run_event_loop` polling, and `main.rs` guard drop ordering implemented and tested.

### Finding Origins & Tracker IDs
1. **Finding 1 (`BUG-0034`, low):** Origin `coder` — `apply_insert("")` returns a no-op `BufferEdit` that `UndoStack::record` still pushes.
2. **Finding 2 (`BUG-0036`, low):** Origin `pre-existing` — `import_locale_files` does not roll back `dic_dest` if the second `fs::rename` fails.
3. **Finding 3 (`BUG-0035`, medium):** Origin `plan` + `coder` — `run_event_loop` does not ignore `ErrorKind::Interrupted` from `event::poll`/`event::read` when signals arrive.

### Deviations & Deferrals
- No deferred steps in Phase 12 (`STEP 12A`–`12F` all completed).
- Minor code duplication (non-blocking): `create_unique_temp_file` is duplicated between `crates/scriptui-core/src/buffer.rs` and `crates/scriptui-core/src/spell.rs`.

### `Advances:` Items Ready to Close
- `TASK-0060`, `TASK-0061`, `TASK-0062`
- `BUG-0017`, `BUG-0018`, `BUG-0019`, `BUG-0020`, `BUG-0021`, `BUG-0022`, `BUG-0023`, `BUG-0024`, `BUG-0025`

## Handoff

- **Verdict:** PASS WITH FINDINGS (all 6 steps `12A`–`12F` implemented, `fmt`/`clippy`/`test`/`cargo deny --offline check` pass; 0 regressions, 3 non-blocking findings filed).
- **New pending items by priority:**
  - `medium`: `BUG-0035` (`run_event_loop` `EINTR` handling on `event::poll`/`read`)
  - `low`: `BUG-0034` (no-op `insert_text("")` records undo entry, desyncing `is_at_saved_state()`), `BUG-0036` (`import_locale_files` partial `.dic` on second `fs::rename` failure)
- **Deferred steps to re-split:** None.
- **Decisions for user:** Approve closing Phase 12 `Advances:` items (`TASK-0060`–`TASK-0062`, `BUG-0017`–`BUG-0025`).
