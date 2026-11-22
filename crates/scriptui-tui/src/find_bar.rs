use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use scriptui_core::{Action, EditorState, OverlayMode, ScriptuiError, ScriptuiResult};

use crate::layout_tree::rgb_to_color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FindFocus {
    #[default]
    Query,
    Replacement,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FindBarState {
    pub focus: FindFocus,
}

pub fn render_find_bar(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    bar_state: FindBarState,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let palette = &state.theme.palette;
    let bg = rgb_to_color(palette.surface);
    let base_style = Style::default().fg(rgb_to_color(palette.foreground)).bg(bg);
    let muted_style = Style::default().fg(rgb_to_color(palette.muted)).bg(bg);
    let accent_style = Style::default()
        .fg(rgb_to_color(palette.accent))
        .bg(bg)
        .add_modifier(Modifier::BOLD);

    let counter = if state.find_replace.matches.is_empty() {
        "0/0".to_string()
    } else {
        format!(
            "{}/{}",
            state.find_replace.active_match.unwrap_or(0) + 1,
            state.find_replace.matches.len()
        )
    };

    let query_focused = state.overlay == OverlayMode::Find || bar_state.focus == FindFocus::Query;
    let query_style = if query_focused {
        accent_style
    } else {
        base_style
    };

    let case_style = if state.find_replace.query.case_sensitive {
        accent_style
    } else {
        muted_style
    };
    let word_style = if state.find_replace.query.whole_word {
        accent_style
    } else {
        muted_style
    };
    let regex_style = if state.find_replace.query.use_regex {
        accent_style
    } else {
        muted_style
    };

    let row0 = Line::from(vec![
        Span::styled(" Find: ", muted_style),
        Span::styled(state.find_replace.query.pattern.clone(), query_style),
        Span::styled(format!("  {counter} "), base_style),
        Span::styled("[Aa]", case_style),
        Span::styled(" ", base_style),
        Span::styled("[W]", word_style),
        Span::styled(" ", base_style),
        Span::styled("[.*]", regex_style),
        Span::styled(" ", base_style),
        Span::styled("[Alt+Enter: All]", muted_style),
    ]);

    let mut lines = vec![row0];

    if state.overlay == OverlayMode::Replace && area.height >= 2 {
        let replace_style = if bar_state.focus == FindFocus::Replacement {
            accent_style
        } else {
            base_style
        };
        let row1 = Line::from(vec![
            Span::styled(" Replace: ", muted_style),
            Span::styled(state.find_replace.replacement.clone(), replace_style),
            Span::styled(" [Enter: Replace] [Ctrl+Enter: All]", muted_style),
        ]);
        lines.push(row1);
    }

    let widget = Paragraph::new(lines).style(base_style);
    frame.render_widget(widget, area);
}

pub fn handle_find_bar_key(
    key: KeyEvent,
    state: &mut EditorState,
    bar_state: &mut FindBarState,
) -> ScriptuiResult<bool> {
    if !matches!(state.overlay, OverlayMode::Find | OverlayMode::Replace) {
        return Ok(false);
    }

    match key.code {
        KeyCode::Esc => {
            state.overlay = OverlayMode::None;
            bar_state.focus = FindFocus::Query;
            Ok(true)
        }
        KeyCode::Tab | KeyCode::BackTab => {
            if state.overlay == OverlayMode::Replace {
                bar_state.focus = match bar_state.focus {
                    FindFocus::Query => FindFocus::Replacement,
                    FindFocus::Replacement => FindFocus::Query,
                };
            }
            Ok(true)
        }
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                state.dispatch(Action::SelectAllOccurrences)?;
                state.overlay = OverlayMode::None;
                bar_state.focus = FindFocus::Query;
                Ok(true)
            } else if key.modifiers.contains(KeyModifiers::CONTROL)
                && state.overlay == OverlayMode::Replace
            {
                let count = state.replace_all_matches()?;
                state.status_message = Some(format!("Replaced {count} occurrence(s)"));
                Ok(true)
            } else if key.modifiers.contains(KeyModifiers::SHIFT) {
                state.dispatch(Action::FindPrev)?;
                Ok(true)
            } else if key.modifiers.is_empty() {
                if state.overlay == OverlayMode::Replace
                    && bar_state.focus == FindFocus::Replacement
                {
                    state.replace_active_match()?;
                } else {
                    state.dispatch(Action::FindNext)?;
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::ALT) => {
            match c.to_ascii_lowercase() {
                'c' => {
                    state.find_replace.query.case_sensitive =
                        !state.find_replace.query.case_sensitive;
                    refresh_find_matches(state)?;
                    Ok(true)
                }
                'w' => {
                    state.find_replace.query.whole_word = !state.find_replace.query.whole_word;
                    refresh_find_matches(state)?;
                    Ok(true)
                }
                'r' => {
                    state.find_replace.query.use_regex = !state.find_replace.query.use_regex;
                    refresh_find_matches(state)?;
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
        KeyCode::Backspace => {
            let editing_replacement =
                state.overlay == OverlayMode::Replace && bar_state.focus == FindFocus::Replacement;
            if editing_replacement {
                state.find_replace.replacement.pop();
            } else {
                state.find_replace.query.pattern.pop();
                refresh_find_matches(state)?;
            }
            Ok(true)
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            let editing_replacement =
                state.overlay == OverlayMode::Replace && bar_state.focus == FindFocus::Replacement;
            if editing_replacement {
                state.find_replace.replacement.push(c);
            } else {
                state.find_replace.query.pattern.push(c);
                refresh_find_matches(state)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn refresh_find_matches(state: &mut EditorState) -> ScriptuiResult<()> {
    match state.update_find_matches() {
        Ok(_) => Ok(()),
        Err(ScriptuiError::InvalidRegex { pattern, source }) => {
            state.find_replace.matches.clear();
            state.find_replace.active_match = None;
            state.status_message = Some(format!("Invalid regex '{pattern}': {source}"));
            Ok(())
        }
        Err(err) => Err(err),
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
    fn find_bar_typing_toggles_replace_and_alt_enter_multicursor() {
        let mut state = EditorState::new(AppConfig::default()).unwrap();
        state.insert_text("foo bar foo");
        state.overlay = OverlayMode::Replace;
        let mut bar_state = FindBarState::default();

        for ch in ['f', 'o', 'o'] {
            let handled = handle_find_bar_key(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut state,
                &mut bar_state,
            )
            .unwrap();
            assert!(handled);
        }

        let backend = TestBackend::new(90, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render_find_bar(frame, Rect::new(0, 0, 90, 2), &state, bar_state);
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        let row0: String = (0..90).map(|x| buf[(x, 0)].symbol()).collect();
        let row1: String = (0..90).map(|x| buf[(x, 1)].symbol()).collect();
        assert!(row0.contains("1/2"), "row0 was: {row0}");
        assert!(row0.contains("[Aa]"), "row0 was: {row0}");
        assert!(row1.contains("Replace:"), "row1 was: {row1}");

        assert!(!state.find_replace.query.case_sensitive);
        let handled = handle_find_bar_key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::ALT),
            &mut state,
            &mut bar_state,
        )
        .unwrap();
        assert!(handled);
        assert!(state.find_replace.query.case_sensitive);

        let handled = handle_find_bar_key(
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            &mut state,
            &mut bar_state,
        )
        .unwrap();
        assert!(handled);
        assert_eq!(bar_state.focus, FindFocus::Replacement);

        for ch in ['b', 'a', 'z'] {
            let handled = handle_find_bar_key(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut state,
                &mut bar_state,
            )
            .unwrap();
            assert!(handled);
        }
        assert_eq!(state.find_replace.replacement, "baz");

        let handled = handle_find_bar_key(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT),
            &mut state,
            &mut bar_state,
        )
        .unwrap();
        assert!(handled);
        assert_eq!(state.overlay, OverlayMode::None);
        assert_eq!(state.buffers.active().document.selection().len(), 2);
    }
}
