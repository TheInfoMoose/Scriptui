use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use scriptui_core::{
    builtin_theme_names, Action, CommandPaletteState, EditorState, IndentStyle, OverlayMode,
    ScriptuiResult,
};

use crate::layout_tree::rgb_to_color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteItemKind {
    Action(Action),
    Theme(String),
    Locale(String),
    ToggleIndentStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteEntry {
    pub id: String,
    pub label: String,
    pub kind: PaletteItemKind,
}

pub fn discover_installed_locales(dict_dirs: &[PathBuf]) -> Vec<String> {
    let mut locales = BTreeSet::new();
    locales.insert("en_US".to_string());

    for dir in dict_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let is_dic = path.extension().and_then(|ext| ext.to_str()) == Some("dic");
            if !is_dic {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !stem.is_empty() && path.with_extension("aff").is_file() {
                locales.insert(stem.to_string());
            }
        }
    }

    locales.into_iter().collect()
}

pub fn build_palette_entries(extra_locales: &[String]) -> Vec<PaletteEntry> {
    let mut entries: Vec<PaletteEntry> = CommandPaletteState::default_commands()
        .into_iter()
        .map(|cmd| PaletteEntry {
            id: cmd.id.to_string(),
            label: cmd.label.to_string(),
            kind: PaletteItemKind::Action(cmd.action),
        })
        .collect();

    entries.push(PaletteEntry {
        id: "indent.cycle_style".to_string(),
        label: "Indent: Cycle Style (Spaces 4 / Spaces 2 / Tabs)".to_string(),
        kind: PaletteItemKind::ToggleIndentStyle,
    });

    for &name in builtin_theme_names() {
        entries.push(PaletteEntry {
            id: format!("theme.{name}"),
            label: format!("Theme: {name}"),
            kind: PaletteItemKind::Theme(name.to_string()),
        });
    }

    let mut seen_locales = HashSet::new();
    let mut locales = Vec::new();
    if seen_locales.insert("en_US".to_string()) {
        locales.push("en_US".to_string());
    }
    for loc in extra_locales {
        if !loc.is_empty() && seen_locales.insert(loc.clone()) {
            locales.push(loc.clone());
        }
    }

    for loc in locales {
        entries.push(PaletteEntry {
            id: format!("locale.{loc}"),
            label: format!("Spell Locale: {loc}"),
            kind: PaletteItemKind::Locale(loc),
        });
    }

    entries
}

pub fn filter_palette_entries<'a>(
    entries: &'a [PaletteEntry],
    query: &str,
) -> Vec<&'a PaletteEntry> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return entries.iter().collect();
    }

    let needle = trimmed.to_lowercase();
    let mut substring_matches = Vec::new();
    let mut subsequence_matches = Vec::new();

    for entry in entries {
        let label_lower = entry.label.to_lowercase();
        let id_lower = entry.id.to_lowercase();
        if label_lower.contains(&needle) || id_lower.contains(&needle) {
            substring_matches.push(entry);
        } else if is_ordered_subsequence(&needle, &label_lower)
            || is_ordered_subsequence(&needle, &id_lower)
        {
            subsequence_matches.push(entry);
        }
    }

    substring_matches.extend(subsequence_matches);
    substring_matches
}

fn is_ordered_subsequence(needle: &str, haystack: &str) -> bool {
    let mut haystack_chars = haystack.chars();
    for target in needle.chars() {
        if !haystack_chars.any(|ch| ch == target) {
            return false;
        }
    }
    true
}

pub fn render_command_palette(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    entries: &[PaletteEntry],
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let width = area.width.clamp(20, 60).min(area.width);
    let height = area.height.clamp(5, 14).min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let modal_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, modal_area);

    let palette = &state.theme.palette;
    let bg = rgb_to_color(palette.surface);
    let base_style = Style::default().fg(rgb_to_color(palette.foreground)).bg(bg);
    let prompt_style = Style::default()
        .fg(rgb_to_color(palette.accent))
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let selected_style = Style::default()
        .fg(rgb_to_color(palette.background))
        .bg(rgb_to_color(palette.accent))
        .add_modifier(Modifier::BOLD);
    let border_style = Style::default()
        .fg(rgb_to_color(palette.border_focused))
        .bg(bg);

    let block = Block::default()
        .title(" Command Palette ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(base_style);
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let filtered = filter_palette_entries(entries, &state.palette.query);
    let selected = state
        .palette
        .selected_index
        .min(filtered.len().saturating_sub(1));
    let list_height = usize::from(inner.height.saturating_sub(1));
    let scroll_offset = if list_height == 0 || selected < list_height {
        0
    } else {
        selected + 1 - list_height
    };

    let mut lines = Vec::with_capacity(usize::from(inner.height));
    lines.push(Line::from(vec![
        Span::styled("> ", prompt_style),
        Span::styled(state.palette.query.clone(), base_style),
    ]));

    for (idx, entry) in filtered
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(list_height)
    {
        let style = if idx == selected {
            selected_style
        } else {
            base_style
        };
        lines.push(Line::from(Span::styled(entry.label.clone(), style)));
    }

    let widget = Paragraph::new(lines).style(base_style);
    frame.render_widget(widget, inner);
}

