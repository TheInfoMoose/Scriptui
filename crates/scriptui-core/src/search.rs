use regex::RegexBuilder;

use crate::buffer::{BufferEdit, DocumentBuffer};
use crate::error::{ScriptuiError, ScriptuiResult};
use crate::selection::{Selection, SelectionSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub pattern: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
}

impl SearchQuery {
    pub fn plain(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchMatch {
    pub start: usize,
    pub end: usize,
}

pub fn find_all(buf: &DocumentBuffer, query: &SearchQuery) -> ScriptuiResult<Vec<SearchMatch>> {
    if query.pattern.is_empty() {
        return Ok(Vec::new());
    }

    let base_pattern = if query.use_regex {
        query.pattern.clone()
    } else {
        regex::escape(&query.pattern)
    };

    let full_pattern = if query.whole_word {
        format!(r"\b(?:{base_pattern})\b")
    } else {
        base_pattern
    };

    let regex = RegexBuilder::new(&full_pattern)
        .case_insensitive(!query.case_sensitive)
        .build()
        .map_err(|source| ScriptuiError::InvalidRegex {
            pattern: query.pattern.clone(),
            source,
        })?;

    let text = buf.text();
    let mut matches = Vec::new();
    let mut last_byte = 0usize;
    let mut last_char = 0usize;

    for m in regex.find_iter(&text) {
        if m.start() == m.end() {
            continue;
        }
        let start_byte = m.start();
        let end_byte = m.end();
        let start = last_char + text[last_byte..start_byte].chars().count();
        let end = start + text[start_byte..end_byte].chars().count();
        last_byte = end_byte;
        last_char = end;
        matches.push(SearchMatch { start, end });
    }

    Ok(matches)
}

pub fn next_match(matches: &[SearchMatch], after_char: usize) -> Option<SearchMatch> {
    matches
        .iter()
        .copied()
        .find(|m| m.start >= after_char)
        .or_else(|| matches.first().copied())
}

pub fn prev_match(matches: &[SearchMatch], before_char: usize) -> Option<SearchMatch> {
    matches
        .iter()
        .rev()
        .copied()
        .find(|m| m.start < before_char)
        .or_else(|| matches.last().copied())
}

pub fn select_all_occurrences(
    buf: &mut DocumentBuffer,
    query: &SearchQuery,
) -> ScriptuiResult<usize> {
    let matches = find_all(buf, query)?;
    if matches.is_empty() {
        return Ok(0);
    }

    let count = matches.len();
    let ranges = matches
        .into_iter()
        .map(|m| Selection::new(m.start, m.end))
        .collect();
    buf.set_selection(SelectionSet::from_ranges(ranges));
    Ok(count)
}

pub fn replace_all(
    buf: &mut DocumentBuffer,
    query: &SearchQuery,
    replacement: &str,
) -> ScriptuiResult<Vec<BufferEdit>> {
    let matches = find_all(buf, query)?;
    if matches.is_empty() {
        return Ok(Vec::new());
    }

    let replacement_chars = replacement.chars().count() as isize;
    let total_delta: isize = matches
        .iter()
        .map(|m| replacement_chars - (m.end - m.start) as isize)
        .sum();
    let last_original_end = matches.last().map(|m| m.end).unwrap_or(0);

    let mut edits = Vec::with_capacity(matches.len());
    for m in matches.iter().rev() {
        let edit = buf.apply_raw_edit(m.start, m.end, replacement)?;
        edits.push(edit);
    }

    let final_cursor = (last_original_end as isize + total_delta).max(0) as usize;
    buf.set_selection(SelectionSet::point(final_cursor));

    Ok(edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_all_case_and_whole_word() {
        let buf = DocumentBuffer::from_str_content("Cat cat caterpillar");

        let default_matches = find_all(&buf, &SearchQuery::plain("cat")).unwrap();
        assert_eq!(default_matches.len(), 3);

        let mut case_query = SearchQuery::plain("cat");
        case_query.case_sensitive = true;
        let case_matches = find_all(&buf, &case_query).unwrap();
        assert_eq!(case_matches.len(), 2);

        let mut word_query = SearchQuery::plain("cat");
        word_query.whole_word = true;
        let word_matches = find_all(&buf, &word_query).unwrap();
        assert_eq!(
            word_matches,
            vec![
                SearchMatch { start: 0, end: 3 },
                SearchMatch { start: 4, end: 7 },
            ]
        );
    }

    #[test]
    fn find_all_regex_and_invalid_regex() {
        let buf = DocumentBuffer::from_str_content("v1.0 and v2.15");

        let regex_query = SearchQuery {
            pattern: r"v\d+\.\d+".to_string(),
            case_sensitive: false,
            whole_word: false,
            use_regex: true,
        };
        let matches = find_all(&buf, &regex_query).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], SearchMatch { start: 0, end: 4 });
        assert_eq!(matches[1], SearchMatch { start: 9, end: 14 });

        let invalid_query = SearchQuery {
            pattern: "[unclosed".to_string(),
            case_sensitive: false,
            whole_word: false,
            use_regex: true,
        };
        let err = find_all(&buf, &invalid_query).unwrap_err();
        assert!(matches!(err, ScriptuiError::InvalidRegex { .. }));
    }

    #[test]
    fn select_all_and_live_multi_cursor_change() {
        let mut buf = DocumentBuffer::from_str_content("amber CRT amber");
        let count = select_all_occurrences(&mut buf, &SearchQuery::plain("amber")).unwrap();
        assert_eq!(count, 2);

        let _ = buf.apply_insert("gold");
        assert_eq!(buf.text(), "gold CRT gold");
    }

    #[test]
    fn replace_all_utf8_safe() {
        let mut buf = DocumentBuffer::from_str_content("café test café");
        let edits = replace_all(&mut buf, &SearchQuery::plain("café"), "bistro").unwrap();
        assert_eq!(buf.text(), "bistro test bistro");
        assert_eq!(edits.len(), 2);
        assert_eq!(buf.selection().primary(), Selection::point(18));
    }
}
