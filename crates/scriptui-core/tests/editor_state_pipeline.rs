use std::fs;

use scriptui_core::{
    compute_wrapped_layout, Action, AppConfig, EditorState, KeyChord, KeyCode, KeyModifiers,
    Keymap, OverlayMode, Position, PreviewMode, ScriptuiError, SearchQuery, SelectionSet,
};

#[test]
fn multi_buffer_keymap_find_multicursor_and_undo_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let first_path = dir.path().join("first.md");
    let second_path = dir.path().join("second.md");
    fs::write(&first_path, "alpha beta alpha\n").unwrap();
    fs::write(&second_path, "# Second\n").unwrap();

    let mut state = EditorState::new(AppConfig::default()).unwrap();
    state.buffers.open_file(&first_path).unwrap();
    state.buffers.open_file(&second_path).unwrap();

    let prev_buffer = state
        .keymap
        .lookup(KeyChord::new(KeyCode::PageUp, KeyModifiers::CTRL));
    assert_eq!(prev_buffer, Some(Action::PrevBuffer));
    state.dispatch(prev_buffer.unwrap()).unwrap();
    assert_eq!(state.buffers.active().display_name(), "first.md");

    state.find_replace.query = SearchQuery::plain("alpha");
    let select_all = state
        .keymap
        .lookup(KeyChord::new(KeyCode::Char('l'), KeyModifiers::CTRL_SHIFT));
    assert_eq!(select_all, Some(Action::SelectAllOccurrences));
    state.dispatch(select_all.unwrap()).unwrap();
    state.insert_text("gamma");
    assert_eq!(state.buffers.active().document.text(), "gamma beta gamma\n");

    let undo = state
        .keymap
        .lookup(KeyChord::new(KeyCode::Char('z'), KeyModifiers::CTRL));
    assert_eq!(undo, Some(Action::Undo));
    state.dispatch(undo.unwrap()).unwrap();
    assert_eq!(state.buffers.active().document.text(), "alpha beta alpha\n");

    let redo = state
        .keymap
        .lookup(KeyChord::new(KeyCode::Char('y'), KeyModifiers::CTRL));
    assert_eq!(redo, Some(Action::Redo));
    state.dispatch(redo.unwrap()).unwrap();
    assert_eq!(state.buffers.active().document.text(), "gamma beta gamma\n");

    let quit = state
        .keymap
        .lookup(KeyChord::new(KeyCode::Char('q'), KeyModifiers::CTRL));
    assert_eq!(quit, Some(Action::Quit));
    state.dispatch(quit.unwrap()).unwrap();
    assert_eq!(state.overlay, OverlayMode::ConfirmQuit);
    assert!(!state.should_quit);

    let save = state
        .keymap
        .lookup(KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL));
    assert_eq!(save, Some(Action::Save));
    state.dispatch(save.unwrap()).unwrap();

    let escape = state
        .keymap
        .lookup(KeyChord::new(KeyCode::Escape, KeyModifiers::NONE));
    assert_eq!(escape, Some(Action::Escape));
    state.dispatch(escape.unwrap()).unwrap();
    assert_eq!(state.overlay, OverlayMode::None);

    state.dispatch(quit.unwrap()).unwrap();
    assert!(state.should_quit);
}

#[test]
fn soft_wrap_layout_spell_quick_fix_and_preview_sync() {
    let mut state = EditorState::new(AppConfig::default()).unwrap();
    state.insert_text("# Heading\n\nCheck this documnt in the editor with soft wrap enabled.");

    let typo_pos = Position { line: 2, col: 12 };
    let layout = compute_wrapped_layout(&state.buffers.active().document, 24, true, 4);
    assert!(
        layout.len_visual_lines() > 3,
        "expected soft wrap to produce more than 3 visual lines, got {}",
        layout.len_visual_lines()
    );
    let (visual_row, visual_col) = layout.logical_to_visual(typo_pos);
    assert_eq!(layout.visual_to_logical(visual_row, visual_col), typo_pos);

    let typo_char = state.buffers.active().document.position_to_char(typo_pos);
    state
        .buffers
        .active_mut()
        .document
        .set_selection(SelectionSet::point(typo_char));

    state.dispatch(Action::OpenSpellQuickFix).unwrap();
    assert_eq!(state.overlay, OverlayMode::SpellQuickFix);

    let popup = state
        .spell_popup
        .as_mut()
        .expect("expected spell_popup to be populated");
    let doc_idx = popup
        .suggestions
        .iter()
        .position(|suggestion| suggestion == "document")
        .expect("expected 'document' in spell suggestions");
    popup.selected_index = doc_idx;

    assert!(state.apply_selected_spell_fix().unwrap());
    let updated_text = state.buffers.active().document.text();
    assert!(updated_text.contains("document"));
    assert!(!updated_text.contains("documnt"));

    let preview = state.build_active_preview(60);
    let plain = preview.plain_lines().join("\n");
    assert!(plain.contains("document"));
    assert!(preview.source_line_to_preview_line(2) >= preview.source_line_to_preview_line(0));
}

