use std::ops::Range;

use comrak::nodes::{AstNode, NodeValue, Sourcepos};
use comrak::{parse_document, Arena, Options};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MdTokenKind {
    Heading(u8),
    Emphasis,
    Strong,
    Strikethrough,
    Link,
    InlineCode,
    CodeBlock,
    BlockQuote,
    ListMarker,
    TaskChecked,
    TaskUnchecked,
    Table,
    ThematicBreak,
    AlertNote,
    AlertTip,
    AlertImportant,
    AlertWarning,
    AlertCaution,
    Footnote,
    FrontMatter,
    Html,
    HardBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MdSpan {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub kind: MdTokenKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlockRegion {
    pub start_line: usize,
    pub end_line: usize,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GfmDocument {
    pub spans: Vec<MdSpan>,
    pub code_blocks: Vec<CodeBlockRegion>,
    pub spell_skip_byte_ranges: Vec<Range<usize>>,
}

impl GfmDocument {
    pub fn is_line_in_code_block(&self, line: usize) -> Option<&CodeBlockRegion> {
        self.code_blocks
            .iter()
            .find(|region| line >= region.start_line && line <= region.end_line)
    }

    pub fn spans_for_line(&self, line: usize) -> Vec<MdSpan> {
        self.spans
            .iter()
            .copied()
            .filter(|span| line >= span.start_line && line <= span.end_line)
            .collect()
    }
}

pub fn gfm_options<'a>() -> Options<'a> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.front_matter_delimiter = Some("---".to_string());
    options.extension.tagfilter = true;
    options
}

struct LineIndex {
    /// Byte offset of the start of each 0-based line.
    line_starts: Vec<usize>,
    /// Byte length (excluding trailing `\n` or `\r\n`) of each 0-based line.
    line_lengths: Vec<usize>,
    /// Total byte length of the source document.
    total_len: usize,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        let mut line_lengths = Vec::new();
        let bytes = text.as_bytes();
        let mut start = 0;

        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                let end = if i > start && bytes[i - 1] == b'\r' {
                    i - 1
                } else {
                    i
                };
                line_lengths.push(end - start);
                start = i + 1;
                line_starts.push(start);
            }
        }
        line_lengths.push(bytes.len().saturating_sub(start));

        Self {
            line_starts,
            line_lengths,
            total_len: bytes.len(),
        }
    }

    fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Converts a comrak 1-based `Sourcepos` (inclusive start and end) into a 0-based `MdSpan`
    /// with inclusive `end_line` and exclusive `end_col`.
    /// `line_offset` accounts for lines stripped by comrak's front-matter extraction.
    fn sourcepos_to_span(
        &self,
        pos: Sourcepos,
        line_offset: usize,
        kind: MdTokenKind,
    ) -> Option<MdSpan> {
        if pos.start.line == 0 || pos.end.line == 0 {
            return None;
        }
        let max_line = self.line_count().saturating_sub(1);
        let start_line = (pos.start.line.saturating_sub(1) + line_offset).min(max_line);
        let end_line = (pos.end.line.saturating_sub(1) + line_offset).min(max_line);
        let start_col = pos
            .start
            .column
            .saturating_sub(1)
            .min(self.line_lengths[start_line]);
        let end_col = if pos.end.column == 0 {
            self.line_lengths[end_line]
        } else {
            pos.end.column.min(self.line_lengths[end_line])
        };

        Some(MdSpan {
            start_line,
            start_col,
            end_line,
            end_col,
            kind,
        })
    }

    /// Converts a 0-based `MdSpan` into a clamped document byte `Range<usize>`.
    fn span_to_byte_range(&self, span: &MdSpan) -> Range<usize> {
        let start_base = self
            .line_starts
            .get(span.start_line)
            .copied()
            .unwrap_or(self.total_len);
        let end_base = self
            .line_starts
            .get(span.end_line)
            .copied()
            .unwrap_or(self.total_len);
        let start = (start_base + span.start_col).min(self.total_len);
        let end = (end_base + span.end_col).min(self.total_len);
        if start <= end {
            start..end
        } else {
            start..start
        }
    }

    /// Computes a 0-based `MdSpan` from a document byte `Range<usize>`.
    fn byte_range_to_span(&self, range: Range<usize>, kind: MdTokenKind) -> Option<MdSpan> {
        if range.is_empty() || range.start >= self.total_len {
            return None;
        }
        let start_line = self.line_for_byte(range.start);
        let start_col = range.start.saturating_sub(self.line_starts[start_line]);
        let end_inclusive = range
            .end
            .saturating_sub(1)
            .min(self.total_len.saturating_sub(1));
        let end_line = self.line_for_byte(end_inclusive);
        let end_col =
            (range.end.saturating_sub(self.line_starts[end_line])).min(self.line_lengths[end_line]);
        Some(MdSpan {
            start_line,
            start_col,
            end_line,
            end_col,
            kind,
        })
    }

    fn line_for_byte(&self, byte_offset: usize) -> usize {
        match self.line_starts.binary_search(&byte_offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        }
    }

    fn line_text<'a>(&self, text: &'a str, line: usize) -> Option<&'a str> {
        let start = *self.line_starts.get(line)?;
        let len = *self.line_lengths.get(line)?;
        text.get(start..start + len)
    }

    fn line_byte_range(&self, line: usize) -> Option<Range<usize>> {
        let start = *self.line_starts.get(line)?;
        let len = *self.line_lengths.get(line)?;
        Some(start..start + len)
    }
}

