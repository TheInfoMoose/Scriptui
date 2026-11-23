## Problem Description
PersonalDictionary load/save unused at runtime; editor.rs:427-437 drops personal/ignored words on locale switch; spell.rs:410-412 excludes ASCII digits; markdown.rs:307-438 has duplicate autolink, unclosed fence, ref-def, and CRLF frontmatter edge cases. From Phase 11 audit.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
