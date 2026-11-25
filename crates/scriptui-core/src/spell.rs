use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{ScriptuiError, ScriptuiResult};

static NEXT_SPELL_TEMP_ID: AtomicU64 = AtomicU64::new(0);
const MAX_TEMP_CREATE_ATTEMPTS: usize = 128;

fn create_unique_temp_file(parent: &Path, file_name: &str) -> ScriptuiResult<(PathBuf, File)> {
    let pid = std::process::id();
    let mut last_path = PathBuf::new();

    for _ in 0..MAX_TEMP_CREATE_ATTEMPTS {
        let seq = NEXT_SPELL_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let tmp_name = format!("{file_name}.scriptui.tmp.{pid}.{seq}");
        let tmp_path = if parent.as_os_str().is_empty() {
            PathBuf::from(tmp_name)
        } else {
            parent.join(tmp_name)
        };

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
        {
            Ok(file) => return Ok((tmp_path, file)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                last_path = tmp_path;
            }
            Err(source) => {
                return Err(ScriptuiError::Io {
                    path: tmp_path,
                    source,
                });
            }
        }
    }

    Err(ScriptuiError::Io {
        path: last_path,
        source: io::Error::new(
            io::ErrorKind::AlreadyExists,
            "failed to create unique temporary file after bounded retries",
        ),
    })
}

