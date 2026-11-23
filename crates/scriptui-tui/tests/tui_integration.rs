use std::collections::HashSet;
use std::process::Command;
use std::time::{Duration, Instant};

use clap::error::ErrorKind;
use clap::Parser;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Terminal;
use scriptui_core::theme::DEFAULT_CRT_AMBER_TOML;
use scriptui_core::{
    builtin_theme_names, resolve_theme, AppConfig, DocumentBuffer, EditorState, OverlayMode,
    PreviewMode, RgbColor, SelectionSet,
};
use scriptui_tui::{
    build_palette_entries, discover_installed_locales, execute_dict_import, handle_palette_key,
    rgb_to_color, Cli, FindBarState, FindFocus, MouseRouter, PreviewPaneState, TuiApp,
};

fn make_cli(theme: Option<&str>, locale: Option<&str>) -> Cli {
    Cli {
        command: None,
        theme: theme.map(ToString::to_string),
        locale: locale.map(ToString::to_string),
        files: Vec::new(),
    }
}

fn row_text(buf: &Buffer, y: u16) -> String {
    let area = buf.area;
    (area.x..area.right())
        .map(|x| buf[(x, y)].symbol())
        .collect()
}

fn buffer_text(buf: &Buffer) -> String {
    let area = buf.area;
    (area.y..area.bottom())
        .map(|y| row_text(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

fn press_key(app: &mut TuiApp, code: KeyCode, modifiers: KeyModifiers) {
    let mut clipboard = Vec::new();
    app.handle_key_event(KeyEvent::new(code, modifiers), &mut clipboard)
        .expect("handle key event");
}

fn find_char_x_in_row(buf: &Buffer, y: u16, x_start: u16, x_end: u16, ch: &str) -> Option<u16> {
    (x_start..x_end).find(|&x| buf[(x, y)].symbol() == ch)
}

#[test]
fn all_five_builtin_palettes_render_with_distinct_colors_and_crt_amber_gfm_differentials() {
    let names = builtin_theme_names();
    assert_eq!(names.len(), 5, "expected exactly 5 builtin themes");

    let gfm_sample = "# H1 Title\n## H2 Sub\n**bold** *italic* `code`\n- [x] task\n";
    let mut seen_backgrounds = HashSet::new();

    for &theme_name in names {
        let mut app = TuiApp::from_cli(&make_cli(Some(theme_name), None))
            .expect("initialize app with builtin theme");
        app.state.insert_text(gfm_sample);

        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).expect("create test terminal");
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render frame");

        let last_frame = app.last_frame.as_ref().expect("frame state populated");
        let status_y = last_frame.regions.status_bar.y;
        let editor_rect = last_frame.regions.editor.expect("editor region present");
        let viewport = last_frame.viewport.expect("viewport metrics present");
        let buf = terminal.backend().buffer();

        let status_line = row_text(buf, status_y);
        assert!(
            status_line.contains(theme_name),
            "status bar for '{theme_name}' did not contain theme name: {status_line}"
        );

        let bg = app.state.theme.palette.background;
        let expected_bg = rgb_to_color(bg);
        seen_backgrounds.insert((bg.r, bg.g, bg.b));
        let bg_sample_x = editor_rect.right().saturating_sub(1);
        let bg_sample_y = editor_rect.y.saturating_add(5);
        assert_eq!(
            buf[(bg_sample_x, bg_sample_y)].bg,
            expected_bg,
            "background cell mismatch for theme '{theme_name}'"
        );

        if theme_name == "crt_amber" {
            let palette = &app.state.theme.palette;
            let text_x_start = viewport.text_area.x;
            let text_x_end = viewport.text_area.right();

            let h1_y = viewport.text_area.y;
            let h1_x = find_char_x_in_row(buf, h1_y, text_x_start, text_x_end, "H")
                .expect("locate H1 text cell");
            let h1_fg = buf[(h1_x, h1_y)].fg;
            assert_eq!(
                h1_fg,
                rgb_to_color(palette.md_h1),
                "CRT Amber H1 cell fg should match md_h1"
            );

            let h2_y = viewport.text_area.y + 1;
            let h2_x = find_char_x_in_row(buf, h2_y, text_x_start, text_x_end, "H")
                .expect("locate H2 text cell");
            let h2_fg = buf[(h2_x, h2_y)].fg;
            assert_eq!(
                h2_fg,
                rgb_to_color(palette.md_h2),
                "CRT Amber H2 cell fg should match md_h2"
            );
            assert_ne!(h1_fg, h2_fg, "H1 and H2 fg colors should differ");

            let code_y = viewport.text_area.y + 2;
            let code_x = find_char_x_in_row(buf, code_y, text_x_start, text_x_end, "`")
                .expect("locate inline code cell")
                + 1;
            assert_eq!(buf[(code_x, code_y)].symbol(), "c");
            let code_fg = buf[(code_x, code_y)].fg;
            assert_eq!(
                code_fg,
                rgb_to_color(palette.md_code_inline),
                "CRT Amber inline code cell fg should match md_code_inline"
            );
            assert_ne!(h1_fg, code_fg, "H1 and inline code fg colors should differ");
            assert_ne!(h2_fg, code_fg, "H2 and inline code fg colors should differ");
        }
    }

    assert_eq!(
        seen_backgrounds.len(),
        5,
        "all 5 builtin palettes should have distinct background RGB colors"
    );
}

#[test]
fn preview_modes_sync_scroll_and_independent_wheel_scroll() {
    let mut app =
        TuiApp::from_cli(&make_cli(Some("crt_amber"), None)).expect("initialize default app");
    let mut doc = String::new();
    for i in 1..=40 {
        doc.push_str(&format!(
            "## Section {i}\n\nParagraph content for section {i} with **bold** and `code`.\n\n"
        ));
    }
    app.state.buffers.active_mut().document = DocumentBuffer::from_str_content(&doc);

    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).expect("create test terminal");

    // 1. Collapsed mode: edge tab '◀' visible, preview pane hidden.
    assert_eq!(app.state.preview_mode, PreviewMode::Collapsed);
    terminal
        .draw(|frame| app.render_frame(frame))
        .expect("render collapsed");
    let collapsed_screen = buffer_text(terminal.backend().buffer());
    assert!(
        collapsed_screen.contains('◀'),
        "expected collapsed edge tab '◀' in:\n{collapsed_screen}"
    );
    {
        let frame_state = app.last_frame.as_ref().expect("frame state");
        assert!(frame_state.regions.preview_edge_tab.is_some());
        assert!(frame_state.regions.preview.is_none());
    }

    // 2. Split mode via Ctrl+Shift+P: split divider + "Preview [SYNC]" visible.
    press_key(
        &mut app,
        KeyCode::Char('P'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(
        matches!(app.state.preview_mode, PreviewMode::Split(_)),
        "expected PreviewMode::Split, got {:?}",
        app.state.preview_mode
    );
    terminal
        .draw(|frame| app.render_frame(frame))
        .expect("render split");
    let split_screen = buffer_text(terminal.backend().buffer());
    assert!(
        split_screen.contains("Preview [SYNC]"),
        "expected 'Preview [SYNC]' in split mode:\n{split_screen}"
    );
    let preview_rect = {
        let frame_state = app.last_frame.as_ref().expect("frame state");
        let div = frame_state
            .regions
            .split_divider
            .expect("split divider present");
        let div_sym = terminal.backend().buffer()[(div.x, div.y)].symbol();
        assert_eq!(div_sym, "│", "expected split divider glyph");
        frame_state.regions.preview.expect("preview region present")
    };

    // 3. Wheel scroll over preview region disables sync and clamps scroll.
    let wheel_down = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: preview_rect.x.saturating_add(2),
        row: preview_rect.y.saturating_add(2),
        modifiers: KeyModifiers::NONE,
    };
    let handled = app
        .handle_mouse_event(wheel_down)
        .expect("handle preview wheel down");
    assert!(handled, "wheel scroll over preview should be handled");
    assert!(
        !app.preview_pane.sync_enabled,
        "wheel scroll over preview should disable sync"
    );
    assert!(
        app.preview_pane.scroll_line > 0,
        "wheel scroll down should advance preview scroll_line"
    );

    for _ in 0..500 {
        let _ = app
            .handle_mouse_event(wheel_down)
            .expect("handle repeated preview wheel down");
    }
    let max_scroll = app
        .preview_pane
        .total_lines
        .saturating_sub(app.preview_pane.viewport_height.max(1));
    assert_eq!(
        app.preview_pane.scroll_line, max_scroll,
        "preview scroll should clamp at total_lines - viewport_height"
    );

    // 4. Expanded mode via Alt+P: full-width preview and sync re-enabled.
    press_key(&mut app, KeyCode::Char('p'), KeyModifiers::ALT);
    assert_eq!(app.state.preview_mode, PreviewMode::Expanded);
    assert!(
        app.preview_pane.sync_enabled,
        "expanding preview should re-enable sync"
    );
    terminal
        .draw(|frame| app.render_frame(frame))
        .expect("render expanded");
    {
        let frame_state = app.last_frame.as_ref().expect("frame state");
        assert!(frame_state.regions.editor.is_none());
        let expanded_preview = frame_state
            .regions
            .preview
            .expect("expanded preview region");
        assert_eq!(expanded_preview.width, 100);
    }

    // 5. Disabling sync and toggling preview re-enables sync.
    let expanded_wheel = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };
    let _ = app
        .handle_mouse_event(expanded_wheel)
        .expect("scroll expanded preview");
    assert!(!app.preview_pane.sync_enabled);

    press_key(
        &mut app,
        KeyCode::Char('P'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert!(
        app.preview_pane.sync_enabled,
        "toggling preview should re-enable sync"
    );
}

#[test]
fn multicursor_find_handoff_and_prose_safe_indentation_e2e() {
    // Part A: Ctrl+F -> "item" -> Alt+Enter multi-cursor handoff -> replacement -> Ctrl+Z undo.
    let mut app = TuiApp::from_cli(&make_cli(None, None)).expect("initialize app");
    let original = "alpha item beta item gamma item";
    app.state.buffers.active_mut().document = DocumentBuffer::from_str_content(original);

    press_key(&mut app, KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::Find);

    for ch in ['i', 't', 'e', 'm'] {
        press_key(&mut app, KeyCode::Char(ch), KeyModifiers::NONE);
    }
    assert_eq!(app.state.find_replace.matches.len(), 3);

    press_key(&mut app, KeyCode::Enter, KeyModifiers::ALT);
    assert_eq!(app.state.overlay, OverlayMode::None);
    assert_eq!(app.state.buffers.active().document.selection().len(), 3);

    press_key(&mut app, KeyCode::Char('X'), KeyModifiers::NONE);
    assert_eq!(
        app.state.buffers.active().document.text(),
        "alpha X beta X gamma X"
    );

    press_key(&mut app, KeyCode::Char('z'), KeyModifiers::CONTROL);
    assert_eq!(
        app.state.buffers.active().document.text(),
        original,
        "Ctrl+Z should atomically undo multi-cursor replacement"
    );

    // Part B: Prose-safe indentation: "- first" + Enter -> "- ", second Enter clears, Tab indents, BackTab+SHIFT dedents.
    let mut prose_app = TuiApp::from_cli(&make_cli(None, None)).expect("initialize prose app");
    prose_app.state.insert_text("- first");

    press_key(&mut prose_app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        prose_app.state.buffers.active().document.text(),
        "- first\n- "
    );

    press_key(&mut prose_app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        prose_app.state.buffers.active().document.text(),
        "- first\n"
    );

    press_key(&mut prose_app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(
        prose_app.state.buffers.active().document.text(),
        "- first\n    "
    );

    press_key(&mut prose_app, KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(
        prose_app.state.buffers.active().document.text(),
        "- first\n"
    );
}

#[test]
fn custom_hunspell_locale_import_discovery_and_palette_switch() {
    let src_dir = tempfile::tempdir().expect("create source temp dir");
    let aff_src = src_dir.path().join("xx_TEST.aff");
    let dic_src = src_dir.path().join("xx_TEST.dic");
    std::fs::write(&aff_src, "SET UTF-8\n").expect("write .aff");
    std::fs::write(&dic_src, "2\nzorkle\nhello\n").expect("write .dic");

    let dicts_dir = tempfile::tempdir().expect("create dicts temp dir");
    let (dic_dest, aff_dest) = execute_dict_import("xx_TEST", &dic_src, &aff_src, dicts_dir.path())
        .expect("import custom locale");
    assert!(dic_dest.is_file());
    assert!(aff_dest.is_file());

    let dict_dirs = [dicts_dir.path().to_path_buf()];
    let locales = discover_installed_locales(&dict_dirs);
    assert!(
        locales.iter().any(|loc| loc == "xx_TEST"),
        "expected discovered locales {locales:?} to contain xx_TEST"
    );

    let entries = build_palette_entries(&locales);
    let mut app = TuiApp::from_cli(&make_cli(None, None)).expect("initialize app");
    app.state.insert_text("zorkle");

    app.state.overlay = OverlayMode::CommandPalette;
    for ch in "xx_TEST".chars() {
        let handled = handle_palette_key(
            KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
            &mut app.state,
            &entries,
            &[],
            &dict_dirs,
        )
        .expect("type locale query");
        assert!(handled);
    }

    let handled = handle_palette_key(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        &mut app.state,
        &entries,
        &[],
        &dict_dirs,
    )
    .expect("confirm locale switch");
    assert!(handled);
    assert_eq!(app.state.overlay, OverlayMode::None);
    assert_eq!(app.state.config.spell_locale, "xx_TEST");

    let (_, diagnostics) = app.state.active_analysis();
    assert!(
        diagnostics.is_empty(),
        "expected zero spell diagnostics for 'zorkle' under xx_TEST, got {diagnostics:?}"
    );
}

#[test]
fn large_file_cached_analysis_and_viewport_render_responsiveness() {
    let mut large_doc = String::with_capacity(5_000 * 48);
    for i in 1..=1_670 {
        large_doc.push_str(&format!(
            "# Section {i}\nParagraph with **bold** and `code`.\n\n"
        ));
    }

    let mut app = TuiApp::from_cli(&make_cli(Some("crt_amber"), None)).expect("initialize app");
    app.state.buffers.active_mut().document = DocumentBuffer::from_str_content(&large_doc);
    assert!(
        app.state.buffers.active().document.len_lines() >= 5_000,
        "expected >= 5000 lines"
    );

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).expect("create test terminal");

    // Initial render populates the cached GFM/spell analysis and frame metrics.
    terminal
        .draw(|frame| app.render_frame(frame))
        .expect("render initial large-file frame");
    assert_eq!(
        app.state.analysis_recompute_count(),
        1,
        "initial render should compute analysis exactly once"
    );

    let editor_rect = app
        .last_frame
        .as_ref()
        .and_then(|f| f.regions.editor)
        .expect("editor region present");
    let scroll_down = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: editor_rect.x.saturating_add(5),
        row: editor_rect.y.saturating_add(5),
        modifiers: KeyModifiers::NONE,
    };

    let start = Instant::now();
    for _ in 0..10 {
        let handled = app
            .handle_mouse_event(scroll_down)
            .expect("handle editor wheel scroll");
        assert!(handled);
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render scrolled frame");
    }
    let elapsed = start.elapsed();

    assert_eq!(
        app.state.analysis_recompute_count(),
        1,
        "scrolling should trigger zero redundant GFM/spell recomputes"
    );
    assert!(
        app.state.buffers.active().scroll_line > 0,
        "expected viewport to scroll down"
    );
    assert!(
        elapsed < Duration::from_millis(500),
        "10 scroll frames took {elapsed:?}, expected < 500ms"
    );
}

#[test]
fn cli_version_matches_workspace_version() {
    let err = Cli::try_parse_from(["scriptui", "--version"])
        .expect_err("expected --version to exit with DisplayVersion");
    assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    assert_eq!(
        err.to_string().trim(),
        format!("scriptui {}", env!("CARGO_PKG_VERSION"))
    );
}

fn type_text(app: &mut TuiApp, text: &str) {
    for ch in text.chars() {
        press_key(app, KeyCode::Char(ch), KeyModifiers::NONE);
    }
}

fn press_key_with_clipboard(app: &mut TuiApp, code: KeyCode, modifiers: KeyModifiers) -> String {
    let mut clipboard = Vec::new();
    app.handle_key_event(KeyEvent::new(code, modifiers), &mut clipboard)
        .expect("handle key event");
    String::from_utf8(clipboard).expect("clipboard output should be valid utf-8")
}

fn rect_has_non_empty_cell(buf: &Buffer, rect: Rect) -> bool {
    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            if !buf[(x, y)].symbol().trim().is_empty() {
                return true;
            }
        }
    }
    false
}

#[test]
fn tui_file_buffer_lifecycle_and_dirty_confirm_modals() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let first_path = dir.path().join("first.md");
    let second_path = dir.path().join("second.md");
    std::fs::write(&second_path, "second file body").expect("seed second file");

    let mut app = TuiApp::from_cli(&make_cli(None, None)).expect("initialize app");
    assert_eq!(app.state.buffers.len(), 1);
    assert_eq!(app.state.buffers.active_index(), 0);

    // Ctrl+N creates a new buffer and activates it.
    press_key(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(app.state.buffers.len(), 2);
    assert_eq!(app.state.buffers.active_index(), 1);

    // Ctrl+PageUp / Ctrl+PageDown cycle tabs.
    press_key(&mut app, KeyCode::PageUp, KeyModifiers::CONTROL);
    assert_eq!(app.state.buffers.active_index(), 0);
    press_key(&mut app, KeyCode::PageDown, KeyModifiers::CONTROL);
    assert_eq!(app.state.buffers.active_index(), 1);

    // Type content and Save As via Ctrl+Shift+S.
    type_text(&mut app, "hello");
    assert!(app.state.buffers.active().document.is_dirty());

    press_key(
        &mut app,
        KeyCode::Char('S'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert_eq!(app.state.overlay, OverlayMode::SaveAsPrompt);
    type_text(&mut app, first_path.to_str().expect("utf-8 first path"));
    press_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);

    assert_eq!(app.state.overlay, OverlayMode::None);
    assert!(!app.state.buffers.active().document.is_dirty());
    assert_eq!(
        std::fs::read_to_string(&first_path).expect("read saved file"),
        "hello"
    );

    // Further edit + Ctrl+S saves updated content directly.
    type_text(&mut app, " world");
    assert!(app.state.buffers.active().document.is_dirty());
    press_key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(!app.state.buffers.active().document.is_dirty());
    assert_eq!(
        std::fs::read_to_string(&first_path).expect("read re-saved file"),
        "hello world"
    );

    // Ctrl+O opens OpenFilePrompt and loads second temp file.
    press_key(&mut app, KeyCode::Char('o'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::OpenFilePrompt);
    type_text(&mut app, second_path.to_str().expect("utf-8 second path"));
    press_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);

    assert_eq!(app.state.overlay, OverlayMode::None);
    assert_eq!(app.state.buffers.len(), 3);
    assert_eq!(
        app.state.buffers.active().document.text(),
        "second file body"
    );

    // Dirty edit + Ctrl+W triggers ConfirmCloseBuffer; 'n' cancels, 'y' closes.
    type_text(&mut app, "!");
    assert!(app.state.buffers.active().document.is_dirty());
    press_key(&mut app, KeyCode::Char('w'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::ConfirmCloseBuffer);

    press_key(&mut app, KeyCode::Char('n'), KeyModifiers::NONE);
    assert_eq!(app.state.overlay, OverlayMode::None);
    assert_eq!(app.state.buffers.len(), 3);

    press_key(&mut app, KeyCode::Char('w'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::ConfirmCloseBuffer);
    press_key(&mut app, KeyCode::Char('y'), KeyModifiers::NONE);
    assert_eq!(app.state.overlay, OverlayMode::None);
    assert_eq!(app.state.buffers.len(), 2);

    // Dirty edit + Ctrl+Q triggers ConfirmQuit; 'n' cancels, 'y' sets should_quit.
    type_text(&mut app, "?");
    assert!(app.state.buffers.active().document.is_dirty());
    press_key(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::ConfirmQuit);
    assert!(!app.state.should_quit);

    press_key(&mut app, KeyCode::Char('n'), KeyModifiers::NONE);
    assert_eq!(app.state.overlay, OverlayMode::None);
    assert!(!app.state.should_quit);

    press_key(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::ConfirmQuit);
    press_key(&mut app, KeyCode::Char('y'), KeyModifiers::NONE);
    assert_eq!(app.state.overlay, OverlayMode::None);
    assert!(app.state.should_quit);
}

#[test]
fn tui_find_replace_bar_inline_shortcuts_and_f3_navigation() {
    let mut app = TuiApp::from_cli(&make_cli(None, None)).expect("initialize app");
    app.state.buffers.active_mut().document =
        DocumentBuffer::from_str_content("foo Foo foobar foo");

    // Ctrl+H opens Replace overlay with query focused.
    press_key(&mut app, KeyCode::Char('h'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::Replace);
    assert_eq!(app.find_bar.focus, FindFocus::Query);

    type_text(&mut app, "foo");
    assert_eq!(app.state.find_replace.matches.len(), 4);

    // Alt+C toggles case-sensitive matching.
    press_key(&mut app, KeyCode::Char('c'), KeyModifiers::ALT);
    assert!(app.state.find_replace.query.case_sensitive);
    assert_eq!(app.state.find_replace.matches.len(), 3);

    // Alt+W toggles whole-word matching.
    press_key(&mut app, KeyCode::Char('w'), KeyModifiers::ALT);
    assert!(app.state.find_replace.query.whole_word);
    assert_eq!(app.state.find_replace.matches.len(), 2);

    // Alt+R toggles regex mode.
    press_key(&mut app, KeyCode::Char('r'), KeyModifiers::ALT);
    assert!(app.state.find_replace.query.use_regex);
    assert_eq!(app.state.find_replace.matches.len(), 2);

    // Toggle flags back off to restore all 4 matches.
    press_key(&mut app, KeyCode::Char('r'), KeyModifiers::ALT);
    press_key(&mut app, KeyCode::Char('w'), KeyModifiers::ALT);
    press_key(&mut app, KeyCode::Char('c'), KeyModifiers::ALT);
    assert!(!app.state.find_replace.query.use_regex);
    assert!(!app.state.find_replace.query.whole_word);
    assert!(!app.state.find_replace.query.case_sensitive);
    assert_eq!(app.state.find_replace.matches.len(), 4);

    // Enter advances through matches in Find focus; Shift+Enter returns to previous.
    press_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.state.find_replace.active_match, Some(0));
    press_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.state.find_replace.active_match, Some(1));
    press_key(&mut app, KeyCode::Enter, KeyModifiers::SHIFT);
    assert_eq!(app.state.find_replace.active_match, Some(0));

    // Tab / Shift+Tab switch focus between Find and Replace inputs.
    press_key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.find_bar.focus, FindFocus::Replacement);
    press_key(&mut app, KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(app.find_bar.focus, FindFocus::Query);
    press_key(&mut app, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(app.find_bar.focus, FindFocus::Replacement);

    // Type replacement, Enter replaces active match, Ctrl+Enter replaces all remaining.
    type_text(&mut app, "baz");
    press_key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        app.state.buffers.active().document.text(),
        "baz Foo foobar foo"
    );

    press_key(&mut app, KeyCode::Enter, KeyModifiers::CONTROL);
    assert_eq!(
        app.state.buffers.active().document.text(),
        "baz baz bazbar baz"
    );
    assert!(app.state.find_replace.matches.is_empty());

    // Esc closes the Replace bar.
    press_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(app.state.overlay, OverlayMode::None);

    // Seed a Find query for "baz", close bar, then test F3 / Shift+F3 and Ctrl+Shift+L.
    press_key(&mut app, KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert_eq!(app.state.overlay, OverlayMode::Find);
    for _ in 0..3 {
        press_key(&mut app, KeyCode::Backspace, KeyModifiers::NONE);
    }
    type_text(&mut app, "baz");
    assert_eq!(app.state.find_replace.matches.len(), 4);
    press_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(app.state.overlay, OverlayMode::None);

    press_key(&mut app, KeyCode::F(3), KeyModifiers::NONE);
    let first_f3 = app.state.find_replace.active_match;
    press_key(&mut app, KeyCode::F(3), KeyModifiers::NONE);
    let second_f3 = app.state.find_replace.active_match;
    assert_ne!(second_f3, first_f3);

    press_key(&mut app, KeyCode::F(3), KeyModifiers::SHIFT);
    assert_eq!(app.state.find_replace.active_match, first_f3);

    // Ctrl+Shift+L selects all occurrences as multi-cursors.
    press_key(
        &mut app,
        KeyCode::Char('L'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert_eq!(app.state.buffers.active().document.selection().len(), 4);
}

#[test]
fn tui_editing_selection_osc52_cut_copy_redo_and_bracketed_paste() {
    let mut app = TuiApp::from_cli(&make_cli(None, None)).expect("initialize app");
    app.state.insert_text("alpha beta");

    // Ctrl+A selects all.
    press_key(&mut app, KeyCode::Char('a'), KeyModifiers::CONTROL);
    assert_eq!(app.state.selected_text(), "alpha beta");

    // Ctrl+C emits OSC 52 sequence without mutating text.
    let copy_osc52 = press_key_with_clipboard(&mut app, KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(
        copy_osc52.starts_with("\x1b]52;c;"),
        "expected OSC 52 copy sequence, got {copy_osc52:?}"
    );
    assert_eq!(app.state.buffers.active().document.text(), "alpha beta");

    // Ctrl+X emits OSC 52 sequence and deletes selection.
    let cut_osc52 = press_key_with_clipboard(&mut app, KeyCode::Char('x'), KeyModifiers::CONTROL);
    assert!(
        cut_osc52.starts_with("\x1b]52;c;"),
        "expected OSC 52 cut sequence, got {cut_osc52:?}"
    );
    assert_eq!(app.state.buffers.active().document.text(), "");

    // Bracketed paste inserts normalized text.
    app.handle_paste_event("one\r\ntwo")
        .expect("handle paste event");
    assert_eq!(app.state.buffers.active().document.text(), "one\ntwo");

    // Ctrl+Z undoes, Ctrl+Y redoes, Ctrl+Z undoes, Ctrl+Shift+Z redoes.
    press_key(&mut app, KeyCode::Char('z'), KeyModifiers::CONTROL);
    assert_eq!(app.state.buffers.active().document.text(), "");

    press_key(&mut app, KeyCode::Char('y'), KeyModifiers::CONTROL);
    assert_eq!(app.state.buffers.active().document.text(), "one\ntwo");

    press_key(&mut app, KeyCode::Char('z'), KeyModifiers::CONTROL);
    assert_eq!(app.state.buffers.active().document.text(), "");

    press_key(
        &mut app,
        KeyCode::Char('Z'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert_eq!(app.state.buffers.active().document.text(), "one\ntwo");

    // Shift+Right, Shift+End, Shift+Home extend selection.
    press_key(&mut app, KeyCode::Home, KeyModifiers::NONE);
    press_key(&mut app, KeyCode::Right, KeyModifiers::SHIFT);
    assert_eq!(app.state.selected_text(), "t");

    press_key(&mut app, KeyCode::End, KeyModifiers::SHIFT);
    assert_eq!(app.state.selected_text(), "two");

    press_key(&mut app, KeyCode::End, KeyModifiers::NONE);
    press_key(&mut app, KeyCode::Home, KeyModifiers::SHIFT);
    assert_eq!(app.state.selected_text(), "two");

    // Alt+Z toggles config.soft_wrap.
    let initial_wrap = app.state.config.soft_wrap;
    press_key(&mut app, KeyCode::Char('z'), KeyModifiers::ALT);
    assert_eq!(app.state.config.soft_wrap, !initial_wrap);
    press_key(&mut app, KeyCode::Char('z'), KeyModifiers::ALT);
    assert_eq!(app.state.config.soft_wrap, initial_wrap);

    // Esc collapses multi-cursor back to 1.
    app.state.buffers.active_mut().document = DocumentBuffer::from_str_content("x y x");
    press_key(&mut app, KeyCode::Char('f'), KeyModifiers::CONTROL);
    type_text(&mut app, "x");
    press_key(&mut app, KeyCode::Enter, KeyModifiers::ALT);
    assert_eq!(app.state.buffers.active().document.selection().len(), 2);

    press_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(app.state.buffers.active().document.selection().len(), 1);
}

#[test]
fn tui_mouse_click_drag_tab_right_click_spell_and_split_divider() {
    let mut app =
        TuiApp::from_cli(&make_cli(Some("crt_amber"), None)).expect("initialize default app");
    app.state.buffers.active_mut().document = DocumentBuffer::from_str_content("hello wrld");
    press_key(&mut app, KeyCode::Char('n'), KeyModifiers::CONTROL);
    press_key(&mut app, KeyCode::PageUp, KeyModifiers::CONTROL);
    assert_eq!(app.state.buffers.len(), 2);
    assert_eq!(app.state.buffers.active_index(), 0);

    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| app.render_frame(frame))
        .expect("render initial frame");

    let text_area = app
        .last_frame
        .as_ref()
        .and_then(|f| f.viewport)
        .expect("viewport present")
        .text_area;

    // 1. Left-click inside editor moves caret.
    let handled = app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: text_area.x + 2,
            row: text_area.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("handle editor left click");
    assert!(handled);
    assert_eq!(
        app.state
            .buffers
            .active()
            .document
            .selection()
            .primary()
            .head,
        2
    );
    let _ = app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: text_area.x + 2,
            row: text_area.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("handle mouse up");

    // 2. Left-click + drag + up creates non-empty selection.
    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: text_area.x + 1,
            row: text_area.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("drag start"));
    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: text_area.x + 5,
            row: text_area.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("drag move"));
    let _ = app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: text_area.x + 5,
            row: text_area.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("drag end");
    assert_eq!(app.state.selected_text(), "ello");

    // 3. Right-click on "wrld" moves caret and opens SpellQuickFix; Esc closes it.
    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: text_area.x + 7,
            row: text_area.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("right click misspelled word"));
    assert_eq!(
        app.state
            .buffers
            .active()
            .document
            .selection()
            .primary()
            .head,
        7
    );
    assert_eq!(app.state.overlay, OverlayMode::SpellQuickFix);
    assert!(app.state.spell_popup.is_some());

    press_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(app.state.overlay, OverlayMode::None);

    // 4. Left-click on second tab hitbox switches active tab.
    let second_tab_rect = app
        .last_frame
        .as_ref()
        .expect("frame state")
        .tab_hitboxes
        .get(1)
        .expect("second tab hitbox")
        .rect;
    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: second_tab_rect.x,
            row: second_tab_rect.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("click second tab"));
    assert_eq!(app.state.buffers.active_index(), 1);
    let _ = app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: second_tab_rect.x,
            row: second_tab_rect.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("release second tab");

    // 5. Left-click on preview edge tab opens PreviewMode::Split(_).
    let edge_tab = app
        .last_frame
        .as_ref()
        .and_then(|f| f.regions.preview_edge_tab)
        .expect("preview edge tab present");
    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: edge_tab.x,
            row: edge_tab.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("click preview edge tab"));
    assert!(matches!(app.state.preview_mode, PreviewMode::Split(_)));
    let _ = app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: edge_tab.x,
            row: edge_tab.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("release preview edge tab");

    // 6. Left-click + drag + up on split divider updates preview_split_percent clamped to 20..=80.
    terminal
        .draw(|frame| app.render_frame(frame))
        .expect("render split frame");
    let divider = app
        .last_frame
        .as_ref()
        .and_then(|f| f.regions.split_divider)
        .expect("split divider present");

    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: divider.x,
            row: divider.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("grab split divider"));

    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 30,
            row: divider.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("drag split divider to 70%"));
    assert_eq!(app.state.config.preview_split_percent, 70);
    assert_eq!(app.state.preview_mode, PreviewMode::Split(70));

    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 5,
            row: divider.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("drag split divider past upper clamp"));
    assert_eq!(app.state.config.preview_split_percent, 80);

    assert!(app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 95,
            row: divider.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("drag split divider past lower clamp"));
    assert_eq!(app.state.config.preview_split_percent, 20);

    let _ = app
        .handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 95,
            row: divider.y,
            modifiers: KeyModifiers::NONE,
        })
        .expect("release split divider");
}

