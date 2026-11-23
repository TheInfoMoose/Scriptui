# Phase 9 review

Range: `487a69a0..8462beb1` (`487a69a0f57842866b7885424610d7b99b7baad0..8462beb1417cfce4a2051ce054d3192a969d4d8a`)

## Independent pass

### Findings

1. **`crates/scriptui-core/src/preview.rs:859`** — Unconditional `SoftBreak` → `HardBreak` breaks CommonMark/GFM paragraph reflow (`regression`)
   - **Trigger:** Rendering a hard-wrapped Markdown paragraph (e.g., 72–80-column prose) in `build_preview` at any width narrower or wider than the source wrap width (such as a 40–60-column split preview pane or a 100+-column full pane).
   - **Effect:** `NodeValue::SoftBreak` now emits `InlineToken::HardBreak` instead of `" "`, forcing a line break at every single source newline. In narrow split panes (below the source wrap width), wrapped lines break early and leave short orphan lines on every source line; in wide panes, paragraphs fail to reflow. At `487a69a0`, `SoftBreak` emitted `" "` and reflowed paragraphs normally.

2. **`crates/scriptui-core/src/preview.rs:371-389`** (interacts with `:859`, `:993`) — Multi-line Setext heading emits non-monotonic `source_line` on its underline rule (`old bug`)
   - **Trigger:** Rendering a multi-line Setext heading (`Line one\nLine two\n========`) in `build_preview`.
   - **Effect:** `render_wrapped_tokens` emits the heading lines with `source_line` and `source_line + 1` (advancing `self.last_source_line`), then line 377 emits the H1/H2 underline rule with the original `source_line`, producing non-monotonic `PreviewLine.source_line` values (`0, 1, 0`) that violate the monotonicity invariant assumed by `source_line_to_preview_line` / `preview_line_to_source_line`. Previously reachable only with an explicit `LineBreak` inside a Setext heading; now triggered by any soft-broken multi-line Setext heading.

3. **`crates/scriptui-core/src/spell.rs:205-235`** — Curly-apostrophe asymmetry between `ignore_word` / `PersonalDictionary::add` and `is_correct` (`new defect`)
   - **Trigger:** Ignoring or adding a word containing a curly apostrophe (`\u{2019}`, e.g. `foo\u{2019}s`) via `ignore_word` or `PersonalDictionary::add`, then spellchecking the straight-apostrophe form (`foo's`).
   - **Effect:** `ignore_word` and `PersonalDictionary::add` store the raw unnormalized string, while `is_correct` only normalizes `\u{2019}` → `'` on lookup. Adding `foo's` accepts `foo\u{2019}s`, but ignoring/adding `foo\u{2019}s` (e.g., directly from a diagnostic token) still flags `foo's`.

4. **`crates/scriptui-core/src/spell.rs:237-243,500-517`** — `suggest()` curly-apostrophe normalization path is untested (`test defect`)
   - **Trigger:** Calling `SpellEngine::suggest("don\u{2019}tt", 5)` (or any misspelled word with `\u{2019}`).
   - **Effect:** The new normalization in `suggest()` (`spell.rs:238-240`) has no unit test; reverting line 238–240 would still pass `cargo test --workspace`.

