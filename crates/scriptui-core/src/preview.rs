use comrak::nodes::{AstNode, ListType, NodeValue, Sourcepos, TableAlignment};
use comrak::{parse_document, Arena};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::highlight::{CodeHighlighter, StyleFlags, StyledSegment};
use crate::markdown::gfm_options;
use crate::theme::{RgbColor, Theme, ThemePalette};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewLine {
    pub source_line: usize,
    pub segments: Vec<StyledSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewDocument {
    pub lines: Vec<PreviewLine>,
}

impl PreviewDocument {
    pub fn source_line_to_preview_line(&self, source_line: usize) -> usize {
        if self.lines.is_empty() {
            return 0;
        }
        self.lines
            .iter()
            .position(|line| line.source_line >= source_line)
            .unwrap_or_else(|| self.lines.len().saturating_sub(1))
    }

    pub fn preview_line_to_source_line(&self, preview_line: usize) -> usize {
        if let Some(line) = self.lines.get(preview_line) {
            line.source_line
        } else if let Some(last) = self.lines.last() {
            last.source_line
        } else {
            0
        }
    }

    pub fn plain_lines(&self) -> Vec<String> {
        self.lines
            .iter()
            .map(|line| {
                line.segments
                    .iter()
                    .map(|seg| seg.text.as_str())
                    .collect::<String>()
            })
            .collect()
    }
}

pub fn build_preview(markdown: &str, theme: &Theme, width: usize) -> PreviewDocument {
    let effective_width = width.max(20);
    let arena = Arena::new();
    let options = gfm_options();
    let root = parse_document(&arena, markdown, &options);
    let highlighter = CodeHighlighter::new();

    let mut renderer = PreviewRenderer {
        palette: &theme.palette,
        highlighter: &highlighter,
        width: effective_width,
        line_offset: 0,
        last_source_line: 0,
        lines: Vec::new(),
        prefix_stack: Vec::new(),
        pending_first_prefix: None,
    };

    renderer.render_document(root);

    PreviewDocument {
        lines: renderer.lines,
    }
}

#[derive(Debug, Clone, Copy)]
struct InlineStyle {
    fg: RgbColor,
    bg: Option<RgbColor>,
    flags: StyleFlags,
}

impl InlineStyle {
    fn plain(fg: RgbColor) -> Self {
        Self {
            fg,
            bg: None,
            flags: StyleFlags::default(),
        }
    }
}

enum InlineToken {
    Segment(StyledSegment),
    HardBreak,
}

#[derive(Debug, Clone, Copy)]
enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AlertKind {
    fn label(self) -> &'static str {
        match self {
            AlertKind::Note => "[NOTE]",
            AlertKind::Tip => "[TIP]",
            AlertKind::Important => "[IMPORTANT]",
            AlertKind::Warning => "[WARNING]",
            AlertKind::Caution => "[CAUTION]",
        }
    }

    fn color(self, palette: &ThemePalette) -> RgbColor {
        match self {
            AlertKind::Note => palette.md_alert_note,
            AlertKind::Tip => palette.md_alert_tip,
            AlertKind::Important => palette.accent,
            AlertKind::Warning => palette.md_alert_warning,
            AlertKind::Caution => palette.md_alert_caution,
        }
    }
}

struct PreviewRenderer<'a> {
    palette: &'a ThemePalette,
    highlighter: &'a CodeHighlighter,
    width: usize,
    line_offset: usize,
    last_source_line: usize,
    lines: Vec<PreviewLine>,
    prefix_stack: Vec<StyledSegment>,
    pending_first_prefix: Option<(StyledSegment, StyledSegment)>,
}

impl<'a> PreviewRenderer<'a> {
    fn source_line_from_pos(&self, pos: Sourcepos) -> usize {
        if pos.start.line > 0 {
            pos.start.line.saturating_sub(1) + self.line_offset
        } else {
            self.last_source_line
        }
    }

    fn end_line_from_pos(&self, pos: Sourcepos) -> usize {
        if pos.end.line > 0 {
            pos.end.line.saturating_sub(1) + self.line_offset
        } else {
            self.source_line_from_pos(pos)
        }
    }

    fn prefix_char_width(&self) -> usize {
        let stack_width: usize = self
            .prefix_stack
            .iter()
            .map(|seg| UnicodeWidthStr::width(seg.text.as_str()))
            .sum();
        let pending_width = self
            .pending_first_prefix
            .as_ref()
            .map(|(first, _)| UnicodeWidthStr::width(first.text.as_str()))
            .unwrap_or(0);
        stack_width + pending_width
    }

    fn available_width(&self) -> usize {
        self.width.saturating_sub(self.prefix_char_width()).max(8)
    }

    fn emit_line(&mut self, source_line: usize, body: Vec<StyledSegment>) {
        self.last_source_line = source_line;
        let mut segments: Vec<StyledSegment> =
            Vec::with_capacity(self.prefix_stack.len() + body.len() + 1);

        for seg in &self.prefix_stack {
            push_coalesced(&mut segments, &seg.text, seg.fg, seg.bg, seg.flags);
        }

        if let Some((first_seg, cont_seg)) = self.pending_first_prefix.take() {
            push_coalesced(
                &mut segments,
                &first_seg.text,
                first_seg.fg,
                first_seg.bg,
                first_seg.flags,
            );
            self.prefix_stack.push(cont_seg);
        }

        for seg in body {
            push_coalesced(&mut segments, &seg.text, seg.fg, seg.bg, seg.flags);
        }

        self.lines.push(PreviewLine {
            source_line,
            segments,
        });
    }