#[test]
fn tui_narrow_viewport_and_long_lines_render_without_panic() {
    let mut app =
        TuiApp::from_cli(&make_cli(Some("crt_amber"), None)).expect("initialize default app");
    let long_line = format!("{} wrld {}", "a".repeat(97), "b".repeat(97));
    assert_eq!(long_line.chars().count(), 200);
    app.state.buffers.active_mut().document = DocumentBuffer::from_str_content(&long_line);

    let backend = TestBackend::new(40, 12);
    let mut terminal = Terminal::new(backend).expect("create narrow test terminal");

    for soft_wrap in [true, false] {
        app.state.config.soft_wrap = soft_wrap;
        app.state.overlay = OverlayMode::None;

        // Base render without overlay.
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render narrow base frame");
        {
            let frame_state = app.last_frame.as_ref().expect("frame state");
            let text_area = frame_state.viewport.expect("viewport").text_area;
            assert!(
                rect_has_non_empty_cell(terminal.backend().buffer(), text_area),
                "expected non-empty viewport cells (soft_wrap={soft_wrap})"
            );
        }

        // Overlay 1: Find bar.
        press_key(&mut app, KeyCode::Char('f'), KeyModifiers::CONTROL);
        assert_eq!(app.state.overlay, OverlayMode::Find);
        app.state.find_replace.query.pattern.clear();
        type_text(&mut app, "wrld");
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render narrow find overlay");
        {
            let frame_state = app.last_frame.as_ref().expect("frame state");
            let text_area = frame_state.viewport.expect("viewport").text_area;
            let find_rect = frame_state.regions.find_bar.expect("find bar region");
            let buf = terminal.backend().buffer();
            assert!(rect_has_non_empty_cell(buf, text_area));
            assert!(rect_has_non_empty_cell(buf, find_rect));
        }
        press_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);

        // Overlay 2: CommandPalette.
        press_key(&mut app, KeyCode::Char('p'), KeyModifiers::CONTROL);
        assert_eq!(app.state.overlay, OverlayMode::CommandPalette);
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render narrow command palette overlay");
        {
            let frame_state = app.last_frame.as_ref().expect("frame state");
            let text_area = frame_state.viewport.expect("viewport").text_area;
            let buf = terminal.backend().buffer();
            assert!(rect_has_non_empty_cell(buf, text_area));
            assert!(rect_has_non_empty_cell(buf, frame_state.total_area));
        }
        press_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);

        // Overlay 3: SpellQuickFix on "wrld" (char offset 98..102).
        app.state
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(99));
        press_key(&mut app, KeyCode::Char('.'), KeyModifiers::CONTROL);
        assert_eq!(app.state.overlay, OverlayMode::SpellQuickFix);
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render narrow spell quick-fix overlay");
        {
            let frame_state = app.last_frame.as_ref().expect("frame state");
            let text_area = frame_state.viewport.expect("viewport").text_area;
            let buf = terminal.backend().buffer();
            assert!(rect_has_non_empty_cell(buf, text_area));
            let rendered = buffer_text(buf);
            assert!(
                rendered.contains("wrld"),
                "expected spell quick-fix overlay to render 'wrld' in:\n{rendered}"
            );
        }
        press_key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    }
}