pub fn handle_palette_key(
    key: KeyEvent,
    state: &mut EditorState,
    entries: &[PaletteEntry],
    theme_dirs: &[PathBuf],
    dict_dirs: &[PathBuf],
) -> ScriptuiResult<bool> {
    if state.overlay != OverlayMode::CommandPalette {
        return Ok(false);
    }

    match key.code {
        KeyCode::Esc => {
            state.overlay = OverlayMode::None;
            state.palette.query.clear();
            state.palette.selected_index = 0;
            Ok(true)
        }
        KeyCode::Up => {
            let filtered_len = filter_palette_entries(entries, &state.palette.query).len();
            if filtered_len == 0 {
                state.palette.selected_index = 0;
            } else {
                let current = state.palette.selected_index.min(filtered_len - 1);
                state.palette.selected_index = current.saturating_sub(1);
            }
            Ok(true)
        }
        KeyCode::Down => {
            let filtered_len = filter_palette_entries(entries, &state.palette.query).len();
            if filtered_len == 0 {
                state.palette.selected_index = 0;
            } else {
                state.palette.selected_index =
                    (state.palette.selected_index + 1).min(filtered_len - 1);
            }
            Ok(true)
        }
        KeyCode::Backspace => {
            state.palette.query.pop();
            state.palette.selected_index = 0;
            Ok(true)
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            state.palette.query.push(c);
            state.palette.selected_index = 0;
            Ok(true)
        }
        KeyCode::Enter => {
            let selected_kind = {
                let filtered = filter_palette_entries(entries, &state.palette.query);
                filtered
                    .get(state.palette.selected_index)
                    .or_else(|| filtered.first())
                    .map(|entry| entry.kind.clone())
            };

            state.overlay = OverlayMode::None;
            state.palette.query.clear();
            state.palette.selected_index = 0;

            if let Some(kind) = selected_kind {
                match kind {
                    PaletteItemKind::Action(action) => {
                        state.dispatch(action)?;
                    }
                    PaletteItemKind::Theme(name) => {
                        state.switch_theme(&name, theme_dirs)?;
                        state.status_message = Some(format!("Theme: {name}"));
                    }
                    PaletteItemKind::Locale(loc) => {
                        state.switch_spell_locale(&loc, dict_dirs)?;
                        state.status_message = Some(format!("Spell locale: {loc}"));
                    }
                    PaletteItemKind::ToggleIndentStyle => {
                        state.toggle_indent_style();
                        let label = match state.config.indent.style {
                            IndentStyle::Spaces(n) => format!("Spaces ({n})"),
                            IndentStyle::Tabs => "Tabs".to_string(),
                        };
                        state.status_message = Some(format!("Indent style: {label}"));
                    }
                }
            }

            Ok(true)
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use scriptui_core::{AppConfig, EditorState, OverlayMode};

    use super::*;

    #[test]
    fn command_palette_fuzzy_filters_and_switches_theme_and_renders() {
        let mut state = EditorState::new(AppConfig::default()).unwrap();
        let entries = build_palette_entries(&[]);
        state.overlay = OverlayMode::CommandPalette;

        for ch in ['p', 'h', 'g', 'r'] {
            let handled = handle_palette_key(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut state,
                &entries,
                &[],
                &[],
            )
            .unwrap();
            assert!(handled);
        }

        let filtered = filter_palette_entries(&entries, &state.palette.query);
        assert!(!filtered.is_empty());
        assert_eq!(filtered[0].label, "Theme: phosphor_green");

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render_command_palette(frame, Rect::new(0, 0, 80, 20), &state, &entries);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        let screen: String = (0..20)
            .map(|y| (0..80).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            screen.contains("Command Palette"),
            "screen did not contain title:\n{screen}"
        );
        assert!(
            screen.contains("phosphor_green"),
            "screen did not contain phosphor_green:\n{screen}"
        );

        let handled = handle_palette_key(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut state,
            &entries,
            &[],
            &[],
        )
        .unwrap();
        assert!(handled);
        assert_eq!(state.theme.name, "phosphor_green");
        assert_eq!(state.overlay, OverlayMode::None);
    }
}
