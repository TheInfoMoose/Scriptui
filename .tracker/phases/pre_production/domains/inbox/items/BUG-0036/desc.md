## Problem Description
In crates/scriptui-core/src/spell.rs:452-467, import_locale_files renames dic_tmp -> dic_dest and then aff_tmp -> aff_dest. If the second rename fails, dic_dest has already been written/replaced without rollback, leaving a partial locale pair on disk.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
