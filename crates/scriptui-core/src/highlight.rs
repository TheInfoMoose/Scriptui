use std::ops::Range;

use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

use crate::markdown::{GfmDocument, MdTokenKind};
use crate::theme::{RgbColor, Theme, ThemePalette};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StyleFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
    pub strikethrough: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSegment {
    pub text: String,
    pub fg: RgbColor,
    pub bg: Option<RgbColor>,
    pub flags: StyleFlags,
}

#[derive(Debug, Clone, Default)]
pub struct LineOverlay {
    pub selection_cols: Vec<Range<usize>>,
    pub search_match_cols: Vec<Range<usize>>,
    pub active_search_col: Option<Range<usize>>,
    pub spell_error_cols: Vec<Range<usize>>,
}

pub fn style_for_md_token(kind: MdTokenKind, palette: &ThemePalette) -> (RgbColor, StyleFlags) {
    let bold = StyleFlags {
        bold: true,
        ..StyleFlags::default()
    };
    let italic = StyleFlags {
        italic: true,
        ..StyleFlags::default()
    };
    let underline = StyleFlags {
        underline: true,
        ..StyleFlags::default()
    };
    let strikethrough = StyleFlags {
        strikethrough: true,
        ..StyleFlags::default()
    };
    let plain = StyleFlags::default();

    match kind {
        MdTokenKind::Heading(1) => (palette.md_h1, bold),
        MdTokenKind::Heading(2) => (palette.md_h2, bold),
        MdTokenKind::Heading(_) => (palette.md_h3, bold),
        MdTokenKind::Strong => (palette.md_strong, bold),
        MdTokenKind::Emphasis => (palette.md_emphasis, italic),
        MdTokenKind::Strikethrough => (palette.md_strikethrough, strikethrough),
        MdTokenKind::Link => (palette.md_link, underline),
        MdTokenKind::InlineCode => (palette.md_code_inline, plain),
        MdTokenKind::CodeBlock => (palette.md_code_block, plain),
        MdTokenKind::BlockQuote => (palette.md_blockquote, italic),
        MdTokenKind::AlertNote => (palette.md_alert_note, bold),
        MdTokenKind::AlertTip => (palette.md_alert_tip, bold),
        MdTokenKind::AlertImportant => (palette.accent, bold),
        MdTokenKind::AlertWarning => (palette.md_alert_warning, bold),
        MdTokenKind::AlertCaution => (palette.md_alert_caution, bold),
        MdTokenKind::Table => (palette.md_table_border, plain),
        MdTokenKind::ThematicBreak => (palette.md_hr, plain),
        MdTokenKind::ListMarker | MdTokenKind::TaskChecked | MdTokenKind::TaskUnchecked => {
            (palette.md_list_marker, plain)
        }
        MdTokenKind::FrontMatter
        | MdTokenKind::Html
        | MdTokenKind::Footnote
        | MdTokenKind::HardBreak => (palette.muted, plain),
    }
}

pub struct CodeHighlighter {
    syntax_set: SyntaxSet,
}

impl CodeHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    pub fn highlight_block(
        &self,
        language: &str,
        lines: &[&str],
        palette: &ThemePalette,
    ) -> Vec<Vec<StyledSegment>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut parse_state = ParseState::new(syntax);
        let mut scope_stack = ScopeStack::new();
        let mut result = Vec::with_capacity(lines.len());

        for &line in lines {
            let mut line_with_nl = String::with_capacity(line.len() + 1);
            line_with_nl.push_str(line);
            if !line.ends_with('\n') {
                line_with_nl.push('\n');
            }

            let Ok(ops) = parse_state.parse_line(&line_with_nl, &self.syntax_set) else {
                if line.is_empty() {
                    result.push(Vec::new());
                } else {
                    result.push(vec![StyledSegment {
                        text: line.to_string(),
                        fg: palette.md_code_block,
                        bg: None,
                        flags: StyleFlags::default(),
                    }]);
                }
                continue;
            };

            let mut segments: Vec<StyledSegment> = Vec::new();
            let mut prev_pos = 0usize;

            for (offset, op) in ops {
                let clamped = offset.min(line.len());
                if clamped > prev_pos && line.is_char_boundary(clamped) {
                    let (fg, flags) = style_for_scope_stack(&scope_stack, palette);
                    push_coalesced(&mut segments, &line[prev_pos..clamped], fg, None, flags);
                    prev_pos = clamped;
                }
                if scope_stack.apply(&op).is_err() {
                    scope_stack = ScopeStack::new();
                }
            }

            if prev_pos < line.len() {
                let (fg, flags) = style_for_scope_stack(&scope_stack, palette);
                push_coalesced(&mut segments, &line[prev_pos..], fg, None, flags);
            }

            result.push(segments);
        }

        result
    }
}

