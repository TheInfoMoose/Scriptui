use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use ropey::Rope;

use crate::error::{ScriptuiError, ScriptuiResult};
use crate::selection::{Position, Selection, SelectionSet};

static NEXT_DOCUMENT_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);
const MAX_TEMP_CREATE_ATTEMPTS: usize = 128;
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

fn next_document_id() -> u64 {
    NEXT_DOCUMENT_ID.fetch_add(1, Ordering::Relaxed)
}

fn create_unique_temp_file(parent: &Path, file_name: &str) -> ScriptuiResult<(PathBuf, File)> {
    let pid = std::process::id();
    let mut last_path = PathBuf::new();

    for _ in 0..MAX_TEMP_CREATE_ATTEMPTS {
        let seq = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferEdit {
    pub start: usize,
    pub end: usize,
    pub inserted: String,
    pub removed: String,
}

#[derive(Debug, Clone)]
pub struct DocumentBuffer {
    doc_id: u64,
    rope: Rope,
    path: Option<PathBuf>,
    dirty: bool,
    last_disk_mtime: Option<SystemTime>,
    selection: SelectionSet,
    revision: u64,
}

impl DocumentBuffer {
    pub fn empty() -> Self {
        Self {
            doc_id: next_document_id(),
            rope: Rope::new(),
            path: None,
            dirty: false,
            last_disk_mtime: None,
            selection: SelectionSet::point(0),
            revision: 0,
        }
    }

    pub fn empty_with_path(path: PathBuf) -> Self {
        Self {
            doc_id: next_document_id(),
            rope: Rope::new(),
            path: Some(path),
            dirty: false,
            last_disk_mtime: None,
            selection: SelectionSet::point(0),
            revision: 0,
        }
    }

    pub fn from_str_content(text: &str) -> Self {
        Self {
            doc_id: next_document_id(),
            rope: Rope::from_str(text),
            path: None,
            dirty: false,
            last_disk_mtime: None,
            selection: SelectionSet::point(0),
            revision: 0,
        }
    }

    pub fn from_file(path: &Path) -> ScriptuiResult<Self> {
        let meta = fs::metadata(path).map_err(|source| ScriptuiError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if meta.len() > MAX_FILE_BYTES {
            return Err(ScriptuiError::Io {
                path: path.to_path_buf(),
                source: io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "file size {} bytes exceeds maximum allowed size of {} bytes",
                        meta.len(),
                        MAX_FILE_BYTES
                    ),
                ),
            });
        }
        let content = fs::read_to_string(path).map_err(|source| ScriptuiError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let last_disk_mtime = meta.modified().ok();

        Ok(Self {
            doc_id: next_document_id(),
            rope: Rope::from_str(&content),
            path: Some(path.to_path_buf()),
            dirty: false,
            last_disk_mtime,
            selection: SelectionSet::point(0),
            revision: 0,
        })
    }

    pub fn save_atomic(&mut self) -> ScriptuiResult<()> {
        let target = match &self.path {
            Some(path) => path.clone(),
            None => {
                return Err(ScriptuiError::Io {
                    path: PathBuf::new(),
                    source: io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "cannot save buffer without an associated file path",
                    ),
                });
            }
        };

        let parent = target.parent().unwrap_or_else(|| Path::new(""));
        let file_name = target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("buffer");
        let (tmp_path, mut file) = create_unique_temp_file(parent, file_name)?;

        let write_result = (|| -> io::Result<()> {
            self.rope.write_to(&mut file)?;
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

        if let Err(source) = fs::rename(&tmp_path, &target) {
            let _ = fs::remove_file(&tmp_path);
            return Err(ScriptuiError::Io {
                path: target,
                source,
            });
        }

        self.last_disk_mtime = fs::metadata(&target)
            .ok()
            .and_then(|meta| meta.modified().ok());
        self.dirty = false;
        Ok(())
    }

    pub fn save_as_atomic(&mut self, path: PathBuf) -> ScriptuiResult<()> {
        self.path = Some(path);
        self.save_atomic()
    }

    pub fn has_external_modification(&self) -> ScriptuiResult<bool> {
        let (Some(path), Some(recorded_mtime)) = (&self.path, self.last_disk_mtime) else {
            return Ok(false);
        };

        let meta = fs::metadata(path).map_err(|source| ScriptuiError::Io {
            path: path.clone(),
            source,
        })?;
        let current_mtime = meta.modified().map_err(|source| ScriptuiError::Io {
            path: path.clone(),
            source,
        })?;

        Ok(current_mtime != recorded_mtime)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn doc_id(&self) -> u64 {
        self.doc_id
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn line_text(&self, line_idx: usize) -> Option<String> {
        if line_idx >= self.rope.len_lines() {
            return None;
        }
        let mut line = self.rope.line(line_idx).to_string();
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        Some(line)
    }

    pub fn selection(&self) -> &SelectionSet {
        &self.selection
    }

    pub fn set_selection(&mut self, mut selection: SelectionSet) {
        selection.clamp(self.len_chars());
        self.selection = selection;
    }

    pub fn char_to_position(&self, char_idx: usize) -> Position {
        let clamped = char_idx.min(self.len_chars());
        let line = self.rope.char_to_line(clamped);
        let line_start = self.rope.line_to_char(line);
        Position {
            line,
            col: clamped.saturating_sub(line_start),
        }
    }

    pub fn position_to_char(&self, pos: Position) -> usize {
        let last_line = self.rope.len_lines().saturating_sub(1);
        let line = pos.line.min(last_line);
        let line_slice = self.rope.line(line);
        let mut line_len = line_slice.len_chars();
        if line_len > 0 && line_slice.char(line_len - 1) == '\n' {
            line_len -= 1;
            if line_len > 0 && line_slice.char(line_len - 1) == '\r' {
                line_len -= 1;
            }
        }
        self.rope.line_to_char(line) + pos.col.min(line_len)
    }

    pub fn apply_insert(&mut self, text: &str) -> Vec<BufferEdit> {
        let primary_index = self.selection.primary_index();
        let len_chars = self.len_chars();
        let spans: Vec<(usize, usize)> = self
            .selection
            .ranges()
            .iter()
            .map(|sel| (sel.start().min(len_chars), sel.end().min(len_chars)))
            .collect();

        let inserted_chars = text.chars().count();
        let mut edits = Vec::with_capacity(spans.len());

        for &(start, end) in spans.iter().rev() {
            if text.is_empty() && start == end {
                continue;
            }
            let removed = self.rope.slice(start..end).to_string();
            if start < end {
                self.rope.remove(start..end);
            }
            if !text.is_empty() {
                self.rope.insert(start, text);
            }
            edits.push(BufferEdit {
                start,
                end,
                inserted: text.to_string(),
                removed,
            });
        }

        let mut new_ranges = Vec::with_capacity(spans.len());
        let mut delta: isize = 0;
        for &(start, end) in &spans {
            let shifted_start = (start as isize + delta).max(0) as usize;
            let caret = shifted_start + inserted_chars;
            new_ranges.push(Selection::point(caret));
            delta += inserted_chars as isize - (end - start) as isize;
        }

        self.selection = SelectionSet::from_ranges_with_primary(new_ranges, primary_index);
        self.selection.clamp(self.len_chars());
        if !edits.is_empty()
            && edits
                .iter()
                .any(|edit| edit.start < edit.end || !text.is_empty())
        {
            self.dirty = true;
            self.revision = self.revision.wrapping_add(1);
        }
        edits
    }

    pub fn apply_delete(&mut self, backward: bool) -> Vec<BufferEdit> {
        let primary_index = self.selection.primary_index();
        let len_chars = self.len_chars();
        let spans: Vec<(usize, usize)> = self
            .selection
            .ranges()
            .iter()
            .map(|sel| {
                let start = sel.start().min(len_chars);
                let end = sel.end().min(len_chars);
                if start < end {
                    (start, end)
                } else if backward {
                    if start >= 2
                        && self.rope.char(start - 2) == '\r'
                        && self.rope.char(start - 1) == '\n'
                    {
                        (start - 2, start)
                    } else if start > 0 {
                        (start - 1, start)
                    } else {
                        (0, 0)
                    }
                } else if start + 1 < len_chars
                    && self.rope.char(start) == '\r'
                    && self.rope.char(start + 1) == '\n'
                {
                    (start, start + 2)
                } else if start < len_chars {
                    (start, start + 1)
                } else {
                    (len_chars, len_chars)
                }
            })
            .collect();

        let mut non_empty: Vec<(usize, usize)> = spans
            .iter()
            .copied()
            .filter(|&(start, end)| start < end)
            .collect();
        non_empty.sort_unstable();

        let mut merged_deletes: Vec<(usize, usize)> = Vec::with_capacity(non_empty.len());
        for (start, end) in non_empty {
            if let Some((_, last_end)) = merged_deletes.last_mut() {
                if start < *last_end {
                    *last_end = (*last_end).max(end);
                    continue;
                }
            }
            merged_deletes.push((start, end));
        }

        let mut edits = Vec::with_capacity(merged_deletes.len());
        for &(start, end) in merged_deletes.iter().rev() {
            let removed = self.rope.slice(start..end).to_string();
            self.rope.remove(start..end);
            edits.push(BufferEdit {
                start,
                end,
                inserted: String::new(),
                removed,
            });
        }

        let mut new_ranges = Vec::with_capacity(spans.len());
        for &(orig_start, _) in &spans {
            let mut shift = 0usize;
            for &(del_start, del_end) in &merged_deletes {
                if del_end <= orig_start {
                    shift += del_end - del_start;
                } else if del_start < orig_start {
                    shift += orig_start - del_start;
                    break;
                } else {
                    break;
                }
            }
            new_ranges.push(Selection::point(orig_start.saturating_sub(shift)));
        }

        self.selection = SelectionSet::from_ranges_with_primary(new_ranges, primary_index);
        self.selection.clamp(self.len_chars());
        if !edits.is_empty() {
            self.dirty = true;
            self.revision = self.revision.wrapping_add(1);
        }
        edits
    }

    pub fn apply_raw_edit(
        &mut self,
        start: usize,
        end: usize,
        replacement: &str,
    ) -> ScriptuiResult<BufferEdit> {
        let len = self.len_chars();
        if start > end || end > len {
            let offset = if end > len { end } else { start };
            return Err(ScriptuiError::OutOfBounds { offset, len });
        }

        let removed = self.rope.slice(start..end).to_string();
        if start < end {
            self.rope.remove(start..end);
        }
        if !replacement.is_empty() {
            self.rope.insert(start, replacement);
        }
        self.selection.clamp(self.len_chars());
        self.dirty = true;
        self.revision = self.revision.wrapping_add(1);

        Ok(BufferEdit {
            start,
            end,
            inserted: replacement.to_string(),
            removed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_cursor_insert_replaces_all_selections() {
        let mut buf = DocumentBuffer::from_str_content("foo bar foo");
        buf.set_selection(SelectionSet::from_ranges(vec![
            Selection::new(0, 3),
            Selection::new(8, 11),
        ]));
        let edits = buf.apply_insert("baz");
        assert_eq!(edits.len(), 2);
        assert_eq!(buf.text(), "baz bar baz");
        assert!(buf.is_dirty());
        assert_eq!(
            buf.selection().ranges(),
            &[Selection::point(3), Selection::point(11)]
        );
    }

    #[test]
    fn multi_cursor_backspace_and_delete() {
        let mut buf = DocumentBuffer::from_str_content("ab cd");
        buf.set_selection(SelectionSet::from_ranges(vec![
            Selection::point(2),
            Selection::point(5),
        ]));
        let _ = buf.apply_delete(true);
        assert_eq!(buf.text(), "a c");

        buf.set_selection(SelectionSet::from_ranges(vec![
            Selection::point(0),
            Selection::point(2),
        ]));
        let _ = buf.apply_delete(false);
        assert_eq!(buf.text(), " ");
    }

    #[test]
    fn atomic_save_and_external_modification() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("sample.txt");

        let mut buf = DocumentBuffer::from_str_content("hello\nworld\n");
        buf.save_as_atomic(p.clone()).unwrap();
        assert!(!buf.is_dirty());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "hello\nworld\n");
        assert!(!buf.has_external_modification().unwrap());

        std::fs::write(&p, "updated\ncontent\n").unwrap();
        let reloaded = DocumentBuffer::from_file(&p).unwrap();
        assert_eq!(reloaded.text(), "updated\ncontent\n");
        assert!(!reloaded.is_dirty());
    }

    #[test]
    fn position_char_roundtrip() {
        let buf = DocumentBuffer::from_str_content("abc\ndef\nghi");
        assert_eq!(buf.char_to_position(5), Position { line: 1, col: 1 });
        assert_eq!(buf.position_to_char(Position { line: 2, col: 2 }), 10);
    }

    #[test]
    fn document_revision_increments_on_content_mutations() {
        let mut buf = DocumentBuffer::from_str_content("hello");
        assert_eq!(buf.revision(), 0);

        buf.set_selection(SelectionSet::point(5));
        assert_eq!(buf.revision(), 0);

        let _ = buf.apply_insert(" world");
        assert_eq!(buf.revision(), 1);

        let _ = buf.apply_delete(true);
        assert_eq!(buf.revision(), 2);

        let _ = buf.apply_raw_edit(0, 5, "hi").unwrap();
        assert_eq!(buf.revision(), 3);

        buf.set_selection(SelectionSet::point(0));
        assert_eq!(buf.revision(), 3);
    }

    #[test]
    fn empty_insert_does_not_mark_buffer_dirty_and_empty_with_path_is_clean() {
        let mut buf = DocumentBuffer::empty_with_path(PathBuf::from("new.md"));
        assert!(!buf.is_dirty());
        assert_eq!(buf.path(), Some(Path::new("new.md")));
        assert_eq!(buf.revision(), 0);

        let _ = buf.apply_insert("");
        assert!(!buf.is_dirty());
        assert_eq!(buf.revision(), 0);
    }

    #[test]
    fn save_atomic_skips_preexisting_temp_collision_with_o_excl() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("sample.txt");

        let pid = std::process::id();
        let legacy_stale = dir.path().join(format!("sample.txt.scriptui.tmp.{pid}"));
        std::fs::write(&legacy_stale, "legacy stale").unwrap();

        let next_seq = NEXT_TEMP_FILE_ID.load(Ordering::Relaxed);
        let colliding_tmp = dir
            .path()
            .join(format!("sample.txt.scriptui.tmp.{pid}.{next_seq}"));
        std::fs::write(&colliding_tmp, "preexisting collision").unwrap();

        let mut buf = DocumentBuffer::from_str_content("atomic content\n");
        buf.save_as_atomic(target.clone()).unwrap();

        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "atomic content\n"
        );
        assert_eq!(
            std::fs::read_to_string(&legacy_stale).unwrap(),
            "legacy stale"
        );
        assert_eq!(
            std::fs::read_to_string(&colliding_tmp).unwrap(),
            "preexisting collision"
        );

        let mut remaining: Vec<PathBuf> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        remaining.sort();

        let mut expected = vec![target, legacy_stale, colliding_tmp];
        expected.sort();
        assert_eq!(remaining, expected);
    }

    #[test]
    fn empty_insert_on_point_selection_returns_no_edits() {
        let mut buf = DocumentBuffer::empty();
        let edits = buf.apply_insert("");
        assert!(edits.is_empty());
        assert!(!buf.is_dirty());
        assert_eq!(buf.revision(), 0);
    }

    #[test]
    fn apply_insert_and_delete_preserve_nonzero_primary_index() {
        let mut buf = DocumentBuffer::from_str_content("alpha beta gamma");
        buf.set_selection(SelectionSet::from_ranges_with_primary(
            vec![Selection::point(5), Selection::point(10)],
            1,
        ));

        let _ = buf.apply_insert("!");
        assert_eq!(buf.selection().primary_index(), 1);
        assert_eq!(buf.selection().primary(), Selection::point(12));

        let _ = buf.apply_delete(true);
        assert_eq!(buf.selection().primary_index(), 1);
        assert_eq!(buf.selection().primary(), Selection::point(10));
    }

    #[test]
    fn from_file_rejects_files_exceeding_max_file_bytes() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.as_file().set_len(MAX_FILE_BYTES + 1).unwrap();

        let err = DocumentBuffer::from_file(tmp.path()).unwrap_err();
        match err {
            ScriptuiError::Io { source, .. } => {
                assert_eq!(source.kind(), std::io::ErrorKind::InvalidData);
                assert!(
                    source.to_string().contains("exceeds maximum allowed size"),
                    "unexpected error message: {source}"
                );
            }
            other => panic!("expected ScriptuiError::Io, got {other:?}"),
        }
    }

    #[test]
    fn apply_delete_backward_and_forward_remove_crlf_atomically() {
        let mut buf = DocumentBuffer::from_str_content("ab\r\ncd");
        buf.set_selection(SelectionSet::point(4));
        let edits = buf.apply_delete(true);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, 2);
        assert_eq!(edits[0].end, 4);
        assert_eq!(edits[0].removed, "\r\n");
        assert_eq!(buf.text(), "abcd");
        assert_eq!(buf.selection().primary(), Selection::point(2));

        let mut buf = DocumentBuffer::from_str_content("ab\r\ncd");
        buf.set_selection(SelectionSet::point(2));
        let edits = buf.apply_delete(false);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, 2);
        assert_eq!(edits[0].end, 4);
        assert_eq!(edits[0].removed, "\r\n");
        assert_eq!(buf.text(), "abcd");
        assert_eq!(buf.selection().primary(), Selection::point(2));

        let mut buf = DocumentBuffer::from_str_content("a\rb");
        buf.set_selection(SelectionSet::point(2));
        let edits = buf.apply_delete(true);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].removed, "\r");
        assert_eq!(buf.text(), "ab");
        assert_eq!(buf.selection().primary(), Selection::point(1));

        let mut buf = DocumentBuffer::from_str_content("a\rb");
        buf.set_selection(SelectionSet::point(1));
        let edits = buf.apply_delete(false);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].removed, "\r");
        assert_eq!(buf.text(), "ab");
        assert_eq!(buf.selection().primary(), Selection::point(1));

        let mut buf = DocumentBuffer::from_str_content("a\nb");
        buf.set_selection(SelectionSet::point(2));
        let edits = buf.apply_delete(true);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].removed, "\n");
        assert_eq!(buf.text(), "ab");
        assert_eq!(buf.selection().primary(), Selection::point(1));

        let mut buf = DocumentBuffer::from_str_content("a\nb");
        buf.set_selection(SelectionSet::point(1));
        let edits = buf.apply_delete(false);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].removed, "\n");
        assert_eq!(buf.text(), "ab");
        assert_eq!(buf.selection().primary(), Selection::point(1));

        let mut buf = DocumentBuffer::from_str_content("ab\r\ncd");
        buf.set_selection(SelectionSet::from_ranges(vec![
            Selection::point(3),
            Selection::point(4),
        ]));
        let edits = buf.apply_delete(true);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].removed, "\r\n");
        assert_eq!(buf.text(), "abcd");
        assert_eq!(buf.selection().ranges(), &[Selection::point(2)]);

        let mut buf = DocumentBuffer::from_str_content("ab\r\ncd");
        buf.set_selection(SelectionSet::from_ranges(vec![
            Selection::point(2),
            Selection::point(3),
        ]));
        let edits = buf.apply_delete(false);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].removed, "\r\n");
        assert_eq!(buf.text(), "abcd");
        assert_eq!(buf.selection().ranges(), &[Selection::point(2)]);
    }
}
