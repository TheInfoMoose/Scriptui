use crate::buffer::DocumentBuffer;
use crate::selection::Position;
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualLine {
    pub logical_line: usize,
    pub wrap_index: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub indent_guides: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrappedLayout {
    pub visual_lines: Vec<VisualLine>,
    pub viewport_width: usize,
    pub soft_wrap: bool,
}

impl WrappedLayout {
    pub fn len_visual_lines(&self) -> usize {
        self.visual_lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.visual_lines.is_empty()
    }

    pub fn logical_to_visual(&self, pos: Position) -> (usize, usize) {
        let mut last_for_line: Option<(usize, &VisualLine)> = None;
        for (visual_row, vl) in self.visual_lines.iter().enumerate() {
            if vl.logical_line == pos.line {
                if pos.col >= vl.char_start && pos.col < vl.char_end {
                    return (visual_row, pos.col.saturating_sub(vl.char_start));
                }
                last_for_line = Some((visual_row, vl));
            }
        }

        if let Some((visual_row, vl)) = last_for_line {
            let clamped_col = pos.col.min(vl.char_end);
            return (visual_row, clamped_col.saturating_sub(vl.char_start));
        }

        match self.visual_lines.last() {
            Some(last) => (
                self.visual_lines.len() - 1,
                last.char_end.saturating_sub(last.char_start),
            ),
            None => (0, 0),
        }
    }

    pub fn visual_to_logical(&self, visual_row: usize, visual_col: usize) -> Position {
        if self.visual_lines.is_empty() {
            return Position { line: 0, col: 0 };
        }
        let row = visual_row.min(self.visual_lines.len() - 1);
        let vl = &self.visual_lines[row];
        Position {
            line: vl.logical_line,
            col: vl.char_start.saturating_add(visual_col).min(vl.char_end),
        }
    }

    pub fn logical_to_visual_cells(
        &self,
        buf: &DocumentBuffer,
        pos: Position,
        tab_width: usize,
    ) -> (usize, usize) {
        let mut last_for_line: Option<(usize, &VisualLine)> = None;
        for (visual_row, vl) in self.visual_lines.iter().enumerate() {
            if vl.logical_line == pos.line {
                if pos.col >= vl.char_start && pos.col < vl.char_end {
                    let line = buf.line_text(vl.logical_line).unwrap_or_default();
                    return (
                        visual_row,
                        logical_col_to_display_col(&line, vl.char_start, pos.col, tab_width),
                    );
                }
                last_for_line = Some((visual_row, vl));
            }
        }

        if let Some((visual_row, vl)) = last_for_line {
            let clamped_col = pos.col.min(vl.char_end);
            let line = buf.line_text(vl.logical_line).unwrap_or_default();
            return (
                visual_row,
                logical_col_to_display_col(&line, vl.char_start, clamped_col, tab_width),
            );
        }

        match self.visual_lines.last() {
            Some(last) => {
                let line = buf.line_text(last.logical_line).unwrap_or_default();
                (
                    self.visual_lines.len() - 1,
                    logical_col_to_display_col(&line, last.char_start, last.char_end, tab_width),
                )
            }
            None => (0, 0),
        }
    }

    pub fn visual_cells_to_logical(
        &self,
        buf: &DocumentBuffer,
        visual_row: usize,
        cell_col: usize,
        tab_width: usize,
    ) -> Position {
        if self.visual_lines.is_empty() {
            return Position { line: 0, col: 0 };
        }
        let row = visual_row.min(self.visual_lines.len() - 1);
        let vl = &self.visual_lines[row];
        let line = buf.line_text(vl.logical_line).unwrap_or_default();
        Position {
            line: vl.logical_line,
            col: display_col_to_logical_col(&line, vl.char_start, vl.char_end, cell_col, tab_width),
        }
    }
}

pub fn compute_indent_guides(line: &str, tab_width: usize) -> Vec<usize> {
    let step = tab_width.max(1);
    let mut leading_width = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => leading_width += 1,
            '\t' => leading_width += step,
            _ => break,
        }
    }
    let levels = leading_width / step;
    (0..levels).map(|level| level * step).collect()
}

