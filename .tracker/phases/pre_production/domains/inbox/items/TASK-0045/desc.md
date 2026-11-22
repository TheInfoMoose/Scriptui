## Problem Description
STEP 8A spec gap: `[POST_GATE]` invokes `./target/release/scriptui --version`, but the Orrery environment sets `CARGO_TARGET_DIR=/workspace/target/orrery`, so `cargo build --release` writes the binary to `/workspace/target/orrery/release/scriptui` (which prints `scriptui 0.1.0`) and `./target/release/scriptui` does not exist. Location: `STEPPED_IMPLEMENTATION_PLAN.md` STEP 8A `[POST_GATE]`.
Suggested post_gate amendment: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --release && "${CARGO_TARGET_DIR:-target}/release/scriptui" --version | grep -qE '^scriptui 0\.1\.0$' && test -f LICENSE`
Lapse: L4 other
Kind: scope-lapse
Found during: STEP 8A (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
