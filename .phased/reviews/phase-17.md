# Phase 17 review

Range: `8cc9b14fa3f1f4f45af2731be2b6d9254c2bfd87..b5ac8578801484bee025196526725c84452c7f1f`

## Independent pass

### Findings

1. **Regression — Closed fenced code blocks inside blockquotes or indented list items include the closing fence in `CodeBlockRegion`** (`crates/scriptui-core/src/markdown.rs:376-393`, `crates/scriptui-core/src/markdown.rs:525-539`)
   - **Trigger:** Parse a closed fenced code block nested inside a blockquote (`> ```rust\n> let x = 1;\n> ``` `) or indented ≥4 spaces inside a list item (`- item\n    ```rust\n    let x = 1;\n    ``` `).
   - **Effect:** `is_closing_fence` inspects the raw document line and returns `false` whenever `leading_spaces > 3` or the line has a blockquote prefix (`>`). `has_closing_fence` evaluates to `false`, setting `CodeBlockRegion.end_line = span.end_line` instead of `span.end_line - 1`. This erroneously treats the closing fence line as code-block body (for syntax highlighting and `is_line_in_code_block`). At `<base>`, `span.end_line - 1` excluded the closing fence for all multi-line fenced code blocks.
   - **Class:** `regression`

2. **New defect — `switch_spell_locale` after `spell_enabled = false` startup clobbers existing `personal.dic`** (`crates/scriptui-core/src/editor.rs:288-298`, `crates/scriptui-core/src/editor.rs:486-499`, `crates/scriptui-core/src/editor.rs:501-515`)
   - **Trigger:** Initialize `EditorState::new_with_dirs` with `config.spell_enabled = false` when a non-empty `personal.dic` already exists in `dict_dirs[0]`, then invoke `switch_spell_locale(...)`, then add a word via `add_spell_popup_word_to_personal_dict()`.
   - **Effect:** At startup `spell_engine` is `None`, so `personal.dic` is never loaded while `personal_dict_path` is still `Some(...)`. `switch_spell_locale` only copies user words from `self.spell_engine` (which is `None`) and does not load `personal.dic` from disk. Adding a word then calls `engine.personal().save_to_file(path)`, overwriting `personal.dic` on disk with only the single newly added word and silently destroying all previously saved entries.
   - **Class:** `new defect`

3. **Test defect — `spell_handles_ascii_digit_tokens_without_flagging_pure_numbers` vacuously passes if `diags` is empty** (`crates/scriptui-core/src/spell.rs:763-770`)
   - **Trigger:** `let diags = engine.check_document("1st 2nd 2026 utf8", &[]); for diag in &diags { ... assert_eq!(diag.word, "utf8"); }`
   - **Effect:** The test never asserts `diags.len() == 1` (or `!diags.is_empty()`). If `check_document` returned an empty vector (e.g., if alphanumeric tokens were erroneously skipped or spellchecking broke), the loop body would never execute and the test would still pass.
   - **Class:** `test defect`

4. **Old bug (sibling) — Control characters remain unsanitized in `find_bar.rs` and `palette.rs`** (`crates/scriptui-tui/src/find_bar.rs:77,98`, `crates/scriptui-tui/src/palette.rs:256,270`)
   - **Trigger:** Render find/replace inputs (`find_replace.query.pattern`, `find_replace.replacement`), command palette query (`palette.query`), or dynamic palette entry labels (`entry.label`, e.g., theme/locale names from disk) containing C0/C1 control characters (`\x1b`, `\x07`, `\u{9b}`).
   - **Effect:** Unlike `chrome.rs`, `modals.rs`, and `preview_pane.rs`, these render paths pass raw strings directly to `Span::styled` without calling `sanitize_control_chars`, allowing raw terminal control sequences into rendered cells.
   - **Class:** `old bug`

5. **New defect — `collect_reference_link_def_skip_ranges` skips paragraph-continuation lines, and `autolink_cursor` is not advanced past `NodeValue::Image`** (`crates/scriptui-core/src/markdown.rs:307-349`, `crates/scriptui-core/src/markdown.rs:541-561`)
   - **Trigger:** (a) A paragraph continuation line whose text starts with `[label]: dest` (e.g., `"See below\n[note]: wrld"`), which is ordinary paragraph text in CommonMark because link reference definitions cannot interrupt a paragraph. (b) An image followed by an autolink with the same URL in the same paragraph (`![alt](https://example.com/a) https://example.com/a`).
   - **Effect:** (a) Ordinary misspelled prose on that line is silently skipped by spellchecking. (b) Because `NodeValue::Image` is not matched in `CollectCtx::collect`, `autolink_cursor` stays before the image, causing the subsequent autolink search to match the URL inside the image syntax instead of the autolink's actual byte range.
   - **Class:** `new defect`

