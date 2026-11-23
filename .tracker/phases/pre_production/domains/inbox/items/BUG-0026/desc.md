## Problem Description
find_bar.rs:117-208 returns Ok(false) for Delete/arrows/Home/End/PageUp/PageDown causing app.rs:182-187 to fall through to editor; focus and match state leak across overlay/buffer switches. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
