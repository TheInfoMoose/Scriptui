## Problem Description
In crates/scriptui-core/src/editor.rs:288-298 and 486-515, starting with spell_enabled=false leaves spell_engine=None so personal.dic is never loaded, while personal_dict_path is Some(...). Calling switch_spell_locale creates a fresh SpellEngine with an empty PersonalDictionary, and subsequently adding a word calls save_to_file(), overwriting personal.dic and destroying previously saved words.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