    fn emit_blank_separator(&mut self, source_line: usize) {
        if let Some(last) = self.lines.last() {
            if !last.segments.is_empty() {
                let mut prefix_only = Vec::new();
                for seg in &self.prefix_stack {
                    push_coalesced(&mut prefix_only, &seg.text, seg.fg, seg.bg, seg.flags);
                }
                self.lines.push(PreviewLine {
                    source_line,
                    segments: prefix_only,
                });
            }
        }
    }

    fn render_document<'ast>(&mut self, root: &'ast AstNode<'ast>) {
        let mut footnotes: Vec<&'ast AstNode<'ast>> = Vec::new();
        let mut rendered_any = false;

        for child in root.children() {
            let is_front_matter = {
                let data = child.data.borrow();
                if let NodeValue::FrontMatter(raw) = &data.value {
                    self.line_offset = raw.bytes().filter(|&b| b == b'\n').count();
                    true
                } else {
                    false
                }
            };
            if is_front_matter {
                continue;
            }

            let is_footnote = matches!(child.data.borrow().value, NodeValue::FootnoteDefinition(_));
            if is_footnote {
                footnotes.push(child);
                continue;
            }

            if rendered_any {
                let src = self.source_line_from_pos(child.data.borrow().sourcepos);
                self.emit_blank_separator(src);
            }
            self.render_block(child);
            rendered_any = true;
        }