impl Default for CodeHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

fn style_for_scope_stack(stack: &ScopeStack, palette: &ThemePalette) -> (RgbColor, StyleFlags) {
    for scope in stack.as_slice().iter().rev() {
        let name = scope.build_string();
        if name == "comment" || name.starts_with("comment.") {
            return (
                palette.muted,
                StyleFlags {
                    italic: true,
                    ..StyleFlags::default()
                },
            );
        }
        if name == "string" || name.starts_with("string.") {
            return (palette.md_alert_tip, StyleFlags::default());
        }
        if name == "keyword"
            || name.starts_with("keyword.")
            || name == "storage"
            || name.starts_with("storage.")
        {
            return (
                palette.accent,
                StyleFlags {
                    bold: true,
                    ..StyleFlags::default()
                },
            );
        }
        if name == "constant" || name.starts_with("constant.") {
            return (palette.md_alert_warning, StyleFlags::default());
        }
        if name.starts_with("entity.name.function")
            || name.starts_with("support.function")
            || name.starts_with("variable.function")
            || name.starts_with("entity.name.type")
            || name.starts_with("entity.name.struct")
            || name.starts_with("entity.name.enum")
            || name.starts_with("entity.name.trait")
            || name.starts_with("entity.name.class")
            || name.starts_with("entity.name.interface")
            || name.starts_with("support.type")
            || name.starts_with("support.class")
        {
            return (palette.md_code_inline, StyleFlags::default());
        }
    }

    (palette.md_code_block, StyleFlags::default())
}

pub fn highlight_editor_line(
    line_idx: usize,
    line_text: &str,
    doc: &GfmDocument,
    theme: &Theme,
    overlay: &LineOverlay,
) -> Vec<StyledSegment> {
    if line_text.is_empty() {
        return Vec::new();
    }

    let char_to_byte: Vec<usize> = line_text
        .char_indices()
        .map(|(byte_idx, _)| byte_idx)
        .chain(std::iter::once(line_text.len()))
        .collect();
    let char_len = char_to_byte.len() - 1;

    let mut md_ranges: Vec<(Range<usize>, MdTokenKind)> = Vec::new();
    let mut boundaries: Vec<usize> = vec![0, char_len];

    for span in doc.spans_for_line(line_idx) {
        let start = if span.start_line < line_idx {
            0
        } else {
            byte_col_to_char_col(line_text, span.start_col)
        };
        let end = if span.end_line > line_idx {
            char_len
        } else {
            byte_col_to_char_col(line_text, span.end_col)
        };
        let clamped_start = start.min(char_len);
        let clamped_end = end.min(char_len);
        if clamped_start < clamped_end {
            boundaries.push(clamped_start);
            boundaries.push(clamped_end);
            md_ranges.push((clamped_start..clamped_end, span.kind));
        }
    }

    let push_range = |bounds: &mut Vec<usize>, r: &Range<usize>| {
        let s = r.start.min(char_len);
        let e = r.end.min(char_len);
        if s < e {
            bounds.push(s);
            bounds.push(e);
        }
    };

    for r in &overlay.search_match_cols {
        push_range(&mut boundaries, r);
    }
    if let Some(r) = &overlay.active_search_col {
        push_range(&mut boundaries, r);
    }
    for r in &overlay.selection_cols {
        push_range(&mut boundaries, r);
    }
    for r in &overlay.spell_error_cols {
        push_range(&mut boundaries, r);
    }

    boundaries.sort_unstable();
    boundaries.dedup();

    let palette = &theme.palette;
    let mut segments: Vec<StyledSegment> = Vec::new();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end {
            continue;
        }

        let mut fg = palette.foreground;
        let mut bg: Option<RgbColor> = None;
        let mut flags = StyleFlags::default();

        for (range, kind) in &md_ranges {
            if range.start <= start && end <= range.end {
                let (token_fg, token_flags) = style_for_md_token(*kind, palette);
                fg = token_fg;
                flags = token_flags;
            }
        }

        if overlay
            .search_match_cols
            .iter()
            .any(|r| r.start <= start && end <= r.end)
        {
            bg = Some(palette.search_match_bg);
        }
        if let Some(r) = &overlay.active_search_col {
            if r.start <= start && end <= r.end {
                bg = Some(palette.search_active_bg);
            }
        }
        if overlay
            .selection_cols
            .iter()
            .any(|r| r.start <= start && end <= r.end)
        {
            bg = Some(palette.selection_bg);
        }
        if overlay
            .spell_error_cols
            .iter()
            .any(|r| r.start <= start && end <= r.end)
        {
            fg = palette.spell_error;
            flags.underline = true;
        }

        let text = &line_text[char_to_byte[start]..char_to_byte[end]];
        push_coalesced(&mut segments, text, fg, bg, flags);
    }

    segments
}

