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

**Phase goal (Phase 7 — Complete):** Resolved both Phase 6 review findings (`BUG-0010` and `BUG-0011`). All planned phases (Phases 1–7) and tracked items are closed.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = `STEP <ID>`); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

_None — Phase 7 complete. Awaiting Phase 8 scope or release cut._

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

_None — all planned phases (Phases 1–7) are complete._

### Open Questions

_None._

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.
_None — all tracked tasks and bugs are closed._
