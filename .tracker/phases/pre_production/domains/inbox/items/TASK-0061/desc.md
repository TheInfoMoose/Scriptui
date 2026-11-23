## Problem Description
STEP 11A spec gap: [MUTATION_SPEC] requires `[bans] wildcards = "deny"` and `[POST_GATE]` requires `cargo deny check` to exit 0, while `[SCOPE_LOCK]` forbids modifying `crates/scriptui-tui/Cargo.toml`. `cargo-deny 0.20.2` rejects `scriptui-core = { path = "../scriptui-core" }` at `crates/scriptui-tui/Cargo.toml:16` as `error[wildcard]` (even with `allow-wildcard-paths = true`, because `scriptui-tui` does not set `publish = false`). Passing `cargo deny check` with `wildcards = "deny"` requires editing `crates/scriptui-tui/Cargo.toml:16` (e.g., adding `version = "0.1.0"` to the `scriptui-core` path dependency, or setting `publish = false` alongside `allow-wildcard-paths = true`). Location: `crates/scriptui-tui/Cargo.toml:16`.
Lapse: L2 file outside ALLOWED_FILES
Kind: scope-lapse
Found during: STEP 11A (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
