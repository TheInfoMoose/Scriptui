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

**Phase goal (Phase 9):** Fix the two open functional bugs before functional verification (`Phase 10`): preview markdown soft-break rendering in block quotes, lists, and paragraphs (`BUG-0012`, `STEP 9A`), and bundled `en_US` Hunspell dictionary coverage for contractions and standard English vocabulary (`BUG-0013`, `STEP 9B`).

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 9A** | `/goal` | preview | `crates/scriptui-core/src/preview.rs` | — |
| 2 | **STEP 9B** | `/goal` | spell | `crates/scriptui-core/assets/dicts/en_US.aff`, `crates/scriptui-core/assets/dicts/en_US.dic`, `crates/scriptui-core/src/spell.rs` | — |

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

- **OQ-1 (Phase 13):** The old STEP 8C POST_GATE runs `python3 -c 'import yaml'`. PyYAML may not be on the Orrery host. Confirm it's installed, or use a gate with no extra package (`actionlint` if present). Waits on: user.
- **OQ-2 (Phase 13):** How to exercise `release.yml` before the real tag: push a prerelease tag (`v0.1.0-rc.1`) and delete the release afterward, or add a `workflow_dispatch` trigger that builds without publishing. Waits on: user.
- **OQ-3 (Phase 10):** Which OS + terminal emulator pairs does the `[USER]` checklist run on (OSC 52 and mouse behavior differ by terminal)? Waits on: user.

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.
_None — all tracked tasks and bugs are closed._

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