        if !footnotes.is_empty() {
            let first_src = self.source_line_from_pos(footnotes[0].data.borrow().sourcepos);
            let sep_line = first_src.max(self.last_source_line);
            if rendered_any {
                self.emit_blank_separator(sep_line);
            }
            let rule_len = self.available_width().min(20);
            self.emit_line(
                sep_line,
                vec![StyledSegment {
                    text: "─".repeat(rule_len),
                    fg: self.palette.md_hr,
                    bg: None,
                    flags: StyleFlags::default(),
                }],
            );
            for fn_node in footnotes {
                self.render_footnote_definition(fn_node);
            }
        }
    }

    fn render_block<'ast>(&mut self, node: &'ast AstNode<'ast>) {
        let data = node.data.borrow();
        let sourcepos = data.sourcepos;
        let source_line = self.source_line_from_pos(sourcepos);
        let end_line = self.end_line_from_pos(sourcepos);

        match &data.value {
            NodeValue::Heading(heading) => {
                let level = heading.level;
                drop(data);
                self.render_heading(node, level, source_line);
            }
            NodeValue::Paragraph => {
                drop(data);
                let tokens =
                    self.collect_inlines(node, InlineStyle::plain(self.palette.foreground));
                self.render_wrapped_tokens(tokens, source_line);
            }
            NodeValue::BlockQuote => {
                drop(data);
                self.render_blockquote(node, source_line);
            }
            NodeValue::List(list) => {
                let list_type = list.list_type;
                let start = list.start;
                drop(data);
                self.render_list(node, list_type, start);
            }
            NodeValue::CodeBlock(cb) => {
                let info = cb.info.clone();
                let literal = cb.literal.clone();
                let fenced = cb.fenced;
                drop(data);
                self.render_code_block(&info, &literal, fenced, source_line, end_line);
            }
            NodeValue::Table(table) => {
                let alignments = table.alignments.clone();
                drop(data);
                self.render_table(node, &alignments, source_line, end_line);
            }
            NodeValue::ThematicBreak => {
                drop(data);
                let avail = self.available_width();
                self.emit_line(
                    source_line,
                    vec![StyledSegment {
                        text: "─".repeat(avail),
                        fg: self.palette.md_hr,
                        bg: None,
                        flags: StyleFlags::default(),
                    }],
                );
            }
            NodeValue::HtmlBlock(html) => {
                let literal = html.literal.clone();
                drop(data);
                for (idx, line) in literal.lines().enumerate() {
                    self.emit_line(
                        source_line + idx,
                        vec![StyledSegment {
                            text: line.to_string(),
                            fg: self.palette.muted,
                            bg: None,
                            flags: StyleFlags::default(),
                        }],
                    );
                }
            }
            NodeValue::FootnoteDefinition(_) => {
                drop(data);
                self.render_footnote_definition(node);
            }
            _ => {
                drop(data);
                for child in node.children() {
                    self.render_block(child);
                }
            }
        }
    }

    fn render_heading<'ast>(&mut self, node: &'ast AstNode<'ast>, level: u8, source_line: usize) {
        let fg = match level {
            1 => self.palette.md_h1,
            2 => self.palette.md_h2,
            _ => self.palette.md_h3,
        };
        let style = InlineStyle {
            fg,
            bg: None,
            flags: StyleFlags {
                bold: true,
                ..StyleFlags::default()
            },
        };
        let tokens = self.collect_inlines(node, style);
        self.render_wrapped_tokens(tokens, source_line);

        if level == 1 || level == 2 {
            let rule_char = if level == 1 { "━" } else { "─" };
            let avail = self.available_width();
            self.emit_line(
                source_line,
                vec![StyledSegment {
                    text: rule_char.repeat(avail),
                    fg,
                    bg: None,
                    flags: StyleFlags {
                        dim: true,
                        ..StyleFlags::default()
                    },
                }],
            );
        }
    }

    fn render_blockquote<'ast>(&mut self, node: &'ast AstNode<'ast>, source_line: usize) {
        if let Some((alert_kind, skip_count)) = detect_alert_in_blockquote(node) {
            let color = alert_kind.color(self.palette);
            let bar = StyledSegment {
                text: "│ ".to_string(),
                fg: color,
                bg: None,
                flags: StyleFlags::default(),
            };
            self.prefix_stack.push(bar);
            self.emit_line(
                source_line,
                vec![StyledSegment {
                    text: alert_kind.label().to_string(),
                    fg: color,
                    bg: None,
                    flags: StyleFlags {
                        bold: true,
                        ..StyleFlags::default()
                    },
                }],
            );

            let mut children = node.children();
            if let Some(first_para) = children.next() {
                let remaining_inlines: Vec<&'ast AstNode<'ast>> =
                    first_para.children().skip(skip_count).collect();
                if !remaining_inlines.is_empty() {
                    let mut tokens = Vec::new();
                    let base_style = InlineStyle::plain(self.palette.foreground);
                    for inline in remaining_inlines {
                        self.collect_inline_node(inline, base_style, &mut tokens);
                    }
                    let body_line = source_line + 1;
                    self.render_wrapped_tokens(tokens, body_line);
                }
            }

            for child in children {
                self.render_block(child);
            }

            self.prefix_stack.pop();
            return;
        }

        let bar = StyledSegment {
            text: "│ ".to_string(),
            fg: self.palette.md_blockquote,
            bg: None,
            flags: StyleFlags::default(),
        };
        self.prefix_stack.push(bar);

        let mut first = true;
        for child in node.children() {
            if !first {
                let src = self.source_line_from_pos(child.data.borrow().sourcepos);
                self.emit_blank_separator(src);
            }
            self.render_block(child);
            first = false;
        }

        self.prefix_stack.pop();
    }

    fn render_list<'ast>(&mut self, node: &'ast AstNode<'ast>, list_type: ListType, start: usize) {
        let mut ordinal = start.max(1);
        for item in node.children() {
            let marker_text = {
                let data = item.data.borrow();
                match &data.value {
                    NodeValue::TaskItem(checked) => match checked {
                        Some('x' | 'X') => "[x] ".to_string(),
                        _ => "[ ] ".to_string(),
                    },
                    NodeValue::Item(_) => match list_type {
                        ListType::Bullet => "• ".to_string(),
                        ListType::Ordered => {
                            let label = format!("{ordinal}. ");
                            ordinal += 1;
                            label
                        }
                    },
                    _ => "• ".to_string(),
                }
            };

            let marker_fg = if marker_text == "[x] " {
                self.palette.status_ok
            } else {
                self.palette.md_list_marker
            };
            let marker_width = UnicodeWidthStr::width(marker_text.as_str());
            let first_prefix = StyledSegment {
                text: marker_text,
                fg: marker_fg,
                bg: None,
                flags: StyleFlags {
                    bold: true,
                    ..StyleFlags::default()
                },
            };
            let cont_prefix = StyledSegment {
                text: " ".repeat(marker_width),
                fg: self.palette.foreground,
                bg: None,
                flags: StyleFlags::default(),
            };

            let stack_len_before = self.prefix_stack.len();
            self.pending_first_prefix = Some((first_prefix, cont_prefix));

            let mut has_children = false;
            for child in item.children() {
                has_children = true;
                self.render_block(child);
            }

            if !has_children {
                let src = self.source_line_from_pos(item.data.borrow().sourcepos);
                self.emit_line(src, Vec::new());
            }

            // If pending_first_prefix was consumed, it pushed cont_prefix onto prefix_stack.
            self.pending_first_prefix = None;
            self.prefix_stack.truncate(stack_len_before);
        }
    }

    fn render_code_block(
        &mut self,
        info: &str,
        literal: &str,
        fenced: bool,
        source_line: usize,
        end_line: usize,
    ) {
        let lang = info.split_whitespace().next().unwrap_or_default();
        let avail = self.available_width();
        let border_fg = self.palette.md_table_border;

        let mut top_segments = Vec::new();
        if lang.is_empty() {
            let dashes = avail.saturating_sub(2);
            top_segments.push(StyledSegment {
                text: format!("┌{}┐", "─".repeat(dashes)),
                fg: border_fg,
                bg: None,
                flags: StyleFlags::default(),
            });
        } else {
            let label = format!(" {lang} ");
            let used = 2 + UnicodeWidthStr::width(label.as_str()) + 1; // "┌─" + label + "┐"
            let trailing_dashes = avail.saturating_sub(used);
            top_segments.push(StyledSegment {
                text: "┌─".to_string(),
                fg: border_fg,
                bg: None,
                flags: StyleFlags::default(),
            });
            top_segments.push(StyledSegment {
                text: label,
                fg: self.palette.accent,
                bg: None,
                flags: StyleFlags {
                    bold: true,
                    ..StyleFlags::default()
                },
            });
            top_segments.push(StyledSegment {
                text: format!("{}┐", "─".repeat(trailing_dashes)),
                fg: border_fg,
                bg: None,
                flags: StyleFlags::default(),
            });
        }
        self.emit_line(source_line, top_segments);

        let trimmed = literal.strip_suffix('\n').unwrap_or(literal);
        let code_lines: Vec<&str> = if trimmed.is_empty() {
            Vec::new()
        } else {
            trimmed.split('\n').collect()
        };
        let highlighted = self
            .highlighter
            .highlight_block(lang, &code_lines, self.palette);
        let body_start_line = if fenced { source_line + 1 } else { source_line };

        for (idx, segs) in highlighted.into_iter().enumerate() {
            let mut row = Vec::with_capacity(segs.len() + 1);
            row.push(StyledSegment {
                text: "│ ".to_string(),
                fg: border_fg,
                bg: None,
                flags: StyleFlags::default(),
            });
            row.extend(segs);
            self.emit_line(body_start_line + idx, row);
        }

        let bottom_dashes = avail.saturating_sub(2);
        self.emit_line(
            end_line.max(body_start_line),
            vec![StyledSegment {
                text: format!("└{}┘", "─".repeat(bottom_dashes)),
                fg: border_fg,
                bg: None,
                flags: StyleFlags::default(),
            }],
        );
    }

    fn render_table<'ast>(
        &mut self,
        node: &'ast AstNode<'ast>,
        alignments: &[TableAlignment],
        source_line: usize,
        end_line: usize,
    ) {
        struct TableRowData {
            is_header: bool,
            source_line: usize,
            cells: Vec<Vec<StyledSegment>>,
        }

        let mut rows: Vec<TableRowData> = Vec::new();
        let mut col_count = alignments.len();

        for row_node in node.children() {
            let (is_header, row_src) = {
                let data = row_node.data.borrow();
                let is_hdr = matches!(data.value, NodeValue::TableRow(true));
                (is_hdr, self.source_line_from_pos(data.sourcepos))
            };

            let base_style = if is_header {
                InlineStyle {
                    fg: self.palette.md_strong,
                    bg: None,
                    flags: StyleFlags {
                        bold: true,
                        ..StyleFlags::default()
                    },
                }
            } else {
                InlineStyle::plain(self.palette.foreground)
            };

            let mut cells = Vec::new();
            for cell_node in row_node.children() {
                let tokens = self.collect_inlines(cell_node, base_style);
                let mut cell_segs = Vec::new();
                for token in tokens {
                    match token {
                        InlineToken::Segment(seg) => {
                            push_coalesced(&mut cell_segs, &seg.text, seg.fg, seg.bg, seg.flags);
                        }
                        InlineToken::HardBreak => {
                            push_coalesced(
                                &mut cell_segs,
                                " ",
                                base_style.fg,
                                base_style.bg,
                                base_style.flags,
                            );
                        }
                    }
                }
                cells.push(cell_segs);
            }
            col_count = col_count.max(cells.len());
            rows.push(TableRowData {
                is_header,
                source_line: row_src,
                cells,
            });
        }

        if col_count == 0 || rows.is_empty() {
            return;
        }

        let mut col_widths = vec![1usize; col_count];
        for row in &rows {
            for (c, cell) in row.cells.iter().enumerate() {
                let w = segments_char_width(cell);
                if w > col_widths[c] {
                    col_widths[c] = w;
                }
            }
        }

        let avail = self.available_width();
        let border_overhead = col_count * 3 + 1;
        let max_content = avail.saturating_sub(border_overhead);
        let total_content: usize = col_widths.iter().sum();
        if total_content > max_content {
            let target = max_content.max(col_count);
            let orig_widths = col_widths.clone();
            let mut remainders: Vec<(usize, usize)> = Vec::with_capacity(col_count);
            for (idx, &w) in orig_widths.iter().enumerate() {
                let scaled_num = w.saturating_mul(target);
                let scaled = (scaled_num / total_content).max(1);
                let rem = scaled_num % total_content;
                col_widths[idx] = scaled;
                remainders.push((idx, rem));
            }

            let mut current_sum: usize = col_widths.iter().sum();
            while current_sum > target {
                if let Some((idx, &max_w)) = col_widths.iter().enumerate().max_by_key(|&(_, w)| w) {
                    if max_w <= 1 {
                        break;
                    }
                    col_widths[idx] -= 1;
                    current_sum -= 1;
                } else {
                    break;
                }
            }

            if current_sum < target {
                remainders.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                let mut remaining = target - current_sum;
                while remaining > 0 {
                    let mut progressed = false;
                    for &(idx, _) in &remainders {
                        if remaining == 0 {
                            break;
                        }
                        if col_widths[idx] < orig_widths[idx] {
                            col_widths[idx] += 1;
                            remaining -= 1;
                            progressed = true;
                        }
                    }
                    if !progressed {
                        break;
                    }
                }
            }
        }

        let border_fg = self.palette.md_table_border;
        let top_border = build_table_rule('┌', '┬', '┐', &col_widths);
        self.emit_line(
            source_line,
            vec![StyledSegment {
                text: top_border,
                fg: border_fg,
                bg: None,
                flags: StyleFlags::default(),
            }],
        );

        for row in &rows {
            let mut line_segs: Vec<StyledSegment> = Vec::new();
            push_coalesced(&mut line_segs, "│", border_fg, None, StyleFlags::default());

            for (c, &col_w) in col_widths.iter().enumerate() {
                let align = alignments.get(c).copied().unwrap_or(TableAlignment::None);
                let cell = row.cells.get(c).map(Vec::as_slice).unwrap_or(&[]);
                let fitted = truncate_segments(cell, col_w);
                let cell_w = segments_char_width(&fitted);
                let pad = col_w.saturating_sub(cell_w);
                let (left_pad, right_pad) = match align {
                    TableAlignment::Right => (pad, 0),
                    TableAlignment::Center => (pad / 2, pad - pad / 2),
                    TableAlignment::None | TableAlignment::Left => (0, pad),
                };

                push_coalesced(
                    &mut line_segs,
                    &" ".repeat(1 + left_pad),
                    self.palette.foreground,
                    None,
                    StyleFlags::default(),
                );
                for seg in fitted {
                    push_coalesced(&mut line_segs, &seg.text, seg.fg, seg.bg, seg.flags);
                }
                push_coalesced(
                    &mut line_segs,
                    &" ".repeat(1 + right_pad),
                    self.palette.foreground,
                    None,
                    StyleFlags::default(),
                );
                push_coalesced(&mut line_segs, "│", border_fg, None, StyleFlags::default());
            }

            self.emit_line(row.source_line, line_segs);

            if row.is_header {
                let mid_border = build_table_rule('├', '┼', '┤', &col_widths);
                self.emit_line(
                    row.source_line,
                    vec![StyledSegment {
                        text: mid_border,
                        fg: border_fg,
                        bg: None,
                        flags: StyleFlags::default(),
                    }],
                );
            }
        }

        let bottom_border = build_table_rule('└', '┴', '┘', &col_widths);
        self.emit_line(
            end_line.max(self.last_source_line),
            vec![StyledSegment {
                text: bottom_border,
                fg: border_fg,
                bg: None,
                flags: StyleFlags::default(),
            }],
        );
    }

    fn render_footnote_definition<'ast>(&mut self, node: &'ast AstNode<'ast>) {
        let (name, source_line) = {
            let data = node.data.borrow();
            let name = match &data.value {
                NodeValue::FootnoteDefinition(fd) => fd.name.clone(),
                _ => String::new(),
            };
            let src = self
                .source_line_from_pos(data.sourcepos)
                .max(self.last_source_line);
            (name, src)
        };

        let label = format!("[{name}]: ");
        let label_width = UnicodeWidthStr::width(label.as_str());
        let first_prefix = StyledSegment {
            text: label,
            fg: self.palette.muted,
            bg: None,
            flags: StyleFlags {
                bold: true,
                ..StyleFlags::default()
            },
        };
        let cont_prefix = StyledSegment {
            text: " ".repeat(label_width),
            fg: self.palette.foreground,
            bg: None,
            flags: StyleFlags::default(),
        };

        let stack_len_before = self.prefix_stack.len();
        self.pending_first_prefix = Some((first_prefix, cont_prefix));

        let mut has_children = false;
        for child in node.children() {
            has_children = true;
            self.render_block(child);
        }
        if !has_children {
            self.emit_line(source_line, Vec::new());
        }

        self.pending_first_prefix = None;
        self.prefix_stack.truncate(stack_len_before);
    }

    fn collect_inlines<'ast>(
        &self,
        node: &'ast AstNode<'ast>,
        base_style: InlineStyle,
    ) -> Vec<InlineToken> {
        let mut tokens = Vec::new();
        for child in node.children() {
            self.collect_inline_node(child, base_style, &mut tokens);
        }
        tokens
    }

    fn collect_inline_node<'ast>(
        &self,
        node: &'ast AstNode<'ast>,
        style: InlineStyle,
        out: &mut Vec<InlineToken>,
    ) {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::Text(text) => {
                if !text.is_empty() {
                    out.push(InlineToken::Segment(StyledSegment {
                        text: text.clone(),
                        fg: style.fg,
                        bg: style.bg,
                        flags: style.flags,
                    }));
                }
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                out.push(InlineToken::HardBreak);
            }
            NodeValue::Code(code) => {
                out.push(InlineToken::Segment(StyledSegment {
                    text: code.literal.clone(),
                    fg: self.palette.md_code_inline,
                    bg: Some(self.palette.surface),
                    flags: StyleFlags::default(),
                }));
            }
            NodeValue::Strong => {
                drop(data);
                let mut next = style;
                next.fg = self.palette.md_strong;
                next.flags.bold = true;
                for child in node.children() {
                    self.collect_inline_node(child, next, out);
                }
            }
            NodeValue::Emph => {
                drop(data);
                let mut next = style;
                next.fg = self.palette.md_emphasis;
                next.flags.italic = true;
                for child in node.children() {
                    self.collect_inline_node(child, next, out);
                }
            }
            NodeValue::Strikethrough => {
                drop(data);
                let mut next = style;
                next.fg = self.palette.md_strikethrough;
                next.flags.strikethrough = true;
                for child in node.children() {
                    self.collect_inline_node(child, next, out);
                }
            }
            NodeValue::Link(link) => {
                let url = link.url.clone();
                drop(data);
                let mut link_style = style;
                link_style.fg = self.palette.md_link;
                link_style.flags.underline = true;

                let before_len = out.len();
                for child in node.children() {
                    self.collect_inline_node(child, link_style, out);
                }
                let label: String = out[before_len..]
                    .iter()
                    .filter_map(|tok| match tok {
                        InlineToken::Segment(seg) => Some(seg.text.as_str()),
                        InlineToken::HardBreak => None,
                    })
                    .collect();

                if !url.is_empty() && label != url {
                    out.push(InlineToken::Segment(StyledSegment {
                        text: format!(" ({url})"),
                        fg: self.palette.muted,
                        bg: None,
                        flags: StyleFlags {
                            dim: true,
                            ..StyleFlags::default()
                        },
                    }));
                }
            }
            NodeValue::Image(image) => {
                let url = image.url.clone();
                drop(data);
                let mut alt_style = style;
                alt_style.fg = self.palette.md_link;
                for child in node.children() {
                    self.collect_inline_node(child, alt_style, out);
                }
                if !url.is_empty() {
                    out.push(InlineToken::Segment(StyledSegment {
                        text: format!(" ({url})"),
                        fg: self.palette.muted,
                        bg: None,
                        flags: StyleFlags {
                            dim: true,
                            ..StyleFlags::default()
                        },
                    }));
                }
            }
            NodeValue::FootnoteReference(fr) => {
                let label = format!("[{}]", fr.name);
                out.push(InlineToken::Segment(StyledSegment {
                    text: label,
                    fg: self.palette.muted,
                    bg: None,
                    flags: StyleFlags {
                        dim: true,
                        ..StyleFlags::default()
                    },
                }));
            }
            NodeValue::HtmlInline(raw) => {
                out.push(InlineToken::Segment(StyledSegment {
                    text: raw.clone(),
                    fg: self.palette.muted,
                    bg: None,
                    flags: StyleFlags::default(),
                }));
            }
            _ => {
                drop(data);
                for child in node.children() {
                    self.collect_inline_node(child, style, out);
                }
            }
        }
    }

    fn render_wrapped_tokens(&mut self, tokens: Vec<InlineToken>, base_source_line: usize) {
        let mut hard_chunks: Vec<Vec<StyledSegment>> = vec![Vec::new()];
        for token in tokens {
            match token {
                InlineToken::Segment(seg) => {
                    if let Some(chunk) = hard_chunks.last_mut() {
                        chunk.push(seg);
                    }
                }
                InlineToken::HardBreak => {
                    hard_chunks.push(Vec::new());
                }
            }
        }

        for (break_idx, chunk) in hard_chunks.into_iter().enumerate() {
            let line_src = base_source_line + break_idx;
            let avail = self.available_width();
            let wrapped = wrap_segments(&chunk, avail);
            if wrapped.is_empty() {
                self.emit_line(line_src, Vec::new());
            } else {
                for line_segs in wrapped {
                    self.emit_line(line_src, line_segs);
                }
            }
        }
    }
}

