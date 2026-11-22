use std::ops::Range;

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;
use scriptui_core::{
    compute_wrapped_layout, highlight_editor_line, parse_gfm, DocumentBuffer, EditorState,
    IndentStyle, LineOverlay, WrappedLayout,
};

use crate::layout_tree::{rgb_to_color, segment_to_style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportMetrics {
    pub area: Rect,
    pub gutter_width: u16,
    pub text_area: Rect,
}

pub fn compute_gutter_width(total_lines: usize, show_line_numbers: bool) -> u16 {
    if !show_line_numbers {
        0
    } else {
        (total_lines.max(1).to_string().len() as u16).saturating_add(2)
    }
}

pub fn ensure_cursor_visible(
    scroll_line: &mut usize,
    scroll_col: &mut usize,
    cursor_visual: (usize, usize),
    viewport_size: (usize, usize),
    total_visual_lines: usize,
    soft_wrap: bool,
) {
    let (cursor_visual_row, cursor_visual_col) = cursor_visual;
    let (viewport_height, viewport_width) = viewport_size;
    let max_scroll_line = total_visual_lines.saturating_sub(1);
    if *scroll_line > max_scroll_line {
        *scroll_line = max_scroll_line;
    }

    let height = viewport_height.max(1);
    if cursor_visual_row < *scroll_line {
        *scroll_line = cursor_visual_row;
    } else if cursor_visual_row >= scroll_line.saturating_add(height) {
        *scroll_line = cursor_visual_row.saturating_add(1).saturating_sub(height);
    }

    if soft_wrap {
        *scroll_col = 0;
    } else {
        let width = viewport_width.max(1);
        if cursor_visual_col < *scroll_col {
            *scroll_col = cursor_visual_col;
        } else if cursor_visual_col >= scroll_col.saturating_add(width) {
            *scroll_col = cursor_visual_col.saturating_add(1).saturating_sub(width);
        }
    }
}

fn char_range_on_line(
    doc: &DocumentBuffer,
    line_idx: usize,
    line_char_len: usize,
    start_char: usize,
    end_char: usize,
) -> Option<Range<usize>> {
    if start_char >= end_char {
        return None;
    }
    let start_pos = doc.char_to_position(start_char);
    let end_pos = doc.char_to_position(end_char);
    if end_pos.line < line_idx || start_pos.line > line_idx {
        return None;
    }

    let col_start = if start_pos.line < line_idx {
        0
    } else {
        start_pos.col.min(line_char_len)
    };
    let col_end = if end_pos.line > line_idx {
        line_char_len
    } else {
        end_pos.col.min(line_char_len)
    };

    if col_start < col_end {
        Some(col_start..col_end)
    } else {
        None
    }
}

pub fn render_editor_viewport(
    frame: &mut Frame,
    area: Rect,
    state: &mut EditorState,
    follow_cursor: bool,
) -> (ViewportMetrics, WrappedLayout) {
    let total_lines = state.buffers.active().document.len_lines();
    let gutter_width = compute_gutter_width(total_lines, state.config.line_numbers)
        .min(area.width.saturating_sub(1));
    let text_area = Rect {
        x: area.x.saturating_add(gutter_width),
        y: area.y,
        width: area.width.saturating_sub(gutter_width),
        height: area.height,
    };
    let metrics = ViewportMetrics {
        area,
        gutter_width,
        text_area,
    };

    let soft_wrap = state.config.soft_wrap;
    let tab_width = match state.config.indent.style {
        IndentStyle::Spaces(w) => w.max(1) as usize,
        IndentStyle::Tabs => 4,
    };

    let layout = {
        let active = state.buffers.active();
        compute_wrapped_layout(
            &active.document,
            text_area.width.max(1) as usize,
            soft_wrap,
            tab_width,
        )
    };

    let primary_pos = {
        let active = state.buffers.active();
        active
            .document
            .char_to_position(active.document.selection().primary().head)
    };
    let cursor_visual = layout.logical_to_visual(primary_pos);

    {
        let active = state.buffers.active_mut();
        if follow_cursor {
            ensure_cursor_visible(
                &mut active.scroll_line,
                &mut active.scroll_col,
                cursor_visual,
                (text_area.height as usize, text_area.width as usize),
                layout.len_visual_lines(),
                soft_wrap,
            );
        } else {
            active.scroll_line = active
                .scroll_line
                .min(layout.len_visual_lines().saturating_sub(1));
            if soft_wrap {
                active.scroll_col = 0;
            }
        }
    }

    if area.width == 0 || area.height == 0 {
        return (metrics, layout);
    }

    let active = state.buffers.active();
    let scroll_line = active.scroll_line;
    let h_scroll = if soft_wrap { 0 } else { active.scroll_col };

    let text = active.document.text();
    let gfm_doc = parse_gfm(&text);
    let spell_diagnostics = state
        .spell_engine
        .as_ref()
        .map(|engine| engine.check_document(&text, &gfm_doc.spell_skip_byte_ranges))
        .unwrap_or_default();

    let palette = &state.theme.palette;
    let bg_color = rgb_to_color(palette.background);
    let base_style = Style::default()
        .fg(rgb_to_color(palette.foreground))
        .bg(bg_color);
    let muted_fg = rgb_to_color(palette.muted);

    let buf = frame.buffer_mut();
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(" ");
                cell.set_style(base_style);
            }
        }
    }

    let visible_rows = text_area.height as usize;
    let visible_width = text_area.width as usize;

    for row_offset in 0..visible_rows {
        let visual_row = scroll_line.saturating_add(row_offset);
        let Some(vl) = layout.visual_lines.get(visual_row) else {
            break;
        };
        let y = text_area.y.saturating_add(row_offset as u16);

        if gutter_width > 0 {
            let gutter_fg = if vl.logical_line == primary_pos.line {
                rgb_to_color(palette.gutter_active_fg)
            } else {
                rgb_to_color(palette.gutter_fg)
            };
            let gutter_style = Style::default().fg(gutter_fg).bg(bg_color);
            let gutter_text = if gutter_width >= 2 {
                let num_width = (gutter_width - 2) as usize;
                if vl.wrap_index == 0 {
                    format!("{:>width$} │", vl.logical_line + 1, width = num_width)
                } else {
                    format!("{} │", " ".repeat(num_width))
                }
            } else {
                "│".to_string()
            };

            for (col_idx, ch) in gutter_text.chars().take(gutter_width as usize).enumerate() {
                let x = area.x.saturating_add(col_idx as u16);
                if let Some(cell) = buf.cell_mut((x, y)) {
                    let mut sym = [0u8; 4];
                    cell.set_symbol(ch.encode_utf8(&mut sym));
                    cell.set_style(gutter_style);
                }
            }
        }

        let line_text = active
            .document
            .line_text(vl.logical_line)
            .unwrap_or_default();
        let line_char_len = line_text.chars().count();

        let mut overlay = LineOverlay::default();
        for sel in active.document.selection().ranges() {
            if let Some(range) = char_range_on_line(
                &active.document,
                vl.logical_line,
                line_char_len,
                sel.start(),
                sel.end(),
            ) {
                overlay.selection_cols.push(range);
            }
        }
        for (idx, m) in state.find_replace.matches.iter().enumerate() {
            if let Some(range) = char_range_on_line(
                &active.document,
                vl.logical_line,
                line_char_len,
                m.start,
                m.end,
            ) {
                if state.find_replace.active_match == Some(idx) {
                    overlay.active_search_col = Some(range.clone());
                }
                overlay.search_match_cols.push(range);
            }
        }
        for diag in &spell_diagnostics {
            if diag.line == vl.logical_line && diag.col_start < diag.col_end {
                overlay.spell_error_cols.push(diag.col_start..diag.col_end);
            }
        }

        let segments = highlight_editor_line(
            vl.logical_line,
            &line_text,
            &gfm_doc,
            &state.theme,
            &overlay,
        );

        let mut styled_chars: Vec<(char, Style)> = Vec::with_capacity(line_char_len);
        for seg in &segments {
            let mut style = segment_to_style(seg);
            if seg.bg.is_none() {
                style = style.bg(bg_color);
            }
            for ch in seg.text.chars() {
                styled_chars.push((ch, style));
            }
        }

        let slice_start = vl.char_start.min(styled_chars.len());
        let slice_end = vl.char_end.min(styled_chars.len());
        let visual_slice = &styled_chars[slice_start..slice_end];

        for col_offset in 0..visible_width {
            let visual_col = h_scroll.saturating_add(col_offset);
            let Some(&(ch, style)) = visual_slice.get(visual_col) else {
                break;
            };
            let logical_col = vl.char_start.saturating_add(visual_col);
            let x = text_area.x.saturating_add(col_offset as u16);
            if let Some(cell) = buf.cell_mut((x, y)) {
                if ch == ' ' && vl.indent_guides.contains(&logical_col) {
                    cell.set_symbol("│");
                    cell.set_style(
                        Style::default()
                            .fg(muted_fg)
                            .bg(style.bg.unwrap_or(bg_color)),
                    );
                } else {
                    let mut sym = [0u8; 4];
                    cell.set_symbol(ch.encode_utf8(&mut sym));
                    cell.set_style(style);
                }
            }
        }
    }

    let cursor_style = Style::default()
        .fg(rgb_to_color(palette.cursor_fg))
        .bg(rgb_to_color(palette.cursor_bg));

    for sel in active.document.selection().ranges() {
        let head_pos = active.document.char_to_position(sel.head);
        let (vis_row, vis_col) = layout.logical_to_visual(head_pos);
        if vis_row < scroll_line || vis_row >= scroll_line.saturating_add(visible_rows) {
            continue;
        }
        if vis_col < h_scroll || vis_col >= h_scroll.saturating_add(visible_width) {
            continue;
        }
        let screen_x = text_area.x.saturating_add((vis_col - h_scroll) as u16);
        let screen_y = text_area.y.saturating_add((vis_row - scroll_line) as u16);
        if let Some(cell) = buf.cell_mut((screen_x, screen_y)) {
            cell.set_style(cursor_style);
        }
    }

    (metrics, layout)
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use scriptui_core::{select_all_occurrences, AppConfig, DocumentBuffer, SearchQuery};

    use super::*;

    #[test]
    fn viewport_renders_gutter_gfm_heading_and_indent_guides() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        state.buffers.active_mut().document =
            DocumentBuffer::from_str_content("# Title\n        indented line");

        let backend = TestBackend::new(40, 6);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut metrics = ViewportMetrics {
            area: Rect::default(),
            gutter_width: 0,
            text_area: Rect::default(),
        };

        terminal
            .draw(|frame| {
                let area = frame.area();
                let (m, _) = render_editor_viewport(frame, area, &mut state, true);
                metrics = m;
            })
            .expect("draw viewport");

        let buf = terminal.backend().buffer();
        let gutter_row0: String = (0..metrics.gutter_width)
            .map(|x| buf[(x, 0)].symbol())
            .collect();
        assert!(
            gutter_row0.contains('1'),
            "expected line number 1 in gutter, got {gutter_row0:?}"
        );

        let heading_cell = &buf[(metrics.text_area.x + 2, 0)];
        assert_eq!(heading_cell.fg, rgb_to_color(state.theme.palette.md_h1));

        let row1_text: String = (metrics.text_area.x..metrics.text_area.right())
            .map(|x| buf[(x, 1)].symbol())
            .collect();
        assert!(
            row1_text.contains('│'),
            "expected indent guide on row 1, got {row1_text:?}"
        );
    }

    #[test]
    fn viewport_renders_multi_cursor_and_clamps_scroll() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        {
            let active = state.buffers.active_mut();
            active.document = DocumentBuffer::from_str_content("ab cd ab");
            let count = select_all_occurrences(&mut active.document, &SearchQuery::plain("ab"))
                .expect("select all occurrences");
            assert_eq!(count, 2);
            active.scroll_line = 999;
        }

        let backend = TestBackend::new(40, 6);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut metrics = ViewportMetrics {
            area: Rect::default(),
            gutter_width: 0,
            text_area: Rect::default(),
        };

        terminal
            .draw(|frame| {
                let area = frame.area();
                let (m, _) = render_editor_viewport(frame, area, &mut state, true);
                metrics = m;
            })
            .expect("draw viewport");

        assert_eq!(state.buffers.active().scroll_line, 0);

        let cursor_bg = rgb_to_color(state.theme.palette.cursor_bg);
        let buf = terminal.backend().buffer();
        let ranges = state.buffers.active().document.selection().ranges();
        assert_eq!(ranges.len(), 2);

        for sel in ranges {
            let pos = state.buffers.active().document.char_to_position(sel.head);
            let x = metrics.text_area.x + pos.col as u16;
            let y = metrics.text_area.y + pos.line as u16;
            assert_eq!(
                buf[(x, y)].bg,
                cursor_bg,
                "expected cursor_bg at ({x}, {y}) for head {}",
                sel.head
            );
        }
    }

    #[test]
    fn viewport_preserves_manual_scroll_when_follow_cursor_is_false() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        let content = (1..=20)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        {
            let active = state.buffers.active_mut();
            active.document = DocumentBuffer::from_str_content(&content);
            active.scroll_line = 10;
        }

        let backend = TestBackend::new(40, 6);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                let _ = render_editor_viewport(frame, area, &mut state, false);
            })
            .expect("draw viewport");

        assert_eq!(state.buffers.active().scroll_line, 10);
    }
}