### What was traced
- Full diffs of `crates/scriptui-core/src/preview.rs`, `crates/scriptui-core/src/spell.rs`, and `crates/scriptui-core/assets/dicts/en_US.aff`.
- Structure, encoding, line count (`49059` lines matching header `49058`), contractions, and `<base>` root coverage of `crates/scriptui-core/assets/dicts/en_US.dic` (confirmed common stopwords `is`, `has`, `after`, `only`, `into`, `unit` are covered via affix flags `i/US`, `ha/SH`, `aft/RZ`, `on/Y`, `to/IU`, `it/USM`; only non-dictionary code jargon from the old 711-word stub like `keymap`/`viewport`/`struct`/`config` is absent).
- All callers of `build_preview`, `SpellEngine`, and `InlineToken::HardBreak` across the workspace; ran `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (all passed).
- Checked changed files for sensitive data (none found).

### What was not checked
- Interactive TUI rendering/scroll behavior in a live terminal (only unit/integration tests and static code tracing were run).
- Upstream licensing/attribution metadata for the bundled `en_US.{aff,dic}` files (no license file checked in this pass).
- Full runtime spellcheck sweep of all 49,058 `en_US.dic` entries or memory/binary-size budgets.

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step compliance
- **STEP 9A (`TASK-0051`, advances `BUG-0012`):** Met all `Contract & Behavior` and `Must still work` requirements. Scope lock honored (`crates/scriptui-core/src/preview.rs` only). Required test `preview_soft_break_in_blockquote_and_bullet_renders_on_new_lines` added and passing. No deferrals or amendments.
- **STEP 9B (`TASK-0052`, advances `BUG-0013`):** Met all `Contract & Behavior` and `Must still work` requirements. Scope lock honored (`crates/scriptui-core/assets/dicts/en_US.{aff,dic}`, `crates/scriptui-core/src/spell.rs` only). Required test `default_en_us_accepts_contractions_and_standard_english_words` added and passing. No deferrals or amendments.

### Independent finding origins & dispositions
| # | Finding | Class | Origin | Neb ID | Notes |
| :---: | :--- | :--- | :--- | :--- | :--- |
| 1 | `preview.rs:859` — Unconditional `SoftBreak` → `HardBreak` breaks paragraph reflow | `regression` | `plan` | `BUG-0014` | Explicitly requested by STEP 9A contract to fix `BUG-0012`; retained as a genuine CommonMark reflow trade-off, not withdrawn. |
| 2 | `preview.rs:371-389` — Multi-line Setext heading non-monotonic `source_line` on underline rule | `old bug` | `pre-existing` | `BUG-0015` | Pre-existing on `LineBreak`, amplified by STEP 9A's `SoftBreak` → `HardBreak`. |
| 3 | `spell.rs:205-235` — `ignore_word` / `PersonalDictionary::add` curly-apostrophe asymmetry | `new defect` | `coder` | `BUG-0016` | Lookup normalizes `\u{2019}` → `'`, but mutators store raw unnormalized strings. |
| 4 | `spell.rs:237-243,500-517` — `suggest()` curly-apostrophe normalization untested | `test defect` | `plan` / `coder` | `TASK-0053` | STEP 9B Tests section omitted `suggest()` with `\u{2019}` and coder did not add one. |

### Deferrals, amendments & unrequested changes
- **Deferrals:** None (`.phased/deferred/` empty).
- **Amendments:** None (`.phased/amendments/` absent).
- **Unrequested changes:** None.

### `Advances:` items ready to close
- `BUG-0012` (`open`, `medium`) — resolved by `STEP 9A` (`cf4c593`).
- `BUG-0013` (`open`, `high`) — resolved by `STEP 9B` (`1fd9ba3`).

## Handoff

- **Verdict:** PASS (with low-severity follow-ups). Both Phase 9 steps (`STEP 9A` / `TASK-0051` and `STEP 9B` / `TASK-0052`) met their contracts with zero deferrals or amendments.
- **Ready to close (pending user `approved`):** `BUG-0012`, `BUG-0013`.
- **New pending items (low priority — candidate for Phase 10/12 or backlog):**
  - `BUG-0014` (`low`, plan): Unconditional `SoftBreak` → `HardBreak` breaks CommonMark paragraph reflow on hard-wrapped Markdown.
  - `BUG-0015` (`low`, pre-existing): Multi-line Setext heading emits non-monotonic `source_line` on underline rule.
  - `BUG-0016` (`low`, coder): `ignore_word` / `PersonalDictionary::add` do not normalize curly apostrophes (`\u{2019}`).
  - `TASK-0053` (`low`, plan/coder): Missing unit test for `SpellEngine::suggest` with curly apostrophe.
- **Deferred steps to re-split:** None.
- **User decisions needed:**
  1. Confirm closing `BUG-0012` and `BUG-0013`.
  2. Decide whether `BUG-0014` (unconditional `SoftBreak` → `HardBreak`) should stay as-is or gain a CommonMark reflow option later.
