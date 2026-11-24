use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::layout::Rect;
use ratatui::Frame;
use scriptui_core::{
    Action, AppConfig, EditorState, OverlayMode, ScriptuiError, ScriptuiResult, WrappedLayout,
};

use crate::chrome::{
    render_preview_edge_tab, render_split_divider, render_status_bar, render_tab_bar, TabHitbox,
};
use crate::cli::Cli;
use crate::find_bar::{handle_find_bar_key, render_find_bar, FindBarState, FindFocus};
use crate::layout_tree::{compute_screen_regions, ScreenRegions};
use crate::modals::{
    handle_confirm_modal_key, handle_file_prompt_key, handle_spell_quick_fix_key,
    render_confirm_modal, render_file_prompt, render_spell_quick_fix,
};
use crate::mouse::{is_over_preview, MouseContext, MouseRouter};
use crate::palette::{
    build_palette_entries_with_themes, discover_custom_themes, discover_installed_locales,
    handle_palette_key, render_command_palette, PaletteEntry,
};
use crate::preview_pane::{render_preview_pane, PreviewPaneState};
use crate::terminal::{crossterm_key_to_chord, write_osc52_clipboard, TerminalGuard};
use crate::viewport::{render_editor_viewport, ViewportMetrics};

fn is_blocking_modal(overlay: OverlayMode) -> bool {
    matches!(
        overlay,
        OverlayMode::CommandPalette
            | OverlayMode::SpellQuickFix
            | OverlayMode::ConfirmQuit
            | OverlayMode::ConfirmCloseBuffer
            | OverlayMode::OpenFilePrompt
            | OverlayMode::SaveAsPrompt
    )
}

#[derive(Debug, Clone)]
pub struct FrameState {
    pub regions: ScreenRegions,
    pub tab_hitboxes: Vec<TabHitbox>,
    pub viewport: Option<ViewportMetrics>,
    pub wrapped_layout: Option<WrappedLayout>,
    pub total_area: Rect,
}

pub struct TuiApp {
    pub state: EditorState,
    pub mouse: MouseRouter,
    pub last_frame: Option<FrameState>,
    pub follow_cursor: bool,
    pub preview_pane: PreviewPaneState,
    pub find_bar: FindBarState,
    pub palette_entries: Vec<PaletteEntry>,
}

impl TuiApp {
    pub fn from_cli(cli: &Cli) -> ScriptuiResult<Self> {
        let config_dir = AppConfig::default_config_dir();
        let theme_dirs = [config_dir.join("themes")];
        let dict_dirs = [config_dir.join("dicts")];
        let config_path = config_dir.join("config.toml");
        let (mut config, config_warning) = match AppConfig::load_or_default(&config_path) {
            Ok(cfg) => (cfg, None),
            Err(err) => (
                AppConfig::default(),
                Some(format!("Config error ({}): {err}", config_path.display())),
            ),
        };
        if let Some(theme) = &cli.theme {
            config.theme = theme.clone();
        }
        if let Some(locale) = &cli.locale {
            config.spell_locale = locale.clone();
        }

        let mut state = EditorState::new_with_dirs(config, &theme_dirs, &dict_dirs)?;
        if let Some(warning) = config_warning {
            state.status_message = Some(match state.status_message.take() {
                Some(existing) if !existing.is_empty() => format!("{warning}; {existing}"),
                _ => warning,
            });
        }
        for path in &cli.files {
            state.buffers.open_or_create_file(path)?;
        }

        Ok(Self {
            state,
            mouse: MouseRouter::default(),
            last_frame: None,
            follow_cursor: true,
            preview_pane: PreviewPaneState::default(),
            find_bar: FindBarState::default(),
            palette_entries: Vec::new(),
        })
    }

    pub fn refresh_palette_entries(&mut self) {
        let config_dir = AppConfig::default_config_dir();
        let theme_dirs = [config_dir.join("themes")];
        let dict_dirs = [config_dir.join("dicts")];
        let custom_themes = discover_custom_themes(&theme_dirs);
        let locales = discover_installed_locales(&dict_dirs);
        self.palette_entries = build_palette_entries_with_themes(&custom_themes, &locales);
    }