#[test]
fn cli_binary_version_help_and_dict_import_with_xdg_config_home() {
    let bin = env!("CARGO_BIN_EXE_scriptui");
    let temp_xdg = tempfile::tempdir().expect("create temp XDG_CONFIG_HOME");

    // 1. --version
    let version_out = Command::new(bin)
        .env("XDG_CONFIG_HOME", temp_xdg.path())
        .arg("--version")
        .output()
        .expect("run scriptui --version");
    assert!(
        version_out.status.success(),
        "expected --version to exit 0, stderr: {}",
        String::from_utf8_lossy(&version_out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&version_out.stdout),
        format!("scriptui {}\n", env!("CARGO_PKG_VERSION"))
    );

    // 2. --help
    let help_out = Command::new(bin)
        .env("XDG_CONFIG_HOME", temp_xdg.path())
        .arg("--help")
        .output()
        .expect("run scriptui --help");
    assert!(
        help_out.status.success(),
        "expected --help to exit 0, stderr: {}",
        String::from_utf8_lossy(&help_out.stderr)
    );
    let help_stdout = String::from_utf8_lossy(&help_out.stdout);
    assert!(
        help_stdout.contains("--theme"),
        "expected --help stdout to contain '--theme':\n{help_stdout}"
    );
    assert!(
        help_stdout.contains("--locale"),
        "expected --help stdout to contain '--locale':\n{help_stdout}"
    );
    assert!(
        help_stdout.contains("dict"),
        "expected --help stdout to contain 'dict':\n{help_stdout}"
    );

    // 3. dict import xx_CLI <dic> <aff>
    let src_dir = tempfile::tempdir().expect("create temp dict source dir");
    let dic_src = src_dir.path().join("xx_CLI.dic");
    let aff_src = src_dir.path().join("xx_CLI.aff");
    let dic_content = "2\nhello\nzorkle\n";
    let aff_content = "SET UTF-8\n";
    std::fs::write(&dic_src, dic_content).expect("write source .dic");
    std::fs::write(&aff_src, aff_content).expect("write source .aff");

    let import_out = Command::new(bin)
        .env("XDG_CONFIG_HOME", temp_xdg.path())
        .args(["dict", "import", "xx_CLI"])
        .arg(&dic_src)
        .arg(&aff_src)
        .output()
        .expect("run scriptui dict import xx_CLI");
    assert!(
        import_out.status.success(),
        "expected valid dict import to exit 0, stderr: {}",
        String::from_utf8_lossy(&import_out.stderr)
    );
    let import_stdout = String::from_utf8_lossy(&import_out.stdout);
    assert!(
        import_stdout.contains("Imported dictionary for 'xx_CLI'"),
        "unexpected dict import stdout: {import_stdout}"
    );

    let dicts_dir = temp_xdg.path().join("scriptui").join("dicts");
    let dic_dest = dicts_dir.join("xx_CLI.dic");
    let aff_dest = dicts_dir.join("xx_CLI.aff");
    assert!(dic_dest.is_file(), "expected imported .dic at {dic_dest:?}");
    assert!(aff_dest.is_file(), "expected imported .aff at {aff_dest:?}");
    assert_eq!(
        std::fs::read_to_string(&dic_dest).expect("read imported .dic"),
        dic_content
    );
    assert_eq!(
        std::fs::read_to_string(&aff_dest).expect("read imported .aff"),
        aff_content
    );

    // 4. dict import ../bad <dic> <aff> rejected
    let bad_out = Command::new(bin)
        .env("XDG_CONFIG_HOME", temp_xdg.path())
        .args(["dict", "import", "../bad"])
        .arg(&dic_src)
        .arg(&aff_src)
        .output()
        .expect("run scriptui dict import ../bad");
    assert!(
        !bad_out.status.success(),
        "expected traversal locale '../bad' to exit non-zero"
    );
    assert!(
        !bad_out.stderr.is_empty(),
        "expected non-empty stderr on invalid locale import"
    );
}

