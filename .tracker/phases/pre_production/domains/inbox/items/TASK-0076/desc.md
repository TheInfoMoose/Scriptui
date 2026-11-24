## Problem Description
crates/scriptui-core/src/buffer.rs:117-134 enforces MAX_FILE_BYTES (64 MiB) in DocumentBuffer::from_file, but no unit or integration test exercises the meta.len() > MAX_FILE_BYTES rejection path (e.g. via a sparse temp file with File::set_len(MAX_FILE_BYTES + 1)).

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
