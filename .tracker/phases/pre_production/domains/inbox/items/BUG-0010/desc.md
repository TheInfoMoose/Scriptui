## Problem Description
`EditorState::active_analysis` caches `(GfmDocument, Vec<SpellDiagnostic>)` keyed on `(buffer_index, doc_revision, spell_revision)` (`crates/scriptui-core/src/editor.rs:275-309`). When ScripTUI launches with no file, `buffers[0]` is an empty `DocumentBuffer` with `revision == 0`, so the initial render caches `(0, 0, 0)`. Opening a file via `Ctrl+O` (`crates/scriptui-tui/src/modals.rs:290`) calls `state.buffers.open_file(...)`, which hits the `replace_initial` branch in `crates/scriptui-core/src/buffers.rs:123-137` and replaces `buffers[0].document` with a new `DocumentBuffer::from_file(...)` whose `revision` is also `0` at `active_index == 0`. The same occurs on `BufferManager::reload_active_from_disk`. Because `analysis_cache` is not invalidated (and `AnalysisCache` does not track a monotonic document identity), `active_analysis()` returns stale empty GFM spans and spell diagnostics until the user edits the buffer.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
