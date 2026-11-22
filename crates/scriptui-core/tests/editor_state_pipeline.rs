use std::fs;

use scriptui_core::{
    compute_wrapped_layout, Action, AppConfig, EditorState, KeyChord, KeyCode, KeyModifiers,
    OverlayMode, Position, SearchQuery, SelectionSet,
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
