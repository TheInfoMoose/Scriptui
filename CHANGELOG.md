# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-22

### Added

- CUA text editing with transactional undo/redo, auto-indent, indent/dedent, soft word wrapping, line-number gutter, and OSC 52 clipboard sync.
- Multiple buffer tabs with file open, save, save-as, close prompts, and tab navigation.
- Inline find and replace with literal, case-sensitive, whole-word, and regular-expression modes, plus multi-cursor selection.
- Live styled GitHub Flavored Markdown (GFM) preview in split-pane and expanded full-pane modes.
- GFM syntax highlighting in the editor viewport.
- Hunspell spellcheck with a bundled `en_US` dictionary, quick-fix modal, and `scriptui dict import` for additional locales.
- Five built-in retro-futuristic palettes (`crt_amber`, `phosphor_green`, `ice_cyan`, `vector_magenta`, `high_contrast_mono`) and custom TOML theme support, switchable via CLI, config, or the Command Palette.
- Mouse support for caret placement, drag selection, independent editor/preview scrolling, tab switching, right-click spellcheck quick-fix, and split-pane resizing.
- CLI options (`--theme`, `--locale`, `[FILE...]`, `dict import`) and optional TOML configuration at `~/.config/scriptui/config.toml` (respecting `$XDG_CONFIG_HOME`).
