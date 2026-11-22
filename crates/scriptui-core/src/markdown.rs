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
        spans: Vec::new(),
        code_blocks: Vec::new(),
        skip_ranges: Vec::new(),
    };

    ctx.collect(root, None);

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
                    self.spans.push(span);
                    if let Some(url_range) =
                        find_link_url_skip_range(self.markdown, full_range, &link.url)
                    {
                        self.skip_ranges.push(url_range);
                    }
                } else {
                    // comrak assigns synthetic (0, 1, 0, 1) sourcepos to GFM autolinks;
                    // locate the literal autolink text within the enclosing block's byte range.
                    let needle = first_text_child(node).unwrap_or_else(|| link.url.clone());
                    let search_range = parent_byte_range.unwrap_or(0..self.markdown.len());
                    if !needle.is_empty() {
                        if let Some(slice) = self.markdown.get(search_range.clone()) {
                            if let Some(rel) = slice.find(&needle) {
                                let start = search_range.start + rel;
                                let range = start..start + needle.len();
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
                    let language = cb
                        .info
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    let (body_start, body_end) = if cb.fenced && span.end_line > span.start_line + 1
                    {
                        (span.start_line + 1, span.end_line - 1)
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
                self.line_offset = raw.bytes().filter(|&b| b == b'\n').count();
                if !raw.is_empty() {
                    let end = raw.len().min(self.markdown.len());
                    let range = 0..end;
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
}