#[test]
fn cli_startup_opens_multiple_files_and_overrides_config() {
    let dir = tempfile::tempdir().expect("create temp dir for multi-file startup");
    let first_path = dir.path().join("first.md");
    let second_path = dir.path().join("second.md");
    std::fs::write(&first_path, "# First Document\n").expect("write first.md");
    std::fs::write(&second_path, "# Second Document\n").expect("write second.md");

    let cli = Cli {
        command: None,
        theme: Some("ice_cyan".to_string()),
        locale: Some("en_US".to_string()),
        files: vec![first_path.clone(), second_path.clone()],
    };
    let app = TuiApp::from_cli(&cli).expect("initialize TuiApp from CLI with two files");
    assert_eq!(app.state.theme.name, "ice_cyan");
    assert_eq!(app.state.config.spell_locale, "en_US");
    assert_eq!(app.state.buffers.len(), 2);
    assert_eq!(app.state.buffers.buffers()[0].display_name(), "first.md");
    assert_eq!(
        app.state.buffers.buffers()[0].document.text(),
        "# First Document\n"
    );
    assert_eq!(app.state.buffers.buffers()[1].display_name(), "second.md");
    assert_eq!(
        app.state.buffers.buffers()[1].document.text(),
        "# Second Document\n"
    );

    // Config-driven TUI rendering: line_numbers = false and spell_enabled = false.
    let config = AppConfig {
        line_numbers: false,
        spell_enabled: false,
        ..AppConfig::default()
    };
    let mut state = EditorState::new(config).expect("initialize EditorState with custom config");
    state.insert_text("hello wrld");
    let mut configured_app = TuiApp {
        state,
        mouse: MouseRouter::default(),
        last_frame: None,
        follow_cursor: true,
        preview_pane: PreviewPaneState::default(),
        find_bar: FindBarState::default(),
        palette_entries: Vec::new(),
    };

    let backend = TestBackend::new(80, 16);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| configured_app.render_frame(frame))
        .expect("render configured app");

    let viewport = configured_app
        .last_frame
        .as_ref()
        .and_then(|frame| frame.viewport)
        .expect("viewport metrics present");
    assert_eq!(
        viewport.gutter_width, 0,
        "expected gutter_width == 0 when line_numbers is false"
    );

    let (_, diagnostics) = configured_app.state.active_analysis();
    assert!(
        diagnostics.is_empty(),
        "expected empty spell diagnostics when spell_enabled is false, got {diagnostics:?}"
    );
}