fn detect_alert_in_blockquote<'ast>(node: &'ast AstNode<'ast>) -> Option<(AlertKind, usize)> {
    let first_child = node.first_child()?;
    if !matches!(first_child.data.borrow().value, NodeValue::Paragraph) {
        return None;
    }

    let mut header_text = String::new();
    let mut consumed = 0usize;

    for inline in first_child.children() {
        let data = inline.data.borrow();
        match &data.value {
            NodeValue::Text(t) => {
                header_text.push_str(t);
                consumed += 1;
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                consumed += 1;
                break;
            }
            _ => return None,
        }
    }

    let trimmed = header_text.trim();
    let kind = if trimmed.eq_ignore_ascii_case("[!NOTE]") {
        AlertKind::Note
    } else if trimmed.eq_ignore_ascii_case("[!TIP]") {
        AlertKind::Tip
    } else if trimmed.eq_ignore_ascii_case("[!IMPORTANT]") {
        AlertKind::Important
    } else if trimmed.eq_ignore_ascii_case("[!WARNING]") {
        AlertKind::Warning
    } else if trimmed.eq_ignore_ascii_case("[!CAUTION]") {
        AlertKind::Caution
    } else {
        return None;
    };

    Some((kind, consumed))
}

fn wrap_segments(segments: &[StyledSegment], max_width: usize) -> Vec<Vec<StyledSegment>> {
    let max_width = max_width.max(1);
    let mut lines: Vec<Vec<StyledSegment>> = Vec::new();
    let mut current: Vec<StyledSegment> = Vec::new();
    let mut current_width = 0usize;

    for seg in segments {
        for piece in split_word_pieces(&seg.text) {
            let is_ws = piece.chars().all(char::is_whitespace);
            let piece_width = UnicodeWidthStr::width(piece);

            if is_ws {
                if current_width == 0 {
                    continue;
                }
                if current_width + piece_width <= max_width {
                    push_coalesced(&mut current, piece, seg.fg, seg.bg, seg.flags);
                    current_width += piece_width;
                } else {
                    trim_trailing_whitespace(&mut current);
                    lines.push(std::mem::take(&mut current));
                    current_width = 0;
                }
                continue;
            }

            if current_width > 0 && current_width + piece_width > max_width {
                trim_trailing_whitespace(&mut current);
                lines.push(std::mem::take(&mut current));
                current_width = 0;
            }

            if piece_width <= max_width {
                push_coalesced(&mut current, piece, seg.fg, seg.bg, seg.flags);
                current_width += piece_width;
            } else {
                let mut chunk = String::new();
                let mut chunk_w = 0usize;
                for ch in piece.chars() {
                    let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
                    if chunk_w > 0 && chunk_w + ch_w > max_width {
                        push_coalesced(&mut current, &chunk, seg.fg, seg.bg, seg.flags);
                        lines.push(std::mem::take(&mut current));
                        chunk.clear();
                        chunk_w = 0;
                        current_width = 0;
                    }
                    chunk.push(ch);
                    chunk_w += ch_w;
                }
                if !chunk.is_empty() {
                    push_coalesced(&mut current, &chunk, seg.fg, seg.bg, seg.flags);
                    current_width += chunk_w;
                }
            }
        }
    }

    if !current.is_empty() {
        trim_trailing_whitespace(&mut current);
        lines.push(current);
    }

    lines
}