#[test]
fn readme_default_cua_keymap_covers_every_documented_shortcut() {
    let keymap = Keymap::default_cua();
    let expected = [
        // File, Buffer & Application
        (
            KeyChord::new(KeyCode::Char('n'), KeyModifiers::CTRL),
            Action::NewBuffer,
        ),
        (
            KeyChord::new(KeyCode::Char('o'), KeyModifiers::CTRL),
            Action::OpenFile,
        ),
        (
            KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL),
            Action::Save,
        ),
        (
            KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL_SHIFT),
            Action::SaveAs,
        ),
        (
            KeyChord::new(KeyCode::Char('w'), KeyModifiers::CTRL),
            Action::CloseBuffer,
        ),
        (
            KeyChord::new(KeyCode::PageDown, KeyModifiers::CTRL),
            Action::NextBuffer,
        ),
        (
            KeyChord::new(KeyCode::PageUp, KeyModifiers::CTRL),
            Action::PrevBuffer,
        ),
        (
            KeyChord::new(KeyCode::Char('q'), KeyModifiers::CTRL),
            Action::Quit,
        ),
        // Editing, History & Indentation
        (
            KeyChord::new(KeyCode::Enter, KeyModifiers::NONE),
            Action::InsertNewline,
        ),
        (
            KeyChord::new(KeyCode::Backspace, KeyModifiers::NONE),
            Action::Backspace,
        ),
        (
            KeyChord::new(KeyCode::Delete, KeyModifiers::NONE),
            Action::Delete,
        ),
        (
            KeyChord::new(KeyCode::Tab, KeyModifiers::NONE),
            Action::Indent,
        ),
        (
            KeyChord::new(KeyCode::Tab, KeyModifiers::SHIFT),
            Action::Dedent,
        ),
        (
            KeyChord::new(KeyCode::BackTab, KeyModifiers::NONE),
            Action::Dedent,
        ),
        (
            KeyChord::new(KeyCode::Char('c'), KeyModifiers::CTRL),
            Action::Copy,
        ),
        (
            KeyChord::new(KeyCode::Char('x'), KeyModifiers::CTRL),
            Action::Cut,
        ),
        (
            KeyChord::new(KeyCode::Char('v'), KeyModifiers::CTRL),
            Action::Paste,
        ),
        (
            KeyChord::new(KeyCode::Char('a'), KeyModifiers::CTRL),
            Action::SelectAll,
        ),
        (
            KeyChord::new(KeyCode::Char('z'), KeyModifiers::CTRL),
            Action::Undo,
        ),
        (
            KeyChord::new(KeyCode::Char('y'), KeyModifiers::CTRL),
            Action::Redo,
        ),
        (
            KeyChord::new(KeyCode::Char('z'), KeyModifiers::CTRL_SHIFT),
            Action::Redo,
        ),
        (
            KeyChord::new(KeyCode::Escape, KeyModifiers::NONE),
            Action::Escape,
        ),
        // Find, Replace & Multi-Cursor
        (
            KeyChord::new(KeyCode::Char('f'), KeyModifiers::CTRL),
            Action::OpenFind,
        ),
        (
            KeyChord::new(KeyCode::Char('h'), KeyModifiers::CTRL),
            Action::OpenReplace,
        ),
        (
            KeyChord::new(KeyCode::F(3), KeyModifiers::NONE),
            Action::FindNext,
        ),
        (
            KeyChord::new(KeyCode::F(3), KeyModifiers::SHIFT),
            Action::FindPrev,
        ),
        (
            KeyChord::new(KeyCode::Char('l'), KeyModifiers::CTRL_SHIFT),
            Action::SelectAllOccurrences,
        ),
        // Overlays & Preview
        (
            KeyChord::new(KeyCode::Char('p'), KeyModifiers::CTRL),
            Action::OpenCommandPalette,
        ),
        (
            KeyChord::new(KeyCode::F(1), KeyModifiers::NONE),
            Action::OpenCommandPalette,
        ),
        (
            KeyChord::new(KeyCode::Char('.'), KeyModifiers::CTRL),
            Action::OpenSpellQuickFix,
        ),
        (
            KeyChord::new(KeyCode::Char('p'), KeyModifiers::CTRL_SHIFT),
            Action::TogglePreview,
        ),
        (
            KeyChord::new(KeyCode::Char('p'), KeyModifiers::ALT),
            Action::ExpandPreview,
        ),
        (
            KeyChord::new(KeyCode::Char('z'), KeyModifiers::ALT),
            Action::ToggleSoftWrap,
        ),
        // Navigation & Selection
        (
            KeyChord::new(KeyCode::Left, KeyModifiers::NONE),
            Action::MoveLeft,
        ),
        (
            KeyChord::new(KeyCode::Right, KeyModifiers::NONE),
            Action::MoveRight,
        ),
        (
            KeyChord::new(KeyCode::Up, KeyModifiers::NONE),
            Action::MoveUp,
        ),
        (
            KeyChord::new(KeyCode::Down, KeyModifiers::NONE),
            Action::MoveDown,
        ),
        (
            KeyChord::new(KeyCode::Home, KeyModifiers::NONE),
            Action::MoveHome,
        ),
        (
            KeyChord::new(KeyCode::End, KeyModifiers::NONE),
            Action::MoveEnd,
        ),
        (
            KeyChord::new(KeyCode::PageUp, KeyModifiers::NONE),
            Action::PageUp,
        ),
        (
            KeyChord::new(KeyCode::PageDown, KeyModifiers::NONE),
            Action::PageDown,
        ),
        (
            KeyChord::new(KeyCode::Left, KeyModifiers::SHIFT),
            Action::SelectLeft,
        ),
        (
            KeyChord::new(KeyCode::Right, KeyModifiers::SHIFT),
            Action::SelectRight,
        ),
        (
            KeyChord::new(KeyCode::Up, KeyModifiers::SHIFT),
            Action::SelectUp,
        ),
        (
            KeyChord::new(KeyCode::Down, KeyModifiers::SHIFT),
            Action::SelectDown,
        ),
        (
            KeyChord::new(KeyCode::Home, KeyModifiers::SHIFT),
            Action::SelectHome,
        ),
        (
            KeyChord::new(KeyCode::End, KeyModifiers::SHIFT),
            Action::SelectEnd,
        ),
    ];

    for (chord, expected_action) in expected {
        assert_eq!(
            keymap.lookup(chord),
            Some(expected_action),
            "expected chord {chord:?} to map to {expected_action:?}"
        );
    }
}