pub fn char_display_width(ch: char, col_cells: usize, tab_width: usize) -> usize {
    if ch == '\t' {
        let step = tab_width.max(1);
        step - (col_cells % step)
    } else {
        UnicodeWidthChar::width(ch).unwrap_or(0)
    }
}

pub fn line_display_width(line: &str, tab_width: usize) -> usize {
    let mut col = 0usize;
    for ch in line.chars() {
        col += char_display_width(ch, col, tab_width);
    }
    col
}

pub fn logical_col_to_display_col(
    line: &str,
    char_start: usize,
    logical_col: usize,
    tab_width: usize,
) -> usize {
    let mut col = 0usize;
    for ch in line
        .chars()
        .skip(char_start)
        .take(logical_col.saturating_sub(char_start))
    {
        col += char_display_width(ch, col, tab_width);
    }
    col
}

pub fn display_col_to_logical_col(
    line: &str,
    char_start: usize,
    char_end: usize,
    target_cell_col: usize,
    tab_width: usize,
) -> usize {
    let mut cell_col = 0usize;
    for (offset, ch) in line
        .chars()
        .skip(char_start)
        .take(char_end.saturating_sub(char_start))
        .enumerate()
    {
        let w = char_display_width(ch, cell_col, tab_width);
        if target_cell_col < cell_col + w {
            return char_start + offset;
        }
        cell_col += w;
    }
    char_end
}

