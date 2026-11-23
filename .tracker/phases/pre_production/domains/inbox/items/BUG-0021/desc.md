## Problem Description
TuiApp::from_cli (app.rs:77-79) calls state.buffers.open_file(path)? which fails on ErrorKind::NotFound instead of creating an empty named buffer ready for first save. From Phase 11 audit (BLK-05).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