pub fn parse_gfm(markdown: &str) -> GfmDocument {
    let arena = Arena::new();
    let options = gfm_options();
    let root = parse_document(&arena, markdown, &options);
    let line_index = LineIndex::new(markdown);

    let mut ctx = CollectCtx {
        markdown,
        line_index: &line_index,
        line_offset: 0,
        autolink_cursor: 0,
        spans: Vec::new(),
        code_blocks: Vec::new(),
        skip_ranges: Vec::new(),
    };

    ctx.collect(root, None);
    collect_reference_link_def_skip_ranges(&mut ctx);

    let spell_skip_byte_ranges = merge_ranges(ctx.skip_ranges);

    GfmDocument {
        spans: ctx.spans,
        code_blocks: ctx.code_blocks,
        spell_skip_byte_ranges,
    }
}

struct CollectCtx<'s> {
    markdown: &'s str,
    line_index: &'s LineIndex,
    line_offset: usize,
    autolink_cursor: usize,
    spans: Vec<MdSpan>,
    code_blocks: Vec<CodeBlockRegion>,
    skip_ranges: Vec<Range<usize>>,
}

impl CollectCtx<'_> {
    fn collect<'a>(&mut self, node: &'a AstNode<'a>, parent_byte_range: Option<Range<usize>>) {
        let data = node.data.borrow();
        let sourcepos = data.sourcepos;
        let offset = self.line_offset;
        let current_byte_range = self
            .line_index
            .sourcepos_to_span(sourcepos, offset, MdTokenKind::Html)
            .map(|s| self.line_index.span_to_byte_range(&s))
            .filter(|r| !r.is_empty())
            .or_else(|| parent_byte_range.clone());

        match &data.value {
            NodeValue::Heading(heading) => {
                if let Some(span) = self.line_index.sourcepos_to_span(
                    sourcepos,
                    offset,
                    MdTokenKind::Heading(heading.level),
                ) {
                    self.spans.push(span);
                }
            }
            NodeValue::Emph => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::Emphasis)
                {
                    self.spans.push(span);
                }
            }
            NodeValue::Strong => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::Strong)
                {
                    self.spans.push(span);
                }
            }
            NodeValue::Strikethrough => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::Strikethrough)
                {
                    self.spans.push(span);
                }
            }
            NodeValue::Link(link) => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::Link)
                {
                    let full_range = self.line_index.span_to_byte_range(&span);
                    self.autolink_cursor = self.autolink_cursor.max(full_range.end);
                    self.spans.push(span);
                    if let Some(url_range) =
                        find_link_url_skip_range(self.markdown, full_range, &link.url)
                    {
                        self.skip_ranges.push(url_range);
                    }
                } else {
                    // comrak assigns synthetic (0, 1, 0, 1) sourcepos to GFM autolinks;
                    // locate the literal autolink text within the enclosing block's byte range
                    // starting at or after the monotonic autolink cursor.
                    let needle = first_text_child(node).unwrap_or_else(|| link.url.clone());
                    let block_range = parent_byte_range.unwrap_or(0..self.markdown.len());
                    let search_start = self
                        .autolink_cursor
                        .max(block_range.start)
                        .min(block_range.end);
                    let search_range = search_start..block_range.end;
                    if !needle.is_empty() {
                        if let Some(slice) = self.markdown.get(search_range.clone()) {
                            if let Some(rel) = slice.find(&needle) {
                                let start = search_range.start + rel;
                                let end = start + needle.len();
                                let range = start..end;
                                self.autolink_cursor = end;
                                if let Some(span) = self
                                    .line_index
                                    .byte_range_to_span(range.clone(), MdTokenKind::Link)
                                {
                                    self.spans.push(span);
                                }
                                self.skip_ranges.push(range);
                            }
                        }
                    }
                }
            }
            NodeValue::Code(_) => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::InlineCode)
                {
                    let range = self.line_index.span_to_byte_range(&span);
                    self.autolink_cursor = self.autolink_cursor.max(range.end);
                    self.spans.push(span);
                    if !range.is_empty() {
                        self.skip_ranges.push(range);
                    }
                }
            }
            NodeValue::CodeBlock(cb) => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::CodeBlock)
                {
                    let range = self.line_index.span_to_byte_range(&span);
                    self.autolink_cursor = self.autolink_cursor.max(range.end);
                    let language = cb
                        .info
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    let (body_start, body_end) = if cb.fenced {
                        let has_closing_fence = span.end_line > span.start_line
                            && self
                                .line_index
                                .line_text(self.markdown, span.end_line)
                                .is_some_and(|line| {
                                    is_closing_fence(line, cb.fence_char, cb.fence_length)
                                });
                        if has_closing_fence {
                            if span.end_line > span.start_line + 1 {
                                (span.start_line + 1, span.end_line - 1)
                            } else {
                                (span.start_line, span.end_line)
                            }
                        } else {
                            ((span.start_line + 1).min(span.end_line), span.end_line)
                        }
                    } else {
                        (span.start_line, span.end_line)
                    };
                    self.code_blocks.push(CodeBlockRegion {
                        start_line: body_start,
                        end_line: body_end,
                        language,
                    });
                    self.spans.push(span);
                    if !range.is_empty() {
                        self.skip_ranges.push(range);
                    }
                }
            }
            NodeValue::BlockQuote => {
                if let Some(mut span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::BlockQuote)
                {
                    let range = self.line_index.span_to_byte_range(&span);
                    if let Some(alert_kind) =
                        self.markdown.get(range).and_then(detect_gfm_alert_kind)
                    {
                        span.kind = alert_kind;
                    }
                    self.spans.push(span);
                }
            }
            NodeValue::Item(_) => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::ListMarker)
                {
                    self.spans.push(span);
                }
            }
            NodeValue::TaskItem(checked) => {
                let kind = match checked {
                    Some('x' | 'X') => MdTokenKind::TaskChecked,
                    _ => MdTokenKind::TaskUnchecked,
                };
                if let Some(span) = self.line_index.sourcepos_to_span(sourcepos, offset, kind) {
                    self.spans.push(span);
                }
            }
            NodeValue::Table(_) => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::Table)
                {
                    self.spans.push(span);
                }
            }
            NodeValue::ThematicBreak => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::ThematicBreak)
                {
                    self.spans.push(span);
                }
            }
            NodeValue::FootnoteDefinition(_) | NodeValue::FootnoteReference(_) => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::Footnote)
                {
                    self.spans.push(span);
                }
            }
            NodeValue::FrontMatter(raw) => {
                // comrak strips front matter before parsing the remainder, shifting subsequent lines.
                let fm_lines = raw.bytes().filter(|&b| b == b'\n').count();
                self.line_offset = fm_lines;
                if !raw.is_empty() {
                    let end = if raw.ends_with('\n') {
                        self.line_index
                            .line_starts
                            .get(fm_lines)
                            .copied()
                            .unwrap_or(self.markdown.len())
                    } else {
                        self.line_index
                            .line_starts
                            .get(fm_lines + 1)
                            .copied()
                            .unwrap_or(self.markdown.len())
                    };
                    let range = 0..end.min(self.markdown.len());
                    self.autolink_cursor = self.autolink_cursor.max(range.end);
                    if let Some(span) = self
                        .line_index
                        .byte_range_to_span(range.clone(), MdTokenKind::FrontMatter)
                    {
                        self.spans.push(span);
                    }
                    if !range.is_empty() {
                        self.skip_ranges.push(range);
                    }
                }
            }
            NodeValue::HtmlBlock(_) | NodeValue::HtmlInline(_) => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::Html)
                {
                    let range = self.line_index.span_to_byte_range(&span);
                    self.autolink_cursor = self.autolink_cursor.max(range.end);
                    self.spans.push(span);
                    if !range.is_empty() {
                        self.skip_ranges.push(range);
                    }
                }
            }
            NodeValue::LineBreak => {
                if let Some(span) =
                    self.line_index
                        .sourcepos_to_span(sourcepos, offset, MdTokenKind::HardBreak)
                {
                    self.spans.push(span);
                }
            }
            _ => {}
        }

        drop(data);

        for child in node.children() {
            self.collect(child, current_byte_range.clone());
        }
    }
}