    pub fn render_frame(&mut self, frame: &mut Frame) {
        let total_area = frame.area();
        let regions =
            compute_screen_regions(total_area, self.state.preview_mode, self.state.overlay);
        let tab_hitboxes = render_tab_bar(frame, regions.tab_bar, &self.state);

        let (viewport, wrapped_layout) = if let Some(editor_rect) = regions.editor {
            let (metrics, layout) =
                render_editor_viewport(frame, editor_rect, &mut self.state, self.follow_cursor);
            (Some(metrics), Some(layout))
        } else {
            (None, None)
        };

        if let Some(edge_rect) = regions.preview_edge_tab {
            render_preview_edge_tab(frame, edge_rect, &self.state.theme);
        }

        if let Some(div_rect) = regions.split_divider {
            render_split_divider(frame, div_rect, &self.state.theme);
        }

        let editor_top_source_line = if let Some(wl) = wrapped_layout.as_ref() {
            wl.visual_lines
                .get(self.state.buffers.active().scroll_line)
                .map_or(0, |vl| vl.logical_line)
        } else {
            self.state.buffers.active().scroll_line
        };

        if let Some(preview_rect) = regions.preview {
            render_preview_pane(
                frame,
                preview_rect,
                &self.state,
                &mut self.preview_pane,
                editor_top_source_line,
            );
        }

        if let Some(find_rect) = regions.find_bar {
            render_find_bar(frame, find_rect, &self.state, self.find_bar);
        }

        render_status_bar(frame, regions.status_bar, &self.state);

        match self.state.overlay {
            OverlayMode::CommandPalette => {
                if self.palette_entries.is_empty() {
                    self.refresh_palette_entries();
                }
                render_command_palette(frame, total_area, &self.state, &self.palette_entries);
            }
            OverlayMode::SpellQuickFix => {
                render_spell_quick_fix(frame, total_area, &self.state);
            }
            OverlayMode::ConfirmQuit | OverlayMode::ConfirmCloseBuffer => {
                render_confirm_modal(frame, total_area, &self.state);
            }
            OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt => {
                render_file_prompt(frame, total_area, &self.state);
            }
            OverlayMode::None | OverlayMode::Find | OverlayMode::Replace => {}
        }

        self.last_frame = Some(FrameState {
            regions,
            tab_hitboxes,
            viewport,
            wrapped_layout,
            total_area,
        });
    }

    pub fn handle_key_event<W: Write>(
        &mut self,
        key: KeyEvent,
        clipboard_out: &mut W,
    ) -> ScriptuiResult<()> {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return Ok(());
        }
        self.follow_cursor = true;

        match self.state.overlay {
            OverlayMode::Find | OverlayMode::Replace => {
                if handle_find_bar_key(key, &mut self.state, &mut self.find_bar)? {
                    return Ok(());
                }
            }
            OverlayMode::CommandPalette => {
                if self.palette_entries.is_empty() {
                    self.refresh_palette_entries();
                }
                let config_dir = AppConfig::default_config_dir();
                let theme_dirs = [config_dir.join("themes")];
                let dict_dirs = [config_dir.join("dicts")];
                let _ = handle_palette_key(
                    key,
                    &mut self.state,
                    &self.palette_entries,
                    &theme_dirs,
                    &dict_dirs,
                )?;
                return Ok(());
            }
            OverlayMode::SpellQuickFix => {
                let _ = handle_spell_quick_fix_key(key, &mut self.state)?;
                return Ok(());
            }
            OverlayMode::ConfirmQuit | OverlayMode::ConfirmCloseBuffer => {
                let _ = handle_confirm_modal_key(key, &mut self.state);
                return Ok(());
            }
            OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt => {
                let _ = handle_file_prompt_key(key, &mut self.state)?;
                return Ok(());
            }
            OverlayMode::None => {}
        }

