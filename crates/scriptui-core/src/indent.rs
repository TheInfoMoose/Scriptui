use serde::{Deserialize, Serialize};

use crate::buffer::{BufferEdit, DocumentBuffer};
use crate::error::ScriptuiResult;
use crate::selection::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentStyle {
    Spaces(u8),
    Tabs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndentConfig {
    pub style: IndentStyle,
    pub smart_gfm_continuation: bool,
}

impl Default for IndentConfig {
    fn default() -> Self {
        Self {
            style: IndentStyle::Spaces(4),
            smart_gfm_continuation: true,
        }
    }
}

impl IndentConfig {
    pub fn indent_unit(&self) -> String {
        match self.style {
            IndentStyle::Tabs => "\t".to_string(),
            IndentStyle::Spaces(n) => " ".repeat(n.max(1) as usize),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnterOutcome {
    Insert(String),
    ClearCurrentLine,
}

pub fn compute_enter_indent(
    current_line: &str,
    in_fenced_code_block: bool,
    config: IndentConfig,
) -> EnterOutcome {
    let trimmed = current_line.trim_start_matches([' ', '\t']);
    let leading_len = current_line.len() - trimmed.len();
    let leading = &current_line[..leading_len];

    if in_fenced_code_block {
        return EnterOutcome::Insert(format!("\n{leading}"));
    }

    if config.smart_gfm_continuation {
        if let Some(outcome) = parse_task_list(leading, trimmed) {
            return outcome;
        }
        if let Some(outcome) = parse_unordered_bullet(leading, trimmed) {
            return outcome;
        }
        if let Some(outcome) = parse_ordered_list(leading, trimmed) {
            return outcome;
        }
        if let Some(outcome) = parse_blockquote(leading, trimmed) {
            return outcome;
        }
    }

    EnterOutcome::Insert("\n".to_string())
}

fn parse_task_list(leading: &str, trimmed: &str) -> Option<EnterOutcome> {
    let bytes = trimmed.as_bytes();
    if bytes.len() < 6 {
        return None;
    }
    let marker = bytes[0];
    if !matches!(marker, b'-' | b'*' | b'+') {
        return None;
    }
    let checkbox = &trimmed[1..6];
    if !matches!(checkbox, " [ ] " | " [x] " | " [X] ") {
        return None;
    }
    let rest = &trimmed[6..];
    if rest.trim().is_empty() {
        Some(EnterOutcome::ClearCurrentLine)
    } else {
        let marker_char = marker as char;
        Some(EnterOutcome::Insert(format!(
            "\n{leading}{marker_char} [ ] "
        )))
    }
}

fn parse_unordered_bullet(leading: &str, trimmed: &str) -> Option<EnterOutcome> {
    let bytes = trimmed.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let marker = bytes[0];
    if !matches!(marker, b'-' | b'*' | b'+') || bytes[1] != b' ' {
        return None;
    }
    let rest = &trimmed[2..];
    if rest.trim().is_empty() {
        Some(EnterOutcome::ClearCurrentLine)
    } else {
        let marker_char = marker as char;
        Some(EnterOutcome::Insert(format!("\n{leading}{marker_char} ")))
    }
}

fn parse_ordered_list(leading: &str, trimmed: &str) -> Option<EnterOutcome> {
    let digits_len = trimmed
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits_len == 0 {
        return None;
    }
    let after_digits = &trimmed[digits_len..];
    let delim = if after_digits.starts_with(". ") {
        '.'
    } else if after_digits.starts_with(") ") {
        ')'
    } else {
        return None;
    };
    let rest = &after_digits[2..];
    if rest.trim().is_empty() {
        return Some(EnterOutcome::ClearCurrentLine);
    }
    let num = trimmed[..digits_len].parse::<usize>().ok()?;
    let next = num.saturating_add(1);
    Some(EnterOutcome::Insert(format!("\n{leading}{next}{delim} ")))
}

fn parse_blockquote(leading: &str, trimmed: &str) -> Option<EnterOutcome> {
    if !trimmed.starts_with('>') {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() && bytes[idx] == b'>' {
        idx += 1;
        if idx < bytes.len() && bytes[idx] == b' ' {
            idx += 1;
        }
    }
    let quote_prefix = &trimmed[..idx];
    let rest = &trimmed[idx..];
    if rest.trim().is_empty() {
        Some(EnterOutcome::ClearCurrentLine)
    } else {
        Some(EnterOutcome::Insert(format!("\n{leading}{quote_prefix}")))
    }
}

pub fn indent_lines(
    buf: &mut DocumentBuffer,
    start_line: usize,
    end_line: usize,
    config: IndentConfig,
) -> ScriptuiResult<Vec<BufferEdit>> {
    let total_lines = buf.len_lines();
    if total_lines == 0 || start_line > end_line {
        return Ok(Vec::new());
    }
    let max_line = total_lines - 1;
    let start = start_line.min(max_line);
    let end = end_line.min(max_line);
    let unit = config.indent_unit();

    let mut edits = Vec::with_capacity(end - start + 1);
    for line in (start..=end).rev() {
        let col0 = buf.position_to_char(Position { line, col: 0 });
        let edit = buf.apply_raw_edit(col0, col0, &unit)?;
        edits.push(edit);
    }
    Ok(edits)
}

pub fn dedent_lines(
    buf: &mut DocumentBuffer,
    start_line: usize,
    end_line: usize,
    config: IndentConfig,
) -> ScriptuiResult<Vec<BufferEdit>> {
    let total_lines = buf.len_lines();
    if total_lines == 0 || start_line > end_line {
        return Ok(Vec::new());
    }
    let max_line = total_lines - 1;
    let start = start_line.min(max_line);
    let end = end_line.min(max_line);

    let mut edits = Vec::new();
    for line in (start..=end).rev() {
        let Some(text) = buf.line_text(line) else {
            continue;
        };
        let remove_chars = match config.style {
            IndentStyle::Tabs => usize::from(text.starts_with('\t')),
            IndentStyle::Spaces(n) => text
                .chars()
                .take(n.max(1) as usize)
                .take_while(|c| *c == ' ')
                .count(),
        };
        if remove_chars > 0 {
            let col0 = buf.position_to_char(Position { line, col: 0 });
            let edit = buf.apply_raw_edit(col0, col0 + remove_chars, "")?;
            edits.push(edit);
        }
    }
    Ok(edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prose_paragraph_indent_does_not_auto_indent() {
        let outcome = compute_enter_indent(
            "    First line of paragraph.",
            false,
            IndentConfig::default(),
        );
        assert_eq!(outcome, EnterOutcome::Insert("\n".to_string()));
    }

    #[test]
    fn gfm_list_and_blockquote_continuation() {
        let cfg = IndentConfig::default();
        assert_eq!(
            compute_enter_indent("  - item", false, cfg),
            EnterOutcome::Insert("\n  - ".to_string())
        );
        assert_eq!(
            compute_enter_indent("1. first", false, cfg),
            EnterOutcome::Insert("\n2. ".to_string())
        );
        assert_eq!(
            compute_enter_indent("- [x] done", false, cfg),
            EnterOutcome::Insert("\n- [ ] ".to_string())
        );
        assert_eq!(
            compute_enter_indent("> quote", false, cfg),
            EnterOutcome::Insert("\n> ".to_string())
        );
        assert_eq!(
            compute_enter_indent("  - ", false, cfg),
            EnterOutcome::ClearCurrentLine
        );
    }

    #[test]
    fn fenced_code_block_preserves_indent() {
        let outcome = compute_enter_indent("    let x = 1;", true, IndentConfig::default());
        assert_eq!(outcome, EnterOutcome::Insert("\n    ".to_string()));
    }

    #[test]
    fn block_indent_and_dedent() {
        let mut buf = DocumentBuffer::from_str_content("a\nb\nc");
        let cfg = IndentConfig {
            style: IndentStyle::Spaces(2),
            smart_gfm_continuation: true,
        };
        let indent_edits = indent_lines(&mut buf, 0, 1, cfg).unwrap();
        assert_eq!(indent_edits.len(), 2);
        assert_eq!(buf.text(), "  a\n  b\nc");

        let dedent_edits = dedent_lines(&mut buf, 0, 1, cfg).unwrap();
        assert_eq!(dedent_edits.len(), 2);
        assert_eq!(buf.text(), "a\nb\nc");
    }
}