fn byte_col_to_char_col(line_text: &str, byte_col: usize) -> usize {
    let clamped = byte_col.min(line_text.len());
    let mut count = 0;
    for (idx, _) in line_text.char_indices() {
        if idx >= clamped {
            return count;
        }
        count += 1;
    }
    count
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
    use crate::markdown::parse_gfm;
    use crate::theme::default_theme;

    #[test]
    fn code_highlighter_styles_rust_tokens_on_crt_amber() {
        let highlighter = CodeHighlighter::new();
        let palette = default_theme().unwrap().palette;
        let lines = ["fn main() {", "    // note", "    let s = \"hi\";", "}"];
        let highlighted = highlighter.highlight_block("rust", &lines, &palette);

        assert_eq!(highlighted.len(), lines.len());
        for (line_segments, expected_line) in highlighted.iter().zip(lines.iter()) {
            let joined: String = line_segments.iter().map(|s| s.text.as_str()).collect();
            assert_eq!(joined, *expected_line);
        }

        let fn_seg = highlighted[0]
            .iter()
            .find(|s| s.text == "fn")
            .expect("expected 'fn' segment");
        assert_eq!(fn_seg.fg, palette.accent);
        assert!(fn_seg.flags.bold);

        let comment_seg = highlighted[1]
            .iter()
            .find(|s| s.text == "// note")
            .expect("expected '// note' segment");
        assert_eq!(comment_seg.fg, palette.muted);
        assert!(comment_seg.flags.italic);

        let string_seg = highlighted[2]
            .iter()
            .find(|s| s.text == "\"hi\"")
            .expect("expected '\"hi\"' segment");
        assert_eq!(string_seg.fg, palette.md_alert_tip);
    }

    #[test]
    fn editor_line_layers_gfm_selection_and_spell_error() {
        let text = "# Hello wrld";
        let doc = parse_gfm(text);
        let theme = default_theme().unwrap();
        let overlay = LineOverlay {
            selection_cols: std::iter::once(2..7).collect(),
            spell_error_cols: std::iter::once(8..12).collect(),
            ..LineOverlay::default()
        };

        let segments = highlight_editor_line(0, text, &doc, &theme, &overlay);
        let joined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(joined, text);

        let hello_seg = segments
            .iter()
            .find(|s| s.text == "Hello")
            .expect("expected 'Hello' segment");
        assert_eq!(hello_seg.bg, Some(theme.palette.selection_bg));

        let wrld_seg = segments
            .iter()
            .find(|s| s.text == "wrld")
            .expect("expected 'wrld' segment");
        assert_eq!(wrld_seg.fg, theme.palette.spell_error);
        assert!(wrld_seg.flags.underline);
    }
}
