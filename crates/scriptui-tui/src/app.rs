use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use scriptui_core::{Action, AppConfig, EditorState, ScriptuiError, ScriptuiResult, WrappedLayout};

use crate::chrome::{
    render_preview_edge_tab, render_split_divider, render_status_bar, render_tab_bar, TabHitbox,
};
use crate::cli::Cli;
use crate::layout_tree::{compute_screen_regions, rgb_to_color, ScreenRegions};
use crate::mouse::{MouseContext, MouseRouter};
use crate::terminal::{crossterm_key_to_chord, write_osc52_clipboard, TerminalGuard};
use crate::viewport::{render_editor_viewport, ViewportMetrics};

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
}

impl TuiApp {
    pub fn from_cli(cli: &Cli) -> ScriptuiResult<Self> {
        let config_path = AppConfig::default_config_dir().join("config.toml");
        let mut config = AppConfig::load_or_default(&config_path).unwrap_or_default();
        if let Some(theme) = &cli.theme {
            config.theme = theme.clone();
        }
        if let Some(locale) = &cli.locale {
            config.spell_locale = locale.clone();
        }

        let mut state = EditorState::new(config)?;
        for path in &cli.files {
            state.buffers.open_file(path)?;
        }

        Ok(Self {
            state,
            mouse: MouseRouter::default(),
            last_frame: None,
        })
    }

    pub fn render_frame(&mut self, frame: &mut Frame) {
        let total_area = frame.area();
        let regions =
            compute_screen_regions(total_area, self.state.preview_mode, self.state.overlay);
        let tab_hitboxes = render_tab_bar(frame, regions.tab_bar, &self.state);

        let (viewport, wrapped_layout) = if let Some(editor_rect) = regions.editor {
            let (metrics, layout) = render_editor_viewport(frame, editor_rect, &mut self.state);
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

        if let Some(preview_rect) = regions.preview {
            let inner_width = preview_rect.width.saturating_sub(2).max(1) as usize;
            let preview_doc = self.state.build_active_preview(inner_width);
            let lines: Vec<Line<'static>> = preview_doc
                .plain_lines()
                .into_iter()
                .map(Line::from)
                .collect();
            let palette = &self.state.theme.palette;
            let preview_style = Style::default()
                .fg(rgb_to_color(palette.foreground))
                .bg(rgb_to_color(palette.background));
            let border_style = Style::default()
                .fg(rgb_to_color(palette.border_default))
                .bg(rgb_to_color(palette.background));
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Preview ")
                .border_style(border_style)
                .style(preview_style);
            let widget = Paragraph::new(lines).block(block).style(preview_style);
            frame.render_widget(widget, preview_rect);
        }

        render_status_bar(frame, regions.status_bar, &self.state);

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
        if key.kind != KeyEventKind::Press {
            return Ok(());
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
        let Some(frame) = self.last_frame.as_ref() else {
            return Ok(false);
        };

        let ctx = MouseContext {
            regions: frame.regions,
            tab_hitboxes: &frame.tab_hitboxes,
            viewport: frame.viewport,
            wrapped_layout: frame.wrapped_layout.as_ref(),
            total_area: frame.total_area,
        };

        self.mouse.handle_mouse(mouse, &mut self.state, &ctx)
    }

    pub fn run_event_loop(&mut self, guard: &mut TerminalGuard) -> io::Result<()> {
        while !self.state.should_quit {
            guard.terminal_mut().draw(|f| self.render_frame(f))?;

            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        let mut stdout = io::stdout();
                        if let Err(err) = self.handle_key_event(key, &mut stdout) {
                            self.state.status_message = Some(err.to_string());
                        }
                    }
                    Event::Mouse(mouse) => {
                        if let Err(err) = self.handle_mouse_event(mouse) {
                            self.state.status_message = Some(err.to_string());
                        }
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
}