#[test]
fn config_flags_control_spellcheck_preview_split_and_soft_wrap() {
    let dir = tempfile::tempdir().unwrap();

    let missing_path = dir.path().join("missing.toml");
    let default_cfg = AppConfig::load_or_default(&missing_path).unwrap();
    assert_eq!(default_cfg, AppConfig::default());

    let partial_path = dir.path().join("partial.toml");
    fs::write(
        &partial_path,
        "spell_enabled = false\npreview_split_percent = 35\n",
    )
    .unwrap();
    let partial_cfg = AppConfig::load_or_default(&partial_path).unwrap();
    assert_eq!(partial_cfg.theme, "crt_amber");
    assert_eq!(partial_cfg.spell_locale, "en_US");
    assert!(!partial_cfg.spell_enabled);
    assert!(partial_cfg.line_numbers);
    assert!(partial_cfg.soft_wrap);
    assert_eq!(partial_cfg.preview_split_percent, 35);

    let invalid_path = dir.path().join("invalid.toml");
    fs::write(&invalid_path, "preview_split_percent = \"not_a_number\"\n").unwrap();
    let invalid_res = AppConfig::load_or_default(&invalid_path);
    assert!(matches!(invalid_res, Err(ScriptuiError::TomlParse { .. })));

    let mut state = EditorState::new(partial_cfg).unwrap();
    state.insert_text("hello wrld");
    let (_, diagnostics) = state.active_analysis();
    assert!(
        diagnostics.is_empty(),
        "expected zero spell diagnostics when spell_enabled = false, got {diagnostics:?}"
    );

    assert_eq!(state.preview_mode, PreviewMode::Collapsed);
    state.dispatch(Action::TogglePreview).unwrap();
    assert_eq!(state.preview_mode, PreviewMode::Split(35));

    assert!(state.config.soft_wrap);
    state.dispatch(Action::ToggleSoftWrap).unwrap();
    assert!(!state.config.soft_wrap);
    state.dispatch(Action::ToggleSoftWrap).unwrap();
    assert!(state.config.soft_wrap);

    assert!(state.config.line_numbers);
    state.dispatch(Action::ToggleLineNumbers).unwrap();
    assert!(!state.config.line_numbers);
    state.dispatch(Action::ToggleLineNumbers).unwrap();
    assert!(state.config.line_numbers);
}

