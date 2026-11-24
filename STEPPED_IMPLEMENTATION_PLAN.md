# ScripTUI — Stepped Implementation Plan

## Step Execution Roadmap

**Workflow:** phased

**Base gate:** `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

**Forbidden paths:** `STEPPED_IMPLEMENTATION_PLAN.md:docs/*:.phased/*:.orrery/*:scripts/*`

**Planning files:** `docs/*`

**How to read this:** work goes strictly top to bottom through **Up Next**, one step at a time. The **#** column is the only order. Step IDs are kept because commits and notes cite them. A step whose "Waits on" names an Open Question or a **[USER]** action doesn't start until that's resolved.

**Who runs what (phased workflow + Orrery `gemini-phased`):**
- **Phase planner (fresh Claude session, `/phase-plan`)** writes the active phase as **contract steps**: behavior, public interfaces, exact user-visible strings and test outcomes. Neon chooses the implementation; the planner never pre-writes code. Outlines future phases and stops for user approval before staging.
- **Phase issue fixer (fresh Sonnet session, `/phase-issue`)** handles a phase that stopped early: diagnoses the deferral, amends gates or re-splits steps.
- **Phase reviewer (fresh Opus session, `/phase-review`)** reviews the finished phase in its own session, from snapshot-backed step commits (`git show <sha>`). The planner never reviews its own phase.
- **Orchestrator / Validator (Hydrogen / Orrery `agy-hydrogen-orch` + `agy-hydrogen-val`)** runs the phase. At the start of each step, `phased:step_start` (`scripts/phased-step.sh pregate` + `neb start` + `scripts/phased-step.sh start`) takes a repository snapshot. After Neon finishes, `phased:step_check` (`scripts/phased-step.sh check`) validates scope and `POST_GATE` strictly against that snapshot diff, the validator reads the diff, and `phased:step_commit` commits the step and closes its Nebulous task.
- **Coder (Neon / Orrery `agy-neon-impl`)** executes one step inside `ALLOWED_FILES`. It never commits, never edits this plan or forbidden paths, and files bugs or scope lapses as **pending** Nebulous items.
- **Two code-fault failures** on one step, or an unamendable scope lapse → the step is deferred (`phased:step_defer`: saves attempt patch, restores step files, parks the task). A step whose `PRE_GATE` fails is deferred without dispatching an agent. Deferrals exceeding `max(2, ⌈steps/3⌉)` stop the phase early.

**Phase goal (Phase 15):** Fix Phase 14 review defects (`BUG-0041`, `BUG-0042`) and add the tag-triggered + `workflow_dispatch` dry-run GitHub Actions release workflow (`.github/workflows/release.yml`, resolving OQ-2).

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 15A** | `/goal` | tui | `crates/scriptui-tui/src/find_bar.rs` | — |
| 2 | **STEP 15B** | `/goal` | core | `crates/scriptui-core/src/editor.rs` | — |
| 3 | **STEP 15C** | `/goal` | ci | `.github/workflows/release.yml` | — |

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
| **STEP 7A** | Done (`bac6c81`) | `/goal` | core | `crates/scriptui-core/src/buffer.rs`, `crates/scriptui-core/src/editor.rs`, `crates/scriptui-tui/src/modals.rs` |
| **STEP 7B** | Done (`2860be3`) | `/goal` | core | `crates/scriptui-core/src/layout.rs`, `crates/scriptui-core/src/lib.rs` |
| **STEP 7C** | Done (`d6c1076`) | `/goal` | tui | `crates/scriptui-tui/src/viewport.rs`, `crates/scriptui-tui/src/mouse.rs` |
| **STEP 8D** | Done (`706671e`) | `/goal` | release | `LICENSE`, `Cargo.toml`, `crates/scriptui-core/Cargo.toml`, `crates/scriptui-tui/Cargo.toml`, `crates/scriptui-tui/tests/tui_integration.rs` |
| **STEP 8E** | Done (`4b7b777`) | `/goal` | docs | `CHANGELOG.md`, `README.md` |
| **STEP 9A** | Done (`cf4c593`) | `/goal` | preview | `crates/scriptui-core/src/preview.rs` |
| **STEP 9B** | Done (`1fd9ba3`) | `/goal` | spell | `crates/scriptui-core/assets/dicts/en_US.aff`, `crates/scriptui-core/assets/dicts/en_US.dic`, `crates/scriptui-core/src/spell.rs` |
| **STEP 10A** | Done (`511824c`) | `/goal` | core | `crates/scriptui-core/src/spell.rs`, `crates/scriptui-core/tests/editor_state_pipeline.rs` |
| **STEP 10B** | Done (`ec1a9ee`) | `/goal` | tui | `crates/scriptui-tui/tests/tui_integration.rs` |
| **STEP 10C** | Done (`26bed7f`) | `/goal` | cli | `crates/scriptui-tui/tests/tui_integration.rs` |
| **[USER] 11B** | Done (`25892eb`) | `/teamwork` | audit | `docs/codebase-audit-report.md` |
| **STEP 12A** | Done (`daa9fac`) | `/goal` | tooling | `deny.toml`, `THIRD_PARTY_LICENSES.md`, `Cargo.toml`, `crates/scriptui-core/Cargo.toml`, `crates/scriptui-tui/Cargo.toml` |
| **STEP 12B** | Done (`eff070c`) | `/goal` | core | `crates/scriptui-core/src/indent.rs`, `crates/scriptui-core/src/buffer.rs`, `crates/scriptui-core/src/spell.rs` |
| **STEP 12C** | Done (`b0addbd`) | `/goal` | core | `crates/scriptui-core/src/history.rs`, `crates/scriptui-core/src/editor.rs`, `crates/scriptui-tui/src/modals.rs` |
| **STEP 12D** | Done (`a01c7eb`) | `/goal` | tui | `crates/scriptui-core/src/buffers.rs`, `crates/scriptui-tui/src/app.rs`, `crates/scriptui-tui/src/palette.rs` |
| **STEP 12E** | Done (`f5febeb`) | `/goal` | perf | `crates/scriptui-core/src/highlight.rs`, `crates/scriptui-tui/src/preview_pane.rs` |
| **STEP 12F** | Done (`213e951`) | `/goal` | tui | `crates/scriptui-tui/Cargo.toml`, `Cargo.lock`, `crates/scriptui-tui/src/terminal.rs`, `crates/scriptui-tui/src/app.rs`, `crates/scriptui-tui/src/main.rs` |
| **STEP 13A** | Done (`756f64c`) | `/goal` | tui | `crates/scriptui-tui/src/terminal.rs`, `crates/scriptui-tui/src/app.rs` |
| **STEP 13B** | Done (`16bfde8`) | `/goal` | tui | `crates/scriptui-tui/src/find_bar.rs`, `crates/scriptui-tui/src/app.rs` |
| **STEP 13C** | Done (`82f06dc`) | `/goal` | core | `crates/scriptui-core/src/theme.rs`, `crates/scriptui-core/src/spell.rs`, `crates/scriptui-core/src/buffer.rs` |
| **STEP 13D** | Done (`9bc15ff`) | `/goal` | core | `crates/scriptui-core/src/selection.rs`, `crates/scriptui-core/src/editor.rs` |
| **STEP 13E** | Done (`489e5f2`) | `/goal` | preview | `crates/scriptui-core/src/preview.rs`, `crates/scriptui-tui/src/app.rs` |
| **STEP 14A** | Done (`be6c288`) | `/goal` | tui | `crates/scriptui-tui/src/find_bar.rs`, `crates/scriptui-tui/src/modals.rs`, `crates/scriptui-tui/src/palette.rs` |
| **STEP 14B** | Done (`aed0790`) | `/goal` | core | `crates/scriptui-core/src/buffer.rs` |
| **STEP 14C** | Done (`206333d`) | `/goal` | core | `crates/scriptui-core/src/editor.rs` |
| **STEP 14D** | Done (`5167e8d`) | `/goal` | ci | `.github/workflows/ci.yml` |

### Future phases

**Decisions (2026-11-22):**
- Phase 8 cascaded (8A POST_GATE fault → 8B/8C PRE_GATEs failed); 8C moves out so Phase 8 is local-only release prep (8A + 8B).
- Release targets stay `x86_64-unknown-linux-gnu` + `aarch64-apple-darwin`; no Windows or macOS x86_64 for v0.1.0.
- Known bugs (`BUG-0012`, `BUG-0013`, both `status = "open"` in neb), functional verification, and the full multiagent audit + fixes run **before** CI & release workflows and the `v0.1.0` tag.
- Because the tag now lands several phases after STEP 8B, the CHANGELOG release date is re-stamped in the release-cut phase.

**Phase 9 — Known bug fixes: BUG-0012 & BUG-0013 (routine).**
- Note for `/phase-plan 9`: `BUG-0012` and `BUG-0013` both have `status = "open"` in neb, whereas `neb list --status pending` queries `pending` by default — run `neb show BUG-0012` and `neb show BUG-0013` explicitly when staging Phase 9.
- Goal: fix preview markdown line breaks (`BUG-0012`, block quotes/bullets pressing Enter renders on the previous line in preview) and spellcheck dictionary coverage (`BUG-0013`, `en_US` dictionary rejects contractions like `"don't"` and standard words like `"emphasized"` and `"cascade"`).
- Done: both `BUG-0012` and `BUG-0013` are closed with regression tests that fail before the fix and pass after.
- Steps: 9A fix `BUG-0012` (markdown/preview line break handling + tests); 9B fix `BUG-0013` (validate/update `en_US` dictionary and/or tokenization + tests).

**Phase 10 — Functional verification (routine).**
- Goal: prove every user-facing behavior the README documents actually works, automated where possible and by hand where a real terminal is required.
- Done: a coverage matrix in `docs/` maps every README CLI flag, config field, keybinding, mouse action, theme and spellcheck flow to an automated test or a `[USER]` checklist row; automated gaps are closed; the checklist passes on Linux and macOS (OQ-3); every failure is a neb bug.
- Steps: 10A core-engine coverage gaps (`scriptui-core` tests); 10B TUI coverage gaps (`TestBackend` integration tests: keybindings, mouse, modals, preview, find/replace); 10C CLI/config/theme/dict-import coverage gaps; 10D `[USER]` real-terminal checklist run (raw mode restore, bracketed paste, OSC 52, mouse, resize).

**Phase 11 — Full multiagent app audit (read-only review + tooling).**
- Goal: a whole-codebase multiagent audit before CI and release: security/robustness (file I/O and paths, theme/dict import, regex input, panics/`unwrap`s, terminal restore on panic), correctness (Unicode, large files, undo/redo edge cases), performance, dependency licenses/advisories, and code quality.
- Done: an audit report in `docs/` with each finding rated; every finding filed as a pending neb item; `deny.toml` added and `cargo deny check` (advisories + licenses) runs clean locally or with documented exceptions.
- Steps: 11A `[REVIEW]` multiagent audit writes the audit report in `docs/` and files pending neb items; 11B add `deny.toml` and verify `cargo deny check` locally (CI integration follows in Phase 13).

**Phase 12 — Audit and verification fixes (risky: scope set by findings).**
- Goal: fix the neb bugs from Phases 10–11 (and any picked pending items from earlier phases) selected at the Phase 11 review.
- Done: every picked item closed with a test that fails before the fix; unpicked items stay pending with a target version.
- Steps: staged by `/phase-plan 12` from the picked neb items (one step per fix, 1–3 files each).

**Phase 13 — CI & release workflows (risky: only verifiable on GitHub).**
- Goal: GitHub Actions CI on `master` push/PR (fmt, clippy, test, `cargo deny check`, release build) and a tag-triggered release workflow producing Linux x86_64 + macOS arm64 tarballs with SHA256 files.
- Done: both workflow files exist and pass a local syntax gate; `[USER]` pushes `master` and CI goes green; the release workflow is exercised once per OQ-2 before the real tag.
- Steps: 13A CI workflow (`.github/workflows/ci.yml`); 13B tag-triggered release workflow (`.github/workflows/release.yml`); 13C `[USER]` push `master`, confirm CI green, run the release dry run.
- Carries over deferred STEP 8C / TASK-0044. `/phase-plan 13` replaces the old 8C block (its `python3 yaml` POST_GATE is OQ-1).

**Phase 14 — v0.1.0 release cut.**
- Goal: ship `v0.1.0`.
- Done: CHANGELOG date matches the tag date; `master` pushed; annotated tag `v0.1.0` pushed; GitHub release has both tarballs and `.sha256` files.
- Steps: 14A re-stamp `CHANGELOG.md` `[0.1.0]` date and add any Phase 9–12 fixes under it; 14B `[USER]` push `master`, `git tag -a v0.1.0 -m "ScripTUI v0.1.0"`, push the tag, check the release assets.

### Open Questions

- **OQ-1 (Phase 14 — resolved):** PyYAML (`python3 -c 'import yaml'`) is confirmed installed on the host and used for YAML syntax validation in STEP 14D's `POST_GATE`.
- **OQ-2 (Phase 15 — resolved):** `.github/workflows/release.yml` includes a `workflow_dispatch` trigger that runs the full build and packaging matrix and uploads workflow artifacts without creating a GitHub Release; `publish` runs only on `refs/tags/v*`.
- **OQ-3 (Phase 10 — resolved):** `[USER]` real-terminal checklist targets Linux desktop terminals (e.g., GNOME Terminal / Alacritty / Kitty / WezTerm); checklist lives in `docs/functional-verification-matrix.md`.

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.
Medium and low audit findings (`BUG-0014`–`BUG-0016`, `BUG-0026`–`BUG-0033`, `TASK-0049`, `TASK-0057`–`TASK-0059`, `TASK-0064`) remain pending in `neb` for post-Phase-12 triage.

---

## Phase 8 Step Blocks

### STEP 8A: Release Metadata, MIT `LICENSE` & Release Profile Tuning

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 8A: Add root MIT LICENSE, complete Cargo workspace package metadata and [profile.release] tuning (preserving unwinding panic for TerminalGuard), and verify release binary --version output.

[SCOPE_LOCK]
ALLOWED_FILES:
  - Cargo.toml
  - crates/scriptui-core/Cargo.toml
  - crates/scriptui-tui/Cargo.toml
  - LICENSE
READ_ONLY_REFS:
  - Cargo.toml:1-9 (workspace.package)
  - crates/scriptui-core/Cargo.toml:1-20
  - crates/scriptui-tui/Cargo.toml:1-19
  - crates/scriptui-tui/src/cli.rs:6-11 (clap version attribute)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - Do NOT set `panic = "abort"` in `[profile.release]` (TerminalGuard::drop relies on unwinding to restore raw terminal mode on panic).
  - Do NOT modify any Rust source files or Cargo.lock dependencies.

[PRE_GATE]
grep -qE '^version = "0\.1\.0"' Cargo.toml && grep -qE '^license = "MIT"' Cargo.toml

[MUTATION_SPEC]
1. LICENSE:
   - Create root `LICENSE` containing the standard MIT License text with `Copyright (c) 2026 ScripTUI Contributors`.
2. Cargo.toml:
   - Under `[workspace.package]`, add:
     - `repository = "https://github.com/TheInfoMoose/Scriptui"`
     - `readme = "README.md"`
     - `keywords = ["tui", "editor", "markdown", "ratatui", "terminal"]`
     - `categories = ["command-line-utilities", "text-editors"]`
   - Append `[profile.release]` with:
     - `lto = "thin"`
     - `codegen-units = 1`
     - `strip = "symbols"`
3. crates/scriptui-core/Cargo.toml and crates/scriptui-tui/Cargo.toml:
   - Under `[package]`, add:
     - `repository.workspace = true`
     - `readme.workspace = true`
     - `keywords.workspace = true`
     - `categories.workspace = true`

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --release && "${CARGO_TARGET_DIR:-target}/release/scriptui" --version | grep -qE '^scriptui 0\.1\.0$' && test -f LICENSE

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 8B: `CHANGELOG.md` (v0.1.0) & `README.md` Installation / License Sections

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 8B: Add Keep-a-Changelog CHANGELOG.md for v0.1.0 and update README.md with Cargo installation instructions, changelog link, and MIT license section.

[SCOPE_LOCK]
ALLOWED_FILES:
  - CHANGELOG.md
  - README.md
READ_ONLY_REFS:
  - README.md:14-43 (Quick Start / Build & Run)
  - README.md:185-193 (Development & Verification)
  - LICENSE:1-21
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - Do not remove or weaken any existing README.md tables or keybinding documentation.

[PRE_GATE]
test -f LICENSE && grep -qE '^## Development & Verification' README.md

[MUTATION_SPEC]
1. CHANGELOG.md:
   - Create `CHANGELOG.md` following Keep a Changelog 1.1.0 and Semantic Versioning 2.0.0.
   - Include `## [0.1.0] - 2026-11-22` with concise `### Added` bullets covering:
     - Headless `scriptui-core` engine: Ropey `DocumentBuffer` with monotonic `doc_id`, multi-buffer `BufferManager`, transactional undo/redo, multi-cursor `SelectionSet`, auto-indent/dedent, literal & regex find/replace, and Unicode display-width + tab-stop soft wrapping.
     - GitHub Flavored Markdown & spellcheck pipeline: `comrak` GFM parsing, `syntect` fenced code block highlighting, live styled Markdown preview, cached per-revision analysis, and embedded `en_US` Hunspell spellcheck with custom locale import (`scriptui dict import`).
     - Ratatui + Crossterm TUI (`scriptui-tui`): CLI (`clap`), `TerminalGuard` with bracketed paste and OSC 52 clipboard sync, 5 built-in retro-futuristic palettes + custom TOML themes, tab bar, line-number gutter, display-cell aligned viewport & mouse routing, split/full Markdown preview pane, inline find/replace bar, command palette, and spellcheck/file/unsaved-changes modals.
     - Packaging & release metadata: MIT `LICENSE`, workspace release profile, and multi-palette `TestBackend` integration test suite.
2. README.md:
   - Under `## Quick Start`, add a `### Install` subsection (before or after `### Build & Run`) showing `cargo install --path crates/scriptui-tui` and verifying with `scriptui --version`.
   - At the end of `README.md` (after `## Development & Verification`), add `## Changelog & License` linking to `[CHANGELOG.md](CHANGELOG.md)` and noting the MIT license in `[LICENSE](LICENSE)`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE '^## \[0\.1\.0\]' CHANGELOG.md && grep -qE 'cargo install --path crates/scriptui-tui' README.md && grep -qE '\[LICENSE\]\(LICENSE\)' README.md

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 8C: GitHub Actions CI & Tag-Triggered Release Workflows

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 8C: Add GitHub Actions CI workflow (fmt, clippy, test, release build) and tag-triggered release workflow (v* tags building Linux/macOS tarballs with SHA256 checksums).

[SCOPE_LOCK]
ALLOWED_FILES:
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
READ_ONLY_REFS:
  - Cargo.toml:1-25
  - crates/scriptui-tui/Cargo.toml:1-25
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - Do not hardcode any secrets, tokens, or personal paths (use `${{ secrets.GITHUB_TOKEN }}` / default `GITHUB_TOKEN` permissions only).

[PRE_GATE]
test -f LICENSE && test -f CHANGELOG.md

[MUTATION_SPEC]
1. .github/workflows/ci.yml:
   - Name: `CI`
   - Triggers: `push` on branches `["master"]`, `pull_request` on branches `["master"]`.
   - Permissions: `contents: read`.
   - Job `check` on `ubuntu-latest`:
     - `actions/checkout@v4`
     - `dtolnay/rust-toolchain@stable` with `components: rustfmt, clippy`
     - `Swatinem/rust-cache@v2`
     - Run `cargo fmt --all -- --check`
     - Run `cargo clippy --workspace --all-targets -- -D warnings`
     - Run `cargo test --workspace`
     - Run `cargo build --release`
2. .github/workflows/release.yml:
   - Name: `Release`
   - Triggers: `push` on tags `["v*"]`.
   - Permissions: `contents: write`.
   - Job `build` with `strategy.matrix.include`:
     - `{ os: ubuntu-latest, target: x86_64-unknown-linux-gnu }`
     - `{ os: macos-14, target: aarch64-apple-darwin }`
   - Steps in `build`:
     - `actions/checkout@v4`
     - `dtolnay/rust-toolchain@stable` with `targets: ${{ matrix.target }}`
     - `Swatinem/rust-cache@v2`
     - Build: `cargo build --release --locked --target ${{ matrix.target }} -p scriptui-tui`
     - Package tarball `scriptui-${{ github.ref_name }}-${{ matrix.target }}.tar.gz` containing `target/${{ matrix.target }}/release/scriptui`, `README.md`, `LICENSE`, and `CHANGELOG.md`, plus a `.sha256` checksum file (using `sha256sum` on Linux / `shasum -a 256` on macOS).
     - Upload artifacts with `actions/upload-artifact@v4`.
   - Job `publish` (`needs: build`, `runs-on: ubuntu-latest`):
     - `actions/download-artifact@v4` with `merge-multiple: true` into `dist/`.
     - `softprops/action-gh-release@v2` with `files: dist/*` and `generate_release_notes: true`.

[POST_GATE]
python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml")); yaml.safe_load(open(".github/workflows/release.yml"))' && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 8D: MIT License, Release Metadata and Release Profile

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 8D: Add an MIT LICENSE, inherited release metadata and a tuned [profile.release]; the release binary prints `scriptui 0.1.0` for --version (re-split of deferred STEP 8A).

[SCOPE_LOCK]
ALLOWED_FILES:
  - LICENSE (NEW)
  - Cargo.toml
  - crates/scriptui-core/Cargo.toml
  - crates/scriptui-tui/Cargo.toml
  - crates/scriptui-tui/tests/tui_integration.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/cli.rs:6-21 (Cli: `name = "scriptui"`, `version` already derived from CARGO_PKG_VERSION)
  - crates/scriptui-tui/tests/tui_integration.rs:1-21 (imports; `Cli` is already imported from scriptui_tui)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No dependency added, removed or version-changed (Cargo.lock must not change). No change to `[workspace.package] version`, `edition`, `license` or `description`. No `panic` key in any profile. No source file under `src/`.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: The repo carries a standard MIT license, both crates inherit complete release metadata from the workspace, and `cargo build --release` produces an optimized, stripped `scriptui` binary whose `--version` output is exactly `scriptui 0.1.0`.

Contract & Behavior:
  1. `LICENSE` (new, repo root): the standard OSI MIT License text, unmodified. First line exactly `MIT License`. Copyright line exactly `Copyright (c) 2026 InfoMoose`.
  2. `Cargo.toml` `[workspace.package]`: keep the four existing keys unchanged; add:
     - `readme = "README.md"`
     - `keywords = ["editor", "markdown", "tui", "terminal", "ratatui"]`
     - `categories = ["text-editors", "command-line-utilities"]`
  3. `Cargo.toml`: add a `[profile.release]` table with exactly these keys: `opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true`. Panic strategy stays the default (unwind).
  4. `crates/scriptui-core/Cargo.toml` and `crates/scriptui-tui/Cargo.toml` `[package]`: keep existing keys; add `readme.workspace = true`, `keywords.workspace = true`, `categories.workspace = true`.
  5. `crates/scriptui-tui/tests/tui_integration.rs`: add one integration test (below). Add any `use` it needs (e.g. `clap::Parser`, `clap::error::ErrorKind`); `clap` is already a normal dependency of scriptui-tui, so no Cargo change is needed for it.

Must still work:
  - `cargo build`, `cargo test --workspace` and every existing test in both crates, unchanged.
  - Panic unwinding (terminal restore on panic relies on the default `panic = "unwind"`).
  - `scriptui --help`, the `dict import` subcommand, `--theme` / `--locale` flags (cli.rs is not touched).
  - Cargo.lock unchanged.

Drops: none

Tests:
  - `cli_version_matches_workspace_version`: `Cli::try_parse_from(["scriptui", "--version"])` -> returns `Err(e)` with `e.kind() == ErrorKind::DisplayVersion`, and `e.to_string().trim()` equals `"scriptui 0.1.0"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qx 'MIT License' LICENSE && grep -qx 'Copyright (c) 2026 InfoMoose' LICENSE && grep -qx '\[profile.release\]' Cargo.toml && grep -qx 'lto = true' Cargo.toml && grep -qx 'strip = true' Cargo.toml && grep -qx 'keywords.workspace = true' crates/scriptui-core/Cargo.toml && grep -qx 'keywords.workspace = true' crates/scriptui-tui/Cargo.toml && grep -q 'fn cli_version_matches_workspace_version' crates/scriptui-tui/tests/tui_integration.rs && test "$(cargo run --release -q -p scriptui-tui -- --version)" = 'scriptui 0.1.0'

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 8E: CHANGELOG and README Installation, Releases and License

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 8E: Add CHANGELOG.md with the 0.1.0 entry and add Installation, Releases and License sections to README.md (re-split of deferred STEP 8B).

[SCOPE_LOCK]
ALLOWED_FILES:
  - CHANGELOG.md (NEW)
  - README.md
READ_ONLY_REFS:
  - README.md (all sections: the CHANGELOG lists only features documented here)
  - LICENSE (added by STEP 8D)
  - Cargo.toml ([workspace.package] version, [profile.release])
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No existing README section's heading or text is changed, reordered or removed. No URLs to this project's GitHub repository (none is configured yet).

[PRE_GATE]
test -f LICENSE && grep -qx '\[profile.release\]' Cargo.toml

[MUTATION_SPEC]
Goal: A reader can see what 0.1.0 contains, how to install `scriptui`, how releases are versioned, and that the project is MIT-licensed.

Contract & Behavior:
  1. `CHANGELOG.md` (new, repo root), Keep a Changelog 1.1.0 style:
     - Line 1 exactly `# Changelog`.
     - A short intro stating the format follows Keep a Changelog and the project uses Semantic Versioning.
     - `## [Unreleased]` heading with no entries.
     - `## [0.1.0] - 2026-11-22` heading (date is a placeholder; Phase 13 re-stamps it to the tag date), followed by `### Added` with one bullet per user-facing feature area the README documents: CUA editing with undo/redo and auto-indent, multiple buffers, find/replace (regex) and multi-cursor, GFM Markdown live preview, syntax highlighting, Hunspell spellcheck with bundled `en_US` and `scriptui dict import`, built-in and custom themes, mouse support, CLI options and the configuration file. No feature the README does not document.
     - No link reference definitions (no repository URL exists yet).
  2. `README.md`: insert three new top-level sections; existing content stays byte-identical:
     - `## Installation`, placed directly after the `## Architecture` section and before `## Quick Start`. States that a Rust stable toolchain (via rustup) is required; shows a `bash` fenced block containing exactly `cargo install --path crates/scriptui-tui --locked`; says this installs the `scriptui` binary into `~/.cargo/bin`; shows `scriptui --version` printing `scriptui 0.1.0`. States that prebuilt binaries for Linux x86_64 (`x86_64-unknown-linux-gnu`) and macOS Apple Silicon (`aarch64-apple-darwin`) are planned for GitHub releases; must not claim they are available now.
     - `## Releases`, placed after `## Development & Verification`. States versions follow Semantic Versioning, release builds use the tuned `[profile.release]` (LTO, single codegen unit, stripped), and changes are recorded in `CHANGELOG.md` (relative link `[CHANGELOG.md](CHANGELOG.md)`).
     - `## License`, the last section of the file. States the project is licensed under the MIT License, with relative link `[LICENSE](LICENSE)`.

Must still work:
  - Every existing README section (Architecture through Development & Verification), including the `cargo build --release` comment naming `target/release/scriptui` and the verification commands.
  - Base gate passes (docs-only change).

Drops: none

Tests:
  - None in Rust (docs-only step). POST_GATE greps verify every required heading and the install command.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qx '# Changelog' CHANGELOG.md && grep -qx '## \[Unreleased\]' CHANGELOG.md && grep -qx '## \[0.1.0\] - 2026-11-22' CHANGELOG.md && grep -qx '### Added' CHANGELOG.md && grep -qx '## Installation' README.md && grep -qx '## Releases' README.md && grep -qx '## License' README.md && grep -qx 'cargo install --path crates/scriptui-tui --locked' README.md && grep -q '\[LICENSE\](LICENSE)' README.md && grep -q '\[CHANGELOG.md\](CHANGELOG.md)' README.md && test "$(grep '^## ' README.md | tail -n 1)" = '## License' && test "$(grep -n '^## ' README.md | grep -A1 '## Architecture' | tail -n 1 | cut -d: -f2)" = '## Installation'

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

## Phase 9 Step Blocks

### STEP 9A: Preview Markdown Soft-Break Line Rendering (`BUG-0012`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 9A: Render markdown soft breaks as line breaks in the live preview so Enter in block quotes, bullets, and paragraphs starts a new preview line (BUG-0012).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/preview.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/preview.rs:841-875 (collect_inline_node, SoftBreak, LineBreak)
  - crates/scriptui-core/src/preview.rs:985-1012 (render_wrapped_tokens)
  - crates/scriptui-core/src/preview.rs:645-660 (render_table HardBreak flattening)
  - crates/scriptui-core/src/preview.rs:392-520 (render_blockquote, render_list)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT modify markdown.rs, indent.rs, or any TUI files.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Pressing Enter in the editor inside block quotes, bullet/ordered lists, or paragraphs renders subsequent lines on separate preview lines with active prefixes preserved instead of collapsing onto the previous line.

Contract & Behavior:
  1. `crates/scriptui-core/src/preview.rs` (`PreviewRenderer::collect_inline_node`):
     - Emit `InlineToken::HardBreak` for `NodeValue::SoftBreak` (matching `NodeValue::LineBreak`) instead of a `" "` `StyledSegment`.
     - Preserve `prefix_stack` across soft breaks (blockquote `"│ "` bars and list-item continuation indent) and keep `render_wrapped_tokens` source-line mapping (`base_source_line + break_idx`) monotonic.
     - Table cell rendering (`render_table`) continues flattening `InlineToken::HardBreak` to a single space `" "`.

Must still work:
  - `preview_renders_tables_tasks_alerts_code_and_breaks` (explicit two-space hard breaks, GFM alerts, task lists, tables, code blocks).
  - `preview_scroll_sync_maps_source_and_preview_lines` (monotonic `source_line_to_preview_line` and `preview_line_to_source_line` roundtrip).
  - `full_gfm_document_highlights_previews_and_spellchecks` in `crates/scriptui-core/tests/markdown_spell_pipeline.rs`.

Drops: none

Tests:
  - `preview_soft_break_in_blockquote_and_bullet_renders_on_new_lines`: call `build_preview("> first quote line\n> second quote line\n\n- bullet line one\n  bullet line two\n\nplain one\nplain two\n", &theme, 80)` -> `plain_lines()` places `"second quote line"` on a later line index than `"first quote line"` (both prefixed with `"│ "`), `"bullet line two"` on a later line index than `"bullet line one"` (with continuation indent rather than `"• "`), and `"plain two"` on a later line index than `"plain one"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn preview_soft_break_in_blockquote_and_bullet_renders_on_new_lines' crates/scriptui-core/src/preview.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 9B: Bundled `en_US` Hunspell Dictionary & Contraction Coverage (`BUG-0013`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 9B: Replace the stub en_US dictionary with the full standard en_US Hunspell dictionary and ensure contractions and standard English words pass spellcheck (BUG-0013).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/assets/dicts/en_US.aff
  - crates/scriptui-core/assets/dicts/en_US.dic
  - crates/scriptui-core/src/spell.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/spell.rs:11-12 (BUNDLED_EN_US_AFF, BUNDLED_EN_US_DIC)
  - crates/scriptui-core/src/spell.rs:211-232 (SpellEngine::is_correct, suggest)
  - crates/scriptui-core/src/spell.rs:234-340 (SpellEngine::check_document)
  - crates/scriptui-core/src/spell.rs:387-393 (is_apostrophe, is_word_char)
  - crates/scriptui-core/tests/markdown_spell_pipeline.rs:6-58
  - crates/scriptui-core/tests/editor_state_pipeline.rs:76-115
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT modify Cargo.toml, Cargo.lock, or any TUI files.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: `SpellEngine::default_en_us()` uses a full standard `en_US` Hunspell dictionary so common English words, inflections, and contractions (including `"don't"`, `"emphasized"`, and `"cascade"`) are accepted while genuine misspellings like `"wrld"` and `"documnt"` are still flagged with suggestions.

Contract & Behavior:
  1. `crates/scriptui-core/assets/dicts/en_US.aff` and `crates/scriptui-core/assets/dicts/en_US.dic`:
     - Replace the 712-line stub dictionary with the full standard `en_US` Hunspell dictionary (sourced from `/usr/share/hunspell/en_US.aff` and `/usr/share/hunspell/en_US.dic`, normalized to UTF-8 LF line endings) so `spellbook::Dictionary::new(BUNDLED_EN_US_AFF, BUNDLED_EN_US_DIC)` loads cleanly.
  2. `crates/scriptui-core/src/spell.rs`:
     - `SpellEngine::is_correct`, `SpellEngine::suggest`, and `SpellEngine::check_document` must accept contractions with ASCII `'` and curly `\u{2019}` apostrophes (e.g., `"don't"`, `"don\u{2019}t"`, `"Don't"`), as well as `"emphasized"` and `"cascade"`, while preserving original byte/column spans in `SpellDiagnostic`. If `spellbook` does not fold `\u{2019}` via `ICONV`, normalize `\u{2019}` to `'` when querying the dictionary.

Must still work:
  - `default_en_us_flags_misspelling_and_suggests` (`"hello wrld"` flags `"wrld"` and suggests `"world"`).
  - `spell_skips_code_ranges_and_respects_personal_and_ignore`.
  - `import_and_load_custom_locale_and_reject_traversal`.
  - `full_gfm_document_highlights_previews_and_spellchecks` (`"documnt"` flagged, `"document"` suggested).
  - `soft_wrap_layout_spell_quick_fix_and_preview_sync`.

Drops: none

Tests:
  - `default_en_us_accepts_contractions_and_standard_english_words`: `SpellEngine::default_en_us()` -> `check_document("I don't think you\u{2019}ve emphasized the cascade. Don't stop.", &[])` returns an empty `Vec`, and `is_correct("don't")`, `is_correct("don\u{2019}t")`, `is_correct("emphasized")`, `is_correct("cascade")` all return `true`, while `is_correct("wrld")` returns `false`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn default_en_us_accepts_contractions_and_standard_english_words' crates/scriptui-core/src/spell.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```


---

## Phase 10 Step Blocks

### STEP 10A: Core-Engine README Coverage Gaps & Curly-Apostrophe Suggest Test (`TASK-0053`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 10A: Add unit/integration tests closing scriptui-core README coverage gaps (full CUA keymap, config flags, navigation/selection/escape) and curly-apostrophe spell suggestions (TASK-0053).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/spell.rs
  - crates/scriptui-core/tests/editor_state_pipeline.rs
READ_ONLY_REFS:
  - README.md:60-72 (Configuration fields and defaults)
  - README.md:111-188 (Keybindings CUA Default)
  - crates/scriptui-core/src/keymap.rs:160-385 (Keymap::default_cua)
  - crates/scriptui-core/src/config.rs:24-84 (AppConfig)
  - crates/scriptui-core/src/editor.rs:245-275,665-760 (EditorState::new, dispatch)
  - crates/scriptui-core/src/spell.rs:237-244 (SpellEngine::suggest)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT modify production logic unless a test exposes a genuine bug (if so, file a pending neb bug rather than changing files outside ALLOWED_FILES). Do NOT weaken any existing test.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Every README-documented CUA keybinding, `AppConfig` option, core navigation/selection/escape action, and `SpellEngine::suggest` with curly apostrophes (`\u{2019}`) is verified by automated tests in `scriptui-core`.

Contract & Behavior:
  1. `crates/scriptui-core/src/spell.rs` (unit tests only):
     - Add `default_en_us_suggests_for_curly_apostrophe_misspelling` covering `SpellEngine::suggest` on a misspelled contraction containing `\u{2019}` (closes `TASK-0053`).
  2. `crates/scriptui-core/tests/editor_state_pipeline.rs` (integration tests only):
     - Add `readme_default_cua_keymap_covers_every_documented_shortcut` asserting `Keymap::default_cua().lookup(...)` maps every shortcut documented in `README.md` (`Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S`, `Ctrl+W`, `Ctrl+PageDown`, `Ctrl+PageUp`, `Ctrl+Q`, `Enter`, `Backspace`, `Delete`, `Tab`, `Shift+Tab`/`BackTab`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`, `Ctrl+A`, `Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, `Esc`, `Ctrl+F`, `Ctrl+H`, `F3`, `Shift+F3`, `Ctrl+Shift+L`, `Ctrl+P`, `F1`, `Ctrl+.`, `Ctrl+Shift+P`, `Alt+P`, `Alt+Z`, `Left`/`Right`/`Up`/`Down`, `Home`/`End`, `PageUp`/`PageDown`, `Shift+Left`/`Shift+Right`/`Shift+Up`/`Shift+Down`, `Shift+Home`/`Shift+End`) to its exact `Action`.
     - Add `config_flags_control_spellcheck_preview_split_and_soft_wrap` verifying `AppConfig::load_or_default` (missing path returns defaults; partial TOML merges defaults; invalid TOML returns `Err`), `spell_enabled = false` yields zero spell diagnostics in `EditorState::active_analysis()`, `preview_split_percent = 35` produces `PreviewMode::Split(35)` on `Action::TogglePreview`, and `Action::ToggleSoftWrap` / `Action::ToggleLineNumbers` flip `config.soft_wrap` / `config.line_numbers`.
     - Add `editor_navigation_selection_cut_find_prev_and_escape_collapse` exercising `SelectAll`, `Cut`, `MoveHome`/`MoveEnd`/`PageUp`/`PageDown`, `SelectLeft`/`SelectRight`/`SelectUp`/`SelectDown`/`SelectHome`/`SelectEnd`, `FindNext`/`FindPrev` wrap-around, and `Action::Escape` collapsing a multi-cursor selection back to one cursor.

Must still work:
  - All existing `scriptui-core` unit and integration tests unchanged.

Drops: none

Tests:
  - `default_en_us_suggests_for_curly_apostrophe_misspelling`: `SpellEngine::default_en_us().suggest("don\u{2019}tt", 5)` -> non-empty suggestions containing `"don't"`.
  - `readme_default_cua_keymap_covers_every_documented_shortcut`: `Keymap::default_cua()` -> every README CUA shortcut resolves to its documented `Action`.
  - `config_flags_control_spellcheck_preview_split_and_soft_wrap`: `AppConfig::load_or_default` on missing/partial/invalid temp files + `EditorState::new` with `spell_enabled = false` and `preview_split_percent = 35` -> zero spell diagnostics on `"wrld"`, `TogglePreview` enters `PreviewMode::Split(35)`, and `ToggleSoftWrap`/`ToggleLineNumbers` toggle config booleans.
  - `editor_navigation_selection_cut_find_prev_and_escape_collapse`: `EditorState::new(AppConfig::default())` with multi-line text -> `SelectEnd` selects to line end, `Cut` removes selection, `FindNext`/`FindPrev` cycle matches in both directions, and `Escape` after `SelectAllOccurrences` collapses `selection().len()` to `1`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn default_en_us_suggests_for_curly_apostrophe_misspelling' crates/scriptui-core/src/spell.rs && grep -qE 'fn readme_default_cua_keymap_covers_every_documented_shortcut' crates/scriptui-core/tests/editor_state_pipeline.rs && grep -qE 'fn config_flags_control_spellcheck_preview_split_and_soft_wrap' crates/scriptui-core/tests/editor_state_pipeline.rs && grep -qE 'fn editor_navigation_selection_cut_find_prev_and_escape_collapse' crates/scriptui-core/tests/editor_state_pipeline.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 10B: TUI End-to-End Coverage Gaps (`TestBackend` Keybindings, Modals, Mouse & Narrow Layout)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 10B: Add TestBackend integration tests in scriptui-tui covering README file/buffer modals, Find/Replace inline shortcuts, editing/OSC52/redo, mouse interactions, and narrow-viewport rendering.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/tests/tui_integration.rs
READ_ONLY_REFS:
  - README.md:111-201 (Keybindings and Mouse Interactions)
  - crates/scriptui-tui/src/app.rs:98-355 (render_frame, handle_key_event, handle_mouse_event, handle_paste_event)
  - crates/scriptui-tui/src/modals.rs:103-310 (spell quick fix, confirm modal, file prompt)
  - crates/scriptui-tui/src/find_bar.rs:1-160 (FindFocus, handle_find_bar_key)
  - crates/scriptui-tui/src/mouse.rs:1-180 (MouseRouter::handle_mouse)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT modify any file under `src/`. Do NOT weaken or remove any existing test in `tui_integration.rs`.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: All README-documented interactive TUI workflows (file/buffer lifecycle and dirty confirmation modals, Find/Replace inline shortcuts and F3 navigation, editing/selection/OSC 52 cut & copy/redo/bracketed paste, mouse click/drag/tab/right-click spell/divider resize, and narrow-terminal rendering with long text) are verified through real `TuiApp` event handlers and `TestBackend` rendering.

Contract & Behavior:
  1. `crates/scriptui-tui/tests/tui_integration.rs`:
     - Add 5 integration tests exercising `TuiApp::handle_key_event`, `TuiApp::handle_mouse_event`, `TuiApp::handle_paste_event`, and `TuiApp::render_frame` on `TestBackend`:
       a. `tui_file_buffer_lifecycle_and_dirty_confirm_modals`: tests `Ctrl+N` (new buffer), `Ctrl+PageDown`/`Ctrl+PageUp` (tab switch), `Ctrl+Shift+S` (`SaveAsPrompt` -> type temp file path -> `Enter` writes file to disk), further edit + `Ctrl+S` (saves updated content), `Ctrl+O` (`OpenFilePrompt` -> type second temp file path -> `Enter` loads it), dirty edit + `Ctrl+W` (`ConfirmCloseBuffer` -> `n` keeps buffer, `Ctrl+W` then `y` closes it), dirty edit + `Ctrl+Q` (`ConfirmQuit` -> `n` cancels, `Ctrl+Q` then `y` sets `state.should_quit == true`).
       b. `tui_find_replace_bar_inline_shortcuts_and_f3_navigation`: tests `Ctrl+H` (opens `OverlayMode::Replace`), `Alt+C` (toggles `case_sensitive`), `Alt+W` (toggles `whole_word`), `Alt+R` (toggles `use_regex`), `Tab` / `Shift+Tab` (switches `find_bar.focus`), `Enter` (next match in Find / replace active match in Replace), `Shift+Enter` (previous match), `Ctrl+Enter` (replace all), `Esc` (closes bar), `F3` / `Shift+F3` (navigates matches after closing bar), and `Ctrl+Shift+L` (selects all occurrences as multi-cursors).
       c. `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste`: tests `Ctrl+A` (select all), `Ctrl+C` (writes OSC 52 `\x1b]52;c;` sequence to clipboard buffer), `Ctrl+X` (writes OSC 52 and deletes selection), `handle_paste_event` (inserts pasted text), `Ctrl+Z` (undo), `Ctrl+Y` and `Ctrl+Shift+Z` (redo), `Shift+Right` / `Shift+End` / `Shift+Home` (extends selection), `Alt+Z` (toggles `config.soft_wrap`), and `Esc` (collapses multi-cursor to 1).
       d. `tui_mouse_click_drag_tab_right_click_spell_and_split_divider`: renders `TuiApp` on `TestBackend::new(100, 24)` with two tabs and `"hello wrld"` in the active buffer -> left-click inside editor moves caret, left-click + drag + up creates non-empty selection, right-click on `"wrld"` moves caret and opens `OverlayMode::SpellQuickFix`, `Esc` closes modal, left-click on second tab hitbox switches active tab, left-click on preview edge tab opens `PreviewMode::Split(_)`, left-click + drag + up on split divider updates `config.preview_split_percent` clamped within `20..=80`.
       e. `tui_narrow_viewport_and_long_lines_render_without_panic`: renders `TuiApp` on `TestBackend::new(40, 12)` with a 200-character single line containing a misspelling (`"wrld"`) under both `soft_wrap = true` and `soft_wrap = false`, and with each overlay active (`Find`, `CommandPalette`, `SpellQuickFix`) -> every draw succeeds with non-empty rendered cells in the viewport and overlay regions.

Must still work:
  - All existing tests in `crates/scriptui-tui/tests/tui_integration.rs` unchanged and passing.

Drops: none

Tests:
  - `tui_file_buffer_lifecycle_and_dirty_confirm_modals`: `TuiApp::from_cli` + temp dir -> Save As, Save, Open, tab switch, dirty Close/Quit cancel and confirm all produce exact file/buffer/quit states.
  - `tui_find_replace_bar_inline_shortcuts_and_f3_navigation`: `TuiApp::from_cli` with multi-occurrence text -> all Find/Replace inline shortcuts, `F3`/`Shift+F3`, and `Ctrl+Shift+L` update toggles, matches, text, and selections as documented.
  - `tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste`: `TuiApp::from_cli` -> `Ctrl+C`/`Ctrl+X` emit OSC 52, `Ctrl+Z`/`Ctrl+Y`/`Ctrl+Shift+Z` undo/redo, `Shift+Arrows`/`Home`/`End` extend selection, `Alt+Z` toggles soft wrap, `Esc` collapses multi-cursor.
  - `tui_mouse_click_drag_tab_right_click_spell_and_split_divider`: `TuiApp::render_frame` + `handle_mouse_event` -> editor click/drag, right-click spell quick-fix, tab bar click, preview edge tab click, and split divider drag clamp all succeed.
  - `tui_narrow_viewport_and_long_lines_render_without_panic`: `TestBackend::new(40, 12)` with 200-char line -> renders cleanly across wrap modes and `Find`/`CommandPalette`/`SpellQuickFix` overlays.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn tui_file_buffer_lifecycle_and_dirty_confirm_modals' crates/scriptui-tui/tests/tui_integration.rs && grep -qE 'fn tui_find_replace_bar_inline_shortcuts_and_f3_navigation' crates/scriptui-tui/tests/tui_integration.rs && grep -qE 'fn tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste' crates/scriptui-tui/tests/tui_integration.rs && grep -qE 'fn tui_mouse_click_drag_tab_right_click_spell_and_split_divider' crates/scriptui-tui/tests/tui_integration.rs && grep -qE 'fn tui_narrow_viewport_and_long_lines_render_without_panic' crates/scriptui-tui/tests/tui_integration.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 10C: CLI Binary, `XDG_CONFIG_HOME`, Config & Custom Theme Coverage (`TASK-0048`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 10C: Add end-to-end CLI binary, XDG_CONFIG_HOME dict-import, multi-file startup, config-driven TUI rendering, and custom TOML theme tests, and read CARGO_PKG_VERSION in the version test (TASK-0048).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/tests/tui_integration.rs
READ_ONLY_REFS:
  - README.md:46-109 (CLI Options, Configuration, Themes, Hunspell Dictionaries)
  - crates/scriptui-tui/src/main.rs:1-38 (dict import CLI path and XDG_CONFIG_HOME)
  - crates/scriptui-tui/src/app.rs:65-91 (TuiApp::from_cli)
  - crates/scriptui-core/src/theme.rs:287-330 (resolve_theme, load_theme_file)
  - crates/scriptui-tui/tests/tui_integration.rs:465-471 (cli_version_matches_workspace_version)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT mutate process-global environment variables (`std::env::set_var`) in tests (use `Command::env` for subprocess isolation so parallel tests cannot race). Do NOT modify `src/` files.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: The compiled `scriptui` binary's `--version`, `--help`, and `dict import` (respecting `XDG_CONFIG_HOME` and rejecting invalid locales), `TuiApp::from_cli` multi-file startup and flag overrides, `line_numbers = false` / `spell_enabled = false` TUI rendering, and custom TOML theme overrides are verified end-to-end, and `cli_version_matches_workspace_version` reads `env!("CARGO_PKG_VERSION")` instead of hardcoding `"0.1.0"` (closes `TASK-0048`).

Contract & Behavior:
  1. `crates/scriptui-tui/tests/tui_integration.rs`:
     - Update `cli_version_matches_workspace_version` to compare `err.to_string().trim()` against `format!("scriptui {}", env!("CARGO_PKG_VERSION"))` (closes `TASK-0048`).
     - Add `cli_binary_version_help_and_dict_import_with_xdg_config_home` invoking `std::process::Command::new(env!("CARGO_BIN_EXE_scriptui"))` with `.env("XDG_CONFIG_HOME", temp_xdg)`:
       - `--version` exits 0 and stdout equals `format!("scriptui {}\n", env!("CARGO_PKG_VERSION"))`.
       - `--help` exits 0 and stdout contains `--theme`, `--locale`, and `dict`.
       - `dict import xx_CLI <dic> <aff>` exits 0, stdout contains `"Imported dictionary for 'xx_CLI'"`, and `<temp_xdg>/scriptui/dicts/xx_CLI.dic` and `xx_CLI.aff` exist with the source contents.
       - `dict import ../bad <dic> <aff>` exits non-zero and stderr is non-empty.
     - Add `cli_startup_opens_multiple_files_and_overrides_config`:
       - Invokes `TuiApp::from_cli` with two temporary Markdown files (`first.md`, `second.md`), `theme: Some("ice_cyan")`, `locale: Some("en_US")` -> opens both buffers in tabs with expected names/contents and sets `state.theme.name == "ice_cyan"`.
       - Constructs a `TuiApp` (or `EditorState::new`) with `AppConfig { line_numbers: false, spell_enabled: false, ..AppConfig::default() }`, inserts `"hello wrld"`, renders on `TestBackend::new(80, 16)` -> `viewport.gutter_width == 0` and `active_analysis().1` (spell diagnostics) is empty.
     - Add `custom_toml_theme_overrides_builtin_and_validates_names`:
       - Writes a valid TOML theme overriding `crt_amber` (with a distinct background RGB such as `{ r = 1, g = 2, b = 3 }`) into `<temp>/themes/crt_amber.toml`, calls `scriptui_core::resolve_theme("crt_amber", &[themes_dir])` -> returns the custom theme's RGB rather than the built-in `#0A0A08`, and `resolve_theme("../escape", &[themes_dir])` returns `Err`.

Must still work:
  - All existing and `STEP 10B` tests in `crates/scriptui-tui/tests/tui_integration.rs` unchanged and passing.

Drops: none

Tests:
  - `cli_version_matches_workspace_version`: `Cli::try_parse_from(["scriptui", "--version"])` -> matches `format!("scriptui {}", env!("CARGO_PKG_VERSION"))`.
  - `cli_binary_version_help_and_dict_import_with_xdg_config_home`: real `scriptui` binary via `Command::new(env!("CARGO_BIN_EXE_scriptui"))` with isolated `XDG_CONFIG_HOME` -> `--version`, `--help`, valid `dict import`, and traversal rejection all behave as documented.
  - `cli_startup_opens_multiple_files_and_overrides_config`: `TuiApp::from_cli` with 2 files + `--theme ice_cyan` -> 2 tabs loaded with `"ice_cyan"`; `line_numbers = false` + `spell_enabled = false` renders with `gutter_width == 0` and zero diagnostics.
  - `custom_toml_theme_overrides_builtin_and_validates_names`: `resolve_theme("crt_amber", &[temp_themes_dir])` -> custom background RGB overrides built-in; `resolve_theme("../escape", ...)` -> `Err`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'CARGO_PKG_VERSION' crates/scriptui-tui/tests/tui_integration.rs && grep -qE 'fn cli_binary_version_help_and_dict_import_with_xdg_config_home' crates/scriptui-tui/tests/tui_integration.rs && grep -qE 'fn cli_startup_opens_multiple_files_and_overrides_config' crates/scriptui-tui/tests/tui_integration.rs && grep -qE 'fn custom_toml_theme_overrides_builtin_and_validates_names' crates/scriptui-tui/tests/tui_integration.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```


---

## Phase 11 Step Blocks

### STEP 11A: Add `deny.toml` & Verify `cargo deny check` Locally

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 11A: Add workspace root `deny.toml` enforcing RustSec advisories, SPDX license allowlist, dependency bans, and registry sources, verified cleanly with `cargo deny check`.

[SCOPE_LOCK]
ALLOWED_FILES:
  - deny.toml (NEW)
READ_ONLY_REFS:
  - Cargo.toml:1-18 (workspace package metadata and MIT license)
  - crates/scriptui-core/Cargo.toml:1-23 (core dependencies)
  - crates/scriptui-tui/Cargo.toml:1-22 (tui dependencies)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT modify `Cargo.toml`, `Cargo.lock`, Rust source files, or `docs/*`. Do NOT globally disable `unmaintained`, `unsound`, or `yanked` advisory checks.

[PRE_GATE]
command -v cargo-deny >/dev/null

[MUTATION_SPEC]
Goal: Running `cargo deny check` at the workspace root enforces dependency security advisories, SPDX license compliance, wildcard/version bans, and registry source restrictions and exits `0` with every transitive exception explicitly documented.

Contract & Behavior:
  1. `deny.toml` (cargo-deny v2 schema compatible with `cargo-deny 0.20+`):
     - `[graph]`: set `targets` to `["x86_64-unknown-linux-gnu", "aarch64-apple-darwin"]` and `all-features = true`.
     - `[advisories]`:
       - Keep vulnerability, unsound, and yanked advisory checks enabled at default strictness (`deny`).
       - Ignore only the three unmaintained transitive advisories that have no safe upstream upgrade in the current dependency tree, each with a `reason` documenting its crate chain:
         - `RUSTSEC-2025-0141` (`bincode 1.3.3` via `syntect 5.3.0`)
         - `RUSTSEC-2024-0436` (`paste 1.0.15` via `ratatui 0.29.0`)
         - `RUSTSEC-2024-0320` (`yaml-rust 0.4.5` via `syntect 5.3.0`)
     - `[licenses]`:
       - Set `confidence-threshold = 0.8`.
       - Allow only the SPDX licenses present in the workspace dependency graph: `"MIT"`, `"Apache-2.0"`, `"Apache-2.0 WITH LLVM-exception"`, `"BSD-2-Clause"`, `"BSD-3-Clause"`, `"0BSD"`, `"BSL-1.0"`, `"Zlib"`, `"Unlicense"`, `"Unicode-3.0"`, and `"MPL-2.0"` (with an inline comment noting `"MPL-2.0"` is for `spellbook`, file-level weak copyleft).
     - `[bans]`:
       - `multiple-versions = "warn"`
       - `wildcards = "deny"`
       - `highlight = "all"`
     - `[sources]`:
       - `unknown-registry = "deny"`
       - `unknown-git = "deny"`
       - `allow-registry = ["https://github.com/rust-lang/crates.io-index"]`
       - `allow-git = []`

Must still work:
  - `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` continue to pass unchanged.

Drops: none

Tests:
  - `cargo deny check` (advisories, licenses, bans, sources): executed against the workspace `Cargo.lock` -> exits `0` with zero unapproved licenses, zero disallowed sources, and zero unacknowledged advisories.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo deny check && grep -qE 'RUSTSEC-2025-0141' deny.toml && grep -qE 'RUSTSEC-2024-0436' deny.toml && grep -qE 'RUSTSEC-2024-0320' deny.toml && grep -qE '"MPL-2.0"' deny.toml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

## Phase 12 Step Blocks

### STEP 12A: `deny.toml`, Workspace Release Metadata & Third-Party License Notice (`TASK-0060`, `TASK-0061`, `TASK-0062`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 12A: Add workspace `deny.toml`, `THIRD_PARTY_LICENSES.md`, `repository`/`rust-version` metadata, and pinned `scriptui-core` version in `scriptui-tui/Cargo.toml` so `cargo deny check` passes cleanly (re-split of deferred STEP 11A).

[SCOPE_LOCK]
ALLOWED_FILES:
  - deny.toml (NEW)
  - THIRD_PARTY_LICENSES.md (NEW)
  - Cargo.toml
  - crates/scriptui-core/Cargo.toml
  - crates/scriptui-tui/Cargo.toml
READ_ONLY_REFS:
  - Cargo.toml:1-18 (workspace.package)
  - crates/scriptui-core/Cargo.toml:1-23
  - crates/scriptui-tui/Cargo.toml:1-22 (scriptui-core path dep on line 16)
  - LICENSE:1-21
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT modify `Cargo.lock`, any Rust source files, or `docs/*`. Do NOT globally disable `unmaintained`, `unsound`, or `yanked` advisory checks.

[PRE_GATE]
command -v cargo-deny >/dev/null

[MUTATION_SPEC]
Goal: `cargo deny check` passes with zero errors at the workspace root, both crates inherit `repository` and `rust-version`, and third-party license attributions for bundled dictionaries and `spellbook` (`MPL-2.0`) are documented in `THIRD_PARTY_LICENSES.md`.

Contract & Behavior:
  1. `Cargo.toml` `[workspace.package]`: keep all existing keys unchanged; add:
     - `repository = "https://github.com/TheInfoMoose/Scriptui"`
     - `rust-version = "1.80"`
  2. `crates/scriptui-core/Cargo.toml` and `crates/scriptui-tui/Cargo.toml` `[package]`: keep existing keys; add `repository.workspace = true` and `rust-version.workspace = true`.
  3. `crates/scriptui-tui/Cargo.toml` `[dependencies]`: update `scriptui-core` to `{ version = "0.1.0", path = "../scriptui-core" }` so `cargo-deny` wildcard checks pass.
  4. `deny.toml` (cargo-deny v2 schema compatible with `cargo-deny 0.20+`):
     - `[graph]`: `targets = ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin"]`, `all-features = true`.
     - `[advisories]`: keep vulnerability, unsound, and yanked checks at default strictness; ignore only the three unmaintained transitive advisories with explicit `reason` strings naming their crate chains: `RUSTSEC-2025-0141` (`bincode 1.3.3` via `syntect 5.3.0`), `RUSTSEC-2024-0436` (`paste 1.0.15` via `ratatui 0.29.0`), `RUSTSEC-2024-0320` (`yaml-rust 0.4.5` via `syntect 5.3.0`).
     - `[licenses]`: `confidence-threshold = 0.8`; allow only `"MIT"`, `"Apache-2.0"`, `"Apache-2.0 WITH LLVM-exception"`, `"BSD-2-Clause"`, `"BSD-3-Clause"`, `"0BSD"`, `"BSL-1.0"`, `"Zlib"`, `"Unlicense"`, `"Unicode-3.0"`, and `"MPL-2.0"` (comment noting `spellbook` file-level weak copyleft).
     - `[bans]`: `multiple-versions = "warn"`, `wildcards = "deny"`, `highlight = "all"`.
     - `[sources]`: `unknown-registry = "deny"`, `unknown-git = "deny"`, `allow-registry = ["https://github.com/rust-lang/crates.io-index"]`, `allow-git = []`.
  5. `THIRD_PARTY_LICENSES.md` (new, repo root):
     - Document third-party notices for:
       - `crates/scriptui-core/assets/dicts/en_US.aff` and `en_US.dic` (SCOWL / Hunspell `en_US` English dictionary, BSD/MIT-style permissive copyright notice).
       - `spellbook` crate (`MPL-2.0`, Mozilla Public License 2.0, file-level weak copyleft).
     - Must not contain local host filesystem paths.

Must still work:
  - `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass unchanged.
  - `Cargo.lock` remains unchanged.

Drops: none

Tests:
  - `cargo deny check`: executed at workspace root against `Cargo.lock` -> exits `0` with zero unapproved licenses, zero wildcard errors, zero disallowed sources, and zero unacknowledged advisories.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo deny check && test -f THIRD_PARTY_LICENSES.md && grep -qE 'MPL-2\.0' THIRD_PARTY_LICENSES.md && grep -qE 'en_US' THIRD_PARTY_LICENSES.md && grep -qE '^rust-version = "1\.80"' Cargo.toml && grep -qE '^repository = "https://github\.com/TheInfoMoose/Scriptui"' Cargo.toml && grep -qE 'scriptui-core = \{ version = "0\.1\.0", path = "\.\./scriptui-core" \}' crates/scriptui-tui/Cargo.toml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 12B: UTF-8 Safe Task-List Parsing, `O_EXCL` Atomic File I/O & Buffer Helpers (`BUG-0017`, `BUG-0020` part 1, `BUG-0025`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 12B: Fix UTF-8 slice panic in `indent::parse_task_list`, enforce `O_EXCL` unique temp files in atomic saves and locale import, guard no-op `apply_insert` dirtying, and add `DocumentBuffer::empty_with_path`.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/indent.rs
  - crates/scriptui-core/src/buffer.rs
  - crates/scriptui-core/src/spell.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/indent.rs:74-96 (parse_task_list)
  - crates/scriptui-core/src/buffer.rs:38-145,268-288 (empty, save_atomic, apply_insert)
  - crates/scriptui-core/src/spell.rs:59-110,355-396 (PersonalDictionary::save_to_file, import_locale_files)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT add new external crates or change existing public method signatures.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Multi-byte UTF-8 lines never panic on Enter, atomic saves and dictionary imports create unique temp files with `O_EXCL` (`create_new(true)`) so symlinks/collisions cannot be overwritten, empty no-op inserts leave `dirty` unchanged, and `DocumentBuffer::empty_with_path` constructs a clean named buffer.

Contract & Behavior:
  1. `crates/scriptui-core/src/indent.rs` (`parse_task_list`):
     - Never slice `&trimmed[1..6]` without checking UTF-8 char boundaries or using safe prefix matching (e.g., `trimmed[1..].strip_prefix(" [ ] ")` / `" [x] "` / `" [X] "`).
     - Lines where a multi-byte UTF-8 character straddles byte offset 6 (e.g., `"-    é"`, `"- é"`, `"*  日本語"`) must not panic and must fall through to normal unordered-bullet or plain-newline handling.
  2. `crates/scriptui-core/src/buffer.rs`:
     - Add `pub fn empty_with_path(path: PathBuf) -> Self`: creates an empty buffer with `path: Some(path)`, `dirty: false`, `last_disk_mtime: None`, `selection: SelectionSet::point(0)`, `revision: 0`, and a fresh `doc_id`.
     - `DocumentBuffer::apply_insert`: set `self.dirty = true` and increment `self.revision` ONLY when at least one edit actually changed content (`edit.start < edit.end || !text.is_empty()`). A no-op insert (`""` with point cursor(s)) leaves `dirty` and `revision` unchanged.
     - `DocumentBuffer::save_atomic`: create the sibling temp file using `OpenOptions::new().write(true).create_new(true)` (`O_EXCL`) with a unique name combining PID and a monotonic atomic counter (retrying on `ErrorKind::AlreadyExists` up to a bounded retry limit before returning `ScriptuiError::Io`). Remove the temp file on write/sync/rename error.
  3. `crates/scriptui-core/src/spell.rs`:
     - `PersonalDictionary::save_to_file`: use `OpenOptions::new().write(true).create_new(true)` (`O_EXCL`) with a unique temp filename (PID + atomic counter, bounded retry on `AlreadyExists`), `flush` + `sync_all`, and atomic `fs::rename`, cleaning up the temp file on error.
     - `import_locale_files`: write both `{locale}.dic` and `{locale}.aff` via `O_EXCL` unique temp files in `dest_dir` (`flush` + `sync_all`) before renaming into place; clean up any temp files on failure so partial/corrupted files are not left behind.

Must still work:
  - All existing `indent.rs`, `buffer.rs`, and `spell.rs` unit tests (including task list continuation, empty task list clearing, `save_atomic`, `PersonalDictionary::save_to_file`, and `import_locale_files`).

Drops: none

Tests:
  - `task_list_multibyte_utf8_at_byte_six_does_not_panic` (in `indent.rs`): `compute_enter_outcome("-    é", IndentConfig::default(), true)` and `compute_enter_outcome("-   é", IndentConfig::default(), true)` -> does not panic and returns `EnterOutcome::Insert("\n- ".to_string())`.
  - `empty_insert_does_not_mark_buffer_dirty_and_empty_with_path_is_clean` (in `buffer.rs`): `DocumentBuffer::empty_with_path(PathBuf::from("new.md"))` -> `is_dirty() == false`, `path() == Some(Path::new("new.md"))`, and `apply_insert("")` with point selection keeps `is_dirty() == false` and `revision() == 0`.
  - `save_atomic_skips_preexisting_temp_collision_with_o_excl` (in `buffer.rs`): `save_atomic()` writes target content cleanly even if a stale `.scriptui.tmp.*` file exists in the same directory, leaving no leftover temp files.
  - `personal_dict_and_locale_import_use_atomic_o_excl_writes` (in `spell.rs`): `PersonalDictionary::save_to_file` and `import_locale_files` succeed cleanly into a temp directory and leave no leftover `.tmp` files behind.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn empty_with_path' crates/scriptui-core/src/buffer.rs && grep -qE 'create_new\(true\)' crates/scriptui-core/src/buffer.rs && grep -qE 'create_new\(true\)' crates/scriptui-core/src/spell.rs && grep -qE 'fn task_list_multibyte_utf8_at_byte_six_does_not_panic' crates/scriptui-core/src/indent.rs && grep -qE 'fn empty_insert_does_not_mark_buffer_dirty_and_empty_with_path_is_clean' crates/scriptui-core/src/buffer.rs && grep -qE 'fn personal_dict_and_locale_import_use_atomic_o_excl_writes' crates/scriptui-core/src/spell.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 12C: Clean-Point Undo `dirty` Tracking, Internal `Ctrl+V` Clipboard & `EditorState::new_with_dirs` (`BUG-0019`, `BUG-0020` part 2, `BUG-0018` core)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 12C: Track clean-point save depth in `UndoStack` (wired into Save/SaveAs), implement internal clipboard for `Action::Copy`/`Cut`/`Paste`, and add `EditorState::new_with_dirs` for custom themes and spell locales.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/history.rs
  - crates/scriptui-core/src/editor.rs
  - crates/scriptui-tui/src/modals.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/history.rs:12-80 (UndoStack)
  - crates/scriptui-core/src/editor.rs:225-275,619-649,719-733 (EditorState::new, Save, Undo/Redo, Cut/Copy/Paste)
  - crates/scriptui-core/src/buffer.rs:165-172 (is_dirty, set_dirty)
  - crates/scriptui-core/src/theme.rs:306-330 (resolve_theme)
  - crates/scriptui-core/src/spell.rs:195-215 (SpellEngine::load_locale)
  - crates/scriptui-tui/src/modals.rs:285-307 (SaveAsPrompt Enter)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT break `EditorState::new(config)` or existing `UndoStack` method signatures.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Undoing back to the initial or last-saved state clears `DocumentBuffer::is_dirty()`, `Action::Copy`/`Cut`/`Paste` work through an internal editor clipboard, and `EditorState::new_with_dirs` resolves custom themes and non-default spell locales from configured directories with visible fallback warnings.

Contract & Behavior:
  1. `crates/scriptui-core/src/history.rs` (`UndoStack`):
     - Track the saved stack depth (`saved_depth: Option<usize>`, initialized to `Some(0)` in `UndoStack::new` and reset to `Some(0)` in `UndoStack::clear`).
     - Add `pub fn mark_saved(&mut self)` (sets `saved_depth = Some(self.undo_entries.len())`) and `pub fn is_at_saved_state(&self) -> bool` (returns `self.saved_depth == Some(self.undo_entries.len())`).
     - In `record`: if `saved_depth` is `Some(d)` and `d > self.undo_entries.len()` (branching after undoing past the saved point), set `saved_depth = None`. When `undo_entries.len() > self.max_entries` evicts index 0, decrement `saved_depth` by 1 if `d > 0`, or set `saved_depth = None` if `d == 0`.
     - In `undo` and `redo`: after applying raw edits and restoring selection, call `buf.set_dirty(!self.is_at_saved_state())`.
  2. `crates/scriptui-core/src/editor.rs` (`EditorState`):
     - Add `pub fn new_with_dirs(config: AppConfig, theme_dirs: &[PathBuf], dict_dirs: &[PathBuf]) -> ScriptuiResult<Self>`:
       - Resolve `config.theme` via `resolve_theme(&config.theme, theme_dirs)`. On `Err`, fall back to `default_theme()?` and record a non-fatal warning in `status_message` naming the missing/invalid theme.
       - When `config.spell_enabled` is `true`, if `config.spell_locale.is_empty()` or `config.spell_locale == "en_US"`, load `SpellEngine::default_en_us().ok()`; otherwise call `SpellEngine::load_locale(&config.spell_locale, dict_dirs)` and on `Err`, fall back to `SpellEngine::default_en_us().ok()` while recording a non-fatal warning in `status_message` naming the locale.
     - Keep `pub fn new(config: AppConfig) -> ScriptuiResult<Self>` delegating to `Self::new_with_dirs(config, &[], &[])`.
     - Add internal clipboard storage on `EditorState` with `pub fn clipboard(&self) -> &str` and `pub fn set_clipboard(&mut self, text: impl Into<String>)`.
     - In `EditorState::dispatch`:
       - `Action::Save`: after `save_atomic()?`, call `active.history.mark_saved()`.
       - `Action::Copy`: if `self.selected_text()` is non-empty, store it in `self.clipboard`.
       - `Action::Cut`: if `self.selected_text()` is non-empty, store it in `self.clipboard` before calling `self.handle_delete(true)`.
       - `Action::Paste`: if `!self.clipboard.is_empty()`, clone and insert it via `self.insert_text(&text)`.
  3. `crates/scriptui-tui/src/modals.rs` (`handle_file_prompt_key`):
     - In `OverlayMode::SaveAsPrompt` `KeyCode::Enter`: after `save_as_atomic(...)` succeeds, call `active.history.mark_saved()`.

Must still work:
  - All existing `history.rs`, `editor.rs`, `modals.rs`, and `tui_integration.rs` tests.
  - `EditorState::new(config)` behavior for built-in themes and `en_US` spellcheck.

Drops: none

Tests:
  - `undo_back_to_initial_or_saved_state_clears_dirty_flag` (in `history.rs`): edit -> `is_dirty() == true` -> `undo()` -> `is_dirty() == false` -> `redo()` -> `is_dirty() == true`; then `mark_saved()` + `buf.set_dirty(false)` -> second edit -> `is_dirty() == true` -> `undo()` -> `is_dirty() == false`; undoing past saved point and recording a new edit makes `is_at_saved_state() == false`.
  - `editor_copy_cut_and_paste_roundtrip_via_internal_clipboard` (in `editor.rs`): insert `"hello world"`, select `"hello"`, `dispatch(Action::Copy)`, move to end, `dispatch(Action::Paste)` -> `"hello worldhello"`; select `"world"`, `dispatch(Action::Cut)`, `dispatch(Action::Paste)` -> restores `"world"` and records undoable transactions.
  - `editor_new_with_dirs_loads_custom_theme_and_locale_or_warns` (in `editor.rs`): temp theme TOML and temp locale `.dic`/`.aff` in `theme_dirs`/`dict_dirs` -> `EditorState::new_with_dirs` loads both without warnings; unknown theme/locale falls back to defaults and sets `status_message.is_some()`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn mark_saved' crates/scriptui-core/src/history.rs && grep -qE 'pub fn new_with_dirs' crates/scriptui-core/src/editor.rs && grep -qE 'pub fn clipboard' crates/scriptui-core/src/editor.rs && grep -qE 'mark_saved\(\)' crates/scriptui-tui/src/modals.rs && grep -qE 'fn undo_back_to_initial_or_saved_state_clears_dirty_flag' crates/scriptui-core/src/history.rs && grep -qE 'fn editor_copy_cut_and_paste_roundtrip_via_internal_clipboard' crates/scriptui-core/src/editor.rs && grep -qE 'fn editor_new_with_dirs_loads_custom_theme_and_locale_or_warns' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 12D: Startup Config/Theme/Locale Wiring, CLI New-File Creation, Custom-Theme Palette & `KeyEventKind::Repeat` (`BUG-0018`, `BUG-0019` TUI, `BUG-0021`, `BUG-0022`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 12D: Wire TuiApp startup to load custom themes/locales and warn on bad config.toml, open non-existent CLI paths as clean named buffers, list custom themes in CommandPalette, accept `KeyEventKind::Repeat`, and forward `Action::Copy` to `EditorState`.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/buffers.rs
  - crates/scriptui-tui/src/app.rs
  - crates/scriptui-tui/src/palette.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffers.rs:106-152 (BufferManager::open_file)
  - crates/scriptui-core/src/buffer.rs:38-65 (DocumentBuffer::empty_with_path)
  - crates/scriptui-core/src/editor.rs:245-275 (EditorState::new_with_dirs)
  - crates/scriptui-tui/src/app.rs:65-100,172-268 (from_cli, refresh_palette_entries, handle_key_event)
  - crates/scriptui-tui/src/palette.rs:30-105 (discover_installed_locales, build_palette_entries)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT break `BufferManager::open_file` or `build_palette_entries(extra_locales)` signatures.

[PRE_GATE]
grep -qE 'pub fn empty_with_path' crates/scriptui-core/src/buffer.rs && grep -qE 'pub fn new_with_dirs' crates/scriptui-core/src/editor.rs

[MUTATION_SPEC]
Goal: Launching `scriptui` with a non-existent file creates a clean named buffer ready to save, `--theme`/`--locale` and `config.toml` resolve custom themes/dictionaries and surface config parse warnings, custom themes appear in the command palette, held keys (`KeyEventKind::Repeat`) repeat normally, and `Ctrl+C` + `Ctrl+V` works end-to-end in `TuiApp`.

Contract & Behavior:
  1. `crates/scriptui-core/src/buffers.rs` (`BufferManager`):
     - Add `pub fn open_or_create_file(&mut self, path: &Path) -> ScriptuiResult<BufferId>`:
       - Calls `self.open_file(path)`; if that returns `Err(ScriptuiError::Io { source, .. })` with `source.kind() == io::ErrorKind::NotFound`, creates a clean `DocumentBuffer::empty_with_path(path.to_path_buf())`, reusing the initial empty untitled buffer if appropriate (matching `open_file`'s `replace_initial` rule) and deduplicating against any already-open buffer with the same `path`.
       - Non-`NotFound` I/O errors (e.g., permission denied, directory path) still return `Err`.
  2. `crates/scriptui-tui/src/palette.rs`:
     - Add `pub fn discover_custom_themes(theme_dirs: &[PathBuf]) -> Vec<String>`: scans `theme_dirs` for valid `*.toml` files (stem passes safe theme identifier rules, sorted and deduplicated).
     - Add `pub fn build_palette_entries_with_themes(extra_themes: &[String], extra_locales: &[String]) -> Vec<PaletteEntry>`: includes all built-in themes plus any non-duplicate `extra_themes` as `PaletteItemKind::Theme(name)` entries (`id: format!("theme.{name}")`, `label: format!("Theme: {name}")`).
     - Keep `pub fn build_palette_entries(extra_locales: &[String]) -> Vec<PaletteEntry>` delegating to `build_palette_entries_with_themes(&[], extra_locales)`.
  3. `crates/scriptui-tui/src/app.rs` (`TuiApp`):
     - `TuiApp::from_cli`:
       - Compute `config_dir = AppConfig::default_config_dir()`, `theme_dirs = [config_dir.join("themes")]`, `dict_dirs = [config_dir.join("dicts")]`.
       - Load `config_path = config_dir.join("config.toml")` via `AppConfig::load_or_default(&config_path)`. On `Err(err)`, fall back to `AppConfig::default()` and preserve a visible warning string (`format!("Config error ({}): {err}", config_path.display())`) to store in `state.status_message` (joining with any theme/locale warning if both occur).
       - Initialize `EditorState::new_with_dirs(config, &theme_dirs, &dict_dirs)?`.
       - Open each path in `&cli.files` via `state.buffers.open_or_create_file(path)?`.
     - `TuiApp::refresh_palette_entries`: discover custom themes from `config_dir.join("themes")` alongside locales from `config_dir.join("dicts")` and call `build_palette_entries_with_themes(&custom_themes, &locales)`.
     - `TuiApp::handle_key_event`:
       - Accept `KeyEventKind::Press | KeyEventKind::Repeat` (return `Ok(())` only for `KeyEventKind::Release` / other kinds).
       - In the `Action::Copy` match arm, call `self.state.dispatch(Action::Copy)?;` in addition to emitting OSC 52 so `Ctrl+C` populates the internal clipboard for `Ctrl+V`.

Must still work:
  - All existing `buffers.rs`, `palette.rs`, `app.rs`, and `tui_integration.rs` tests (including OSC 52 copy/cut, CLI startup with existing files, and `KeyEventKind::Release` being ignored).

Drops: none

Tests:
  - `open_or_create_file_creates_clean_named_buffer_for_missing_path` (in `buffers.rs`): `BufferManager::open_or_create_file(&nonexistent_path)` -> succeeds, replaces initial empty untitled tab, `is_dirty() == false`, `path() == Some(&nonexistent_path)`, and second call with same path deduplicates to the same `BufferId`.
  - `discover_custom_themes_and_palette_entries_include_custom_toml` (in `palette.rs`): temp dir with `solarized.toml` -> `discover_custom_themes` returns `["solarized"]` and `build_palette_entries_with_themes` contains `PaletteItemKind::Theme("solarized")`.
  - `tui_app_handles_repeat_keys_ctrl_c_v_and_nonexistent_cli_file` (in `app.rs`): `TuiApp::from_cli` with a non-existent temp file path succeeds with a clean named buffer; sending `KeyEventKind::Repeat` character `'a'` inserts `'a'` while `KeyEventKind::Release` is ignored; `Ctrl+A` + `Ctrl+C` + `End` + `Ctrl+V` duplicates the text and emits OSC 52.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'pub fn open_or_create_file' crates/scriptui-core/src/buffers.rs && grep -qE 'pub fn discover_custom_themes' crates/scriptui-tui/src/palette.rs && grep -qE 'pub fn build_palette_entries_with_themes' crates/scriptui-tui/src/palette.rs && grep -qE 'KeyEventKind::Repeat' crates/scriptui-tui/src/app.rs && grep -qE 'fn open_or_create_file_creates_clean_named_buffer_for_missing_path' crates/scriptui-core/src/buffers.rs && grep -qE 'fn discover_custom_themes_and_palette_entries_include_custom_toml' crates/scriptui-tui/src/palette.rs && grep -qE 'fn tui_app_handles_repeat_keys_ctrl_c_v_and_nonexistent_cli_file' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 12E: Static `SyntaxSet` Caching & Revision-Keyed Preview Pane Cache (`BUG-0023`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 12E: Cache syntect's default `SyntaxSet` in a process-wide `LazyLock` and cache `PreviewDocument` inside `PreviewPaneState` keyed by `(doc_id, revision, theme_name, width)` to eliminate per-frame rebuilds.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/highlight.rs
  - crates/scriptui-tui/src/preview_pane.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/highlight.rs:80-130 (CodeHighlighter)
  - crates/scriptui-core/src/preview.rs:15-50 (PreviewDocument, build_preview)
  - crates/scriptui-tui/src/preview_pane.rs:10-70 (PreviewPaneState, render_preview_pane)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT change `render_preview_pane`'s function signature or `CodeHighlighter::new()` / `highlight_block()` signatures.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Creating `CodeHighlighter::new()` reuses a single lazily initialized process-wide `SyntaxSet`, and `render_preview_pane` reuses its cached `PreviewDocument` across frames whenever the active document id, revision, theme name, and preview inner width are unchanged.

Contract & Behavior:
  1. `crates/scriptui-core/src/highlight.rs` (`CodeHighlighter`):
     - Store `SyntaxSet::load_defaults_newlines()` in a `static DEFAULT_SYNTAX_SET: LazyLock<SyntaxSet>` (or `OnceLock<SyntaxSet>`).
     - `CodeHighlighter` borrows `&'static SyntaxSet` so `CodeHighlighter::new()` performs zero deserialization after first initialization.
     - Expose `pub fn syntax_set_ptr(&self) -> *const SyntaxSet` (or `#[cfg(test)]` / `pub` helper) so unit tests can assert two `CodeHighlighter::new()` instances share the identical `SyntaxSet` allocation.
  2. `crates/scriptui-tui/src/preview_pane.rs` (`PreviewPaneState`, `render_preview_pane`):
     - Keep `pub scroll_line`, `pub total_lines`, `pub viewport_height`, `pub sync_enabled` and `Default` / `Debug` / `Clone` on `PreviewPaneState`.
     - Add an internal cache storing `(doc_id: u64, doc_revision: u64, theme_name: String, inner_width: usize, preview_doc: PreviewDocument)` and a counter `rebuild_count: u64` with `pub fn rebuild_count(&self) -> u64`.
     - In `render_preview_pane`: only call `state.build_active_preview(inner_width)` and increment `rebuild_count` when the cache is empty or any of `(doc_id, doc_revision, state.theme.name, inner_width)` changed; otherwise render directly from the cached `PreviewDocument` (still updating scroll sync for `editor_top_source_line` and `viewport_height`).

Must still work:
  - All existing `highlight.rs`, `preview_pane.rs`, and `tui_integration.rs` tests, including narrow preview rendering (`area.width < 2 || area.height < 2` early return) and scroll sync when `editor_top_source_line` changes without a document edit.

Drops:
  - `PreviewPaneState` no longer derives `Copy`, `PartialEq`, or `Eq` (it now owns the cached `PreviewDocument`; no callers in the workspace rely on `Copy` or `PartialEq` for `PreviewPaneState`).

Tests:
  - `code_highlighter_reuses_static_syntax_set` (in `highlight.rs`): two `CodeHighlighter::new()` instances share the same `SyntaxSet` pointer and both highlight a Rust snippet identically.
  - `render_preview_pane_caches_until_doc_theme_or_width_changes` (in `preview_pane.rs`): rendering the same `EditorState` twice at `(60, 16)` keeps `pane_state.rebuild_count() == 1` even when `editor_top_source_line` changes; inserting text increments `rebuild_count()` to `2`; changing `area.width` to `40` increments `rebuild_count()` to `3`; changing `state.theme` increments `rebuild_count()` to `4`; rendering at `(40, 16)` again keeps `rebuild_count() == 4`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'LazyLock|OnceLock' crates/scriptui-core/src/highlight.rs && grep -qE 'pub fn rebuild_count' crates/scriptui-tui/src/preview_pane.rs && grep -qE 'fn code_highlighter_reuses_static_syntax_set' crates/scriptui-core/src/highlight.rs && grep -qE 'fn render_preview_pane_caches_until_doc_theme_or_width_changes' crates/scriptui-tui/src/preview_pane.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

### STEP 12F: Unix Signal Handling (`SIGTERM`/`SIGHUP`/`SIGTSTP`) & Alt-Screen Exit Error Ordering (`BUG-0024`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 12F: Register Unix `SIGTERM`/`SIGHUP`/`SIGTSTP` signal flags in `TerminalGuard`, poll them in `TuiApp::run_event_loop` for clean terminal restore/suspend, and drop `TerminalGuard` before printing exit errors in `main.rs`.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/Cargo.toml
  - Cargo.lock
  - crates/scriptui-tui/src/terminal.rs
  - crates/scriptui-tui/src/app.rs
  - crates/scriptui-tui/src/main.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/Cargo.toml:15-22 (dependencies; signal-hook 0.3.18 is already in Cargo.lock)
  - crates/scriptui-tui/src/terminal.rs:19-76 (TerminalGuard, restore_terminal_best_effort)
  - crates/scriptui-tui/src/app.rs:380-418 (run_event_loop)
  - crates/scriptui-tui/src/main.rs:31-53 (TerminalGuard::enter, run_event_loop, error exit)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT add any crate not already present in `Cargo.lock` (`signal-hook = "0.3"` only under `[target.'cfg(unix)'.dependencies]`). Do NOT weaken `deny.toml`.

[PRE_GATE]
test -f deny.toml && command -v cargo-deny >/dev/null

[MUTATION_SPEC]
Goal: Receiving `SIGTERM` or `SIGHUP` exits the event loop cleanly so `TerminalGuard::drop` restores the terminal, `SIGTSTP` restores the terminal before suspending and re-enters TUI mode on resume, and any `run_event_loop` error is printed to stderr only after `TerminalGuard` has dropped and left the alternate screen.

Contract & Behavior:
  1. `crates/scriptui-tui/Cargo.toml` and `Cargo.lock`:
     - Add `[target.'cfg(unix)'.dependencies]` with `signal-hook = "0.3"` (matching `signal-hook 0.3.18` already in `Cargo.lock`; `cargo deny check` must still exit `0`).
  2. `crates/scriptui-tui/src/terminal.rs`:
     - Introduce a small helper `SignalFlags` (containing `terminate: Arc<AtomicBool>` and `suspend: Arc<AtomicBool>`) with testable methods `should_terminate(&self) -> bool` and `take_suspend(&self) -> bool` (`swap(false, Ordering::SeqCst)`).
     - On `#[cfg(unix)]`, `TerminalGuard::enter` registers `signal_hook::flag::register` for `SIGTERM` and `SIGHUP` -> `terminate`, and `SIGTSTP` -> `suspend`, storing the `SigId`s and unregistering them in `TerminalGuard::drop` alongside `restore_terminal_best_effort()`. On non-Unix targets, `SignalFlags` stays `false` (no-op).
     - Expose on `TerminalGuard`:
       - `pub fn should_terminate(&self) -> bool`
       - `pub fn handle_suspend_if_requested(&mut self) -> io::Result<bool>`: if `take_suspend()` is `true`, on Unix restores the terminal (`restore_terminal_best_effort()`), raises `SIGSTOP` (`signal_hook::low_level::raise(signal_hook::consts::SIGSTOP)`), and upon `SIGCONT` resume re-enables raw mode, alt screen, mouse capture, bracketed paste, and calls `self.terminal.clear()?`, returning `Ok(true)`; returns `Ok(false)` when no suspend was requested.
  3. `crates/scriptui-tui/src/app.rs` (`TuiApp::run_event_loop`):
     - At the top of each loop iteration (and before redraw/poll), check `if guard.should_terminate() { self.state.should_quit = true; break; }`.
     - Poll `if guard.handle_suspend_if_requested()? { needs_redraw = true; }`.
  4. `crates/scriptui-tui/src/main.rs`:
     - Scope `guard` in a block so `guard` drops (leaving the alternate screen and disabling raw mode) BEFORE `eprintln!("ScripTUI exited with error: {err}")` runs on `Err(err)`.

Must still work:
  - All existing `terminal.rs`, `app.rs`, and `tui_integration.rs` tests.
  - `TerminalGuard` panic hook still restores the terminal on unwind.
  - `cargo deny check` exits `0`.

Drops: none

Tests:
  - `signal_flags_report_terminate_and_consume_suspend_once` (in `terminal.rs`): toggling `terminate` makes `should_terminate() == true`; toggling `suspend` makes the first `take_suspend()` return `true` and the second return `false`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo deny check && grep -qE 'signal-hook' crates/scriptui-tui/Cargo.toml && grep -qE 'pub fn should_terminate' crates/scriptui-tui/src/terminal.rs && grep -qE 'pub fn handle_suspend_if_requested' crates/scriptui-tui/src/terminal.rs && grep -qE 'should_terminate\(\)' crates/scriptui-tui/src/app.rs && grep -qE 'fn signal_flags_report_terminate_and_consume_suspend_once' crates/scriptui-tui/src/terminal.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```


---

## Phase 13 Step Blocks

### STEP 13A: TUI Input Modifier Filtering, OSC 52 Cap & `EINTR` Event Loop Retry

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0028`, `BUG-0035`, `BUG-0032` (OSC 52 cap)

```text
[STEP_ID]
STEP 13A: Ignore Super/Hyper/Meta modifiers in key chord mapping and char insertion, cap OSC 52 clipboard output at MAX_OSC52_BYTES, and retry crossterm event::poll/event::read on ErrorKind::Interrupted (EINTR).

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/terminal.rs
  - crates/scriptui-tui/src/app.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/keymap.rs:1-80 (KeyChord, KeyModifiers)
  - crates/scriptui-tui/src/terminal.rs:177-247 (crossterm_key_to_chord, write_osc52_clipboard)
  - crates/scriptui-tui/src/app.rs:237-286,399-445 (handle_key_event, run_event_loop)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - Do not alter internal EditorState clipboard behavior for Action::Copy/Action::Cut.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Pressing Super/Hyper/Meta shortcuts (e.g. Super+s or Super+Left) neither inserts characters nor triggers editor navigation, oversized copies do not emit unbounded OSC 52 terminal sequences, and Unix signal interruption (EINTR) in event::poll/event::read retries the event loop instead of exiting with an error.

Contract & Behavior:
  1. `crates/scriptui-tui/src/terminal.rs`:
     - In `crossterm_key_to_chord(event: KeyEvent) -> Option<KeyChord>`, return `None` whenever `event.modifiers.intersects(CrosstermKeyModifiers::SUPER | CrosstermKeyModifiers::HYPER | CrosstermKeyModifiers::META)`.
     - Define `pub const MAX_OSC52_BYTES: usize = 74_994;` (100 KiB base64-encoded limit). In `write_osc52_clipboard<W: Write>(writer: &mut W, text: &str) -> io::Result<()>`, if `text.len() > MAX_OSC52_BYTES`, return `Ok(())` without writing an OSC 52 escape sequence.
  2. `crates/scriptui-tui/src/app.rs`:
     - In `TuiApp::handle_key_event`, in the `KeyCode::Char(c)` fallback branch, require `!key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META)` before inserting text.
     - In `TuiApp::run_event_loop` (and via a testable helper `poll_and_read_event` or equivalent taking poll/read closures), treat `Err(err)` where `err.kind() == io::ErrorKind::Interrupted` from either `event::poll` or `event::read` as `Ok(None)` so the loop continues and checks `guard.should_terminate()` / `guard.handle_suspend_if_requested()`. Non-`Interrupted` errors still propagate.

Must still work:
  - Normal `Ctrl`, `Alt`, `Shift`, and unmodified keybindings and text input.
  - Normal-sized OSC 52 copy/cut sequences (`<= MAX_OSC52_BYTES`) and internal `EditorState` clipboard for all sizes.

Drops: none

Tests:
  - `crossterm_key_to_chord_ignores_super_hyper_meta`: `KeyEvent` with `SUPER`, `HYPER`, or `META` on `Char('s')` and `Left` -> `crossterm_key_to_chord` returns `None`.
  - `write_osc52_clipboard_skips_oversized_payload`: writing `MAX_OSC52_BYTES + 1` bytes of `'a'` -> `Ok(())` and output buffer remains empty, while `"hello"` still writes `\x1b]52;c;aGVsbG8=\x07`.
  - `super_modifier_key_does_not_insert_text`: `TuiApp::handle_key_event` with `Char('s')` + `KeyModifiers::SUPER` -> buffer text remains empty.
  - `eintr_retry_continues_on_interrupted`: helper returns `Ok(None)` when poll or read returns `ErrorKind::Interrupted`, and propagates `ErrorKind::BrokenPipe`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'MAX_OSC52_BYTES' crates/scriptui-tui/src/terminal.rs && grep -qE 'fn crossterm_key_to_chord_ignores_super_hyper_meta' crates/scriptui-tui/src/terminal.rs && grep -qE 'fn write_osc52_clipboard_skips_oversized_payload' crates/scriptui-tui/src/terminal.rs && grep -qE 'ErrorKind::Interrupted' crates/scriptui-tui/src/app.rs && grep -qE 'fn eintr_retry_continues_on_interrupted' crates/scriptui-tui/src/app.rs && grep -qE 'fn super_modifier_key_does_not_insert_text' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 13B: Find/Replace Key Capture & Focus/Match State Reset

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0026`

```text
[STEP_ID]
STEP 13B: Prevent unhandled navigation/Delete keys in Find/Replace from leaking to the editor, reset FindFocus::Query on open/close, and clear stale find matches across buffer switches.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/find_bar.rs
  - crates/scriptui-tui/src/app.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/find_bar.rs:108-210 (handle_find_bar_key)
  - crates/scriptui-tui/src/app.rs:190-287,289-340 (handle_key_event, handle_mouse_event)
  - crates/scriptui-core/src/editor.rs:480-560 (Action dispatch for OpenFind/OpenReplace/buffer switching)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.
  - Do not break F3/Shift+F3 navigation after closing Find/Replace with Esc.

[PRE_GATE]
grep -qE 'fn super_modifier_key_does_not_insert_text' crates/scriptui-tui/src/app.rs

[MUTATION_SPEC]
Goal: When the Find/Replace bar is open, Delete and cursor/page navigation keys (Left, Right, Up, Down, Home, End, PageUp, PageDown) never mutate or move the underlying editor; reopening Find/Replace starts with FindFocus::Query; and switching/closing/opening buffers clears stale find_replace.matches and active_match.

Contract & Behavior:
  1. `crates/scriptui-tui/src/find_bar.rs`:
     - In `handle_find_bar_key`, consume (`Ok(true)`) `KeyCode::Delete`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`, `PageDown` (and any non-hotkey unmodified/Shift character or Backspace) without mutating the editor.
     - Only allow explicit global/find-mode shortcuts (`Ctrl+F`, `Ctrl+H`, `F3`, `Shift+F3`, `Ctrl+Q`, `Ctrl+S`) to return `Ok(false)` so `app.rs` can handle them; all other unhandled keys while Find/Replace is open must not fall through to editor text insertion or cursor movement.
  2. `crates/scriptui-tui/src/app.rs`:
     - When `OverlayMode::Find` or `OverlayMode::Replace` is entered (via keybinding or palette) or exited, reset `self.find_bar.focus = FindFocus::Query`.
     - Whenever switching, closing, or opening a buffer (`NextBuffer`, `PrevBuffer`, `CloseBuffer`, `NewBuffer`, mouse tab click, or `OpenFilePrompt` completion), clear `self.state.find_replace.matches` and set `self.state.find_replace.active_match = None` (and reset `self.find_bar.focus = FindFocus::Query` if an overlay was open).

Must still work:
  - `Esc` closes Find/Replace, `Tab`/`BackTab` toggles focus in Replace mode, `Alt+C`/`Alt+W`/`Alt+R` toggle find options, `Enter`/`Shift+Enter`/`Ctrl+Enter`/`Alt+Enter` work as before.
  - `F3` / `Shift+F3` after closing Find/Replace still jumps to matches for the current buffer.

Drops: none

Tests:
  - `find_bar_consumes_navigation_and_delete_keys`: `handle_find_bar_key` with `OverlayMode::Find` and `Delete`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown` -> returns `Ok(true)` and leaves editor text/cursor untouched when dispatched through `TuiApp::handle_key_event`.
  - `find_bar_focus_and_matches_reset_across_buffer_switch`: focus `FindFocus::Replacement` in Replace mode with active matches on buffer 0, switch to buffer 1 via `Ctrl+Tab` (or `NextBuffer`), and reopen Replace -> `find_replace.matches` is empty, `active_match` is `None`, and `find_bar.focus` is `FindFocus::Query`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn find_bar_consumes_navigation_and_delete_keys' crates/scriptui-tui/src/find_bar.rs && grep -qE 'fn find_bar_focus_and_matches_reset_across_buffer_switch' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 13C: Core Symlink Containment, Locale Rename Rollback, Max File Size & Empty Insert Guard

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0032` (core), `BUG-0034`, `BUG-0036`

```text
[STEP_ID]
STEP 13C: Enforce canonical directory containment on theme and locale loads, roll back dic_dest if aff_dest rename fails in import_locale_files, cap DocumentBuffer::from_file at MAX_FILE_BYTES, and avoid pushing no-op BufferEdits on empty point-selection inserts.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/theme.rs
  - crates/scriptui-core/src/spell.rs
  - crates/scriptui-core/src/buffer.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/theme.rs:306-331 (resolve_theme)
  - crates/scriptui-core/src/spell.rs:205-247,405-470 (load_locale, import_locale_files)
  - crates/scriptui-core/src/buffer.rs:116-135,290-338 (from_file, apply_insert)
  - crates/scriptui-core/src/editor.rs:370-388 (insert_text calling UndoStack::record)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Symlinks escaping theme or dictionary search directories are rejected, failed `.aff` rename in `import_locale_files` leaves no partial `.dic` on disk, files larger than `MAX_FILE_BYTES` fail cleanly before reading into memory, and `apply_insert("")` on a point selection emits no `BufferEdit` so `UndoStack::is_at_saved_state()` stays accurate.

Contract & Behavior:
  1. `crates/scriptui-core/src/theme.rs`:
     - In `resolve_theme(name: &str, search_dirs: &[PathBuf])`, canonicalize each existing `dir` and `candidate` path; only load `candidate` if `canonical_candidate.starts_with(&canonical_dir)` (rejecting/skipping symlinks that escape `dir`).
  2. `crates/scriptui-core/src/spell.rs`:
     - In `SpellEngine::load_locale(locale: &str, search_dirs: &[PathBuf])`, canonicalize each `dir`, `aff_path`, and `dic_path`; only load if both canonical paths start with `canonical_dir`.
     - In `import_locale_files`, before overwriting `dic_dest`, record whether `dic_dest` already existed (and its prior bytes if so). If `fs::rename(&aff_tmp, &aff_dest)` fails, remove `aff_tmp` and restore `dic_dest` (write back prior bytes if it existed, or remove `dic_dest` if it did not) before returning `Err`.
  3. `crates/scriptui-core/src/buffer.rs`:
     - Define `pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;` (64 MiB). In `DocumentBuffer::from_file(path: &Path)`, check `fs::metadata(path)?.len() > MAX_FILE_BYTES` before `fs::read_to_string`, returning `ScriptuiError::Io` with `io::ErrorKind::InvalidData`.
     - In `DocumentBuffer::apply_insert(&mut self, text: &str)`, skip pushing a `BufferEdit` when `text.is_empty() && start == end`, so `apply_insert("")` on point selections returns an empty `Vec`.

Must still work:
  - Valid custom themes and locales inside their respective directories load normally.
  - Non-empty inserts and empty inserts replacing a non-empty selection (`start < end`) still record `BufferEdit` and mark `dirty = true`.

Drops: none

Tests:
  - `resolve_theme_rejects_symlink_escape` (in `theme.rs`, `#[cfg(unix)]`): symlink inside theme dir pointing outside -> `resolve_theme` does not load the outside file.
  - `load_locale_rejects_symlink_escape` (in `spell.rs`, `#[cfg(unix)]`): symlinked `.dic`/`.aff` pointing outside dict dir -> `load_locale` returns `Err`.
  - `import_locale_files_rolls_back_dic_on_aff_rename_failure` (in `spell.rs`): simulate failure on second rename (e.g. `aff_dest` is a non-empty directory) -> `import_locale_files` returns `Err` and `dic_dest` is absent (or restored to prior content).
  - `empty_insert_on_point_selection_returns_no_edits` (in `buffer.rs`): `DocumentBuffer::empty()` with `apply_insert("")` -> returns empty `Vec`, `dirty() == false`, `revision() == 0`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'MAX_FILE_BYTES' crates/scriptui-core/src/buffer.rs && grep -qE 'fn resolve_theme_rejects_symlink_escape' crates/scriptui-core/src/theme.rs && grep -qE 'fn load_locale_rejects_symlink_escape' crates/scriptui-core/src/spell.rs && grep -qE 'fn import_locale_files_rolls_back_dic_on_aff_rename_failure' crates/scriptui-core/src/spell.rs && grep -qE 'fn empty_insert_on_point_selection_returns_no_edits' crates/scriptui-core/src/buffer.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 13D: Multi-Cursor Preservation & `SelectionSet::normalize` Primary Remapping

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0029`

```text
[STEP_ID]
STEP 13D: Remap SelectionSet::primary across sort/merge in normalize(), and preserve all cursors in handle_indent, move_primary_selection, and ClearCurrentLine.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/selection.rs
  - crates/scriptui-core/src/editor.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/selection.rs:56-145 (SelectionSet, normalize)
  - crates/scriptui-core/src/editor.rs:843-990 (handle_insert_newline, handle_indent, move_primary_selection)
  - crates/scriptui-core/src/indent.rs:1-120 (indent_lines, dedent_lines)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qE 'fn empty_insert_on_point_selection_returns_no_edits' crates/scriptui-core/src/buffer.rs

[MUTATION_SPEC]
Goal: Sorting or merging ranges in SelectionSet::normalize() keeps `primary` pointing at the original primary range (or the merged range containing it), and indent/dedent, cursor movement, and ClearCurrentLine preserve multiple cursors instead of collapsing to primary.

Contract & Behavior:
  1. `crates/scriptui-core/src/selection.rs`:
     - In `SelectionSet::normalize(&mut self)`, tag each range with whether it was the `primary` element before sorting; after sorting and merging overlapping/adjacent ranges, set `self.primary` to the index of the merged range that absorbed the tagged primary element (falling back to `0` only if empty).
  2. `crates/scriptui-core/src/editor.rs`:
     - In `handle_indent(&mut self, indent: bool)`, collect the unique line ranges covered by all selections in `selection_before.ranges()`, indent/dedent each affected line once (in reverse or merged line-range order so offsets stay consistent), and shift every selection range in `selection_before` rather than collapsing to `primary`.
     - In `move_primary_selection` (and `Action::Move*` / `Action::Select*`), apply the motion to every selection in `doc.selection()` and rebuild `SelectionSet` preserving `primary` rather than collapsing multi-cursors to a single selection.
     - In `EnterOutcome::ClearCurrentLine` (inside `handle_insert_newline`), clear the current list-prefix line(s) for all cursors without collapsing multi-cursors (or apply per-cursor line clears in reverse offset order).

Must still work:
  - Single-cursor indent/dedent, cursor movement, selection extension, and `Enter` list continuation/clear.
  - `Escape` still collapses multi-cursors to primary when no overlay is open.

Drops: none

Tests:
  - `normalize_remaps_primary_across_sort_and_merge` (in `selection.rs`): build a `SelectionSet` where `primary` is at a higher offset that sorts earlier or merges with another range -> `set.primary()` returns the range containing that original primary selection.
  - `multi_cursor_preserved_across_indent_move_and_clear_line` (in `editor.rs`): with 2 cursors on different lines, `Action::Indent`, `Action::MoveRight`, and `Enter` on an empty bullet line (`- `) -> `selection().len() == 2` after each operation and both lines are updated.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn normalize_remaps_primary_across_sort_and_merge' crates/scriptui-core/src/selection.rs && grep -qE 'fn multi_cursor_preserved_across_indent_move_and_clear_line' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 13E: `PreviewMode::Expanded` Scroll Sync & `unicode_width` Table/Preview Layout

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0027`

```text
[STEP_ID]
STEP 13E: Measure preview text and table widths with unicode_width, shrink wide tables to fit narrow viewports, and map scroll_line to logical source line in PreviewMode::Expanded.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/preview.rs
  - crates/scriptui-tui/src/app.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/preview.rs:150-185,480-560,670-715,1050-1170 (table widths, wrapping, borders)
  - crates/scriptui-core/src/layout.rs:1-120 (WrappedLayout::compute)
  - crates/scriptui-tui/src/app.rs:116-155 (render_frame, editor_top_source_line)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qE 'fn find_bar_focus_and_matches_reset_across_buffer_switch' crates/scriptui-tui/src/app.rs

[MUTATION_SPEC]
Goal: CJK/emoji/wide characters in Markdown preview align and wrap by terminal display width instead of `chars().count()`, wide tables shrink to fit narrow preview widths, and `PreviewMode::Expanded` syncs preview scroll using the logical source line corresponding to visual `scroll_line`.

Contract & Behavior:
  1. `crates/scriptui-core/src/preview.rs`:
     - Replace `.chars().count()` width measurements (table cells, borders, labels, inline wrapping, list markers) with `unicode_width::UnicodeWidthStr::width` (treating `unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)` for per-char wrapping).
     - When rendering a table whose total column + border width exceeds the target `width`, shrink column widths proportionally (down to a minimum column width of 1) and wrap/truncate cell content to fit within `width`.
  2. `crates/scriptui-tui/src/app.rs`:
     - In `TuiApp::render_frame`, when `regions.editor` is `None` (i.e. `PreviewMode::Expanded`) and `regions.preview` is `Some(preview_rect)`, compute `editor_top_source_line` by mapping the active buffer's `scroll_line` through `WrappedLayout::compute` (using the last editor width or `preview_rect.width.max(1)` as fallback) instead of passing visual `scroll_line` directly as logical line.

Must still work:
  - ASCII Markdown tables, headings, lists, blockquotes, and code blocks render identically at standard widths.
  - `PreviewMode::Split` scroll sync continues to use the rendered `wrapped_layout`.

Drops: none

Tests:
  - `wide_table_shrinks_and_measures_display_width` (in `preview.rs`): render a table with wide CJK text (`"日本語テスト"`) and long column text at `width = 24` -> every rendered line's `UnicodeWidthStr::width` is `<= 24`.
  - `expanded_preview_scroll_sync_maps_visual_scroll_to_logical_line` (in `app.rs`): buffer with a long wrapped paragraph on line 0 followed by a heading on line 1, `scroll_line > 0` within the wrapped paragraph, and `PreviewMode::Expanded` -> `render_frame` maps `editor_top_source_line` to `0` (not `scroll_line`).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'UnicodeWidthStr' crates/scriptui-core/src/preview.rs && grep -qE 'fn wide_table_shrinks_and_measures_display_width' crates/scriptui-core/src/preview.rs && grep -qE 'fn expanded_preview_scroll_sync_maps_visual_scroll_to_logical_line' crates/scriptui-tui/src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

## Phase 14 Step Blocks

### STEP 14A: Find/Replace Global Shortcut Passthrough & Modal/Palette Modifier Filtering

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0037`, `BUG-0040`

```text
[STEP_ID]
STEP 14A: Pass global shortcuts through handle_find_bar_key instead of swallowing non-whitelisted keys, and filter SUPER/HYPER/META (and confirm-modal modifiers) in modals.rs and palette.rs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/find_bar.rs
  - crates/scriptui-tui/src/modals.rs
  - crates/scriptui-tui/src/palette.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/find_bar.rs:108-235,332-405 (handle_find_bar_key, find_bar_consumes_navigation_and_delete_keys)
  - crates/scriptui-tui/src/app.rs:344-379 (Action::NextBuffer/PrevBuffer/CloseBuffer/NewBuffer find-bar reset, SUPER/HYPER/META filter)
  - crates/scriptui-tui/src/modals.rs:198-217,260-285 (handle_confirm_modal_key, handle_file_prompt_key)
  - crates/scriptui-tui/src/palette.rs:285-330 (handle_palette_key)
  - crates/scriptui-core/src/keymap.rs:161-260 (Keymap::default_cua global shortcuts)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do not modify app.rs or keymap.rs.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: While Find/Replace is open, unmodified/Shift navigation and delete keys stay captured inside the find bar while all global shortcuts (including Ctrl+PageDown/Up, Ctrl+W/N/O, Ctrl+Shift+S/P, Ctrl+P, F1, Alt+P, Alt+Z) pass through to TuiApp::handle_key_event; meanwhile, OpenFilePrompt, SaveAsPrompt, CommandPalette, and ConfirmQuit/ConfirmCloseBuffer ignore SUPER/HYPER/META (and modified confirm) keys.

Contract & Behavior:
  1. `crates/scriptui-tui/src/find_bar.rs` (`handle_find_bar_key`):
     - Consume `Delete`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`, and `PageDown` only when `!key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META)` (so unmodified and `Shift`-modified navigation/delete keys do not leak into the editor buffer, while `Ctrl+PageDown`/`Ctrl+PageUp` pass through).
     - For `Alt+<char>` (when `key.modifiers.contains(KeyModifiers::ALT)` and `!key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META)`), consume only `'c' | 'w' | 'r'` (case-insensitive) to toggle case-sensitivity, whole-word, and regex. Return `Ok(false)` for other `Alt` chords (such as `Alt+P`, `Alt+Z`).
     - Return `Ok(false)` for unhandled keys (do not swallow unmatched keys with `_ => Ok(true)`), so global shortcuts (`Ctrl+F`, `Ctrl+H`, `Ctrl+Q`, `Ctrl+S`, `Ctrl+Shift+S`, `Ctrl+N`, `Ctrl+O`, `Ctrl+W`, `Ctrl+P`, `Ctrl+Shift+P`, `Ctrl+PageDown`, `Ctrl+PageUp`, `F1`, `F3`, `Shift+F3`, `Alt+P`, `Alt+Z`) pass through to `TuiApp::handle_key_event`.
  2. `crates/scriptui-tui/src/modals.rs`:
     - In `handle_file_prompt_key`, filter `KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META` on `KeyCode::Char(c)` so `Super`/`Hyper`/`Meta`-modified characters are not appended to `state.file_prompt_input`.
     - In `handle_confirm_modal_key`, accept `KeyCode::Char('y' | 'Y' | 'n' | 'N')` only when `!key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META)`, and accept `KeyCode::Enter` / `KeyCode::Esc` only when `key.modifiers.is_empty()`. Return `false` for modified keys (e.g. `Ctrl+Y`, `Super+Y`, `Alt+N`, `Ctrl+Enter`).
  3. `crates/scriptui-tui/src/palette.rs`:
     - In `handle_palette_key`, filter `KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META` on `KeyCode::Char(c)` so `Super`/`Hyper`/`Meta`-modified characters are not appended to `state.palette.query`.

Must still work:
  - `find_bar_consumes_navigation_and_delete_keys` (unmodified Delete/Left/Right/Up/Down/Home/End/PageUp/PageDown remain consumed and never mutate the editor buffer or selection).
  - `find_bar_typing_toggles_replace_and_alt_enter_multicursor` (typing, `Alt+C`/`Alt+W`/`Alt+R`, `Tab`, `Enter`, `Shift+Enter`, `Ctrl+Enter`, `Alt+Enter`, `Esc`).
  - Existing modal and command palette typing, uppercase (`Shift+Char`), `Enter`, `Backspace`, and `Esc` tests.

Drops: none

Tests:
  - `find_bar_passes_through_global_shortcuts_and_resets_on_ctrl_pagedown` (in `find_bar.rs`): verify `handle_find_bar_key` returns `Ok(false)` for `Ctrl+PageDown`, `Ctrl+PageUp`, `Ctrl+W`, `Ctrl+N`, `Ctrl+O`, `Ctrl+Shift+S`, `Ctrl+P`, `Ctrl+Shift+P`, `F1`, `Alt+P`, and `Alt+Z`; then on a `TuiApp` with 2 buffers, `OverlayMode::Replace`, `bar_state.focus = FindFocus::Replacement`, and populated `find_replace.matches`, dispatch `Ctrl+PageDown` through `TuiApp::handle_key_event` -> active buffer switches, `find_replace.matches` is empty, `find_replace.active_match` is `None`, and `find_bar.focus == FindFocus::Query`.
  - `modals_ignore_super_hyper_meta_and_modified_confirm_keys` (in `modals.rs`): in `OverlayMode::OpenFilePrompt`, `Super+s`, `Hyper+a`, and `Meta+b` leave `file_prompt_input` empty while plain `'a'` appends `'a'`; in `OverlayMode::ConfirmQuit`, `Ctrl+Y`, `Super+Y`, `Alt+N`, and `Ctrl+Enter` return `false` and leave `overlay == OverlayMode::ConfirmQuit`, while plain `'n'` returns `true` and dismisses the modal.
  - `palette_ignores_super_hyper_meta_char_input` (in `palette.rs`): in `OverlayMode::CommandPalette`, `Super+s`, `Hyper+x`, and `Meta+z` leave `palette.query` empty while plain `'t'` appends `'t'`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn find_bar_passes_through_global_shortcuts_and_resets_on_ctrl_pagedown' crates/scriptui-tui/src/find_bar.rs && grep -qE 'fn modals_ignore_super_hyper_meta_and_modified_confirm_keys' crates/scriptui-tui/src/modals.rs && grep -qE 'fn palette_ignores_super_hyper_meta_char_input' crates/scriptui-tui/src/palette.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 14B: Preserve `SelectionSet::primary_index` in `DocumentBuffer` Edits & Test `MAX_FILE_BYTES`

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0038`, `TASK-0076`

```text
[STEP_ID]
STEP 14B: Preserve SelectionSet::primary_index in DocumentBuffer::apply_insert and apply_delete, and add a unit test for DocumentBuffer::from_file MAX_FILE_BYTES rejection.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/buffer.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/buffer.rs:117-145,305-413 (from_file, MAX_FILE_BYTES, apply_insert, apply_delete)
  - crates/scriptui-core/src/selection.rs:64-95 (SelectionSet::from_ranges_with_primary, primary_index)
  - crates/scriptui-core/Cargo.toml:24-25 (tempfile dev-dependency)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do not change MAX_FILE_BYTES value or SelectionSet API.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Multi-cursor inserts and deletes in `DocumentBuffer` preserve which cursor is primary (`primary_index`) instead of resetting it to `0`, and `DocumentBuffer::from_file` rejecting files larger than `MAX_FILE_BYTES` is covered by an automated unit test.

Contract & Behavior:
  1. `crates/scriptui-core/src/buffer.rs`:
     - In `DocumentBuffer::apply_insert` and `DocumentBuffer::apply_delete`, capture `self.selection.primary_index()` before rebuilding `self.selection` and construct the new selection via `SelectionSet::from_ranges_with_primary(new_ranges, primary_index)` so `primary_index` tracks the same cursor (or the merged range containing it after `normalize`).
     - Keep `self.selection.clamp(self.len_chars())`, empty-insert no-op behavior, `dirty`, and `revision` semantics unchanged.

Must still work:
  - Single-cursor and multi-cursor insert/delete offsets, undo/redo inverse edits, and `empty_insert_does_not_dirty_or_bump_revision`.
  - `DocumentBuffer::from_file` reading normal files up to `MAX_FILE_BYTES`.

Drops: none

Tests:
  - `apply_insert_and_delete_preserve_nonzero_primary_index` (in `buffer.rs`): build a `DocumentBuffer` from `"alpha beta gamma"` with `SelectionSet::from_ranges_with_primary(vec![Selection::point(5), Selection::point(10)], 1)` -> after `apply_insert("!")`, `selection().primary_index() == 1` and `selection().primary() == Selection::point(12)`; after `apply_delete(true)`, `selection().primary_index() == 1` and `selection().primary() == Selection::point(10)`.
  - `from_file_rejects_files_exceeding_max_file_bytes` (in `buffer.rs`): create a temporary sparse file (`tempfile::NamedTempFile`) with `set_len(MAX_FILE_BYTES + 1)` -> `DocumentBuffer::from_file(tmp.path())` returns `Err(ScriptuiError::Io { source, .. })` where `source.kind() == std::io::ErrorKind::InvalidData` and `source.to_string()` contains `"exceeds maximum allowed size"`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'from_ranges_with_primary' crates/scriptui-core/src/buffer.rs && grep -qE 'fn apply_insert_and_delete_preserve_nonzero_primary_index' crates/scriptui-core/src/buffer.rs && grep -qE 'fn from_file_rejects_files_exceeding_max_file_bytes' crates/scriptui-core/src/buffer.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 14C: Mixed-Cursor `handle_insert_newline` Per-Cursor Handling

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0039`

```text
[STEP_ID]
STEP 14C: Evaluate EnterOutcome per cursor line in EditorState::handle_insert_newline so mixed ClearCurrentLine and Insert cursors both apply in one atomic undoable transaction.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/editor.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/editor.rs:844-906 (handle_insert_newline)
  - crates/scriptui-core/src/indent.rs (compute_enter_indent, EnterOutcome)
  - crates/scriptui-core/src/selection.rs:64-95 (SelectionSet::from_ranges_with_primary)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do not modify indent.rs or buffer.rs.

[PRE_GATE]
grep -qE 'from_ranges_with_primary' crates/scriptui-core/src/buffer.rs

[MUTATION_SPEC]
Goal: Pressing Enter with multiple cursors where some cursors sit on empty list/quote markers (`EnterOutcome::ClearCurrentLine`) and others sit on normal text or non-empty list items (`EnterOutcome::Insert`) applies each cursor's own `EnterOutcome` in one undoable transaction while preserving all cursors and `primary_index`.

Contract & Behavior:
  1. `crates/scriptui-core/src/editor.rs` (`EditorState::handle_insert_newline`):
     - Compute `EnterOutcome` (`compute_enter_indent`) for each cursor's line (respecting `gfm.is_line_in_code_block(line)` and `self.config.indent`).
     - Apply all per-cursor edits in reverse document order so earlier offsets remain valid:
       - For `EnterOutcome::ClearCurrentLine`: clear that line's full content (`line_start..line_end` -> `""`), coalescing multiple cursors on the same cleared line so the line is cleared at most once, and place resulting caret(s) at `line_start`.
       - For `EnterOutcome::Insert(text)`: replace that cursor's selection span (`sel.start()..sel.end()`) with `text` and place the resulting caret at `shifted_start + text.chars().count()`.
     - Rebuild `SelectionSet` via `SelectionSet::from_ranges_with_primary(..., selection_before.primary_index())`, clamp to `doc.len_chars()`, and record all edits in a single `EditTransaction` in `active.history` so one `Action::Undo` restores the exact pre-Enter text and `SelectionSet`.

Must still work:
  - Single-cursor auto-indent, Markdown list/task/quote continuation, empty list-marker clearing, and fenced code-block Enter behavior.
  - `multi_cursor_preserved_across_indent_move_and_clear_line`.
  - Homogeneous multi-cursor Enter (all `Insert` or all `ClearCurrentLine`).

Drops: none

Tests:
  - `mixed_multicursor_enter_handles_clear_and_insert_lines_together` (in `editor.rs`):
    - Case 1 (primary on empty bullet): text `"- \nhello"`, cursors at `2` (primary, index `0`) and `8` -> `Action::InsertNewline` produces `"\nhello\n"`, `selection().len() == 2`, `selection().primary_index() == 0`; `Action::Undo` restores `"- \nhello"` and the original `SelectionSet`.
    - Case 2 (primary on normal text): text `"- \nhello"`, cursors at `2` and `8` (primary, index `1`) -> `Action::InsertNewline` produces `"\nhello\n"`, `selection().len() == 2`, `selection().primary_index() == 1`; `Action::Undo` restores `"- \nhello"` and the original `SelectionSet`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn mixed_multicursor_enter_handles_clear_and_insert_lines_together' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 14D: Full-Suite GitHub Actions CI Workflow (`.github/workflows/ci.yml`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 14D: Add GitHub Actions CI workflow running rustfmt, clippy, full workspace tests (Linux + macOS), cargo deny check, and locked release build on master pushes and PRs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - .github/workflows/ci.yml (NEW)
READ_ONLY_REFS:
  - Cargo.toml:1-20 (workspace members, rust-version, release profile)
  - deny.toml:1-38 (cargo-deny configuration and target triples)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do NOT create .github/workflows/release.yml in this step. Do not hardcode secrets, tokens, or personal paths.

[PRE_GATE]
test -f deny.toml

[MUTATION_SPEC]
Goal: Every push to `master` and pull request targeting `master` runs the full CI verification suite (formatting, clippy, full workspace test suite on Linux and macOS, `cargo deny check`, and locked release build).

Contract & Behavior:
  1. `.github/workflows/ci.yml` (new):
     - `name: CI`
     - Triggers: `push` on `branches: [master]` and `pull_request` on `branches: [master]`.
     - Top-level `permissions:` with `contents: read` only.
     - Job `check` (`runs-on: ubuntu-latest`):
       - `actions/checkout@v4`
       - `dtolnay/rust-toolchain@stable` with `components: rustfmt, clippy`
       - `Swatinem/rust-cache@v2`
       - Run `cargo fmt --all -- --check`
       - Run `cargo clippy --workspace --all-targets -- -D warnings`
       - Run `cargo test --workspace`
       - Run `EmbarkStudios/cargo-deny-action@v2` with `command: check`
       - Run `cargo build --release --locked`
     - Job `macos-test` (`runs-on: macos-14`):
       - `actions/checkout@v4`
       - `dtolnay/rust-toolchain@stable`
       - `Swatinem/rust-cache@v2`
       - Run `cargo test --workspace`

Must still work:
  - Local workspace build, clippy, formatting, and all unit/integration tests unchanged.

Drops: none

Tests:
  - None in Rust (workflow YAML step). `POST_GATE` parses `.github/workflows/ci.yml` with PyYAML and asserts the `name`, `check` job, `macos-test` job, and required CI commands are present.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && python3 -c 'import yaml; d = yaml.safe_load(open(".github/workflows/ci.yml")); assert d["name"] == "CI" and "check" in d["jobs"] and "macos-test" in d["jobs"]' && grep -qE 'cargo fmt --all -- --check' .github/workflows/ci.yml && grep -qE 'cargo clippy --workspace --all-targets -- -D warnings' .github/workflows/ci.yml && grep -qE 'cargo test --workspace' .github/workflows/ci.yml && grep -qE 'cargo-deny-action' .github/workflows/ci.yml && grep -qE 'cargo build --release --locked' .github/workflows/ci.yml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```


### STEP 15A: Restrict Find/Replace Bar Passthrough to Non-Editing Global Shortcuts

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0041`

```text
[STEP_ID]
STEP 15A: Trap document-editing, history, selection, multi-cursor, spell-quick-fix, and unbound chords in `handle_find_bar_key` while continuing to pass through non-editing global shortcuts.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-tui/src/find_bar.rs
READ_ONLY_REFS:
  - crates/scriptui-tui/src/app.rs:260-380 (TuiApp::handle_key_event overlay dispatch and keymap fallback)
  - crates/scriptui-tui/src/terminal.rs:175-210 (crossterm_key_to_chord)
  - crates/scriptui-core/src/keymap.rs:99-365 (Action enum and Keymap::default_cua)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do not modify TuiApp::handle_key_event or Keymap::default_cua.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Pressing document-editing or selection shortcuts (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, `Ctrl+X`, `Ctrl+C`, `Ctrl+V`, `Ctrl+A`, `Ctrl+Shift+L`, `Ctrl+.`) or unbound chords while `OverlayMode::Find` or `OverlayMode::Replace` is open never mutates the underlying document, selection, clipboard, or overlay, while non-editing global shortcuts still pass through to `TuiApp`.

Contract & Behavior:
  1. `crates/scriptui-tui/src/find_bar.rs` (`handle_find_bar_key`):
     - Preserve direct handling of Find/Replace bar controls: `Esc`, `Tab`/`BackTab`, `Enter` variants (`Alt+Enter`, `Ctrl+Enter`, `Shift+Enter`, plain `Enter`), `Alt+C`/`Alt+W`/`Alt+R` toggles (without `CONTROL|SUPER|HYPER|META`), `Backspace`, printable characters without `CONTROL|ALT|SUPER|HYPER|META`, and plain navigation/delete keys.
     - For any other key (including `Alt+<char>` other than `c`/`w`/`r` and the fallback arm), look up `crate::terminal::crossterm_key_to_chord(key).and_then(|c| state.keymap.lookup(c))`.
     - Return `Ok(false)` (pass through to `TuiApp`) ONLY when the resolved `Action` is a non-editing global/window/view/buffer/search action:
       `Action::Save`, `Action::SaveAs`, `Action::OpenFile`, `Action::NewBuffer`, `Action::CloseBuffer`, `Action::NextBuffer`, `Action::PrevBuffer`, `Action::Quit`, `Action::OpenFind`, `Action::OpenReplace`, `Action::FindNext`, `Action::FindPrev`, `Action::OpenCommandPalette`, `Action::TogglePreview`, `Action::ExpandPreview`, `Action::ToggleSoftWrap`, `Action::ToggleLineNumbers`.
     - Return `Ok(true)` (consume/trap) for all other keys: document-editing/history/selection/navigation/multi-cursor/spell actions (`Undo`, `Redo`, `Cut`, `Copy`, `Paste`, `SelectAll`, `SelectAllOccurrences`, `OpenSpellQuickFix`, `Select*`, `Move*`, `Page*`, `InsertNewline`, `Backspace`, `Delete`, `Indent`, `Dedent`, `Escape`), `SUPER`/`HYPER`/`META` chords, and any unbound chord.

Must still work:
  - Find/Replace query and replacement typing, `Backspace`, `Tab`/`BackTab`, `Esc`, `Enter` variants, and `Alt+C`/`Alt+W`/`Alt+R`.
  - Existing tests `find_bar_consumes_navigation_and_delete_keys` and `find_bar_passes_through_global_shortcuts_and_resets_on_ctrl_pagedown` (all non-editing global shortcuts still return `Ok(false)` and trigger their `TuiApp` actions).

Drops: none

Tests:
  - `find_bar_traps_document_editing_shortcuts_end_to_end`: Create a `TuiApp`, insert `"alpha beta"` then `" gamma"` (so undo history is non-empty), select text or position the caret, open `OverlayMode::Find` (`Ctrl+F`), and dispatch `Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`, `Ctrl+Shift+L`, `Ctrl+.`, and unbound `Alt+X` through both `handle_find_bar_key` (asserting `Ok(true)`) and `TuiApp::handle_key_event` (asserting document text, selection, OSC 52 clipboard output buffer, and `OverlayMode::Find` are unchanged); repeat with `OverlayMode::Replace` (`Ctrl+H`) active.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn find_bar_traps_document_editing_shortcuts_end_to_end' crates/scriptui-tui/src/find_bar.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 15B: Prevent Overlapping Edits in Multi-Cursor `handle_insert_newline`

* **Recommended Command:** `/goal`
* **Advances:** `BUG-0042`

```text
[STEP_ID]
STEP 15B: Guard `EditorState::handle_insert_newline` so `ClearCurrentLine` never overlaps another selection or non-empty selection and `planned_edits` are validated disjoint before any buffer mutation.

[SCOPE_LOCK]
ALLOWED_FILES:
  - crates/scriptui-core/src/editor.rs
READ_ONLY_REFS:
  - crates/scriptui-core/src/indent.rs:1-80 (EnterOutcome, compute_enter_indent)
  - crates/scriptui-core/src/selection.rs:1-140 (Selection, SelectionSet::from_ranges_with_primary)
  - crates/scriptui-core/src/buffer.rs:160-240 (apply_raw_edit)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do not change EnterOutcome or compute_enter_indent semantics.

[PRE_GATE]
true

[MUTATION_SPEC]
Goal: Multi-cursor `Action::InsertNewline` never returns partial unrecorded `OutOfBounds` errors or corrupts surrounding text when a selection starts, ends, or spans inside a line whose `compute_enter_indent` returns `EnterOutcome::ClearCurrentLine`.

Contract & Behavior:
  1. `crates/scriptui-core/src/editor.rs` (`EditorState::handle_insert_newline`):
     - When `compute_enter_indent` returns `EnterOutcome::ClearCurrentLine` for a cursor on `line` (`[line_start, line_end)`), emit a `ClearCurrentLine` edit `(line_start, line_end, "")` ONLY when:
       (a) that cursor's selection is empty (`sel.is_empty()`), AND
       (b) no other selection in `selection_before.ranges()` overlaps `[line_start, line_end)` (strictly: `other_start < line_end && other_end > line_start`).
     - Otherwise (when the cursor itself is a non-empty selection, or another selection overlaps `[line_start, line_end)`), fall back to inserting `"\n"` at `(sel_start, sel_end)` for that cursor so all planned edits remain strictly disjoint.
     - After sorting `planned_edits` by `(start, end)` and BEFORE calling `active.document.apply_raw_edit`, verify every edit satisfies `start <= end && end <= len_chars` and consecutive edits satisfy `prev.end <= next.start`; if violated, return `Err(ScriptuiError::OutOfBounds { .. })` without mutating `active.document`.
     - Preserve caret-offset shifting, `SelectionSet::from_ranges_with_primary(..., selection_before.primary_index())`, and single atomic `EditTransaction` undo/redo recording.

Must still work:
  - Single-cursor and multi-cursor `Action::InsertNewline` on regular lines, indented/bullet/quote continuation lines, multiple point cursors on the same empty bullet line, and non-overlapping mixed `ClearCurrentLine` + `Insert` cursors (including `insert_newline_multicursor_clears_empty_bullet_and_inserts_on_other_line`).
  - `Action::Undo` and `Action::Redo` restore exact document text and `SelectionSet` (including `primary_index`).

Drops: none

Tests:
  - `insert_newline_multicursor_overlapping_clear_line_and_selection_is_atomic`:
    - Case A: `"hello\n- "` with `[Selection::new(7, 2), Selection::point(8)]` (`primary_index = 0`) -> `dispatch(Action::InsertNewline)` returns `Ok(())`, buffer becomes `"he\n \n"`, and `dispatch(Action::Undo)` restores `"hello\n- "` with `[Selection::new(7, 2), Selection::point(8)]`.
    - Case B: `"- \nhello"` with `[Selection::point(0), Selection::new(1, 5)]` (`primary_index = 0`) -> `dispatch(Action::InsertNewline)` returns `Ok(())`, buffer becomes `"\n-\nllo"` with point selections at `1` and `3`, and `dispatch(Action::Undo)` restores `"- \nhello"` with `[Selection::point(0), Selection::new(1, 5)]`.
    - Case C: `"hello\n- "` with single non-empty selection `[Selection::new(2, 8)]` -> `dispatch(Action::InsertNewline)` returns `Ok(())`, buffer becomes `"he\n"`, and `dispatch(Action::Undo)` restores `"hello\n- "`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && grep -qE 'fn insert_newline_multicursor_overlapping_clear_line_and_selection_is_atomic' crates/scriptui-core/src/editor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 15C: Tag-Triggered and Dry-Run GitHub Actions Release Workflow (`.github/workflows/release.yml`)

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 15C: Add `.github/workflows/release.yml` building Linux x86_64 and macOS arm64 tarballs with SHA256 checksums on `v*.*.*` tags (publishing GitHub Release) and on `workflow_dispatch` (dry-run artifact upload without publishing).

[SCOPE_LOCK]
ALLOWED_FILES:
  - .github/workflows/release.yml (NEW)
READ_ONLY_REFS:
  - .github/workflows/ci.yml:1-41 (existing GitHub Actions conventions and action versions)
  - Cargo.toml:1-20 (workspace metadata and release profile)
  - crates/scriptui-tui/Cargo.toml:1-27 (binary name `scriptui`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Do not modify `.github/workflows/ci.yml`, `CHANGELOG.md`, or `Cargo.toml`. Do not hardcode secrets, tokens, or personal paths.

[PRE_GATE]
test -f .github/workflows/ci.yml && test -f THIRD_PARTY_LICENSES.md && test -f LICENSE

[MUTATION_SPEC]
Goal: Pushing a `v*.*.*` tag builds and packages `scriptui` release tarballs and `.sha256` files for `x86_64-unknown-linux-gnu` and `aarch64-apple-darwin` and publishes them to a GitHub Release, while triggering `workflow_dispatch` runs the full build and packaging matrix and uploads workflow artifacts without creating a GitHub Release.

Contract & Behavior:
  1. `.github/workflows/release.yml` (new):
     - `name: Release`
     - Triggers:
       - `push` with `tags: ["v*.*.*"]`
       - `workflow_dispatch:` (no required inputs)
     - Top-level `permissions:` with `contents: read` only.
     - Job `build`:
       - `strategy.fail-fast: false`
       - `strategy.matrix.include` with exactly two entries:
         - `os: ubuntu-latest`, `target: x86_64-unknown-linux-gnu`
         - `os: macos-14`, `target: aarch64-apple-darwin`
       - `runs-on: ${{ matrix.os }}`
       - Steps:
         - `actions/checkout@v4`
         - `dtolnay/rust-toolchain@stable` with `targets: ${{ matrix.target }}`
         - `Swatinem/rust-cache@v2`
         - Build locked release binary: `cargo build --release --locked --target ${{ matrix.target }}`
         - Package `scriptui-${VERSION}-${{ matrix.target }}.tar.gz` (using `${GITHUB_REF_NAME}` on tag push and `dry-run-${GITHUB_SHA::7}` on `workflow_dispatch`) containing the `scriptui` binary (`target/${{ matrix.target }}/release/scriptui`), `README.md`, `LICENSE`, and `THIRD_PARTY_LICENSES.md`, and generate `scriptui-${VERSION}-${{ matrix.target }}.tar.gz.sha256` (supporting both `sha256sum` on Linux and `shasum -a 256` on macOS).
         - Upload the `.tar.gz` and `.tar.gz.sha256` files via `actions/upload-artifact@v4`.
     - Job `publish`:
       - `needs: [build]`
       - `if: startsWith(github.ref, 'refs/tags/v')`
       - `runs-on: ubuntu-latest`
       - `permissions:` with `contents: write`
       - Steps:
         - Download all built artifacts via `actions/download-artifact@v4` (with `merge-multiple: true` into a staging directory such as `dist`).
         - Publish the GitHub Release via `softprops/action-gh-release@v2` attaching `dist/*.tar.gz` and `dist/*.tar.gz.sha256` with `generate_release_notes: true`.

Must still work:
  - `.github/workflows/ci.yml` unchanged; local workspace build, clippy, formatting, and all unit/integration tests unchanged.

Drops: none

Tests:
  - None in Rust (workflow YAML step). `POST_GATE` parses `.github/workflows/release.yml` with PyYAML and asserts `name == "Release"`, triggers (`push.tags` contains `v*.*.*` and `workflow_dispatch` present), exact two-target `build` matrix (`x86_64-unknown-linux-gnu` on `ubuntu-latest`, `aarch64-apple-darwin` on `macos-14`), `publish` job tag-only `if` condition (`refs/tags/v`) and `contents: write` permission, and required build/release steps.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && python3 -c 'import yaml; d = yaml.safe_load(open(".github/workflows/release.yml")); on = d.get("on", d.get(True)); assert d["name"] == "Release" and "workflow_dispatch" in on and "v*.*.*" in on["push"]["tags"]; targets = {m["target"]: m["os"] for m in d["jobs"]["build"]["strategy"]["matrix"]["include"]}; assert targets == {"x86_64-unknown-linux-gnu": "ubuntu-latest", "aarch64-apple-darwin": "macos-14"}; pub = d["jobs"]["publish"]; assert "refs/tags/v" in pub["if"] and pub["permissions"]["contents"] == "write"' && grep -qE 'cargo build --release --locked --target' .github/workflows/release.yml && grep -qE 'action-gh-release' .github/workflows/release.yml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```
