# ScripTUI

ScripTUI is a retro-futuristic terminal text editor and GitHub Flavored Markdown (GFM) IDE built in Rust with [Ratatui](https://ratatui.rs/) and [Crossterm](https://github.com/crossterm-rs/crossterm).

## Architecture

ScripTUI is organized as a two-crate Cargo workspace:

| Crate | Role | Key Capabilities |
| :--- | :--- | :--- |
| `crates/scriptui-core` | Headless editor engine (no TUI toolkit imports) | Ropey rope buffer (`DocumentBuffer`), multi-buffer manager, transactional undo/redo, multi-cursor selections, Unicode display-width & tab-stop soft wrapping, regex/literal find & replace, GFM syntax highlighting & live styled preview, embedded Hunspell `en_US` spellcheck, TOML themes & config, and CUA keymap. |
| `crates/scriptui-tui` | Ratatui + Crossterm TUI frontend (`scriptui` binary) | CLI parsing (`clap`), raw-mode `TerminalGuard` with bracketed paste and OSC 52 clipboard support, tab bar, line-number gutter & editor viewport, split/full Markdown preview pane, inline find/replace bar, command palette, spellcheck quick-fix & file/confirmation modals, and mouse routing. |

## Installation

Building and installing ScripTUI from source requires a Rust stable toolchain (installed via [rustup](https://rustup.rs/)).

```bash
cargo install --path crates/scriptui-tui --locked
```

This installs the `scriptui` binary into `~/.cargo/bin`. Verify the installation:

```bash
scriptui --version
# scriptui 0.1.0
```

Prebuilt binaries for Linux x86_64 (`x86_64-unknown-linux-gnu`) and macOS Apple Silicon (`aarch64-apple-darwin`) are planned for future GitHub releases and are not yet available.

## Quick Start

### Build & Run

```bash
# Build an optimized release binary (target/release/scriptui)
cargo build --release

# Run directly via Cargo with zero or more files
cargo run -p scriptui-tui -- [FILE...]

# Launch with a specific palette and spellcheck locale
cargo run -p scriptui-tui -- --theme phosphor_green --locale en_US README.md
```

### CLI Options & Subcommands

```text
scriptui [OPTIONS] [FILE...]
scriptui dict import <LOCALE> <DIC> <AFF>
```

| Flag / Subcommand | Description |
| :--- | :--- |
| `--theme <NAME>` | Select a built-in palette or custom TOML theme by name (overrides `config.toml`). |
| `--locale <LOCALE>` | Select the Hunspell spellcheck locale (default: `en_US`, overrides `config.toml`). |
| `[FILE...]` | One or more files to open in tabs on startup. |
| `dict import <LOCALE> <DIC> <AFF>` | Import a Hunspell `.dic` and `.aff` dictionary pair for `<LOCALE>` into the user dictionary directory (`~/.config/scriptui/dicts/`). |

### Configuration

ScripTUI reads `~/.config/scriptui/config.toml` (or `$XDG_CONFIG_HOME/scriptui/config.toml` when `XDG_CONFIG_HOME` is set). All fields are optional and fall back to defaults:

```toml
theme = "crt_amber"
spell_locale = "en_US"
spell_enabled = true
line_numbers = true
soft_wrap = true
preview_split_percent = 50
```

## Themes & Custom Palettes

### Built-in Palettes

ScripTUI ships with 5 built-in retro-futuristic palettes (switchable via `--theme`, `config.toml`, or the Command Palette `Ctrl+P` / `F1`):

| Palette Name | Default | Visual Style | Background / Foreground |
| :--- | :---: | :--- | :--- |
| `crt_amber` | Yes | Warm amber CRT phosphor on deep charcoal | `#0A0A08` / `#FFB000` |
| `phosphor_green` | No | Classic P31 green phosphor terminal | `#080C09` / `#40F074` |
| `ice_cyan` | No | Cool ice-cyan vector terminal | `#080C10` / `#70E4FF` |
| `vector_magenta` | No | Magenta-violet vector phosphor display | `#0E0910` / `#F284FF` |
| `high_contrast_mono` | No | Pure black & white high-contrast monochrome | `#000000` / `#F5F5F5` |

### Installing Custom Themes

Place custom TOML theme files in `~/.config/scriptui/themes/<name>.toml` (or `$XDG_CONFIG_HOME/scriptui/themes/<name>.toml`). Theme names may contain ASCII alphanumeric characters, `_`, and `-`. Custom files in this directory take precedence over built-in palettes of the same name.

```toml
name = "custom_amber"

[palette]
background = { r = 10, g = 10, b = 8 }
surface = { r = 20, g = 18, b = 14 }
foreground = { r = 255, g = 176, b = 0 }
# ...all ThemePalette RGB fields (see crates/scriptui-core/src/theme.rs)
```

## Hunspell Dictionaries & Spellcheck

ScripTUI bundles an embedded `en_US` Hunspell dictionary so spellcheck works out of the box with zero external setup. To add another locale (for example `en_GB` or `fr_FR`), import its `.dic` and `.aff` files:

```bash
scriptui dict import en_GB /path/to/en_GB.dic /path/to/en_GB.aff
```

Dictionaries are stored in `~/.config/scriptui/dicts/<LOCALE>.{dic,aff}` (or `$XDG_CONFIG_HOME/scriptui/dicts/`) and can be selected via `--locale <LOCALE>`, `spell_locale` in `config.toml`, or the Command Palette.

## Keybindings (CUA Default)

### File, Buffer & Application

| Shortcut | Action |
| :--- | :--- |
| `Ctrl+N` | New empty buffer |
| `Ctrl+O` | Open file prompt |
| `Ctrl+S` | Save active buffer |
| `Ctrl+Shift+S` | Save active buffer as |
| `Ctrl+W` | Close active buffer |
| `Ctrl+PageDown` | Next buffer tab |
| `Ctrl+PageUp` | Previous buffer tab |
| `Ctrl+Q` | Quit ScripTUI |

### Editing, History & Indentation

| Shortcut | Action |
| :--- | :--- |
| `Enter` | Insert newline (with auto-indent) |
| `Backspace` | Delete character/selection backward |
| `Delete` | Delete character/selection forward |
| `Tab` | Indent selection or insert indent |
| `Shift+Tab` | Dedent current line or selection |
| `Ctrl+C` | Copy selection (with OSC 52 clipboard sync) |
| `Ctrl+X` | Cut selection |
| `Ctrl+V` | Paste from clipboard |
| `Ctrl+A` | Select all |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Redo |
| `Esc` | Close active overlay or collapse multi-cursor |

### Find, Replace & Multi-Cursor

| Shortcut | Action |
| :--- | :--- |
| `Ctrl+F` | Open Find bar |
| `Ctrl+H` | Open Find & Replace bar |
| `F3` | Find next match |
| `Shift+F3` | Find previous match |
| `Ctrl+Shift+L` | Select all occurrences of current selection/word (multi-cursor) |

#### Find / Replace Bar Inline Shortcuts

When the Find or Replace bar is open:

| Shortcut | Action |
| :--- | :--- |
| `Alt+C` | Toggle case-sensitive matching (`[Aa]`) |
| `Alt+W` | Toggle whole-word matching (`[W]`) |
| `Alt+R` | Toggle regular-expression mode (`[.*]`) |
| `Alt+Enter` | Select all matches as multi-cursors and close bar |
| `Enter` | Jump to next match (Find field) or replace active match (Replace field) |
| `Shift+Enter` | Jump to previous match |
| `Ctrl+Enter` | Replace all matches (Replace mode) |
| `Tab` / `Shift+Tab` | Switch focus between Find and Replace inputs |
| `Esc` | Close Find/Replace bar |

### Overlays & Preview

| Shortcut | Action |
| :--- | :--- |
| `Ctrl+P` / `F1` | Open Command Palette |
| `Ctrl+.` | Open Spellcheck Quick-Fix at cursor |
| `Ctrl+Shift+P` | Toggle Markdown preview (hidden / split pane) |
| `Alt+P` | Toggle expanded full-pane Markdown preview |
| `Alt+Z` | Toggle soft word wrapping |

### Navigation & Selection

| Shortcut | Action |
| :--- | :--- |
| `Left` / `Right` / `Up` / `Down` | Move cursor |
| `Home` / `End` | Move to start / end of line |
| `PageUp` / `PageDown` | Scroll one page up / down |
| `Shift+Left` / `Shift+Right` / `Shift+Up` / `Shift+Down` | Extend selection by character / line |
| `Shift+Home` / `Shift+End` | Extend selection to start / end of line |

## Mouse Interactions

| Action | Behavior |
| :--- | :--- |
| **Left-click in editor** | Places the caret at the clicked visual cell. |
| **Left-click & drag in editor** | Creates and extends a text selection. |
| **Scroll wheel over editor** | Scrolls the editor viewport (3 visual lines per notch). |
| **Scroll wheel over preview** | Scrolls the Markdown preview pane independently. |
| **Left-click on tab bar** | Switches to the clicked buffer tab. |
| **Right-click in editor** | Moves the caret and opens the Spellcheck Quick-Fix modal for a misspelled word. |
| **Left-click on preview edge tab** | Toggles the split Markdown preview pane open or closed. |
| **Left-click & drag split divider** | Resizes the editor / preview split ratio (clamped between 20% and 80%). |

## Development & Verification

Run the full formatting, lint, and test suite from the workspace root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Releases

ScripTUI follows [Semantic Versioning](https://semver.org/). Release builds use the workspace's tuned `[profile.release]` configuration (link-time optimization, a single codegen unit, and stripped symbols). Notable changes for each version are recorded in [CHANGELOG.md](CHANGELOG.md).

## License

ScripTUI is licensed under the MIT License. See [LICENSE](LICENSE) for details.
