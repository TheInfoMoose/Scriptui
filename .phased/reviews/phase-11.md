# Phase 11 review

Range: `9389f4d21b6db7e0500937109bd50f73486ad91a..4d524fe86fb4833219df1c82cb781e1a1ca03658`

## Independent pass

### Diff & Working-Tree Inventory
- Committed code files changed in `9389f4d..4d524fe` (excluding `.tracker/` and `.phased/`): **none (0 files)**. No Rust source, manifest, script, or test files were modified, so there are zero code regressions in this range.
- Untracked working-tree files present at review time:
  - `docs/codebase-audit-report.md` (314 lines, 49,351 bytes — audit report)
  - `.agents/teamwork/` (agent workspace directory)

### Findings

1. **`docs/codebase-audit-report.md:3, 78` — Internal host path leaked in documentation (`new defect`)**
   - **Trigger:** Inspecting or publishing `docs/codebase-audit-report.md`.
   - **Effect:** Lines 3 and 78 embed the literal local mount path `/srv/devshare/Scriptui` instead of a repository-relative reference or placeholder.

2. **`.gitignore:1-5` & working tree (`docs/codebase-audit-report.md`, `.agents/teamwork/`) — Uncommitted deliverable and untracked `.agents/` workspace pollution (`new defect`)**
   - **Trigger:** Checking repository state (`git status --short`) after the phase run.
   - **Effect:** `docs/codebase-audit-report.md` is untracked (absent from `HEAD`), and `.agents/teamwork/` remains untracked in the repository root with no `.agents/` entry in `.gitignore`, risking accidental commits or dirty-tree failures in subsequent phases.

3. **`crates/scriptui-core/src/indent.rs:83` — Unchecked UTF-8 byte slice `&trimmed[1..6]` panics on multi-byte task-list prefix (`old bug`, verified directly)**
   - **Trigger:** Pressing `Enter` on a line starting with `-`, `*`, or `+` where a multi-byte UTF-8 character straddles byte index `6` (e.g., `"-    é"`).
   - **Effect:** Runtime panic (`byte index 6 is not a char boundary`). Present at base commit `9389f4d`.

4. **`crates/scriptui-core/src/editor.rs:246-254, 732`, `crates/scriptui-tui/src/app.rs:68-79, 177-179`, `crates/scriptui-tui/src/preview_pane.rs:56`, `crates/scriptui-core/src/highlight.rs:87`, `crates/scriptui-tui/Cargo.toml:16` — Pre-existing codebase defects corroborated from `docs/codebase-audit-report.md` (`old bug`, spot-checked directly)**
   - **Trigger / Effect:** Spot-checked directly against source at `HEAD` (identical to `9389f4d`):
     - `editor.rs:246-254` & `app.rs:68-76`: startup silently ignores `config.spell_locale` and custom theme search dirs (`&[]`), and swallows `config.toml` load errors.
     - `editor.rs:732`: `Action::Paste` is a no-op (`{}`).
     - `app.rs:77-79`: opening a non-existent CLI file fails startup.
     - `app.rs:177-179`: `KeyEventKind::Repeat` events are dropped.
     - `preview_pane.rs:56` & `highlight.rs:87`: `SyntaxSet::load_defaults_newlines()` and full preview build run synchronously on every frame.
     - `crates/scriptui-tui/Cargo.toml:16`: `scriptui-core` path dependency omits `version`.

### What Was Not Checked
- Did not re-run `cargo test`, `cargo clippy`, `cargo deny`, or `cargo tree` (zero Rust/Cargo files changed in `9389f4d..4d524fe`).
- Did not verify every medium/low line-number citation in `docs/codebase-audit-report.md` beyond the high/critical blockers spot-checked above.
- Did not inspect `.agents/teamwork/` contents.

<!-- SEALED: Independent pass locked -->

## Contract pass

### Step-by-Step Contract Evaluation
- **`STEP 11A` (`TASK-0060` — Add `deny.toml` & verify `cargo deny check`):** **UNMET / DEFERRED** (`.phased/deferred/11A.md`).
  - **Root cause (`TASK-0061`):** Plan scope lapse (`L2`). `STEP 11A` required `[bans] wildcards = "deny"` and `cargo deny check` exiting `0`, while `[SCOPE_LOCK]` forbade editing `crates/scriptui-tui/Cargo.toml`. `cargo-deny 0.20.2` rejects `scriptui-core = { path = "../scriptui-core" }` at `crates/scriptui-tui/Cargo.toml:16` as `error[wildcard]` unless `version = "0.1.0"` is added (or `publish = false` + `allow-wildcard-paths = true`).
  - **Amendments:** None.
  - **Recommendation:** Re-stage `STEP 11A` in Phase 12 with `crates/scriptui-tui/Cargo.toml` added to `ALLOWED_FILES` (adding `version = "0.1.0"` to `scriptui-core`).