        if let Some(action) = crossterm_key_to_chord(key).and_then(|c| self.state.keymap.lookup(c))
        {
            match action {
                Action::Copy => {
                    let selected = self.state.selected_text();
                    if !selected.is_empty() {
                        write_osc52_clipboard(clipboard_out, &selected).map_err(|source| {
                            ScriptuiError::Io {
                                path: PathBuf::from("clipboard"),
                                source,
                            }
                        })?;
                    }
                    self.state.dispatch(Action::Copy)?;
                }
                Action::Cut => {
                    let selected = self.state.selected_text();
                    if !selected.is_empty() {
                        write_osc52_clipboard(clipboard_out, &selected).map_err(|source| {
                            ScriptuiError::Io {
                                path: PathBuf::from("clipboard"),
                                source,
                            }
                        })?;
                    }
                    self.state.dispatch(Action::Cut)?;
                }
                Action::TogglePreview | Action::ExpandPreview => {
                    self.preview_pane.sync_enabled = true;
                    self.state.dispatch(action)?;
                }
                Action::OpenCommandPalette => {
                    self.refresh_palette_entries();
                    self.state.dispatch(Action::OpenCommandPalette)?;
                }
                other => {
                    self.state.dispatch(other)?;
                }
            }
        } else if let KeyCode::Char(c) = key.code {
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
            {
                let s = c.to_string();
                self.state.insert_text(&s);
            }
        }