### What was traced and verified clean
- `crates/scriptui-core/src/buffer.rs`: `apply_delete` backward/forward CRLF atomic deletion, overlapping multi-cursor span merging, caret shift calculation, reverse edit ordering, and unit tests.
- `crates/scriptui-core/src/markdown.rs`: CRLF front-matter byte range & `line_offset` alignment, duplicate autolinks within a block, UTF-8 slice boundaries in `is_reference_link_definition`.
- `crates/scriptui-tui/src/chrome.rs`, `modals.rs`, `preview_pane.rs`: `sanitize_control_chars` behavior on C0/C1/tab characters, narrow-width rendering, and confirm modal static strings.
- `crates/scriptui-tui/src/viewport.rs`: combining-mark attachment (including at right edge and after straddling wide chars), wide-char right-edge placeholder clamping, multi-cell wide-char cursor styling, and control-char skipping (`w == 0 && ch.is_control()`).
- Workspace static gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all exited `0`.
- Sensitive data scan across all changed hunks: no real IPs, hostnames, emails, tokens, client names, or internal paths found.

### What was not checked
- Live interactive terminal rendering (real terminal emulator / PTY behavior for combining marks, wide-char cursor cells, and OSC52 / control-char handling) — verified only via unit/integration tests and ratatui `TestBackend`.
- End-to-end persistence of `personal.dic` through the full `TuiApp::from_cli` binary path on an actual filesystem outside unit tests.

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step compliance
- **STEP 17A** (`TASK-0085`, `buffer.rs`): All `Contract & Behavior`, `Must still work`, `Tests`, and `[POST_GATE]` requirements met. No unrequested changes.
- **STEP 17B** (`TASK-0086`, `viewport.rs`): All `Contract & Behavior`, `Must still work`, `Tests`, and `[POST_GATE]` requirements met. No unrequested changes.
- **STEP 17C** (`TASK-0087`, `chrome.rs`, `preview_pane.rs`, `modals.rs`): All `Contract & Behavior`, `Must still work`, `Tests`, and `[POST_GATE]` requirements met. Sibling modules (`find_bar.rs`, `palette.rs`) were outside `[SCOPE_LOCK]`.
- **STEP 17D** (`TASK-0088`, `markdown.rs`): All 4 `Contract & Behavior` items implemented as specified, but item 2's literal `<=3 leading spaces` closing-fence rule violates `Must still work: Closed fenced code blocks` for closed fences inside blockquotes or ≥4-space indented list items (`BUG-0044`).
- **STEP 17E** (`TASK-0089`, `spell.rs`, `editor.rs`): All `Contract & Behavior` items implemented as specified, but the spec omitted loading `personal.dic` on `switch_spell_locale` after `spell_enabled = false` startup (`BUG-0045`), and the new test in `spell.rs` uses a vacuous loop (`TASK-0090`).

### Deferrals & amendments
- None in Phase 17 (all 5 steps `17A`–`17E` completed and closed).

### Finding origins & tracker IDs
| # | Finding | Class | Origin | Neb ID |
|---|---------|-------|--------|--------|
| 1 | Closed fenced code blocks in blockquotes/indented lists include closing fence | regression | `plan` | `BUG-0044` |
| 2 | `switch_spell_locale` after `spell_enabled=false` clobbers `personal.dic` | new defect | `plan` | `BUG-0045` |
| 3 | Vacuous loop in `spell_handles_ascii_digit_tokens_without_flagging_pure_numbers` | test defect | `coder` | `TASK-0090` |
| 4 | Control chars unsanitized in `find_bar.rs` and `palette.rs` | old bug | `pre-existing` | `BUG-0046` |
| 5 | `collect_reference_link_def_skip_ranges` paragraph continuation & `NodeValue::Image` autolink cursor | new defect | `plan` | `BUG-0047` |

### `Advances:` items ready to close
- `BUG-0030` (STEP 17A–17C): All listed defects fixed; sibling `find_bar.rs`/`palette.rs` tracked separately as `BUG-0046`.
- `BUG-0031` (STEP 17D–17E): All listed gaps resolved; follow-up regression/defects tracked as `BUG-0044`, `BUG-0045`, `BUG-0047`, `TASK-0090`.

## Handoff

**Verdict:** PASS WITH FOLLOW-UPS (all 5 steps complete, `fmt`/`clippy`/`test` green; 1 regression + 2 medium defects + 2 low items filed as `pending`).

**New pending items by priority:**
- **Medium:**
  - `BUG-0044` — Regression: `is_closing_fence` (`markdown.rs`) rejects blockquote-prefixed and ≥4-space indented closing fences, including them in `CodeBlockRegion`.
  - `BUG-0045` — `switch_spell_locale` (`editor.rs`) after `spell_enabled=false` startup leaves `PersonalDictionary` empty and clobbers `personal.dic` on add.
  - `BUG-0046` — Sibling: `find_bar.rs` and `palette.rs` render unsanitized C0/C1 control chars.
- **Low:**
  - `BUG-0047` — `markdown.rs` ref-def heuristic skips paragraph continuations; `autolink_cursor` misses `NodeValue::Image`.
  - `TASK-0090` — `spell_handles_ascii_digit_tokens_without_flagging_pure_numbers` needs a non-empty / `len() == 1` assertion.

**Deferred steps to re-split:** None.

**Decisions for user:**
- Approve closing `BUG-0030` and `BUG-0031` (follow-ups tracked under `BUG-0044`–`BUG-0047`, `TASK-0090`).