#[test]
fn custom_toml_theme_overrides_builtin_and_validates_names() {
    let temp = tempfile::tempdir().expect("create temp dir for custom themes");
    let themes_dir = temp.path().join("themes");
    std::fs::create_dir_all(&themes_dir).expect("create themes directory");

    let custom_bg = RgbColor { r: 1, g: 2, b: 3 };
    let builtin_bg_line = "background = { r = 10, g = 10, b = 8 }";
    let custom_bg_line = "background = { r = 1, g = 2, b = 3 }";
    assert!(
        DEFAULT_CRT_AMBER_TOML.contains(builtin_bg_line),
        "expected DEFAULT_CRT_AMBER_TOML to contain built-in background line"
    );
    let custom_toml = DEFAULT_CRT_AMBER_TOML.replace(builtin_bg_line, custom_bg_line);
    std::fs::write(themes_dir.join("crt_amber.toml"), custom_toml)
        .expect("write custom crt_amber.toml");

    let resolved = resolve_theme("crt_amber", std::slice::from_ref(&themes_dir))
        .expect("resolve overridden crt_amber theme");
    assert_eq!(resolved.palette.background, custom_bg);
    assert_ne!(resolved.palette.background, RgbColor { r: 10, g: 10, b: 8 });

    let err = resolve_theme("../escape", std::slice::from_ref(&themes_dir))
        .expect_err("expected path traversal theme name to fail");
    assert!(
        !err.to_string().is_empty(),
        "expected non-empty error for invalid theme name"
    );
}
