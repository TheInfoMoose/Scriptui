## Problem Description
CHANGELOG.md:7,9 use [Unreleased] and [0.1.0] link-reference headings with no definitions, so brackets render literally. Blocked on a repository URL; add [Unreleased]/[0.1.0] compare links (and repository = in [workspace.package]) when the GitHub remote exists.
Found during: Phase 8 review.

## Reproduction Steps
Render CHANGELOG.md on GitHub: headings show literal [Unreleased] and [0.1.0] text instead of links.

## Fix Log & Agent Notes