fn split_word_pieces(text: &str) -> Vec<&str> {
    let mut pieces = Vec::new();
    let mut iter = text.char_indices();
    let Some((mut start, first_ch)) = iter.next() else {
        return pieces;
    };
    let mut in_ws = first_ch.is_whitespace();

    for (idx, ch) in iter {
        let ch_ws = ch.is_whitespace();
        if ch_ws != in_ws {
            pieces.push(&text[start..idx]);
            start = idx;
            in_ws = ch_ws;
        }
    }
    pieces.push(&text[start..]);
    pieces
}

fn trim_trailing_whitespace(segments: &mut Vec<StyledSegment>) {
    while let Some(last) = segments.last_mut() {
        let trimmed = last.text.trim_end();
        if trimmed.is_empty() {
            segments.pop();
        } else {
            if trimmed.len() != last.text.len() {
                last.text.truncate(trimmed.len());
            }
            break;
        }
    }
}

fn segments_char_width(segments: &[StyledSegment]) -> usize {
    segments
        .iter()
        .map(|s| UnicodeWidthStr::width(s.text.as_str()))
        .sum()
}

fn truncate_segments(segments: &[StyledSegment], max_width: usize) -> Vec<StyledSegment> {
    let mut out = Vec::new();
    let mut remaining = max_width;
    for seg in segments {
        if remaining == 0 {
            break;
        }
        let seg_w = UnicodeWidthStr::width(seg.text.as_str());
        if seg_w <= remaining {
            out.push(seg.clone());
            remaining -= seg_w;
        } else {
            let mut truncated = String::new();
            let mut used = 0usize;
            for ch in seg.text.chars() {
                let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
                if used + ch_w > remaining {
                    break;
                }
                truncated.push(ch);
                used += ch_w;
            }
            if !truncated.is_empty() {
                out.push(StyledSegment {
                    text: truncated,
                    fg: seg.fg,
                    bg: seg.bg,
                    flags: seg.flags,
                });
            }
            break;
        }
    }
    out
}

