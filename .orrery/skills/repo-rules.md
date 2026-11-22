# ScripTUI — Repo Rules

ScripTUI is a retro-futuristic Rust TUI (`ratatui`) text editor and GitHub Flavored Markdown (GFM) IDE baseline organized as a Cargo workspace (`crates/scriptui-core` + future `crates/scriptui-tui`).

## Plan & Workflow
- Plan file: `/workspace/STEPPED_IMPLEMENTATION_PLAN.md` (`**Workflow:** phased`).
- Forbidden paths (never edit): `STEPPED_IMPLEMENTATION_PLAN.md`, `docs/*`, `.phased/*`, `.orrery/*`, `scripts/*`.

## Mandatory Gates
- Base Rust gate: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- No `.unwrap()` or `.expect()` in production non-test code; use `ScriptuiError` / `ScriptuiResult`.
- Keep `scriptui-core` free of TUI toolkit imports (`ratatui`, `crossterm`, `clap`).
- Test fixtures must use reserved placeholder data (`192.0.2.0/24`, `example.com`, `testuser`).