fn is_closing_fence(line: &str, fence_char: u8, fence_length: usize) -> bool {
    if fence_char != b'`' && fence_char != b'~' {
        return false;
    }
    let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
    if leading_spaces > 3 {
        return false;
    }
    let rest = &line[leading_spaces..];
    let fence_count = rest.bytes().take_while(|&b| b == fence_char).count();
    if fence_count < fence_length.max(3) {
        return false;
    }
    rest[fence_count..].trim().is_empty()
}

fn collect_reference_link_def_skip_ranges(ctx: &mut CollectCtx<'_>) {
    for line in ctx.line_offset..ctx.line_index.line_count() {
        let in_code_or_front_matter = ctx.spans.iter().any(|span| {
            matches!(span.kind, MdTokenKind::CodeBlock | MdTokenKind::FrontMatter)
                && line >= span.start_line
                && line <= span.end_line
        });
        if in_code_or_front_matter {
            continue;
        }
        if let Some(text) = ctx.line_index.line_text(ctx.markdown, line) {
            if is_reference_link_definition(text) {
                if let Some(range) = ctx.line_index.line_byte_range(line) {
                    if !range.is_empty() {
                        ctx.skip_ranges.push(range);
                    }
                }
            }
        }
    }
}

fn is_reference_link_definition(line: &str) -> bool {
    let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
    if leading_spaces > 3 {
        return false;
    }
    let rest = &line[leading_spaces..];
    let Some(after_open) = rest.strip_prefix('[') else {
        return false;
    };

    let bytes = after_open.as_bytes();
    let mut i = 0;
    let mut close_idx = None;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'[' => return false,
            b']' => {
                if bytes.get(i + 1) == Some(&b':') {
                    close_idx = Some(i);
                    break;
                }
                return false;
            }
            _ => i += 1,
        }
    }

    let Some(close_idx) = close_idx else {
        return false;
    };
    let label = &after_open[..close_idx];
    if label.trim().is_empty() || label.starts_with('^') {
        return false;
    }

    let after_colon = after_open[close_idx + 2..].trim_start_matches([' ', '\t']);
    if after_colon.is_empty() {
        return false;
    }

    let after_dest = if let Some(angle_inner) = after_colon.strip_prefix('<') {
        let angle_bytes = angle_inner.as_bytes();
        let mut j = 0;
        let mut end_angle = None;
        while j < angle_bytes.len() {
            match angle_bytes[j] {
                b'\\' => {
                    j += 2;
                    continue;
                }
                b'<' => return false,
                b'>' => {
                    end_angle = Some(j);
                    break;
                }
                _ => j += 1,
            }
        }
        let Some(end_angle) = end_angle else {
            return false;
        };
        if angle_inner[..end_angle].is_empty() {
            return false;
        }
        &angle_inner[end_angle + 1..]
    } else {
        let dest_end = after_colon.find([' ', '\t']).unwrap_or(after_colon.len());
        let dest = &after_colon[..dest_end];
        if dest.is_empty() || dest.bytes().any(|b| b.is_ascii_control()) {
            return false;
        }
        &after_colon[dest_end..]
    };

    let trailing = after_dest.trim();
    if trailing.is_empty() {
        return true;
    }
    if !after_dest.starts_with([' ', '\t']) {
        return false;
    }
    (trailing.len() >= 2 && trailing.starts_with('"') && trailing.ends_with('"'))
        || (trailing.len() >= 2 && trailing.starts_with('\'') && trailing.ends_with('\''))
        || (trailing.len() >= 2 && trailing.starts_with('(') && trailing.ends_with(')'))
}

