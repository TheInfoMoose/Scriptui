use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use scriptui_core::{EditorState, IndentStyle, Theme};

use crate::layout_tree::rgb_to_color;

pub(crate) fn sanitize_control_chars(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch == '\t' {
                ' '
            } else if ch.is_control() {
                '?'
            } else {
                ch
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabHitbox {
    pub rect: Rect,
    pub buffer_index: usize,
}

pub fn render_tab_bar(frame: &mut Frame, area: Rect, state: &EditorState) -> Vec<TabHitbox> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let palette = &state.theme.palette;
    let bar_style = Style::default()
        .fg(rgb_to_color(palette.foreground))
        .bg(rgb_to_color(palette.surface));
    let active_style = Style::default()
        .fg(rgb_to_color(palette.background))
        .bg(rgb_to_color(palette.accent))
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default()
        .fg(rgb_to_color(palette.foreground))
        .bg(rgb_to_color(palette.surface));

    let active_idx = state.buffers.active_index();
    let mut spans = Vec::new();
    let mut hitboxes = Vec::new();
    let mut cursor_x = area.x;
    let area_right = area.right();

    for (idx, buf) in state.buffers.buffers().iter().enumerate() {
        if cursor_x >= area_right {
            break;
        }

        let dirty_marker = if buf.is_dirty() { "*" } else { "" };
        let sanitized_name = sanitize_control_chars(&buf.display_name());
        let full_label = format!(" {sanitized_name}{dirty_marker} ");
        let remaining = area_right.saturating_sub(cursor_x) as usize;
        let visible_label: String = full_label.chars().take(remaining).collect();
        let tab_width = visible_label.chars().count().min(u16::MAX as usize) as u16;
        if tab_width == 0 {
            break;
        }

        let style = if idx == active_idx {
            active_style
        } else {
            inactive_style
        };
        spans.push(Span::styled(visible_label, style));
        hitboxes.push(TabHitbox {
            rect: Rect::new(cursor_x, area.y, tab_width, 1),
            buffer_index: idx,
        });
        cursor_x = cursor_x.saturating_add(tab_width);
    }

    let widget = Paragraph::new(Line::from(spans)).style(bar_style);
    frame.render_widget(widget, area);

    hitboxes
}

pub fn render_preview_edge_tab(frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let palette = &theme.palette;
    let surface_bg = rgb_to_color(palette.surface);
    let base_style = Style::default()
        .fg(rgb_to_color(palette.accent))
        .bg(surface_bg);
    let border_style = Style::default()
        .fg(rgb_to_color(palette.border_default))
        .bg(surface_bg);
    let arrow_style = Style::default()
        .fg(rgb_to_color(palette.accent))
        .bg(surface_bg)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default()
        .fg(rgb_to_color(palette.accent))
        .bg(surface_bg);

    let glyphs = ['◀', 'P', 'R', 'E', 'V', 'I', 'E', 'W'];
    let height = area.height as usize;
    let top_pad = height.saturating_sub(glyphs.len()) / 2;

    let mut lines = Vec::with_capacity(height);
    for row in 0..height {
        let glyph = row
            .checked_sub(top_pad)
            .and_then(|idx| glyphs.get(idx).copied());

        let line = if area.width >= 3 {
            match glyph {
                Some('◀') => Line::from(vec![
                    Span::styled("│", border_style),
                    Span::styled("◀", arrow_style),
                    Span::styled(" ", base_style),
                ]),
                Some(ch) => Line::from(vec![
                    Span::styled("│", border_style),
                    Span::styled(ch.to_string(), label_style),
                    Span::styled(" ", base_style),
                ]),
                None => Line::from(vec![
                    Span::styled("│", border_style),
                    Span::styled("  ", base_style),
                ]),
            }
        } else if area.width == 2 {
            match glyph {
                Some('◀') => Line::from(vec![
                    Span::styled("│", border_style),
                    Span::styled("◀", arrow_style),
                ]),
                Some(ch) => Line::from(vec![
                    Span::styled("│", border_style),
                    Span::styled(ch.to_string(), label_style),
                ]),
                None => Line::from(vec![
                    Span::styled("│", border_style),
                    Span::styled(" ", base_style),
                ]),
            }
        } else {
            match glyph {
                Some('◀') => Line::from(Span::styled("◀", arrow_style)),
                Some(ch) => Line::from(Span::styled(ch.to_string(), label_style)),
                None => Line::from(Span::styled(" ", base_style)),
            }
        };
        lines.push(line);
    }

    let widget = Paragraph::new(lines).style(base_style);
    frame.render_widget(widget, area);
}

pub fn render_split_divider(frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let palette = &theme.palette;
    let divider_style = Style::default()
        .fg(rgb_to_color(palette.border_focused))
        .bg(rgb_to_color(palette.background));

    let lines: Vec<Line<'static>> = (0..area.height)
        .map(|_| Line::from(Span::styled("│", divider_style)))
        .collect();

    let widget = Paragraph::new(lines).style(divider_style);
    frame.render_widget(widget, area);
}

pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &EditorState) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let palette = &state.theme.palette;
    let bg = rgb_to_color(palette.surface);
    let fg_style = Style::default().fg(rgb_to_color(palette.foreground)).bg(bg);
    let accent_style = Style::default()
        .fg(rgb_to_color(palette.accent))
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let muted_style = Style::default().fg(rgb_to_color(palette.muted)).bg(bg);

    let active = state.buffers.active();
    let dirty_marker = if active.is_dirty() { "*" } else { "" };
    let left_name = format!(
        " {}{}",
        sanitize_control_chars(&active.display_name()),
        dirty_marker
    );

    let status_msg = state
        .status_message
        .as_deref()
        .filter(|msg| !msg.is_empty())
        .map(|msg| format!(" │ {}", sanitize_control_chars(msg)))
        .unwrap_or_default();

    let selection = active.document.selection();
    let pos = active.document.char_to_position(selection.primary().head);
    let cursor_str = format!("Ln {}, Col {}", pos.line + 1, pos.col + 1);
    let multi_cursor_str = if selection.len() > 1 {
        format!(" │ {} cursors", selection.len())
    } else {
        String::new()
    };
    let wrap_str = if state.config.soft_wrap {
        "WRAP"
    } else {
        "NOWRAP"
    };
    let indent_str = match state.config.indent.style {
        IndentStyle::Spaces(w) => format!("SPC {}", w.max(1)),
        IndentStyle::Tabs => "TAB 4".to_string(),
    };
    let locale_str = sanitize_control_chars(state.config.spell_locale.as_str());
    let theme_str = sanitize_control_chars(state.theme.name.as_str());

    let right_text = format!(
        "{cursor_str}{multi_cursor_str} │ {wrap_str} │ {indent_str} │ {locale_str} │ {theme_str} "
    );

    let total_width = area.width as usize;
    let right_len = right_text.chars().count();
    let left_full_len = left_name.chars().count() + status_msg.chars().count();

    let mut spans = Vec::new();
    if left_full_len + 1 + right_len <= total_width {
        spans.push(Span::styled(left_name, accent_style));
        if !status_msg.is_empty() {
            spans.push(Span::styled(status_msg, fg_style));
        }
        let pad = total_width.saturating_sub(left_full_len + right_len);
        spans.push(Span::styled(" ".repeat(pad), muted_style));
        spans.push(Span::styled(cursor_str, accent_style));
        if !multi_cursor_str.is_empty() {
            spans.push(Span::styled(multi_cursor_str, fg_style));
        }
        spans.push(Span::styled(" │ ", muted_style));
        spans.push(Span::styled(wrap_str, fg_style));
        spans.push(Span::styled(" │ ", muted_style));
        spans.push(Span::styled(indent_str, fg_style));
        spans.push(Span::styled(" │ ", muted_style));
        spans.push(Span::styled(locale_str, muted_style));
        spans.push(Span::styled(" │ ", muted_style));
        spans.push(Span::styled(format!("{theme_str} "), accent_style));
    } else if right_len <= total_width {
        let left_budget = total_width.saturating_sub(right_len + 1);
        let left_combined = format!("{left_name}{status_msg}");
        let truncated_left: String = left_combined.chars().take(left_budget).collect();
        let left_used = truncated_left.chars().count();
        if !truncated_left.is_empty() {
            spans.push(Span::styled(truncated_left, accent_style));
        }
        let pad = total_width.saturating_sub(left_used + right_len);
        spans.push(Span::styled(" ".repeat(pad), muted_style));
        spans.push(Span::styled(right_text, fg_style));
    } else {
        let truncated_right: String = right_text.chars().take(total_width).collect();
        spans.push(Span::styled(truncated_right, fg_style));
    }

    let widget = Paragraph::new(Line::from(spans)).style(fg_style);
    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use scriptui_core::{AppConfig, EditorState, OverlayMode, PreviewMode};

    use super::*;
    use crate::layout_tree::compute_screen_regions;

    #[test]
    fn screen_regions_match_all_three_preview_modes_and_find_bar() {
        let area = Rect::new(0, 0, 100, 30);

        let collapsed = compute_screen_regions(area, PreviewMode::Collapsed, OverlayMode::None);
        assert_eq!(collapsed.tab_bar.height, 1);
        assert_eq!(collapsed.status_bar.height, 1);
        assert!(collapsed.editor.is_some());
        assert_eq!(collapsed.preview_edge_tab.map(|r| r.width), Some(3));
        assert!(collapsed.split_divider.is_none());
        assert!(collapsed.preview.is_none());
        assert!(collapsed.find_bar.is_none());

        let split = compute_screen_regions(area, PreviewMode::Split(50), OverlayMode::None);
        assert!(split.editor.is_some());
        assert_eq!(split.split_divider.map(|r| r.width), Some(1));
        assert!(split.preview.is_some());
        assert!(split.preview_edge_tab.is_none());

        let expanded = compute_screen_regions(area, PreviewMode::Expanded, OverlayMode::None);
        assert!(expanded.editor.is_none());
        assert!(expanded.preview_edge_tab.is_none());
        assert!(expanded.split_divider.is_none());
        assert!(expanded.preview.is_some());

        let replace = compute_screen_regions(area, PreviewMode::Collapsed, OverlayMode::Replace);
        assert_eq!(replace.find_bar.map(|r| r.height), Some(2));
    }

    #[test]
    fn chrome_renders_tabs_edge_handle_and_telemetry_status_bar() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        state.buffers.new_buffer();

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut hitboxes = Vec::new();

        terminal
            .draw(|frame| {
                let regions =
                    compute_screen_regions(frame.area(), state.preview_mode, state.overlay);
                hitboxes = render_tab_bar(frame, regions.tab_bar, &state);
                if let Some(edge) = regions.preview_edge_tab {
                    render_preview_edge_tab(frame, edge, &state.theme);
                }
                render_status_bar(frame, regions.status_bar, &state);
            })
            .expect("draw chrome");

        assert_eq!(hitboxes.len(), state.buffers.len());

        let buffer = terminal.backend().buffer();
        let mut all_text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                all_text.push_str(buffer[(x, y)].symbol());
            }
            all_text.push('\n');
        }

        assert!(all_text.contains('◀'));
        assert!(all_text.contains("Ln 1, Col 1"));
        assert!(all_text.contains("crt_amber"));
    }

    #[test]
    fn chrome_sanitizes_control_chars_in_tabs_and_status_bar() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        state.buffers.active_mut().document = scriptui_core::DocumentBuffer::empty_with_path(
            std::path::PathBuf::from("\x1b[31mred\x07\r.md"),
        );
        state.status_message = Some("warn\x1b[2J\x08\u{9b}!".to_string());

        for width in [80u16, 20u16] {
            let mut tab_terminal =
                Terminal::new(TestBackend::new(width, 1)).expect("tab test terminal");
            tab_terminal
                .draw(|frame| {
                    render_tab_bar(frame, Rect::new(0, 0, width, 1), &state);
                })
                .expect("draw tab bar");

            let tab_buf = tab_terminal.backend().buffer();
            let mut tab_text = String::new();
            for x in 0..width {
                let sym = tab_buf[(x, 0)].symbol();
                assert!(
                    !sym.chars().any(char::is_control),
                    "tab bar cell ({x}, 0) at width {width} contained control char: {sym:?}"
                );
                tab_text.push_str(sym);
            }
            assert!(
                tab_text.contains("red"),
                "expected 'red' in tab bar at width {width}, got: {tab_text:?}"
            );

            let mut status_terminal =
                Terminal::new(TestBackend::new(width, 1)).expect("status test terminal");
            status_terminal
                .draw(|frame| {
                    render_status_bar(frame, Rect::new(0, 0, width, 1), &state);
                })
                .expect("draw status bar");

            let status_buf = status_terminal.backend().buffer();
            let mut status_text = String::new();
            for x in 0..width {
                let sym = status_buf[(x, 0)].symbol();
                assert!(
                    !sym.chars().any(char::is_control),
                    "status bar cell ({x}, 0) at width {width} contained control char: {sym:?}"
                );
                status_text.push_str(sym);
            }
            if width == 80 {
                assert!(
                    status_text.contains("red"),
                    "expected 'red' in status bar at width {width}, got: {status_text:?}"
                );
                assert!(
                    status_text.contains("warn"),
                    "expected 'warn' in status bar at width {width}, got: {status_text:?}"
                );
            }
        }
    }
}
