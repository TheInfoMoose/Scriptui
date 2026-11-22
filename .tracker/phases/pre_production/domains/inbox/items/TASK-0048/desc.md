## Problem Description
crates/scriptui-tui/tests/tui_integration.rs:470 cli_version_matches_workspace_version asserts the literal "scriptui 0.1.0"; it fails on every version bump and does not check the workspace version its name claims. Fix: concat!("scriptui ", env!("CARGO_PKG_VERSION")) or rename to say it pins 0.1.0.
Found during: Phase 8 review.

## Reproduction Steps
Bump [workspace.package] version in Cargo.toml to 0.1.1, run cargo test -p scriptui-tui cli_version: test fails.

## Fix Log & Agent Notes