fn first_text_child<'a>(node: &'a AstNode<'a>) -> Option<String> {
    let child = node.first_child()?;
    let data = child.data.borrow();
    if let NodeValue::Text(text) = &data.value {
        Some(text.clone())
    } else {
        None
    }
}

fn detect_gfm_alert_kind(blockquote_src: &str) -> Option<MdTokenKind> {
    let first_line = blockquote_src.lines().next()?;
    let trimmed = first_line.trim_start();
    let after_gt = trimmed.strip_prefix('>')?.trim();
    if after_gt.eq_ignore_ascii_case("[!NOTE]") {
        Some(MdTokenKind::AlertNote)
    } else if after_gt.eq_ignore_ascii_case("[!TIP]") {
        Some(MdTokenKind::AlertTip)
    } else if after_gt.eq_ignore_ascii_case("[!IMPORTANT]") {
        Some(MdTokenKind::AlertImportant)
    } else if after_gt.eq_ignore_ascii_case("[!WARNING]") {
        Some(MdTokenKind::AlertWarning)
    } else if after_gt.eq_ignore_ascii_case("[!CAUTION]") {
        Some(MdTokenKind::AlertCaution)
    } else {
        None
    }
}

fn find_link_url_skip_range(
    markdown: &str,
    full_range: Range<usize>,
    url: &str,
) -> Option<Range<usize>> {
    if full_range.is_empty() {
        return None;
    }
    let slice = markdown.get(full_range.clone())?;

    // Inline markdown link `[text](url ...)` — skip only the destination part `(url ...)`,
    // preserving link text for spell checking.
    if slice.starts_with('[') {
        if let Some(dest_start) = slice.rfind("](") {
            let start = full_range.start + dest_start + 2;
            let end = full_range.end;
            if start < end {
                return Some(start..end);
            }
        }
        if !url.is_empty() {
            if let Some(rel) = slice.rfind(url) {
                let start = full_range.start + rel;
                return Some(start..start + url.len());
            }
        }
        return None;
    }

    // Autolinks (`https://...` or `<https://...>`) — skip the entire autolink span.
    Some(full_range)
}

