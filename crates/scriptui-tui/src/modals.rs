use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use scriptui_core::{EditorState, OverlayMode, ScriptuiResult};

use crate::layout_tree::rgb_to_color;

pub fn centered_rect(area: Rect, max_width: u16, max_height: u16) -> Rect {
    let width = max_width.min(area.width);
    let height = max_height.min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width, height)
}

pub fn render_spell_quick_fix(frame: &mut Frame, area: Rect, state: &EditorState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let Some(popup) = &state.spell_popup else {
        return;
    };

    let word = &popup.diagnostic.word;
    let title = format!(" Spellcheck: {word} ");
    let mut items: Vec<String> = popup.suggestions.clone();
    items.push(format!("Add '{word}' to Personal Dictionary"));
    items.push(format!("Ignore '{word}' (Session)"));

    let max_item_len = items
        .iter()
        .map(|item| item.chars().count())
        .max()
        .unwrap_or(0)
        .max(title.chars().count());
    let desired_width = u16::try_from(max_item_len)
        .unwrap_or(u16::MAX)
        .saturating_add(4)
        .max(40);
    let desired_height = u16::try_from(items.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2);
    let modal_area = centered_rect(area, desired_width, desired_height);

    frame.render_widget(Clear, modal_area);

    let palette = &state.theme.palette;
    let bg = rgb_to_color(palette.surface);
    let base_style = Style::default().fg(rgb_to_color(palette.foreground)).bg(bg);
    let selected_style = Style::default()
        .fg(rgb_to_color(palette.background))
        .bg(rgb_to_color(palette.accent))
        .add_modifier(Modifier::BOLD);
    let border_style = Style::default()
        .fg(rgb_to_color(palette.border_focused))
        .bg(bg);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(base_style);
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let selected = popup.selected_index.min(items.len().saturating_sub(1));
    let visible_rows = usize::from(inner.height);
    let scroll_offset = if selected < visible_rows {
        0
    } else {
        selected + 1 - visible_rows
    };

    let lines: Vec<Line> = items
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_rows)
        .map(|(idx, text)| {
            let style = if idx == selected {
                selected_style
            } else {
                base_style
            };
            Line::from(Span::styled(text.clone(), style))
        })
        .collect();

    let widget = Paragraph::new(lines).style(base_style);
    frame.render_widget(widget, inner);
}