fn build_table_rule(left: char, mid: char, right: char, col_widths: &[usize]) -> String {
    let mut out = String::new();
    out.push(left);
    for (idx, &w) in col_widths.iter().enumerate() {
        if idx > 0 {
            out.push(mid);
        }
        for _ in 0..(w + 2) {
            out.push('─');
        }
    }
    out.push(right);
    out
}

fn push_coalesced(
    segments: &mut Vec<StyledSegment>,
    text: &str,
    fg: RgbColor,
    bg: Option<RgbColor>,
    flags: StyleFlags,
) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = segments.last_mut() {
        if last.fg == fg && last.bg == bg && last.flags == flags {
            last.text.push_str(text);
            return;
        }
    }
    segments.push(StyledSegment {
        text: text.to_string(),
        fg,
        bg,
        flags,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::default_theme;

    #[test]
    fn preview_renders_tables_tasks_alerts_code_and_breaks() {
        let md = concat!(
            "# Title\n",
            "\n",
            "> [!TIP]\n",
            "> helpful\n",
            "\n",
            "- [x] done\n",
            "\n",
            "| A | B |\n",
            "|---|---|\n",
            "| 1 | 2 |\n",
            "\n",
            "```rust\n",
            "fn main() {}\n",
            "```\n",
            "\n",
            "line1  \n",
            "line2\n",
        );

        let theme = default_theme().expect("default theme");
        let doc = build_preview(md, &theme, 80);
        let plain = doc.plain_lines();
        let joined = plain.join("\n");

        assert!(joined.contains("Title"), "missing Title in:\n{joined}");
        assert!(joined.contains("[TIP]"), "missing [TIP] in:\n{joined}");
        assert!(joined.contains("[x]"), "missing [x] in:\n{joined}");
        assert!(joined.contains('┌'), "missing ┌ in:\n{joined}");
        assert!(joined.contains('│'), "missing │ in:\n{joined}");
        assert!(joined.contains("rust"), "missing rust in:\n{joined}");

        let line1_idx = plain
            .iter()
            .position(|l| l.contains("line1"))
            .expect("missing line1");
        let line2_idx = plain
            .iter()
            .position(|l| l.contains("line2"))
            .expect("missing line2");
        assert_ne!(
            line1_idx, line2_idx,
            "expected line1 and line2 on separate lines, got:\n{joined}"
        );
    }

    #[test]
    fn preview_scroll_sync_maps_source_and_preview_lines() {
        let md = concat!(
            "# Section One\n",
            "\n",
            "First paragraph text.\n",
            "\n",
            "## Section Two\n",
            "\n",
            "Second paragraph text.\n",
            "\n",
            "### Section Three\n",
            "\n",
            "Third paragraph text.\n",
        );

        let theme = default_theme().expect("default theme");
        let doc = build_preview(md, &theme, 80);
        let total_src_lines = md.lines().count();

        let mut prev_mapped = 0usize;
        for src in 0..total_src_lines {
            let mapped = doc.source_line_to_preview_line(src);
            assert!(
                mapped >= prev_mapped,
                "non-monotonic mapping at src={src}: {mapped} < {prev_mapped}"
            );
            prev_mapped = mapped;
        }

        for heading_src in [0usize, 4, 8] {
            let preview_idx = doc.source_line_to_preview_line(heading_src);
            let roundtrip = doc.preview_line_to_source_line(preview_idx);
            assert!(
                roundtrip >= heading_src,
                "heading src={heading_src} mapped to preview={preview_idx} -> src={roundtrip}"
            );
        }
    }

    #[test]
    fn preview_soft_break_in_blockquote_and_bullet_renders_on_new_lines() {
        let md = concat!(
            "> first quote line\n",
            "> second quote line\n",
            "\n",
            "- bullet line one\n",
            "  bullet line two\n",
            "\n",
            "plain one\n",
            "plain two\n",
        );

        let theme = default_theme().expect("default theme");
        let doc = build_preview(md, &theme, 80);
        let plain = doc.plain_lines();
        let joined = plain.join("\n");

        let quote1_idx = plain
            .iter()
            .position(|l| l.contains("first quote line"))
            .expect("missing first quote line");
        let quote2_idx = plain
            .iter()
            .position(|l| l.contains("second quote line"))
            .expect("missing second quote line");
        assert!(
            quote2_idx > quote1_idx,
            "expected second quote line after first quote line, got:\n{joined}"
        );
        assert!(
            plain[quote1_idx].starts_with("│ "),
            "expected blockquote prefix on first quote line, got: {:?}",
            plain[quote1_idx]
        );
        assert!(
            plain[quote2_idx].starts_with("│ "),
            "expected blockquote prefix on second quote line, got: {:?}",
            plain[quote2_idx]
        );

        let bullet1_idx = plain
            .iter()
            .position(|l| l.contains("bullet line one"))
            .expect("missing bullet line one");
        let bullet2_idx = plain
            .iter()
            .position(|l| l.contains("bullet line two"))
            .expect("missing bullet line two");
        assert!(
            bullet2_idx > bullet1_idx,
            "expected bullet line two after bullet line one, got:\n{joined}"
        );
        assert!(
            plain[bullet1_idx].starts_with("• "),
            "expected bullet marker on first bullet line, got: {:?}",
            plain[bullet1_idx]
        );
        assert!(
            plain[bullet2_idx].starts_with("  ") && !plain[bullet2_idx].starts_with("• "),
            "expected continuation indent without bullet marker on second bullet line, got: {:?}",
            plain[bullet2_idx]
        );

        let plain1_idx = plain
            .iter()
            .position(|l| l.contains("plain one"))
            .expect("missing plain one");
        let plain2_idx = plain
            .iter()
            .position(|l| l.contains("plain two"))
            .expect("missing plain two");
        assert!(
            plain2_idx > plain1_idx,
            "expected plain two after plain one, got:\n{joined}"
        );
    }

    #[test]
    fn wide_table_shrinks_and_measures_display_width() {
        let md = concat!(
            "| 項目 | 説明テキスト長い列 |\n",
            "|------|--------------------|\n",
            "| 日本語テスト | これは非常に長い日本語テーブルセルの内容です |\n",
        );

        let theme = default_theme().expect("default theme");
        let doc = build_preview(md, &theme, 24);
        let plain = doc.plain_lines();
        assert!(!plain.is_empty(), "expected rendered table lines");

        for line in &plain {
            let display_w = UnicodeWidthStr::width(line.as_str());
            assert!(
                display_w <= 24,
                "expected line display width <= 24, got {display_w} for line: {line:?}"
            );
        }
    }
}
