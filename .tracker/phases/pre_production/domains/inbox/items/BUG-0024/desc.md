## Problem Description
terminal.rs:19-75 installs no SIGTERM/SIGHUP/SIGTSTP handlers leaving PTY in raw/mouse mode on signal; main.rs:47-50 prints exit errors before TerminalGuard drops. From Phase 11 audit (BLK-08a).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
