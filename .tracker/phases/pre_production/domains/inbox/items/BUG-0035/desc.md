## Problem Description
In crates/scriptui-tui/src/app.rs:415-416, event::poll(Duration::from_millis(50))? and event::read()? propagate any io::Error without ignoring ErrorKind::Interrupted (EINTR). If a Unix signal (SIGTERM, SIGHUP, SIGTSTP) interrupts the underlying poll/read syscall, run_event_loop exits with Err(Interrupted) instead of continuing to check guard.should_terminate() or guard.handle_suspend_if_requested().

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