        Ok(())
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) -> ScriptuiResult<bool> {
        if is_blocking_modal(self.state.overlay) {
            return Ok(false);
        }

        let Some(frame) = self.last_frame.as_ref() else {
            return Ok(false);
        };

        if matches!(
            mouse.kind,
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
        ) && is_over_preview(frame.regions, mouse.column, mouse.row)
        {
            self.preview_pane.sync_enabled = false;
            let max_scroll = self
                .preview_pane
                .total_lines
                .saturating_sub(self.preview_pane.viewport_height.max(1));
            self.preview_pane.scroll_line = match mouse.kind {
                MouseEventKind::ScrollUp => self
                    .preview_pane
                    .scroll_line
                    .saturating_sub(3)
                    .min(max_scroll),
                MouseEventKind::ScrollDown => self
                    .preview_pane
                    .scroll_line
                    .saturating_add(3)
                    .min(max_scroll),
                _ => self.preview_pane.scroll_line,
            };
            return Ok(true);
        }

        match mouse.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                self.follow_cursor = false;
            }
            MouseEventKind::Down(MouseButton::Left | MouseButton::Right) => {
                self.follow_cursor = true;
            }
            _ => {}
        }

        let ctx = MouseContext {
            regions: frame.regions,
            tab_hitboxes: &frame.tab_hitboxes,
            viewport: frame.viewport,
            wrapped_layout: frame.wrapped_layout.as_ref(),
            total_area: frame.total_area,
        };

        self.mouse.handle_mouse(mouse, &mut self.state, &ctx)
    }

    pub fn handle_paste_event(&mut self, text: &str) -> ScriptuiResult<()> {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let single_line = normalized.replace('\n', " ");

        match self.state.overlay {
            OverlayMode::None => {
                self.state.insert_text(&normalized);
                self.follow_cursor = true;
            }
            OverlayMode::Find => {
                self.state.find_replace.query.pattern.push_str(&single_line);
                match self.state.update_find_matches() {
                    Ok(_) => {}
                    Err(ScriptuiError::InvalidRegex { pattern, source }) => {
                        self.state.find_replace.matches.clear();
                        self.state.find_replace.active_match = None;
                        self.state.status_message =
                            Some(format!("Invalid regex '{pattern}': {source}"));
                    }
                    Err(err) => return Err(err),
                }
            }
            OverlayMode::Replace => {
                if self.find_bar.focus == FindFocus::Replacement {
                    self.state.find_replace.replacement.push_str(&single_line);
                } else {
                    self.state.find_replace.query.pattern.push_str(&single_line);
                    match self.state.update_find_matches() {
                        Ok(_) => {}
                        Err(ScriptuiError::InvalidRegex { pattern, source }) => {
                            self.state.find_replace.matches.clear();
                            self.state.find_replace.active_match = None;
                            self.state.status_message =
                                Some(format!("Invalid regex '{pattern}': {source}"));
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
            OverlayMode::CommandPalette => {
                self.state.palette.query.push_str(&single_line);
                self.state.palette.selected_index = 0;
            }
            OverlayMode::OpenFilePrompt | OverlayMode::SaveAsPrompt => {
                self.state.file_prompt_input.push_str(&single_line);
            }
            OverlayMode::SpellQuickFix
            | OverlayMode::ConfirmQuit
            | OverlayMode::ConfirmCloseBuffer => {}
        }

        Ok(())
    }

    pub fn run_event_loop(&mut self, guard: &mut TerminalGuard) -> io::Result<()> {
        let mut needs_redraw = true;
        while !self.state.should_quit {
            if needs_redraw {
                guard.terminal_mut().draw(|f| self.render_frame(f))?;
                needs_redraw = false;
            }

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        let mut stdout = io::stdout();
                        if let Err(err) = self.handle_key_event(key, &mut stdout) {
                            self.state.status_message = Some(err.to_string());
                        }
                        needs_redraw = true;
                    }
                    Event::Mouse(mouse) => {
                        if let Err(err) = self.handle_mouse_event(mouse) {
                            self.state.status_message = Some(err.to_string());
                        }
                        needs_redraw = true;
                    }
                    Event::Paste(text) => {
                        if let Err(err) = self.handle_paste_event(&text) {
                            self.state.status_message = Some(err.to_string());
                        }
                        needs_redraw = true;
                    }
                    Event::Resize(_, _) => {
                        needs_redraw = true;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;

    #[test]
    fn tui_app_renders_full_shell_and_handles_typing_and_osc52_copy() {
        let cli = Cli {
            command: None,
            theme: Some("crt_amber".to_string()),
            locale: None,
            files: Vec::new(),
        };
        let mut app = TuiApp::from_cli(&cli).expect("TuiApp should initialize from CLI");

        let mut clipboard_buf = Vec::new();
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT),
            &mut clipboard_buf,
        )
        .expect("typing H should succeed");
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("typing i should succeed");
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+A should succeed");
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+C should succeed");

        let clipboard_str =
            String::from_utf8(clipboard_buf).expect("OSC 52 output should be valid UTF-8");
        assert!(
            clipboard_str.contains("SGk="),
            "expected OSC 52 buffer to contain base64 'SGk=', got {clipboard_str:?}"
        );

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render_frame should succeed");

        assert!(app.last_frame.is_some());

        let buffer = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                rendered.push_str(buffer[(x, y)].symbol());
            }
            rendered.push('\n');
        }

        assert!(
            rendered.contains("Hi"),
            "expected rendered buffer to contain 'Hi', got:\n{rendered}"
        );
        assert!(
            rendered.contains("crt_amber"),
            "expected rendered buffer to contain 'crt_amber', got:\n{rendered}"
        );
    }

    #[test]
    fn mouse_wheel_scroll_not_snapped_back_on_next_render() {
        let cli = Cli {
            command: None,
            theme: None,
            locale: None,
            files: Vec::new(),
        };
        let mut app = TuiApp::from_cli(&cli).expect("TuiApp should initialize from CLI");
        let content = (1..=20)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        app.state.buffers.active_mut().document =
            scriptui_core::DocumentBuffer::from_str_content(&content);

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("initial render_frame should succeed");

        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 10,
            row: 2,
            modifiers: KeyModifiers::NONE,
        })
        .expect("ScrollDown should succeed");
        assert!(!app.follow_cursor);
        assert_eq!(app.state.buffers.active().scroll_line, 3);

        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("second render_frame should succeed");
        assert_eq!(app.state.buffers.active().scroll_line, 3);

        let mut clipboard_buf = Vec::new();
        app.handle_key_event(
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("Right key should succeed");
        assert!(app.follow_cursor);

        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("third render_frame should succeed");
        assert_eq!(app.state.buffers.active().scroll_line, 0);
    }

    fn render_to_string(terminal: &mut Terminal<TestBackend>, app: &mut TuiApp) -> String {
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render_frame should succeed");
        let buffer = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                rendered.push_str(buffer[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        rendered
    }

    #[test]
    fn tui_app_overlays_and_styled_preview_integration() {
        let cli = Cli {
            command: None,
            theme: Some("crt_amber".to_string()),
            locale: None,
            files: Vec::new(),
        };
        let mut app = TuiApp::from_cli(&cli).expect("TuiApp should initialize from CLI");
        app.state
            .insert_text("# Heading\n\n- [x] Done\n\nfoo bar foo");

        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let mut clipboard_buf = Vec::new();

        // 1. Open split preview (Ctrl+Shift+P), render, assert styled preview title and heading
        app.handle_key_event(
            KeyEvent::new(
                KeyCode::Char('P'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
            &mut clipboard_buf,
        )
        .expect("Ctrl+Shift+P should toggle preview");
        let rendered = render_to_string(&mut terminal, &mut app);
        assert!(
            rendered.contains("Preview [SYNC]"),
            "expected rendered buffer to contain 'Preview [SYNC]', got:\n{rendered}"
        );
        assert!(
            rendered.contains("Heading"),
            "expected rendered buffer to contain 'Heading', got:\n{rendered}"
        );

        // 2. Open Command Palette (Ctrl+P), type "ice_cyan", Enter -> theme switches
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+P should open command palette");
        assert_eq!(app.state.overlay, OverlayMode::CommandPalette);
        for ch in "ice_cyan".chars() {
            app.handle_key_event(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut clipboard_buf,
            )
            .expect("typing palette query should succeed");
        }
        app.handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("Enter in command palette should succeed");
        assert_eq!(app.state.theme.name, "ice_cyan");

        // 3. Open Find bar (Ctrl+F), type "foo", render, assert "1/2", Alt+Enter -> 2 selections
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+F should open find bar");
        assert_eq!(app.state.overlay, OverlayMode::Find);
        for ch in "foo".chars() {
            app.handle_key_event(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut clipboard_buf,
            )
            .expect("typing find query should succeed");
        }
        let rendered = render_to_string(&mut terminal, &mut app);
        assert!(
            rendered.contains("1/2"),
            "expected find bar to show '1/2', got:\n{rendered}"
        );
        app.handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT),
            &mut clipboard_buf,
        )
        .expect("Alt+Enter should select all occurrences");
        assert_eq!(app.state.buffers.active().document.selection().len(), 2);
        assert_eq!(app.state.overlay, OverlayMode::None);

        // 4. Ctrl+Q on dirty buffer -> ConfirmQuit modal -> 'y' sets should_quit
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+Q should prompt on dirty buffer");
        assert_eq!(app.state.overlay, OverlayMode::ConfirmQuit);
        let rendered = render_to_string(&mut terminal, &mut app);
        assert!(
            rendered.contains("Quit?"),
            "expected confirm modal to render, got:\n{rendered}"
        );
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("'y' should confirm quit");
        assert!(app.state.should_quit);
    }

    #[test]
    fn blocking_modals_trap_unhandled_keys_and_ignore_mouse() {
        let cli = Cli {
            command: None,
            theme: None,
            locale: None,
            files: Vec::new(),
        };
        let mut app = TuiApp::from_cli(&cli).expect("TuiApp should initialize from CLI");
        app.state.insert_text("hello");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render_frame should succeed");

        let mut clipboard_buf = Vec::new();
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+Q should open ConfirmQuit");
        assert_eq!(app.state.overlay, OverlayMode::ConfirmQuit);

        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("'x' should be trapped");
        app.handle_key_event(
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("Backspace should be trapped");

        let mouse_handled = app
            .handle_mouse_event(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 10,
                row: 2,
                modifiers: KeyModifiers::NONE,
            })
            .expect("mouse event should not error");
        assert!(!mouse_handled);

        assert_eq!(app.state.buffers.active().document.text(), "hello");
        assert_eq!(app.state.overlay, OverlayMode::ConfirmQuit);
    }

    #[test]
    fn alt_enter_resets_find_focus_and_palette_entries_cached() {
        use crate::find_bar::FindFocus;

        let cli = Cli {
            command: None,
            theme: None,
            locale: None,
            files: Vec::new(),
        };
        let mut app = TuiApp::from_cli(&cli).expect("TuiApp should initialize from CLI");
        app.state.insert_text("foo bar foo");
        assert!(app.palette_entries.is_empty());

        let mut clipboard_buf = Vec::new();
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+H should open Replace");
        assert_eq!(app.state.overlay, OverlayMode::Replace);

        for ch in "foo".chars() {
            app.handle_key_event(
                KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE),
                &mut clipboard_buf,
            )
            .expect("typing find query should succeed");
        }
        app.handle_key_event(
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("Tab should switch focus to Replacement");
        assert_eq!(app.find_bar.focus, FindFocus::Replacement);

        app.handle_key_event(
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT),
            &mut clipboard_buf,
        )
        .expect("Alt+Enter should select all occurrences");
        assert_eq!(app.state.overlay, OverlayMode::None);
        assert_eq!(app.find_bar.focus, FindFocus::Query);

        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+P should open CommandPalette");
        assert_eq!(app.state.overlay, OverlayMode::CommandPalette);
        assert!(!app.palette_entries.is_empty());
        let cached_len = app.palette_entries.len();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| app.render_frame(frame))
            .expect("render_frame should succeed");
        assert_eq!(app.palette_entries.len(), cached_len);
    }

    #[test]
    fn bracketed_paste_routes_to_editor_find_bar_and_ignores_confirm_modal() {
        let cli = Cli {
            command: None,
            theme: None,
            locale: None,
            files: Vec::new(),
        };
        let mut app = TuiApp::from_cli(&cli).expect("TuiApp should initialize from CLI");

        app.handle_paste_event("line1\r\nline2")
            .expect("paste in editor should succeed");
        assert_eq!(app.state.buffers.active().document.text(), "line1\nline2");

        app.state.overlay = OverlayMode::Find;
        app.handle_paste_event("line2")
            .expect("paste in find bar should succeed");
        assert_eq!(app.state.find_replace.query.pattern, "line2");
        assert_eq!(app.state.find_replace.matches.len(), 1);

        app.state.overlay = OverlayMode::ConfirmQuit;
        app.handle_paste_event("ignored")
            .expect("paste in confirm modal should be ignored");
        assert_eq!(app.state.buffers.active().document.text(), "line1\nline2");
    }

    #[test]
    fn tui_app_handles_repeat_keys_ctrl_c_v_and_nonexistent_cli_file() {
        let dir = tempfile::tempdir().expect("tempdir should succeed");
        let missing_path = dir.path().join("fresh.md");

        let cli = Cli {
            command: None,
            theme: None,
            locale: None,
            files: vec![missing_path.clone()],
        };
        let mut app = TuiApp::from_cli(&cli).expect("TuiApp should open missing file as clean");
        assert!(!app.state.buffers.active().is_dirty());
        assert_eq!(
            app.state.buffers.active().document.path(),
            Some(missing_path.as_path())
        );

        let mut clipboard_buf = Vec::new();
        app.handle_key_event(
            KeyEvent::new_with_kind(KeyCode::Char('a'), KeyModifiers::NONE, KeyEventKind::Repeat),
            &mut clipboard_buf,
        )
        .expect("repeat key should insert char");
        app.handle_key_event(
            KeyEvent::new_with_kind(
                KeyCode::Char('b'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            ),
            &mut clipboard_buf,
        )
        .expect("release key should be ignored");
        assert_eq!(app.state.buffers.active().document.text(), "a");

        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+A should select all");
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+C should copy");
        app.handle_key_event(
            KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
            &mut clipboard_buf,
        )
        .expect("End should move cursor to end");
        app.handle_key_event(
            KeyEvent::new(KeyCode::Char('v'), KeyModifiers::CONTROL),
            &mut clipboard_buf,
        )
        .expect("Ctrl+V should paste");

        assert_eq!(app.state.buffers.active().document.text(), "aa");
        let clipboard_str =
            String::from_utf8(clipboard_buf).expect("OSC 52 output should be valid UTF-8");
        assert!(
            clipboard_str.contains("YQ=="),
            "expected OSC 52 buffer to contain base64 'YQ==', got {clipboard_str:?}"
        );
    }
}
