use std::collections::HashSet;
use std::time::{Duration, Instant};

use clap::error::ErrorKind;
use clap::Parser;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;
use scriptui_core::{builtin_theme_names, DocumentBuffer, OverlayMode, PreviewMode};
use scriptui_tui::{
    build_palette_entries, discover_installed_locales, execute_dict_import, handle_palette_key,
    rgb_to_color, Cli, TuiApp,
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
    assert_eq!(err.to_string().trim(), "scriptui 0.1.0");
}