- **`[USER] 11B` (Multiagent read-only codebase audit & report):** **PARTIALLY MET**.
  - **Met:** Comprehensive read-only audit report written at `docs/codebase-audit-report.md` covering all required dimensions (architecture, correctness, security/safety, test coverage, scope lapse, incomplete implementations, code hygiene) with rated findings and remediation roadmap.
  - **Unmet:**
    1. `docs/codebase-audit-report.md` was left untracked (uncommitted) and leaks `/srv/devshare/Scriptui` on lines 3 and 78; `.agents/teamwork/` was left untracked without `.agents/` in `.gitignore` (`TASK-0063`).
    2. Audit findings had not yet been filed as pending `neb` items during `[USER] 11B`; filed now as `BUG-0017`–`BUG-0033` and `TASK-0062`–`TASK-0064`.
- **Plan/tracker drift noted in audit (`SCP-06`, `SCP-07`):**
  - `STEPPED_IMPLEMENTATION_PLAN.md:135` (`### Parked`) says `_None — all tracked tasks and bugs are closed._`, stale against open/pending `neb` items (`BUG-0014`–`BUG-0016`, `TASK-0042`–`0044`, `TASK-0049`–`0050`, `TASK-0057`–`0061`, plus newly filed Phase 11 items).
  - `STEPPED_IMPLEMENTATION_PLAN.md:108` lists `11A` as audit and `11B` as `deny.toml` (inverted vs `### Up Next` lines 31–32).
- **Advances items ready to close:** None (no step completed in Phase 11).

### Independent Finding Origins
1. `docs/codebase-audit-report.md:3, 78` internal host path leak → **`coder`** (audit author; filed `TASK-0063`).
2. Untracked `docs/codebase-audit-report.md` and `.agents/teamwork/` without `.gitignore` entry → **`coder`** / process (filed `TASK-0063`).
3. `crates/scriptui-core/src/indent.rs:83` UTF-8 slice panic → **`pre-existing`** (filed `BUG-0017`).
4. Corroborated high/critical codebase blockers (`editor.rs`, `app.rs`, `preview_pane.rs`, `highlight.rs`, `Cargo.toml`) → **`pre-existing`** (filed `BUG-0018`–`BUG-0025`, `TASK-0062`; `crates/scriptui-tui/Cargo.toml:16` already tracked in `TASK-0061`).

## Handoff

- **Verdict:** PARTIAL / DEFERRED (0 code regressions; `STEP 11A` deferred on `L2` scope lapse; `[USER] 11B` report produced, findings now filed in `neb`).
- **Deferred steps to re-split/re-stage:**
  - `STEP 11A` (`TASK-0060`, `TASK-0061`): re-stage with `ALLOWED_FILES: [deny.toml, crates/scriptui-tui/Cargo.toml]` (add `version = "0.1.0"` to `scriptui-core` path dep).
- **Pending items by priority (for Phase 12 selection):**
  - **Critical:** `BUG-0017` (UTF-8 panic in `indent::parse_task_list`).
  - **High (code/release blockers):** `BUG-0018` (startup `--locale`/custom themes/`config.toml`), `BUG-0019` (`Ctrl+V` no-op), `BUG-0020` (`dirty` after Undo), `BUG-0021` (CLI new-file startup), `BUG-0022` (`KeyEventKind::Repeat`), `BUG-0023` (per-frame `SyntaxSet`/preview rebuild), `BUG-0024` (signal handling + alt-screen exit error), `BUG-0025` (atomic save `O_EXCL` + `import_locale_files`), `TASK-0061` (`scriptui-tui/Cargo.toml` version for `cargo-deny`), `TASK-0062` (third-party licenses + `Cargo.toml` metadata).
  - **Medium:** `TASK-0060` (`deny.toml`), `TASK-0063` (scrub path & commit `docs/codebase-audit-report.md` + gitignore `.agents/`), `TASK-0064` (`TerminalGuard`/event-loop tests + `STEP 10D`), `BUG-0026`–`BUG-0033` (FindBar fallthrough, Expanded preview sync/widths, `Super`/`Meta` leak, multi-cursor collapse, UI control chars/viewport/CRLF, GFM/spellcheck/PersonalDict, symlink/size bounds, tab overflow/overlay UX).
  - **Low (existing):** `BUG-0014`, `BUG-0015`, `BUG-0016`, `TASK-0049`, `TASK-0057`, `TASK-0058`, `TASK-0059`.
- **Decisions for user:**
  1. Confirm Phase 12 scope: all Critical + High blockers (`BUG-0017`–`BUG-0025`, `TASK-0060`–`TASK-0063`) plus which Medium/Low items to include before `v0.1.0`.
  2. Decide whether `Ctrl+V` (`BUG-0019`) should add an internal/system clipboard or be removed from `Keymap`/`README.md` in favor of bracketed paste only.
  3. Confirm removing `.agents/teamwork/` and adding `.agents/` to `.gitignore` (`TASK-0063`).