fn write_unique_temp_file(
    parent: &Path,
    file_name: &str,
    write_fn: impl FnOnce(&mut File) -> io::Result<()>,
) -> ScriptuiResult<PathBuf> {
    let (tmp_path, mut file) = create_unique_temp_file(parent, file_name)?;
    let write_result = (|| -> io::Result<()> {
        write_fn(&mut file)?;
        file.flush()?;
        file.sync_all()?;
        Ok(())
    })();

    drop(file);
    if let Err(source) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(ScriptuiError::Io {
            path: tmp_path,
            source,
        });
    }
    Ok(tmp_path)
}

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

        let tmp_path = write_unique_temp_file(parent, file_name, |file| {
            for word in &self.words {
                writeln!(file, "{word}")?;
            }
            Ok(())
        })?;

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
            let Ok(canonical_dir) = fs::canonicalize(dir) else {
                continue;
            };
            let aff_path = dir.join(&aff_name);
            let dic_path = dir.join(&dic_name);
            if aff_path.is_file() && dic_path.is_file() {
                let (Ok(canonical_aff), Ok(canonical_dic)) =
                    (fs::canonicalize(&aff_path), fs::canonicalize(&dic_path))
                else {
                    continue;
                };
                if !canonical_aff.starts_with(&canonical_dir)
                    || !canonical_dic.starts_with(&canonical_dir)
                {
                    continue;
                }
                let aff =
                    fs::read_to_string(&canonical_aff).map_err(|source| ScriptuiError::Io {
                        path: canonical_aff,
                        source,
                    })?;
                let dic =
                    fs::read_to_string(&canonical_dic).map_err(|source| ScriptuiError::Io {
                        path: canonical_dic,
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

    pub fn personal(&self) -> &PersonalDictionary {
        &self.personal
    }

    pub fn personal_mut(&mut self) -> &mut PersonalDictionary {
        &mut self.personal
    }

    pub fn set_personal(&mut self, personal: PersonalDictionary) {
        self.personal = personal;
    }

    pub fn copy_user_words_from(&mut self, other: &SpellEngine) {
        for word in &other.personal.words {
            self.personal.add(word);
        }
        for word in &other.ignored {
            self.ignore_word(word);
        }
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

            if !word.chars().any(char::is_alphabetic) {
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

    let dic_name = format!("{locale}.dic");
    let aff_name = format!("{locale}.aff");
    let dic_dest = dest_dir.join(&dic_name);
    let aff_dest = dest_dir.join(&aff_name);

    let prior_dic_bytes = match fs::read(&dic_dest) {
        Ok(bytes) => Some(bytes),
        Err(err) if err.kind() == io::ErrorKind::NotFound => None,
        Err(source) => {
            return Err(ScriptuiError::Io {
                path: dic_dest,
                source,
            });
        }
    };

    let dic_tmp = write_unique_temp_file(dest_dir, &dic_name, |file| {
        file.write_all(dic_content.as_bytes())
    })?;
    let aff_tmp = match write_unique_temp_file(dest_dir, &aff_name, |file| {
        file.write_all(aff_content.as_bytes())
    }) {
        Ok(path) => path,
        Err(err) => {
            let _ = fs::remove_file(&dic_tmp);
            return Err(err);
        }
    };

    if let Err(source) = fs::rename(&dic_tmp, &dic_dest) {
        let _ = fs::remove_file(&dic_tmp);
        let _ = fs::remove_file(&aff_tmp);
        return Err(ScriptuiError::Io {
            path: dic_dest,
            source,
        });
    }

    if let Err(source) = fs::rename(&aff_tmp, &aff_dest) {
        let _ = fs::remove_file(&aff_tmp);
        if let Some(prior_bytes) = prior_dic_bytes {
            let _ = fs::write(&dic_dest, prior_bytes);
        } else {
            let _ = fs::remove_file(&dic_dest);
        }
        return Err(ScriptuiError::Io {
            path: aff_dest,
            source,
        });
    }

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
    ch.is_alphabetic() || ch.is_ascii_digit() || is_apostrophe(ch)
}

fn is_all_caps_acronym(word: &str) -> bool {
    let mut alpha_count = 0usize;
    let mut has_digit = false;
    for ch in word.chars() {
        if ch.is_alphabetic() {
            if !ch.is_uppercase() {
                return false;
            }
            alpha_count += 1;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        }
    }
    alpha_count >= 2 || (alpha_count >= 1 && has_digit)
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

    #[test]
    fn default_en_us_suggests_for_curly_apostrophe_misspelling() {
        let engine = SpellEngine::default_en_us().unwrap();
        let suggestions = engine.suggest("don\u{2019}tt", 5);
        assert!(
            !suggestions.is_empty(),
            "expected non-empty suggestions for curly-apostrophe misspelling"
        );
        assert!(
            suggestions.iter().any(|s| s == "don't"),
            "expected \"don't\" in suggestions, got {suggestions:?}"
        );
    }

    #[test]
    fn personal_dict_and_locale_import_use_atomic_o_excl_writes() {
        let dict_dir = tempfile::tempdir().unwrap();
        let dict_path = dict_dir.path().join("personal.dic");

        let mut personal = PersonalDictionary::default();
        personal.add("scriptui");
        personal.add("testuser");
        personal.save_to_file(&dict_path).unwrap();

        let reloaded = PersonalDictionary::load_from_file(&dict_path).unwrap();
        assert!(reloaded.contains("scriptui"));
        assert!(reloaded.contains("testuser"));

        let dict_entries: Vec<String> = fs::read_dir(dict_dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(dict_entries, vec!["personal.dic".to_string()]);

        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let aff_src = src_dir.path().join("de_DE.aff");
        let dic_src = src_dir.path().join("de_DE.dic");
        fs::write(&aff_src, "SET UTF-8\nTRY esianrtolcdugmphbyfvkwz\n").unwrap();
        fs::write(&dic_src, "1\nhallo\n").unwrap();

        let (dic_dest, aff_dest) =
            import_locale_files("de_DE", &dic_src, &aff_src, dest_dir.path()).unwrap();
        assert!(dic_dest.is_file());
        assert!(aff_dest.is_file());

        let mut dest_entries: Vec<String> = fs::read_dir(dest_dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        dest_entries.sort();
        assert_eq!(
            dest_entries,
            vec!["de_DE.aff".to_string(), "de_DE.dic".to_string()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn load_locale_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let dict_dir = root.path().join("dicts");
        fs::create_dir_all(&dict_dir).unwrap();

        let outside_aff = root.path().join("outside.aff");
        let outside_dic = root.path().join("outside.dic");
        fs::write(&outside_aff, "SET UTF-8\nTRY esianrtolcdugmphbyfvkwz\n").unwrap();
        fs::write(&outside_dic, "1\nbonjour\n").unwrap();

        symlink(&outside_aff, dict_dir.join("es_ES.aff")).unwrap();
        symlink(&outside_dic, dict_dir.join("es_ES.dic")).unwrap();

        let result = SpellEngine::load_locale("es_ES", std::slice::from_ref(&dict_dir));
        assert!(matches!(result, Err(ScriptuiError::SpellDict { .. })));
    }

    #[test]
    fn import_locale_files_rolls_back_dic_on_aff_rename_failure() {
        let src_dir = tempfile::tempdir().unwrap();
        let aff_src = src_dir.path().join("it_IT.aff");
        let dic_src = src_dir.path().join("it_IT.dic");
        fs::write(&aff_src, "SET UTF-8\nTRY esianrtolcdugmphbyfvkwz\n").unwrap();
        fs::write(&dic_src, "1\nciao\n").unwrap();

        // Case 1: dic_dest did not previously exist -> removed on aff rename failure.
        let dest_fresh = tempfile::tempdir().unwrap();
        let aff_blocker = dest_fresh.path().join("it_IT.aff");
        fs::create_dir_all(&aff_blocker).unwrap();
        fs::write(aff_blocker.join("keep"), "block rename").unwrap();

        let err = import_locale_files("it_IT", &dic_src, &aff_src, dest_fresh.path()).unwrap_err();
        assert!(matches!(err, ScriptuiError::Io { .. }));
        assert!(!dest_fresh.path().join("it_IT.dic").exists());

        // Case 2: dic_dest existed beforehand -> restored to prior content on aff rename failure.
        let dest_existing = tempfile::tempdir().unwrap();
        let prior_dic = dest_existing.path().join("it_IT.dic");
        fs::write(&prior_dic, "1\nprecedente\n").unwrap();
        let aff_blocker_existing = dest_existing.path().join("it_IT.aff");
        fs::create_dir_all(&aff_blocker_existing).unwrap();
        fs::write(aff_blocker_existing.join("keep"), "block rename").unwrap();

        let err =
            import_locale_files("it_IT", &dic_src, &aff_src, dest_existing.path()).unwrap_err();
        assert!(matches!(err, ScriptuiError::Io { .. }));
        assert_eq!(fs::read_to_string(&prior_dic).unwrap(), "1\nprecedente\n");
    }

    #[test]
    fn spell_handles_ascii_digit_tokens_without_flagging_pure_numbers() {
        let engine = SpellEngine::default_en_us().unwrap();
        assert!(engine.check_document("1st 2nd 2026", &[]).is_empty());

        let diags = engine.check_document("1st 2nd 2026 utf8", &[]);
        for diag in &diags {
            assert_ne!(diag.word, "1st");
            assert_ne!(diag.word, "2nd");
            assert_ne!(diag.word, "2026");
            assert_ne!(diag.word, "utf");
            assert_eq!(diag.word, "utf8");
        }
    }
}
