## Problem Description
`crates/scriptui-tui/tests/tui_integration.rs:465` — `large_file_cached_analysis_and_viewport_render_responsiveness` asserts `10 scroll frames < 500ms` in unoptimized debug builds, which fails (~2.42s in isolation, ~2.68s under parallel `cargo test --workspace`) even though `analysis_recompute_count() == 1` — suggested fix: relax or gate the wall-clock duration assertion for debug builds or optimize viewport rendering per scroll frame.
Kind: bug
Found during: STEP 18A (Neon)

## Reproduction Steps
1. `cargo test -p scriptui-tui --test tui_integration large_file_cached_analysis_and_viewport_render_responsiveness`
2. Observe: `panicked at crates/scriptui-tui/tests/tui_integration.rs:465:5: 10 scroll frames took 2.421307392s, expected < 500ms`

## Fix Log & Agent Notes
