# Phase 13 review

Range: `aedcc05..75dbf32`

## Independent pass

1. **`crates/scriptui-tui/src/find_bar.rs:108-116, 204-212, 232` — `regression`**
   - **Trigger:** While `OverlayMode::Find` or `OverlayMode::Replace` is open, press a global shortcut outside `{Ctrl+F, Ctrl+H, Ctrl+Q, Ctrl+S, F3, Shift+F3}` (e.g., `Ctrl+PageDown`/`Ctrl+PageUp` to switch buffers, `Ctrl+W` to close buffer, `Ctrl+N` for new buffer, `Ctrl+O` to open file, `Ctrl+Shift+S` for save-as, `Ctrl+Shift+P`/`F1` for command palette).
   - **Effect:** `handle_find_bar_key` now swallows all non-whitelisted keys (`KeyCode::PageUp | KeyCode::PageDown => Ok(true)` and `_ => Ok(true)`), whereas at `aedcc05` it returned `Ok(false)` so global shortcuts passed through to `TuiApp::handle_key_event`. This also makes the keyboard-triggered `NextBuffer`/`PrevBuffer`/`CloseBuffer`/`NewBuffer` find-bar reset branch in `crates/scriptui-tui/src/app.rs:348-360` unreachable while Find/Replace is open.

2. **`crates/scriptui-core/src/buffer.rs:345, 405` — `old bug`**
   - **Trigger:** Hold a multi-cursor `SelectionSet` with `primary_index() > 0` and run `DocumentBuffer::apply_insert` or `DocumentBuffer::apply_delete`.
   - **Effect:** Both methods rebuild `self.selection` via `SelectionSet::from_ranges(new_ranges)`, hardcoding `primary = 0` and discarding `self.selection.primary_index()`. Although `SelectionSet::normalize`, cursor motion, indent/dedent, and `ClearCurrentLine` now preserve `primary_index`, typing or deleting silently resets the primary cursor back to index `0`.

3. **`crates/scriptui-core/src/editor.rs:844-905` — `new defect`**
   - **Trigger:** Hold multiple cursors where the primary cursor is on an empty list marker line (`- `, `EnterOutcome::ClearCurrentLine`) and a secondary cursor is on a regular text line (`hello`, `EnterOutcome::Insert("\n")`), then dispatch `Action::InsertNewline` (or the reverse).
   - **Effect:** `handle_insert_newline` branches solely on the primary cursor's line. When primary is `ClearCurrentLine`, secondary cursors on non-clear lines receive no newline at all; when primary is `Insert`, secondary cursors on empty list markers are not cleared.

4. **`crates/scriptui-tui/src/modals.rs:207-214, 277-280` & `crates/scriptui-tui/src/palette.rs:320-323` — `old bug`**
   - **Trigger:** Press a `SUPER`, `HYPER`, or `META`-modified character key (e.g. `Super+S` or `Super+Y`) while `OpenFilePrompt`, `SaveAsPrompt`, `CommandPalette`, or `ConfirmQuit`/`ConfirmCloseBuffer` is active.
   - **Effect:** Unlike `app.rs:366-371` and `find_bar.rs:214-219`, `handle_file_prompt_key` and `handle_palette_key` only filter `CONTROL | ALT` (so `Super+S` inserts `'s'`), and `handle_confirm_modal_key` does not check modifiers at all (so `Super+Y` or `Ctrl+Y` confirms quit/close).

5. **`crates/scriptui-core/src/buffer.rs:117-134` — `test defect`**
   - **Trigger:** `DocumentBuffer::from_file` enforces `MAX_FILE_BYTES` (64 MiB), but no unit or integration test covers the rejection path.
   - **Effect:** Removing or weakening the `meta.len() > MAX_FILE_BYTES` guard would pass `cargo test --workspace`.

### What was checked
- Full diff of all 9 changed files (`buffer.rs`, `editor.rs`, `preview.rs`, `selection.rs`, `spell.rs`, `theme.rs`, `app.rs`, `find_bar.rs`, `terminal.rs`) against `aedcc05`.
- Sibling callers of `SelectionSet::from_ranges`, `KeyCode::Char` modifier guards, `wrapped_layout` consumers, `chars().count()` in `preview.rs`, and filesystem symlink/rollback paths in `spell.rs` and `theme.rs`.
- Sensitive data scan across all diffs (none found).
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (all green).

