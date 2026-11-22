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

**Phase goal (Phase 8):** Local, gate-verifiable `v0.1.0` release prep: add MIT `LICENSE`, complete Cargo workspace release metadata and `[profile.release]` tuning with verified release binary `--version` output (`STEP 8A`), and author `CHANGELOG.md` and update `README.md` with installation/release/license sections (`STEP 8B`). GitHub workflows moved to Phase 9.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 8A** | `/goal` | release | `Cargo.toml`, `crates/scriptui-core/Cargo.toml`, `crates/scriptui-tui/Cargo.toml`, `LICENSE` | — |
| 2 | **STEP 8B** | `/goal` | docs | `CHANGELOG.md`, `README.md` | STEP 8A |
| 3 | **STEP 8C** | `/goal` | ci | `.github/workflows/ci.yml`, `.github/workflows/release.yml` | STEP 8B |

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

### Future phases

**Decisions (2026-11-22):**
- Phase 8 cascaded (8A POST_GATE fault → 8B/8C PRE_GATEs failed); 8C moves out so Phase 8 is local-only release prep (8A + 8B).
- Release targets stay `x86_64-unknown-linux-gnu` + `aarch64-apple-darwin`; no Windows or macOS x86_64 for v0.1.0.
- Functional verification and a full app audit run **before** the `v0.1.0` tag, so the tag ships an audited, verified build.
- Because the tag now lands several phases after STEP 8B, the CHANGELOG release date is re-stamped in the release-cut phase.

**Phase 9 — CI & release workflows (risky: only verifiable on GitHub).**
- Goal: GitHub Actions CI on `master` push/PR (fmt, clippy, test, release build) and a tag-triggered release workflow producing Linux x86_64 + macOS arm64 tarballs with SHA256 files.
- Done: both workflow files exist and pass a local syntax gate; `[USER]` pushes `master` and CI goes green; the release workflow is exercised once per OQ-2 before the real tag.
- Steps: 9A CI workflow (`.github/workflows/ci.yml`); 9B tag-triggered release workflow (`.github/workflows/release.yml`); 9C `[USER]` push `master`, confirm CI green, run the release dry run.
- Carries over deferred STEP 8C / TASK-0044. `/phase-plan 8` removes the 8C row from Up Next; `/phase-plan 9` replaces the old 8C block (its `python3 yaml` POST_GATE is OQ-1).

**Phase 10 — Functional verification (routine).**
- Goal: prove every user-facing behavior the README documents actually works, automated where possible and by hand where a real terminal is required.
- Done: a coverage matrix in `docs/` maps every README CLI flag, config field, keybinding, mouse action, theme and spellcheck flow to an automated test or a `[USER]` checklist row; automated gaps are closed; the checklist passes on Linux and macOS (OQ-3); every failure is a neb bug.
- Steps: 10A core-engine coverage gaps (`scriptui-core` tests); 10B TUI coverage gaps (`TestBackend` integration tests: keybindings, mouse, modals, preview, find/replace); 10C CLI/config/theme/dict-import coverage gaps; 10D `[USER]` real-terminal checklist run (raw mode restore, bracketed paste, OSC 52, mouse, resize).

**Phase 11 — Full app audit (read-only review + tooling).**
- Goal: a whole-codebase audit before release: security/robustness (file I/O and paths, theme/dict import, regex input, panics/`unwrap`s, terminal restore on panic), correctness (Unicode, large files, undo/redo edge cases), performance, dependency licenses/advisories, and code quality.
- Done: an audit report in `docs/` with each finding rated; every finding filed as a pending neb item; `cargo deny` (advisories + licenses) runs clean or with documented exceptions.
- Steps: 11A `[REVIEW]` fresh Claude session writes the audit report and files neb items; 11B add `deny.toml` and wire `cargo deny check` into CI.

**Phase 12 — Audit and verification fixes (risky: scope set by findings).**
- Goal: fix the neb bugs from Phases 10–11 picked at the Phase 11 review.
- Done: every picked item closed with a test that fails before the fix; unpicked items stay pending with a target version.
- Steps: staged by `/phase-plan 12` from the picked neb items (one step per fix, 1–3 files each).

**Phase 13 — v0.1.0 release cut.**
- Goal: ship `v0.1.0`.
- Done: CHANGELOG date matches the tag date; `master` pushed; annotated tag `v0.1.0` pushed; GitHub release has both tarballs and `.sha256` files.
- Steps: 13A re-stamp `CHANGELOG.md` `[0.1.0]` date and add any Phase 10–12 fixes under it; 13B `[USER]` push `master`, `git tag -a v0.1.0 -m "ScripTUI v0.1.0"`, push the tag, check the release assets.

### Open Questions

- **OQ-1 (Phase 9):** The old STEP 8C POST_GATE runs `python3 -c 'import yaml'`. PyYAML may not be on the Orrery host. Confirm it's installed, or use a gate with no extra package (`actionlint` if present). Waits on: user.
- **OQ-2 (Phase 9):** How to exercise `release.yml` before the real tag: push a prerelease tag (`v0.1.0-rc.1`) and delete the release afterward, or add a `workflow_dispatch` trigger that builds without publishing. Waits on: user.
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
