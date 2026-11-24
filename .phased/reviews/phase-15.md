# Phase 15 review

Range: `e8f5015a3c5c3305cf4d8bdf36bb957c3bb3f303..f9aebfdb7f56e9d634946b8c3b50b971e843bc34`

## Independent pass

### Findings

1. **`crates/scriptui-core/src/editor.rs:877-898` (`regression`)** — Point cursors on the same empty list/blockquote continuation line falsely trigger `overlapped_by_other`.
   - **Trigger:** Pressing Enter (`Action::InsertNewline`) with two or more point cursors on the same empty continuation line where at least one cursor is strictly inside `(line_start, line_end)` (e.g., `"- "` with `Selection::point(0)` and `Selection::point(1)`, or `Selection::point(1)` and `Selection::point(2)`; or `"  - "` with `Selection::point(1)` and `Selection::point(2)`).
   - **Effect:** `overlapped_by_other` tests `other_start < line_end && other_end > line_start` without excluding empty point cursors (`!other.is_empty()`). When one point cursor is at a line boundary (`line_start` or `line_end`) and another is strictly inside, the check is asymmetric: the boundary cursor falls back to inserting `"\n"` while the interior cursor clears `(line_start, line_end)`, leaving `"\n"` instead of `""` (and inverting cursor positions when the boundary cursor is at `line_start`). When two point cursors are both strictly inside `(line_start, line_end)`, both fall back to `Insert` and split the marker (`" \n \n- "`) instead of clearing the line. At `e8f5015a`, all point cursors on the same empty continuation line deduplicated via `cleared_lines.insert(line)` and cleared the line cleanly to `""`.

### Traced & verified clean

- **`.github/workflows/release.yml`**: Valid YAML; matrix targets `x86_64-unknown-linux-gnu` (`ubuntu-latest`) and `aarch64-apple-darwin` (`macos-14`) match `[[bin]] name = "scriptui"` in `crates/scriptui-tui/Cargo.toml`; `README.md`, `LICENSE`, `THIRD_PARTY_LICENSES.md`, and `Cargo.lock` are all tracked in git; top-level `permissions: contents: read` with `contents: write` scoped to `publish`; `publish` gated on `startsWith(github.ref, 'refs/tags/v')`; SHA256 generation handles both `sha256sum` and `shasum -a 256`; no sensitive data.
- **`crates/scriptui-tui/src/find_bar.rs`**: `is_find_bar_passthrough_action` whitelists the 17 app/file/buffer/search/view actions and traps all document-editing, undo/redo, clipboard, selection, navigation, and spell quick-fix shortcuts in both `Find` and `Replace` overlays via `crossterm_key_to_chord` + `state.keymap.lookup`; `find_bar_traps_document_editing_shortcuts_end_to_end` exercises both `handle_find_bar_key` and `TuiApp::handle_key_event` end-to-end without loosening or deleting existing tests.
- **`crates/scriptui-core/src/editor.rs:916-933`**: Pre-mutation bounds/ordering/disjointness validation prevents partial `apply_raw_edit` mutations; `insert_newline_multicursor_overlapping_clear_line_and_selection_is_atomic` verifies Cases A, B, and C plus undo restoration.
- **Static & test gates**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (76 core unit, 37 TUI unit, 14 integration tests) all pass. No sensitive data found in any changed file.

### Not checked

- Live GitHub Actions execution of `.github/workflows/release.yml` (cross-platform runner builds, artifact upload/download, and `softprops/action-gh-release@v2` tag publication).

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step-by-step verification

- **STEP 15A (`TASK-0081`, advances `BUG-0041`) — Met:** `crates/scriptui-tui/src/find_bar.rs` matches all `Contract & Behavior`, `Must still work`, and `Tests` requirements within `[SCOPE_LOCK]`. No unrequested changes.
- **STEP 15B (`TASK-0082`, advances `BUG-0042`) — Partial (`Must still work` unmet):** `crates/scriptui-core/src/editor.rs` implements `Contract & Behavior` verbatim within `[SCOPE_LOCK]` and resolves the overlapping-selection crash/corruption (`BUG-0042`). However, `Must still work` explicitly requires "multiple point cursors on the same empty bullet line," which regresses when any point cursor lies strictly inside `(line_start, line_end)` because Contract 1(b)'s formula (`other_start < line_end && other_end > line_start`) fails to exclude empty sibling cursors (`!other.is_empty()`). Validator did not catch this because no test placed two point cursors on the same empty bullet line.
- **STEP 15C (`TASK-0083`) — Met:** `.github/workflows/release.yml` matches all `Contract & Behavior`, `Must still work`, and `POST_GATE` checks within `[SCOPE_LOCK]`. No unrequested changes.
- **Deferrals & amendments:** None in Phase 15.

### Finding origins & tracker actions

| Finding | Location | Class | Origin | Filed |
|---|---|---|---|---|
| 1 | `crates/scriptui-core/src/editor.rs:877-898` | `regression` | `plan` (Contract 1(b) omitted `!other.is_empty()`, contradicting `Must still work`) | `BUG-0043` (`pending`, medium) |

- **`Advances:` ready to close (after `approved`):** `BUG-0041` (STEP 15A), `BUG-0042` (STEP 15B; original overlapping-selection defect resolved, new point-cursor regression tracked separately in `BUG-0043`).

## Handoff

- **Verdict:** Pass with 1 medium regression (`BUG-0043`).
- **New pending items (by priority):**
  - `BUG-0043` (medium): `handle_insert_newline` breaks multiple point cursors on the same empty list/quote line (`crates/scriptui-core/src/editor.rs:877-898`; guard `overlapped_by_other` with `!other.is_empty()` and add multi-point-cursor regression test).
- **Ready to close on `approved`:** `BUG-0041`, `BUG-0042`.
- **Deferred steps to re-split:** None.
- **Not checked:** Live GitHub Actions run of `.github/workflows/release.yml`.
- **Decisions for user:**
  1. Confirm closing `BUG-0041` and `BUG-0042` (`approved`).
  2. Schedule `BUG-0043` in Phase 16 and optionally trigger a `workflow_dispatch` dry-run of `.github/workflows/release.yml` on GitHub.