pub fn compute_wrapped_layout(
    buf: &DocumentBuffer,
    viewport_width: usize,
    soft_wrap: bool,
    tab_width: usize,
) -> WrappedLayout {
    let width = viewport_width.max(1);
    let mut visual_lines = Vec::new();

    for line_idx in 0..buf.len_lines() {
        let line = buf.line_text(line_idx).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        let guides = compute_indent_guides(&line, tab_width);

        if !soft_wrap || line_display_width(&line, tab_width) <= width {
            visual_lines.push(VisualLine {
                logical_line: line_idx,
                wrap_index: 0,
                char_start: 0,
                char_end: chars.len(),
                indent_guides: guides,
            });
            continue;
        }

        let mut start = 0usize;
        let mut wrap_index = 0usize;
        while start < chars.len() {
            let mut idx = start;
            let mut cell_width = 0usize;
            while idx < chars.len() {
                let ch_width = char_display_width(chars[idx], cell_width, tab_width);
                if cell_width + ch_width > width {
                    if idx == start {
                        idx += 1;
                    }
                    break;
                }
                cell_width += ch_width;
                idx += 1;
            }
            let max_end = idx;
            let end = if max_end == chars.len() {
                max_end
            } else if let Some(ws) = (start..max_end)
                .rev()
                .find(|&i| chars[i] == ' ' || chars[i] == '\t')
            {
                ws + 1
            } else {
                max_end
            };

            visual_lines.push(VisualLine {
                logical_line: line_idx,
                wrap_index,
                char_start: start,
                char_end: end,
                indent_guides: if wrap_index == 0 {
                    guides.clone()
                } else {
                    Vec::new()
                },
            });

            start = end;
            wrap_index += 1;
        }
    }

    WrappedLayout {
        visual_lines,
        viewport_width: width,
        soft_wrap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_wrap_breaks_on_word_boundaries_and_hard_breaks_long_words() {
        let words_buf = DocumentBuffer::from_str_content("alpha beta gamma");
        let words_layout = compute_wrapped_layout(&words_buf, 10, true, 4);
        assert_eq!(words_layout.len_visual_lines(), 2);
        assert_eq!(words_layout.visual_lines[0].logical_line, 0);
        assert_eq!(words_layout.visual_lines[0].wrap_index, 0);
        assert_eq!(words_layout.visual_lines[0].char_start, 0);
        assert_eq!(words_layout.visual_lines[0].char_end, 6);
        assert_eq!(words_layout.visual_lines[1].logical_line, 0);
        assert_eq!(words_layout.visual_lines[1].wrap_index, 1);
        assert_eq!(words_layout.visual_lines[1].char_start, 6);
        assert_eq!(words_layout.visual_lines[1].char_end, 16);

        let long_buf = DocumentBuffer::from_str_content("abcdefghijkl");
        let long_layout = compute_wrapped_layout(&long_buf, 5, true, 4);
        assert_eq!(long_layout.len_visual_lines(), 3);
        assert_eq!(
            (
                long_layout.visual_lines[0].char_start,
                long_layout.visual_lines[0].char_end
            ),
            (0, 5)
        );
        assert_eq!(
            (
                long_layout.visual_lines[1].char_start,
                long_layout.visual_lines[1].char_end
            ),
            (5, 10)
        );
        assert_eq!(
            (
                long_layout.visual_lines[2].char_start,
                long_layout.visual_lines[2].char_end
            ),
            (10, 12)
        );
    }

    #[test]
    fn no_wrap_mode_preserves_one_visual_line_per_logical_line() {
        let buf = DocumentBuffer::from_str_content("long line here\nsecond");
        let layout = compute_wrapped_layout(&buf, 5, false, 4);
        assert!(!layout.is_empty());
        assert_eq!(layout.len_visual_lines(), 2);
        assert_eq!(layout.visual_lines[0].logical_line, 0);
        assert_eq!(layout.visual_lines[0].char_start, 0);
        assert_eq!(layout.visual_lines[0].char_end, 14);
        assert_eq!(layout.visual_lines[1].logical_line, 1);
        assert_eq!(layout.visual_lines[1].char_start, 0);
        assert_eq!(layout.visual_lines[1].char_end, 6);
    }

    #[test]
    fn logical_visual_roundtrip_and_indent_guides() {
        let buf = DocumentBuffer::from_str_content("        indented text here");
        let layout = compute_wrapped_layout(&buf, 14, true, 4);
        assert!(layout.len_visual_lines() >= 2);
        assert_eq!(layout.visual_lines[0].indent_guides, vec![0, 4]);
        assert!(layout.visual_lines[1].indent_guides.is_empty());

        for col in [0, 4, 7, 8, 12, 21, 22, 26] {
            let pos = Position { line: 0, col };
            let (vr, vc) = layout.logical_to_visual(pos);
            assert_eq!(layout.visual_to_logical(vr, vc), pos);
        }
    }

    #[test]
    fn soft_wrap_accounts_for_cjk_wide_chars_and_tab_stops() {
        let cjk_buf = DocumentBuffer::from_str_content("日本語テスト");
        assert_eq!(line_display_width("日本語テスト", 4), 12);
        let cjk_layout = compute_wrapped_layout(&cjk_buf, 6, true, 4);
        assert_eq!(cjk_layout.len_visual_lines(), 2);
        assert_eq!(
            (
                cjk_layout.visual_lines[0].char_start,
                cjk_layout.visual_lines[0].char_end
            ),
            (0, 3)
        );
        assert_eq!(
            (
                cjk_layout.visual_lines[1].char_start,
                cjk_layout.visual_lines[1].char_end
            ),
            (3, 6)
        );

        let tab_text = "\tabcdef";
        assert_eq!(line_display_width(tab_text, 4), 10);
        let tab_buf = DocumentBuffer::from_str_content(tab_text);
        let tab_layout = compute_wrapped_layout(&tab_buf, 8, true, 4);
        assert!(tab_layout.len_visual_lines() >= 2);
        assert!(
            (
                tab_layout.visual_lines[0].char_start,
                tab_layout.visual_lines[0].char_end
            ) == (0, 1)
                || (
                    tab_layout.visual_lines[0].char_start,
                    tab_layout.visual_lines[0].char_end
                ) == (0, 5)
        );
        let tab_chars: Vec<char> = tab_text.chars().collect();
        for vl in &tab_layout.visual_lines {
            let segment: String = tab_chars[vl.char_start..vl.char_end].iter().collect();
            assert!(line_display_width(&segment, 4) <= 8);
        }

        let narrow_buf = DocumentBuffer::from_str_content("日");
        let narrow_layout = compute_wrapped_layout(&narrow_buf, 1, true, 4);
        assert_eq!(narrow_layout.len_visual_lines(), 1);
        assert_eq!(
            (
                narrow_layout.visual_lines[0].char_start,
                narrow_layout.visual_lines[0].char_end
            ),
            (0, 1)
        );
    }

    #[test]
    fn display_cell_mapping_handles_tabs_and_cjk_wide_chars() {
        let tab_buf = DocumentBuffer::from_str_content("\tabc");
        let tab_layout = compute_wrapped_layout(&tab_buf, 20, true, 4);
        assert_eq!(
            tab_layout.logical_to_visual_cells(&tab_buf, Position { line: 0, col: 0 }, 4),
            (0, 0)
        );
        assert_eq!(
            tab_layout.logical_to_visual_cells(&tab_buf, Position { line: 0, col: 1 }, 4),
            (0, 4)
        );
        assert_eq!(
            tab_layout.logical_to_visual_cells(&tab_buf, Position { line: 0, col: 2 }, 4),
            (0, 5)
        );
        for cell in 0..=3 {
            assert_eq!(
                tab_layout.visual_cells_to_logical(&tab_buf, 0, cell, 4),
                Position { line: 0, col: 0 }
            );
        }
        assert_eq!(
            tab_layout.visual_cells_to_logical(&tab_buf, 0, 4, 4),
            Position { line: 0, col: 1 }
        );
        assert_eq!(
            tab_layout.visual_cells_to_logical(&tab_buf, 0, 5, 4),
            Position { line: 0, col: 2 }
        );
        assert_eq!(
            tab_layout.visual_cells_to_logical(&tab_buf, 0, 99, 4),
            Position { line: 0, col: 4 }
        );

        let cjk_buf = DocumentBuffer::from_str_content("日本語a");
        let cjk_layout = compute_wrapped_layout(&cjk_buf, 20, true, 4);
        assert_eq!(
            cjk_layout.logical_to_visual_cells(&cjk_buf, Position { line: 0, col: 0 }, 4),
            (0, 0)
        );
        assert_eq!(
            cjk_layout.logical_to_visual_cells(&cjk_buf, Position { line: 0, col: 1 }, 4),
            (0, 2)
        );
        assert_eq!(
            cjk_layout.logical_to_visual_cells(&cjk_buf, Position { line: 0, col: 2 }, 4),
            (0, 4)
        );
        assert_eq!(
            cjk_layout.logical_to_visual_cells(&cjk_buf, Position { line: 0, col: 3 }, 4),
            (0, 6)
        );
        for cell in [0, 1] {
            assert_eq!(
                cjk_layout.visual_cells_to_logical(&cjk_buf, 0, cell, 4),
                Position { line: 0, col: 0 }
            );
        }
        for cell in [2, 3] {
            assert_eq!(
                cjk_layout.visual_cells_to_logical(&cjk_buf, 0, cell, 4),
                Position { line: 0, col: 1 }
            );
        }
        for cell in [4, 5] {
            assert_eq!(
                cjk_layout.visual_cells_to_logical(&cjk_buf, 0, cell, 4),
                Position { line: 0, col: 2 }
            );
        }
        assert_eq!(
            cjk_layout.visual_cells_to_logical(&cjk_buf, 0, 6, 4),
            Position { line: 0, col: 3 }
        );

        let wrapped_buf = DocumentBuffer::from_str_content("日本語テスト");
        let wrapped_layout = compute_wrapped_layout(&wrapped_buf, 6, true, 4);
        assert_eq!(
            wrapped_layout.logical_to_visual_cells(&wrapped_buf, Position { line: 0, col: 4 }, 4),
            (1, 2)
        );
        assert_eq!(
            wrapped_layout.visual_cells_to_logical(&wrapped_buf, 1, 2, 4),
            Position { line: 0, col: 4 }
        );
    }
}