### What was not checked
- Live interactive terminal testing (real TTY signal delivery for `EINTR`, real terminal emulator OSC 52 behavior, real macOS/Linux `SUPER`/`HYPER`/`META` keyboard protocol emission).
- Windows-specific symlink and rename-over-existing behavior (`resolve_theme_rejects_symlink_escape` and `load_locale_rejects_symlink_escape` are `#[cfg(unix)]`).

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step compliance
- **STEP 13A (`TASK-0071`)**: All `Contract & Behavior`, `Must still work`, and `Tests` met. Scope respected.
- **STEP 13B (`TASK-0072`)**: All `Contract & Behavior`, `Must still work`, and `Tests` met as written. However, the contract's strict 6-shortcut passthrough allowlist (`Ctrl+F`, `Ctrl+H`, `F3`, `Shift+F3`, `Ctrl+Q`, `Ctrl+S`) caused Finding 1 (regression on global shortcuts while Find/Replace is open, including keyboard buffer switching).
- **STEP 13C (`TASK-0073`)**: All `Contract & Behavior`, `Must still work`, and `Tests` met. Scope respected. Contract omitted a test for `MAX_FILE_BYTES` (Finding 5).
- **STEP 13D (`TASK-0074`)**: All `Contract & Behavior`, `Must still work`, and `Tests` met. Scope respected. `handle_insert_newline` only handles homogeneous `ClearCurrentLine` vs `Insert` across cursors (Finding 3).
- **STEP 13E (`TASK-0075`)**: All `Contract & Behavior`, `Must still work`, and `Tests` met. Scope respected.

### Deferrals & amendments
- None in Phase 13 (`.phased/deferred/11A.md` is from Phase 11; no `.phased/amendments/` entries).

### Origin classification & filed items
| # | Finding | Class | Origin | Filed |
|---|---------|-------|--------|-------|
| 1 | `find_bar.rs` swallows global shortcuts outside 6-key allowlist | regression | `plan` | `BUG-0037` (medium, pending) |
| 2 | `buffer.rs:345,405` reset `SelectionSet` `primary_index` to `0` | old bug | `pre-existing` | `BUG-0038` (low, pending) |
| 3 | `editor.rs:844-905` mixed-cursor `InsertNewline` ignores secondary lines | new defect | `coder` | `BUG-0039` (low, pending) |
| 4 | `modals.rs` & `palette.rs` missing `SUPER\|HYPER\|META` guards | old bug | `pre-existing` | `BUG-0040` (low, pending) |
| 5 | `buffer.rs:117-134` `MAX_FILE_BYTES` rejection untested | test defect | `plan` | `TASK-0076` (low, pending) |

### Advances items
- Ready to close cleanly: `BUG-0027`, `BUG-0032`, `BUG-0034`, `BUG-0035`, `BUG-0036`.
- Ready to close with follow-ups filed: `BUG-0026` (regression tracked in `BUG-0037`), `BUG-0028` (modal/palette siblings tracked in `BUG-0040`), `BUG-0029` (mixed-cursor newline & `buffer.rs` `primary_index` tracked in `BUG-0038`, `BUG-0039`).

## Handoff
- **Verdict:** PASS WITH FINDINGS — all 5 steps (`13A`–`13E`) passed gates with zero deferrals/amendments; 1 medium plan-origin regression (`BUG-0037`) and 4 low follow-ups filed.
- **Medium priority:**
  - `BUG-0037`: `find_bar.rs` swallows global shortcuts (`Ctrl+PageDown/Up`, `Ctrl+W/N/O`, `Ctrl+Shift+S/P`, `F1`) while Find/Replace is open and makes keyboard buffer-switch reset unreachable.
- **Low priority:**
  - `BUG-0038`: `buffer.rs:345,405` `apply_insert`/`apply_delete` drop `SelectionSet::primary_index()`.
  - `BUG-0039`: `editor.rs:844-905` `handle_insert_newline` ignores secondary cursors on mixed `ClearCurrentLine`/`Insert` lines.
  - `BUG-0040`: `modals.rs` and `palette.rs` do not filter `SUPER|HYPER|META` (and `handle_confirm_modal_key` filters no modifiers).
  - `TASK-0076`: Add unit test for `DocumentBuffer::from_file` `MAX_FILE_BYTES` rejection.
- **Deferred steps to re-split:** none.
- **User decisions needed:**
  - Approve closing `BUG-0026`, `BUG-0027`, `BUG-0028`, `BUG-0029`, `BUG-0032`, `BUG-0034`, `BUG-0035`, `BUG-0036` (or hold `BUG-0026` until `BUG-0037` is fixed).
  - Schedule `BUG-0037` in the next phase before release.
