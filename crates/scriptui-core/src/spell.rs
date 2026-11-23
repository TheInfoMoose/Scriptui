use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::fs::{self, File};
use std::io::{self, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use crate::error::{ScriptuiError, ScriptuiResult};

pub const DEFAULT_LOCALE: &str = "en_US";

const BUNDLED_EN_US_AFF: &str = include_str!("../assets/dicts/en_US.aff");
const BUNDLED_EN_US_DIC: &str = include_str!("../assets/dicts/en_US.dic");

pub fn validate_locale_name(locale: &str) -> ScriptuiResult<()> {
    let valid = !locale.is_empty()
        && locale.len() <= 32
        && !locale.starts_with('.')
        && !locale.contains("..")
        && locale
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');

    if valid {
        Ok(())
    } else {
        Err(ScriptuiError::InvalidLocale(locale.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellDiagnostic {
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub word: String,
}

#[derive(Debug, Clone, Default)]
pub struct PersonalDictionary {
    words: BTreeSet<String>,
}

impl PersonalDictionary {
    pub fn load_from_file(path: &Path) -> ScriptuiResult<Self> {
        let content = fs::read_to_string(path).map_err(|source| ScriptuiError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut dict = Self::default();
        for line in content.lines() {
            dict.add(line);
        }
        Ok(dict)
    }

    pub fn save_to_file(&self, path: &Path) -> ScriptuiResult<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|source| ScriptuiError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
        }

        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("personal_dict");
        let tmp_name = format!("{file_name}.scriptui.tmp.{}", std::process::id());
        let tmp_path = if parent.as_os_str().is_empty() {
            PathBuf::from(tmp_name)
        } else {
            parent.join(tmp_name)
        };

        let write_result = (|| -> io::Result<()> {
            let mut file = File::create(&tmp_path)?;
            for word in &self.words {
                writeln!(file, "{word}")?;
            }
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(source) = write_result {
            let _ = fs::remove_file(&tmp_path);
            return Err(ScriptuiError::Io {
                path: tmp_path,
                source,
            });
        }

        if let Err(source) = fs::rename(&tmp_path, path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(ScriptuiError::Io {
                path: path.to_path_buf(),
                source,
            });
        }

        Ok(())
    }

    pub fn add(&mut self, word: &str) {
        let trimmed = word.trim();
        if !trimmed.is_empty() {
            self.words.insert(trimmed.to_string());
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        self.words.contains(word)
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

pub struct SpellEngine {
    locale: String,
    dict: spellbook::Dictionary,
    personal: PersonalDictionary,
    ignored: HashSet<String>,
}

impl SpellEngine {
    pub fn default_en_us() -> ScriptuiResult<Self> {
        let dict =
            spellbook::Dictionary::new(BUNDLED_EN_US_AFF, BUNDLED_EN_US_DIC).map_err(|err| {
                ScriptuiError::SpellDict {
                    locale: DEFAULT_LOCALE.to_string(),
                    reason: err.to_string(),
                }
            })?;
        Ok(Self {
            locale: DEFAULT_LOCALE.to_string(),
            dict,
            personal: PersonalDictionary::default(),
            ignored: HashSet::new(),
        })
    }

    pub fn load_locale(locale: &str, search_dirs: &[PathBuf]) -> ScriptuiResult<Self> {
        validate_locale_name(locale)?;

        let aff_name = format!("{locale}.aff");
        let dic_name = format!("{locale}.dic");

        for dir in search_dirs {
            let aff_path = dir.join(&aff_name);
            let dic_path = dir.join(&dic_name);
            if aff_path.is_file() && dic_path.is_file() {
                let aff = fs::read_to_string(&aff_path).map_err(|source| ScriptuiError::Io {
                    path: aff_path,
                    source,
                })?;
                let dic = fs::read_to_string(&dic_path).map_err(|source| ScriptuiError::Io {
                    path: dic_path,
                    source,
                })?;
                let dict = spellbook::Dictionary::new(&aff, &dic).map_err(|err| {
                    ScriptuiError::SpellDict {
                        locale: locale.to_string(),
                        reason: err.to_string(),
                    }
                })?;
                return Ok(Self {
                    locale: locale.to_string(),
                    dict,
                    personal: PersonalDictionary::default(),
                    ignored: HashSet::new(),
                });
            }
        }

        if locale == DEFAULT_LOCALE {
            return Self::default_en_us();
        }

        Err(ScriptuiError::SpellDict {
            locale: locale.to_string(),
            reason: format!("dictionary files {aff_name} and {dic_name} not found"),
        })
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn personal_mut(&mut self) -> &mut PersonalDictionary {
        &mut self.personal
    }

    pub fn ignore_word(&mut self, word: &str) {
        let trimmed = word.trim();
        if !trimmed.is_empty() {
            self.ignored.insert(trimmed.to_string());
        }
    }

    pub fn is_correct(&self, word: &str) -> bool {
        let normalized = normalize_apostrophes(word);
        if self.check_exact(word) || (normalized.as_ref() != word && self.check_exact(&normalized))
        {
            return true;
        }
        if normalized.chars().any(char::is_uppercase) {
            let lower = normalized.to_lowercase();
            if self.check_exact(&lower) {
                return true;
            }
            if normalized.as_ref() != word {
                let lower_raw = word.to_lowercase();
                if self.check_exact(&lower_raw) {
                    return true;
                }
            }
        }
        false
    }

    fn check_exact(&self, word: &str) -> bool {
        self.ignored.contains(word) || self.personal.contains(word) || self.dict.check(word)
    }

    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let normalized = normalize_apostrophes(word);
        let mut out = Vec::new();
        self.dict.suggest(&normalized, &mut out);
        out.truncate(limit);
        out
    }

    pub fn check_document(
        &self,
        text: &str,
        skip_byte_ranges: &[Range<usize>],
    ) -> Vec<SpellDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut line = 0usize;
        let mut col = 0usize;

        let mut token_start: Option<(usize, usize)> = None; // (byte_idx, col)
        let mut prev_end_byte = 0usize;
        let mut prev_end_col = 0usize;

        let flush_token = |diagnostics: &mut Vec<SpellDiagnostic>,
                           line: usize,
                           start_byte: usize,
                           start_col: usize,
                           end_byte: usize,
                           end_col: usize| {
            let raw = &text[start_byte..end_byte];
            let leading_apos: usize = raw
                .chars()
                .take_while(|&c| is_apostrophe(c))
                .map(char::len_utf8)
                .sum();
            let leading_apos_chars = raw.chars().take_while(|&c| is_apostrophe(c)).count();
            let trailing_apos: usize = raw
                .chars()
                .rev()
                .take_while(|&c| is_apostrophe(c))
                .map(char::len_utf8)
                .sum();
            let trailing_apos_chars = raw.chars().rev().take_while(|&c| is_apostrophe(c)).count();

            let byte_start = start_byte + leading_apos;
            let byte_end = end_byte.saturating_sub(trailing_apos);
            if byte_start >= byte_end {
                return;
            }

            let col_start = start_col + leading_apos_chars;
            let col_end = end_col.saturating_sub(trailing_apos_chars);
            let word = &text[byte_start..byte_end];

            if word.chars().count() <= 1 {
                return;
            }

            if is_all_caps_acronym(word) {
                return;
            }

            if overlaps_any(byte_start..byte_end, skip_byte_ranges) {
                return;
            }

            if !self.is_correct(word) {
                diagnostics.push(SpellDiagnostic {
                    line,
                    col_start,
                    col_end,
                    byte_start,
                    byte_end,
                    word: word.to_string(),
                });
            }
        };

        for (byte_idx, ch) in text.char_indices() {
            if is_word_char(ch) {
                if token_start.is_none() {
                    token_start = Some((byte_idx, col));
                }
                prev_end_byte = byte_idx + ch.len_utf8();
                prev_end_col = col + 1;
            } else if let Some((start_byte, start_col)) = token_start.take() {
                flush_token(
                    &mut diagnostics,
                    line,
                    start_byte,
                    start_col,
                    prev_end_byte,
                    prev_end_col,
                );
            }

            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        if let Some((start_byte, start_col)) = token_start.take() {
            flush_token(
                &mut diagnostics,
                line,
                start_byte,
                start_col,
                prev_end_byte,
                prev_end_col,
            );
        }

        diagnostics
    }
}

pub fn import_locale_files(
    locale: &str,
    dic_src: &Path,
    aff_src: &Path,
    dest_dir: &Path,
) -> ScriptuiResult<(PathBuf, PathBuf)> {
    validate_locale_name(locale)?;

    let dic_content = fs::read_to_string(dic_src).map_err(|source| ScriptuiError::Io {
        path: dic_src.to_path_buf(),
        source,
    })?;
    let aff_content = fs::read_to_string(aff_src).map_err(|source| ScriptuiError::Io {
        path: aff_src.to_path_buf(),
        source,
    })?;

    spellbook::Dictionary::new(&aff_content, &dic_content).map_err(|err| {
        ScriptuiError::SpellDict {
            locale: locale.to_string(),
            reason: err.to_string(),
        }
    })?;

    fs::create_dir_all(dest_dir).map_err(|source| ScriptuiError::Io {
        path: dest_dir.to_path_buf(),
        source,
    })?;

    let dic_dest = dest_dir.join(format!("{locale}.dic"));
    let aff_dest = dest_dir.join(format!("{locale}.aff"));

    fs::write(&dic_dest, &dic_content).map_err(|source| ScriptuiError::Io {
        path: dic_dest.clone(),
        source,
    })?;
    fs::write(&aff_dest, &aff_content).map_err(|source| ScriptuiError::Io {
        path: aff_dest.clone(),
        source,
    })?;

    Ok((dic_dest, aff_dest))
}

fn normalize_apostrophes(word: &str) -> Cow<'_, str> {
    if word.contains('\u{2019}') {
        Cow::Owned(word.replace('\u{2019}', "'"))
    } else {
        Cow::Borrowed(word)
    }
}

fn is_apostrophe(ch: char) -> bool {
    matches!(ch, '\'' | '\u{2019}')
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphabetic() || is_apostrophe(ch)
}

fn is_all_caps_acronym(word: &str) -> bool {
    let mut alpha_count = 0usize;
    for ch in word.chars() {
        if ch.is_alphabetic() {
            if !ch.is_uppercase() {
                return false;
            }
            alpha_count += 1;
        }
    }
    alpha_count >= 2
}

fn overlaps_any(range: Range<usize>, skip_ranges: &[Range<usize>]) -> bool {
    skip_ranges
        .iter()
        .any(|skip| range.start < skip.end && skip.start < range.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_en_us_flags_misspelling_and_suggests() {
        let engine = SpellEngine::default_en_us().unwrap();
        let diags = engine.check_document("hello wrld", &[]);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].word, "wrld");
        assert_eq!(diags[0].line, 0);
        assert_eq!(diags[0].col_start, 6);
        assert_eq!(diags[0].col_end, 10);

        let suggestions = engine.suggest("wrld", 5);
        assert!(
            suggestions.iter().any(|s| s == "world"),
            "expected 'world' in suggestions, got {suggestions:?}"
        );
    }

    #[test]
    fn spell_skips_code_ranges_and_respects_personal_and_ignore() {
        let mut engine = SpellEngine::default_en_us().unwrap();
        let text = "hello wrld `code_typo`";
        let code_start = text.find("`code_typo`").unwrap();
        let code_end = code_start + "`code_typo`".len();

        let code_range = code_start..code_end;
        let skip_ranges = std::slice::from_ref(&code_range);

        let diags = engine.check_document(text, skip_ranges);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].word, "wrld");

        engine.ignore_word("wrld");
        let after_ignore = engine.check_document(text, skip_ranges);
        assert!(after_ignore.is_empty());

        engine.personal_mut().add("scriptui");
        assert!(engine.is_correct("scriptui"));
    }

    #[test]
    fn import_and_load_custom_locale_and_reject_traversal() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();

        let aff_src = src_dir.path().join("fr_FR.aff");
        let dic_src = src_dir.path().join("fr_FR.dic");
        fs::write(&aff_src, "SET UTF-8\nTRY esianrtolcdugmphbyfvkwz\n").unwrap();
        fs::write(&dic_src, "1\nbonjour\n").unwrap();

        let (dic_dest, aff_dest) =
            import_locale_files("fr_FR", &dic_src, &aff_src, dest_dir.path()).unwrap();
        assert!(dic_dest.is_file());
        assert!(aff_dest.is_file());

        let engine = SpellEngine::load_locale("fr_FR", &[dest_dir.path().to_path_buf()]).unwrap();
        assert!(engine.is_correct("bonjour"));
        assert!(!engine.is_correct("bonjoru"));

        let traversal = validate_locale_name("../etc");
        assert!(matches!(traversal, Err(ScriptuiError::InvalidLocale(_))));
    }

    #[test]
    fn default_en_us_accepts_contractions_and_standard_english_words() {
        let engine = SpellEngine::default_en_us().unwrap();
        let diags = engine.check_document(
            "I don't think you\u{2019}ve emphasized the cascade. Don't stop.",
            &[],
        );
        assert!(
            diags.is_empty(),
            "expected no spell diagnostics for valid English text, got {diags:?}"
        );

        assert!(engine.is_correct("don't"));
        assert!(engine.is_correct("don\u{2019}t"));
        assert!(engine.is_correct("Don't"));
        assert!(engine.is_correct("emphasized"));
        assert!(engine.is_correct("cascade"));
        assert!(!engine.is_correct("wrld"));
    }
}
