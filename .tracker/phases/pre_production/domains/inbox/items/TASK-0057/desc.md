## Problem Description
In crates/scriptui-tui/tests/tui_integration.rs:1051-1081 (tui_narrow_viewport_and_long_lines_render_without_panic), CommandPalette only checks rect_has_non_empty_cell(buf, total_area) (tautological with text_area) and SpellQuickFix only checks buffer_text(buf).contains("wrld") (already present in the editor viewport). Assert overlay-specific content/regions (e.g. Command Palette modal cells and "Spellcheck: wrld").

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