#[test]
fn editor_navigation_selection_cut_find_prev_and_escape_collapse() {
    let mut lines = vec![
        "alpha beta alpha".to_string(),
        "gamma delta".to_string(),
        "alpha omega".to_string(),
    ];
    for idx in 3..25 {
        lines.push(format!("line {idx}"));
    }
    let initial = lines.join("\n");

    let mut state = EditorState::new(AppConfig::default()).unwrap();
    state.insert_text(&initial);

    // SelectAll selects the entire buffer.
    state.dispatch(Action::SelectAll).unwrap();
    let total_chars = state.buffers.active().document.len_chars();
    let all_sel = state.buffers.active().document.selection().primary();
    assert_eq!(all_sel.start(), 0);
    assert_eq!(all_sel.end(), total_chars);

    // MoveHome / MoveEnd / PageDown / PageUp
    let start_pos = state
        .buffers
        .active()
        .document
        .position_to_char(Position { line: 0, col: 4 });
    state
        .buffers
        .active_mut()
        .document
        .set_selection(SelectionSet::point(start_pos));
    state.dispatch(Action::MoveHome).unwrap();
    let pos = state
        .buffers
        .active()
        .document
        .char_to_position(state.buffers.active().document.selection().primary().head);
    assert_eq!(pos, Position { line: 0, col: 0 });

    state.dispatch(Action::MoveEnd).unwrap();
    let pos = state
        .buffers
        .active()
        .document
        .char_to_position(state.buffers.active().document.selection().primary().head);
    assert_eq!(pos, Position { line: 0, col: 16 });

    state.dispatch(Action::PageDown).unwrap();
    let after_page_down = state
        .buffers
        .active()
        .document
        .char_to_position(state.buffers.active().document.selection().primary().head);
    assert!(after_page_down.line > 0);

    state.dispatch(Action::PageUp).unwrap();
    let after_page_up = state
        .buffers
        .active()
        .document
        .char_to_position(state.buffers.active().document.selection().primary().head);
    assert_eq!(after_page_up.line, 0);

    // Selection extension: SelectRight, SelectLeft, SelectDown, SelectUp, SelectHome, SelectEnd
    state
        .buffers
        .active_mut()
        .document
        .set_selection(SelectionSet::point(0));
    state.dispatch(Action::SelectRight).unwrap();
    state.dispatch(Action::SelectRight).unwrap();
    assert_eq!(
        state.buffers.active().document.selection().primary().head,
        2
    );
    state.dispatch(Action::SelectLeft).unwrap();
    assert_eq!(
        state.buffers.active().document.selection().primary().head,
        1
    );
    state.dispatch(Action::SelectDown).unwrap();
    let down_pos = state
        .buffers
        .active()
        .document
        .char_to_position(state.buffers.active().document.selection().primary().head);
    assert_eq!(down_pos, Position { line: 1, col: 1 });
    state.dispatch(Action::SelectUp).unwrap();
    let up_pos = state
        .buffers
        .active()
        .document
        .char_to_position(state.buffers.active().document.selection().primary().head);
    assert_eq!(up_pos, Position { line: 0, col: 1 });
    state.dispatch(Action::SelectHome).unwrap();
    assert_eq!(
        state.buffers.active().document.selection().primary().head,
        0
    );
    state.dispatch(Action::SelectEnd).unwrap();
    let end_sel = state.buffers.active().document.selection().primary();
    assert_eq!(end_sel.start(), 0);
    assert_eq!(end_sel.end(), 16);

    // Cut removes the active selection.
    state.dispatch(Action::Cut).unwrap();
    assert_eq!(
        state.buffers.active().document.line_text(0).as_deref(),
        Some("")
    );
    state.dispatch(Action::Undo).unwrap();
    assert_eq!(state.buffers.active().document.text(), initial);

    // FindNext / FindPrev wrap-around across all three "alpha" matches.
    state
        .buffers
        .active_mut()
        .document
        .set_selection(SelectionSet::point(0));
    state.find_replace.query = SearchQuery::plain("alpha");

    state.dispatch(Action::FindNext).unwrap();
    assert_eq!(state.find_replace.active_match, Some(0));
    state.dispatch(Action::FindNext).unwrap();
    assert_eq!(state.find_replace.active_match, Some(1));
    state.dispatch(Action::FindNext).unwrap();
    assert_eq!(state.find_replace.active_match, Some(2));
    state.dispatch(Action::FindNext).unwrap();
    assert_eq!(state.find_replace.active_match, Some(0));

    state.dispatch(Action::FindPrev).unwrap();
    assert_eq!(state.find_replace.active_match, Some(2));
    state.dispatch(Action::FindPrev).unwrap();
    assert_eq!(state.find_replace.active_match, Some(1));
    state.dispatch(Action::FindPrev).unwrap();
    assert_eq!(state.find_replace.active_match, Some(0));

    // Escape collapses multi-cursor selection back to a single cursor.
    state.dispatch(Action::SelectAllOccurrences).unwrap();
    assert_eq!(state.buffers.active().document.selection().len(), 3);
    state.dispatch(Action::Escape).unwrap();
    assert_eq!(state.buffers.active().document.selection().len(), 1);
}