fn merge_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|r| r.start < r.end);
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort_by_key(|r| (r.start, r.end));

    let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(last) = merged.last_mut() {
            if range.start <= last.end {
                last.end = last.end.max(range.end);
                continue;
            }
        }
        merged.push(range);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_gfm_extracts_all_major_features() {
        let md = concat!(
            "---\n",
            "title: Test\n",
            "---\n",
            "\n",
            "# H1\n",
            "\n",
            "**bold** *italic* ~~strike~~\n",
            "\n",
            "- [x] done\n",
            "\n",
            "> [!WARNING]\n",
            "> watch out\n",
            "\n",
            "| A | B |\n",
            "| - | - |\n",
            "| 1 | 2 |\n",
            "\n",
            "```rust\n",
            "fn main() {}\n",
            "```\n",
            "\n",
            "line1  \n",
            "line2\n",
            "\n",
            "[site](https://example.com)\n",
        );

        let doc = parse_gfm(md);

        let required = [
            MdTokenKind::FrontMatter,
            MdTokenKind::Heading(1),
            MdTokenKind::Strong,
            MdTokenKind::Emphasis,
            MdTokenKind::Strikethrough,
            MdTokenKind::TaskChecked,
            MdTokenKind::AlertWarning,
            MdTokenKind::Table,
            MdTokenKind::CodeBlock,
            MdTokenKind::HardBreak,
            MdTokenKind::Link,
        ];

        for expected in required {
            assert!(
                doc.spans.iter().any(|s| s.kind == expected),
                "missing expected token kind: {expected:?}; spans={:?}",
                doc.spans
            );
        }

        assert_eq!(doc.code_blocks.len(), 1);
        assert_eq!(doc.code_blocks[0].language, "rust");
    }

    #[test]
    fn spell_skip_ranges_cover_code_frontmatter_and_urls() {
        let md = concat!(
            "---\n",
            "author: testuser\n",
            "---\n",
            "\n",
            "Use `foo_bar` here.\n",
            "\n",
            "```rust\n",
            "let misspelled_ident = 1;\n",
            "```\n",
            "\n",
            "Visit https://example.com/typo now\n",
        );

        let doc = parse_gfm(md);

        let is_covered = |needle: &str| {
            let start = md.find(needle).expect("needle in fixture");
            let end = start + needle.len();
            doc.spell_skip_byte_ranges
                .iter()
                .any(|r| r.start <= start && end <= r.end)
        };

        assert!(is_covered("author: testuser"));
        assert!(is_covered("foo_bar"));
        assert!(is_covered("misspelled_ident"));
        assert!(is_covered("https://example.com/typo"));
        assert!(!is_covered("Visit"));
    }

    #[test]
    fn parse_gfm_handles_duplicate_autolinks_unclosed_fence_ref_defs_and_crlf_front_matter() {
        // (a) Duplicate autolinks resolve to distinct spans and both byte ranges are skipped.
        let dup_md = "Visit https://example.com/a and https://example.com/a";
        let dup_doc = parse_gfm(dup_md);
        let link_spans: Vec<MdSpan> = dup_doc
            .spans
            .iter()
            .copied()
            .filter(|s| s.kind == MdTokenKind::Link)
            .collect();
        assert_eq!(link_spans.len(), 2);
        assert_ne!(link_spans[0].start_col, link_spans[1].start_col);
        assert_eq!(link_spans[0].start_col, 6);
        assert_eq!(link_spans[0].end_col, 27);
        assert_eq!(link_spans[1].start_col, 32);
        assert_eq!(link_spans[1].end_col, 53);
        assert!(dup_doc
            .spell_skip_byte_ranges
            .iter()
            .any(|r| r.start <= 6 && 27 <= r.end));
        assert!(dup_doc
            .spell_skip_byte_ranges
            .iter()
            .any(|r| r.start <= 32 && 53 <= r.end));

        // (b) Unclosed fenced code block at EOF preserves body lines through the last line.
        let unclosed_md = "```rust\nlet x = 1;\nlet y = 2;";
        let unclosed_doc = parse_gfm(unclosed_md);
        assert_eq!(unclosed_doc.code_blocks.len(), 1);
        assert_eq!(unclosed_doc.code_blocks[0].start_line, 1);
        assert_eq!(unclosed_doc.code_blocks[0].end_line, 2);
        assert_eq!(unclosed_doc.code_blocks[0].language, "rust");
        assert!(unclosed_doc.is_line_in_code_block(2).is_some());

        // (c) Reference link definitions are covered by spell_skip_byte_ranges.
        let ref_md = "[docs]: https://example.com/wrld\n\nSee [docs].\n";
        let ref_doc = parse_gfm(ref_md);
        let ref_line = "[docs]: https://example.com/wrld";
        let ref_start = ref_md.find(ref_line).expect("ref def line in fixture");
        let ref_end = ref_start + ref_line.len();
        assert!(ref_doc
            .spell_skip_byte_ranges
            .iter()
            .any(|r| r.start <= ref_start && ref_end <= r.end));

        // (d) CRLF front matter spans its full original byte range and keeps heading on line 3.
        let crlf_md = "---\r\ntitle: wrld\r\n---\r\n# Heading\r\n";
        let crlf_doc = parse_gfm(crlf_md);
        let fm_span = crlf_doc
            .spans
            .iter()
            .find(|s| s.kind == MdTokenKind::FrontMatter)
            .expect("expected FrontMatter span");
        assert_eq!(fm_span.start_line, 0);
        assert_eq!(fm_span.end_line, 2);
        let fm_end = crlf_md.find("# Heading").expect("heading in CRLF fixture");
        assert!(crlf_doc
            .spell_skip_byte_ranges
            .iter()
            .any(|r| r.start == 0 && r.end >= fm_end - 2));
        let heading_span = crlf_doc
            .spans
            .iter()
            .find(|s| s.kind == MdTokenKind::Heading(1))
            .expect("expected Heading(1) span");
        assert_eq!(heading_span.start_line, 3);
    }
}