pub fn handle_spell_quick_fix_key(key: KeyEvent, state: &mut EditorState) -> ScriptuiResult<bool> {
    if state.overlay != OverlayMode::SpellQuickFix {
        return Ok(false);
    }

    match key.code {
        KeyCode::Esc => {
            state.spell_popup = None;
            state.overlay = OverlayMode::None;
            Ok(true)
        }
        KeyCode::Up => {
            if let Some(popup) = &mut state.spell_popup {
                let total = popup.suggestions.len() + 2;
                let current = popup.selected_index.min(total - 1);
                popup.selected_index = current.saturating_sub(1);
            }
            Ok(true)
        }
        KeyCode::Down => {
            if let Some(popup) = &mut state.spell_popup {
                let total = popup.suggestions.len() + 2;
                popup.selected_index = (popup.selected_index + 1).min(total - 1);
            }
            Ok(true)
        }
        KeyCode::Enter => {
            let Some((selected, suggestion_count)) = state
                .spell_popup
                .as_ref()
                .map(|popup| (popup.selected_index, popup.suggestions.len()))
            else {
                state.overlay = OverlayMode::None;
                return Ok(true);
            };

            if selected < suggestion_count {
                state.apply_selected_spell_fix()?;
            } else if selected == suggestion_count {
                state.add_spell_popup_word_to_personal_dict();
            } else {
                state.ignore_spell_popup_word();
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn render_confirm_modal(frame: &mut Frame, area: Rect, state: &EditorState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let (title, body) = match state.overlay {
        OverlayMode::ConfirmQuit => (
            " Unsaved Changes — Quit? ",
            "Discard unsaved changes and quit? [Y]es / [N]o",
        ),
        OverlayMode::ConfirmCloseBuffer => (
            " Unsaved Changes — Close Buffer? ",
            "Discard unsaved changes and close? [Y]es / [N]o",
        ),
        _ => return,
    };

    let desired_width = u16::try_from(body.chars().count().max(title.chars().count()))
        .unwrap_or(u16::MAX)
        .saturating_add(4);
    let modal_area = centered_rect(area, desired_width, 3);
    frame.render_widget(Clear, modal_area);

    let palette = &state.theme.palette;
    let bg = rgb_to_color(palette.surface);
    let base_style = Style::default().fg(rgb_to_color(palette.foreground)).bg(bg);
    let border_style = Style::default()
        .fg(rgb_to_color(palette.border_focused))
        .bg(bg);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(base_style);
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let widget = Paragraph::new(Line::from(Span::styled(body, base_style))).style(base_style);
    frame.render_widget(widget, inner);
}

pub fn handle_confirm_modal_key(key: KeyEvent, state: &mut EditorState) -> bool {
    if !matches!(
        state.overlay,
        OverlayMode::ConfirmQuit | OverlayMode::ConfirmCloseBuffer
    ) {
        return false;
    }

    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            state.confirm_dirty_action(true);
            true
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            state.confirm_dirty_action(false);
            true
        }
        _ => false,
    }
}

pub fn render_file_prompt(frame: &mut Frame, area: Rect, state: &EditorState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let title = match state.overlay {
        OverlayMode::OpenFilePrompt => " Open File ",
        OverlayMode::SaveAsPrompt => " Save As ",
        _ => return,
    };

    let modal_area = centered_rect(area, 60, 3);
    frame.render_widget(Clear, modal_area);

    let palette = &state.theme.palette;
    let bg = rgb_to_color(palette.surface);
    let base_style = Style::default().fg(rgb_to_color(palette.foreground)).bg(bg);
    let border_style = Style::default()
        .fg(rgb_to_color(palette.border_focused))
        .bg(bg);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(base_style);
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let widget = Paragraph::new(Line::from(Span::styled(
        state.file_prompt_input.as_str(),
        base_style,
    )))
    .style(base_style);
    frame.render_widget(widget, inner);
}

pub fn handle_file_prompt_key(key: KeyEvent, state: &mut EditorState) -> ScriptuiResult<bool> {
    if !matches!(
        state.overlay,
        OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt
    ) {
        return Ok(false);
    }

    match key.code {
        KeyCode::Esc => {
            state.overlay = OverlayMode::None;
            Ok(true)
        }
        KeyCode::Backspace => {
            state.file_prompt_input.pop();
            Ok(true)
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            state.file_prompt_input.push(c);
            Ok(true)
        }
        KeyCode::Enter => {
            let path = state.file_prompt_input.trim().to_string();
            if !path.is_empty() {
                match state.overlay {
                    OverlayMode::OpenFilePrompt => {
                        state.buffers.open_file(&PathBuf::from(&path))?;
                        state.find_replace.matches.clear();
                        state.find_replace.active_match = None;
                        state.status_message = Some(format!("Opened {path}"));
                        state.overlay = OverlayMode::None;
                    }
                    OverlayMode::SaveAsPrompt => {
                        state
                            .buffers
                            .active_mut()
                            .document
                            .save_as_atomic(PathBuf::from(&path))?;
                        state.status_message = Some(format!("Saved {path}"));
                        state.overlay = OverlayMode::None;
                    }
                    _ => {}
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
    use scriptui_core::{Action, AppConfig, EditorState, OverlayMode};
    use tempfile::tempdir;

    use super::*;

    fn buffer_to_string(terminal: &Terminal<TestBackend>, width: u16, height: u16) -> String {
        let buf = terminal.backend().buffer();
        (0..height)
            .map(|y| (0..width).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn spell_popup_confirm_modal_and_save_as_prompt_roundtrip() {
        let mut state = EditorState::new(AppConfig::default()).unwrap();
        state.insert_text("hello wrld");
        assert!(state.open_spell_quick_fix_at_cursor());
        assert_eq!(state.overlay, OverlayMode::SpellQuickFix);

        let backend = TestBackend::new(70, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                render_spell_quick_fix(frame, Rect::new(0, 0, 70, 16), &state);
            })
            .unwrap();
        let screen = buffer_to_string(&terminal, 70, 16);
        assert!(
            screen.contains("Add 'wrld' to Personal Dictionary"),
            "spell popup screen did not contain expected action:\n{screen}"
        );

        let suggestions_len = state.spell_popup.as_ref().unwrap().suggestions.len();
        for _ in 0..=suggestions_len {
            let handled = handle_spell_quick_fix_key(
                KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                &mut state,
            )
            .unwrap();
            assert!(handled);
        }
        let handled = handle_spell_quick_fix_key(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut state,
        )
        .unwrap();
        assert!(handled);
        assert_eq!(state.overlay, OverlayMode::None);
        assert!(state.spell_popup.is_none());
        let remaining_diags = state
            .spell_engine
            .as_ref()
            .unwrap()
            .check_document("hello wrld", &[]);
        assert!(
            remaining_diags.is_empty(),
            "expected 'wrld' to be ignored, got: {remaining_diags:?}"
        );

        assert!(state.buffers.active().document.is_dirty());
        state.dispatch(Action::Quit).unwrap();
        assert_eq!(state.overlay, OverlayMode::ConfirmQuit);
        assert!(!state.should_quit);

        terminal
            .draw(|frame| {
                render_confirm_modal(frame, Rect::new(0, 0, 70, 16), &state);
            })
            .unwrap();
        let confirm_screen = buffer_to_string(&terminal, 70, 16);
        assert!(
            confirm_screen.contains("Unsaved Changes"),
            "confirm modal screen did not contain title:\n{confirm_screen}"
        );

        assert!(handle_confirm_modal_key(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            &mut state
        ));
        assert!(state.should_quit);
        assert_eq!(state.overlay, OverlayMode::None);

        let dir = tempdir().unwrap();
        let target_path = dir.path().join("saved_note.md");
        let target_str = target_path.to_string_lossy().to_string();

        state.overlay = OverlayMode::SaveAsPrompt;
        state.file_prompt_input.clear();
        for ch in target_str.chars() {
            let handled = handle_file_prompt_key(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut state,
            )
            .unwrap();
            assert!(handled);
        }

        terminal
            .draw(|frame| {
                render_file_prompt(frame, Rect::new(0, 0, 70, 16), &state);
            })
            .unwrap();
        let prompt_screen = buffer_to_string(&terminal, 70, 16);
        assert!(
            prompt_screen.contains("Save As"),
            "file prompt screen did not contain title:\n{prompt_screen}"
        );

        let handled = handle_file_prompt_key(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut state,
        )
        .unwrap();
        assert!(handled);
        assert_eq!(state.overlay, OverlayMode::None);
        assert!(target_path.exists());
        let saved = std::fs::read_to_string(&target_path).unwrap();
        assert_eq!(saved, "hello wrld");
    }
}
