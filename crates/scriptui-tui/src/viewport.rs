use std::ops::Range;

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;
use scriptui_core::{
    char_display_width, compute_wrapped_layout, highlight_editor_line, DocumentBuffer, EditorState,
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
    let cursor_visual =
        layout.logical_to_visual_cells(&state.buffers.active().document, primary_pos, tab_width);

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

    let (gfm_doc, spell_diagnostics) = {
        let (g, s) = state.active_analysis();
        (g.clone(), s.to_vec())
    };

    let active = state.buffers.active();
    let scroll_line = active.scroll_line;
    let h_scroll = if soft_wrap { 0 } else { active.scroll_col };

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

        let mut display_col = 0usize;
        let mut last_cell: Option<(u16, u16)> = None;
        let right_edge = h_scroll.saturating_add(visible_width);
        for &(ch, style) in visual_slice {
            let w = char_display_width(ch, display_col, tab_width);
            if w == 0 {
                if !ch.is_control() {
                    if let Some(pos) = last_cell {
                        if let Some(cell) = buf.cell_mut(pos) {
                            let mut combined = cell.symbol().to_string();
                            combined.push(ch);
                            cell.set_symbol(&combined);
                        }
                    }
                }
                continue;
            }
            if display_col >= right_edge {
                break;
            }
            if ch == '\t' {
                let mut rendered_cell = None;
                for step in 0..w {
                    let cell_col = display_col + step;
                    if cell_col >= h_scroll && cell_col < right_edge {
                        let x = text_area.x.saturating_add((cell_col - h_scroll) as u16);
                        if let Some(cell) = buf.cell_mut((x, y)) {
                            if step == 0
                                && vl.wrap_index == 0
                                && vl.indent_guides.contains(&cell_col)
                            {
                                cell.set_symbol("│");
                                cell.set_style(
                                    Style::default()
                                        .fg(muted_fg)
                                        .bg(style.bg.unwrap_or(bg_color)),
                                );
                            } else {
                                cell.set_symbol(" ");
                                cell.set_style(style);
                            }
                            rendered_cell = Some((x, y));
                        }
                    }
                }
                last_cell = rendered_cell;
            } else if display_col >= h_scroll && display_col.saturating_add(w) <= right_edge {
                let x = text_area.x.saturating_add((display_col - h_scroll) as u16);
                let mut rendered_cell = None;
                if ch == ' ' && vl.wrap_index == 0 && vl.indent_guides.contains(&display_col) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol("│");
                        cell.set_style(
                            Style::default()
                                .fg(muted_fg)
                                .bg(style.bg.unwrap_or(bg_color)),
                        );
                        rendered_cell = Some((x, y));
                    }
                } else {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        let mut sym = [0u8; 4];
                        cell.set_symbol(ch.encode_utf8(&mut sym));
                        cell.set_style(style);
                        rendered_cell = Some((x, y));
                    }
                    for cont in 1..w {
                        if let Some(cell) = buf.cell_mut((x.saturating_add(cont as u16), y)) {
                            cell.set_symbol("");
                            cell.set_style(style);
                        }
                    }
                }
                last_cell = rendered_cell;
            } else if w >= 2
                && display_col >= h_scroll
                && display_col < right_edge
                && display_col.saturating_add(w) > right_edge
            {
                for cell_col in display_col..right_edge {
                    let x = text_area.x.saturating_add((cell_col - h_scroll) as u16);
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(" ");
                        cell.set_style(style);
                    }
                }
                last_cell = None;
            } else {
                last_cell = None;
            }
            display_col += w;
        }
    }

    let cursor_style = Style::default()
        .fg(rgb_to_color(palette.cursor_fg))
        .bg(rgb_to_color(palette.cursor_bg));

    for sel in active.document.selection().ranges() {
        let head_pos = active.document.char_to_position(sel.head);
        let (vis_row, vis_col) =
            layout.logical_to_visual_cells(&active.document, head_pos, tab_width);
        if vis_row < scroll_line || vis_row >= scroll_line.saturating_add(visible_rows) {
            continue;
        }
        if vis_col < h_scroll || vis_col >= h_scroll.saturating_add(visible_width) {
            continue;
        }
        let cursor_w = active
            .document
            .line_text(head_pos.line)
            .and_then(|line| line.chars().nth(head_pos.col))
            .map_or(1, |ch| {
                if ch.is_control() {
                    1
                } else {
                    char_display_width(ch, vis_col, tab_width).max(1)
                }
            });
        let screen_x = text_area.x.saturating_add((vis_col - h_scroll) as u16);
        let screen_y = text_area.y.saturating_add((vis_row - scroll_line) as u16);
        let cursor_end_x = screen_x
            .saturating_add(cursor_w.min(u16::MAX as usize) as u16)
            .min(text_area.right());
        for x in screen_x..cursor_end_x {
            if let Some(cell) = buf.cell_mut((x, screen_y)) {
                cell.set_style(cursor_style);
            }
        }
    }

    (metrics, layout)
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use scriptui_core::{
        select_all_occurrences, AppConfig, DocumentBuffer, Position, SearchQuery, SelectionSet,
    };

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

    #[test]
    fn viewport_expands_tabs_and_aligns_cjk_wide_chars_and_cursor() {
        let mut config = AppConfig::default();
        config.indent.style = IndentStyle::Tabs;
        let mut state = EditorState::new(config).expect("default editor state");
        {
            let active = state.buffers.active_mut();
            active.document = DocumentBuffer::from_str_content("\tabc\n日本語X");
            let a_offset = active
                .document
                .position_to_char(Position { line: 0, col: 1 });
            active.document.set_selection(SelectionSet::point(a_offset));
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
            .expect("draw viewport with cursor on 'a'");

        let cursor_bg = rgb_to_color(state.theme.palette.cursor_bg);
        let buf = terminal.backend().buffer();
        let tx = metrics.text_area.x;
        let ty = metrics.text_area.y;

        assert_eq!(buf[(tx, ty)].symbol(), "│");
        assert_eq!(buf[(tx + 1, ty)].symbol(), " ");
        assert_eq!(buf[(tx + 2, ty)].symbol(), " ");
        assert_eq!(buf[(tx + 3, ty)].symbol(), " ");
        assert_eq!(buf[(tx + 4, ty)].symbol(), "a");
        assert_eq!(buf[(tx + 4, ty)].bg, cursor_bg);

        {
            let active = state.buffers.active_mut();
            let x_offset = active
                .document
                .position_to_char(Position { line: 1, col: 3 });
            active.document.set_selection(SelectionSet::point(x_offset));
        }

        terminal
            .draw(|frame| {
                let area = frame.area();
                let (m, _) = render_editor_viewport(frame, area, &mut state, true);
                metrics = m;
            })
            .expect("draw viewport with cursor on 'X'");

        let buf = terminal.backend().buffer();
        let tx = metrics.text_area.x;
        let row1 = metrics.text_area.y + 1;
        assert_eq!(buf[(tx, row1)].symbol(), "日");
        assert_eq!(buf[(tx + 2, row1)].symbol(), "本");
        assert_eq!(buf[(tx + 4, row1)].symbol(), "語");
        assert_eq!(buf[(tx + 6, row1)].symbol(), "X");
        assert_eq!(buf[(tx + 6, row1)].bg, cursor_bg);
    }

    #[test]
    fn viewport_attaches_combining_marks_and_styles_wide_char_cursor_cells() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        {
            let active = state.buffers.active_mut();
            active.document = DocumentBuffer::from_str_content("e\u{0301} 漢a");
            let wide_offset = active
                .document
                .position_to_char(Position { line: 0, col: 3 });
            active
                .document
                .set_selection(SelectionSet::point(wide_offset));
        }

        let backend = TestBackend::new(40, 6);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut metrics = ViewportMetrics {
            area: Rect::default(),
            gutter_width: 0,
            text_area: Rect::default(),
        };

        let mut frame_buf = ratatui::buffer::Buffer::empty(Rect::default());
        terminal
            .draw(|frame| {
                let area = frame.area();
                let (m, _) = render_editor_viewport(frame, area, &mut state, true);
                metrics = m;
                frame_buf = frame.buffer_mut().clone();
            })
            .expect("draw viewport with combining mark and wide cursor");

        let cursor_fg = rgb_to_color(state.theme.palette.cursor_fg);
        let cursor_bg = rgb_to_color(state.theme.palette.cursor_bg);
        let tx = metrics.text_area.x;
        let ty = metrics.text_area.y;

        assert_eq!(frame_buf[(tx, ty)].symbol(), "e\u{0301}");
        assert_eq!(frame_buf[(tx + 1, ty)].symbol(), " ");
        assert_eq!(frame_buf[(tx + 2, ty)].symbol(), "漢");
        assert_eq!(frame_buf[(tx + 2, ty)].fg, cursor_fg);
        assert_eq!(frame_buf[(tx + 2, ty)].bg, cursor_bg);
        assert_eq!(frame_buf[(tx + 3, ty)].fg, cursor_fg);
        assert_eq!(frame_buf[(tx + 3, ty)].bg, cursor_bg);
        assert_eq!(frame_buf[(tx + 4, ty)].symbol(), "a");
        assert_ne!(frame_buf[(tx + 4, ty)].bg, cursor_bg);

        let backend_buf = terminal.backend().buffer();
        assert_eq!(backend_buf[(tx, ty)].symbol(), "e\u{0301}");
        assert_eq!(backend_buf[(tx + 2, ty)].symbol(), "漢");
        assert_eq!(backend_buf[(tx + 2, ty)].fg, cursor_fg);
        assert_eq!(backend_buf[(tx + 2, ty)].bg, cursor_bg);

        let narrow_config = AppConfig {
            line_numbers: false,
            soft_wrap: false,
            ..AppConfig::default()
        };
        let mut narrow_state = EditorState::new(narrow_config).expect("narrow editor state");
        {
            let active = narrow_state.buffers.active_mut();
            active.document = DocumentBuffer::from_str_content("e\u{0301} 漢a");
            let wide_offset = active
                .document
                .position_to_char(Position { line: 0, col: 3 });
            active
                .document
                .set_selection(SelectionSet::point(wide_offset));
        }

        let narrow_backend = TestBackend::new(3, 2);
        let mut narrow_terminal = Terminal::new(narrow_backend).expect("narrow test terminal");
        let mut narrow_metrics = ViewportMetrics {
            area: Rect::default(),
            gutter_width: 0,
            text_area: Rect::default(),
        };

        narrow_terminal
            .draw(|frame| {
                let area = frame.area();
                let (m, _) = render_editor_viewport(frame, area, &mut narrow_state, false);
                narrow_metrics = m;
            })
            .expect("draw narrow viewport where wide char straddles right edge");

        assert_eq!(narrow_metrics.text_area.width, 3);
        let narrow_buf = narrow_terminal.backend().buffer();
        let ntx = narrow_metrics.text_area.x;
        let nty = narrow_metrics.text_area.y;
        assert_eq!(narrow_buf[(ntx, nty)].symbol(), "e\u{0301}");
        assert_eq!(narrow_buf[(ntx + 1, nty)].symbol(), " ");
        assert_eq!(narrow_buf[(ntx + 2, nty)].symbol(), " ");
        assert_eq!(narrow_buf[(ntx + 2, nty)].fg, cursor_fg);
        assert_eq!(narrow_buf[(ntx + 2, nty)].bg, cursor_bg);
    }
}
