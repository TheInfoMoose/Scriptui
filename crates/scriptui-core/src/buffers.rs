use std::fs;
use std::path::Path;

use crate::buffer::DocumentBuffer;
use crate::error::ScriptuiResult;
use crate::history::UndoStack;

pub type BufferId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseResult {
    Closed,
    NeedsConfirmation,
}

#[derive(Debug, Clone)]
pub struct ManagedBuffer {
    pub id: BufferId,
    pub document: DocumentBuffer,
    pub history: UndoStack,
    pub scroll_line: usize,
    pub scroll_col: usize,
    pub external_change_pending: bool,
}

impl ManagedBuffer {
    fn new_empty(id: BufferId, undo_limit: usize) -> Self {
        Self {
            id,
            document: DocumentBuffer::empty(),
            history: UndoStack::new(undo_limit),
            scroll_line: 0,
            scroll_col: 0,
            external_change_pending: false,
        }
    }

    pub fn display_name(&self) -> String {
        self.document
            .path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("Untitled-{}", self.id))
    }

    pub fn is_dirty(&self) -> bool {
        self.document.is_dirty()
    }
}

#[derive(Debug, Clone)]
pub struct BufferManager {
    buffers: Vec<ManagedBuffer>,
    active_index: usize,
    next_id: BufferId,
    undo_limit: usize,
}

impl BufferManager {
    pub const DEFAULT_UNDO_LIMIT: usize = 500;

    pub fn new() -> Self {
        let initial = ManagedBuffer::new_empty(1, Self::DEFAULT_UNDO_LIMIT);
        Self {
            buffers: vec![initial],
            active_index: 0,
            next_id: 2,
            undo_limit: Self::DEFAULT_UNDO_LIMIT,
        }
    }

    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn buffers(&self) -> &[ManagedBuffer] {
        &self.buffers
    }

    pub fn active(&self) -> &ManagedBuffer {
        &self.buffers[self.active_index]
    }

    pub fn active_mut(&mut self) -> &mut ManagedBuffer {
        &mut self.buffers[self.active_index]
    }

    pub fn new_buffer(&mut self) -> BufferId {
        let id = self.next_id;
        self.next_id += 1;
        self.buffers
            .push(ManagedBuffer::new_empty(id, self.undo_limit));
        self.active_index = self.buffers.len() - 1;
        id
    }

    pub fn open_file(&mut self, path: &Path) -> ScriptuiResult<BufferId> {
        let target_canonical = fs::canonicalize(path).ok();
        if let Some((idx, existing)) = self.buffers.iter().enumerate().find(|(_, buf)| {
            let Some(existing_path) = buf.document.path() else {
                return false;
            };
            match (&target_canonical, fs::canonicalize(existing_path).ok()) {
                (Some(a), Some(b)) => a == &b,
                _ => existing_path == path,
            }
        }) {
            self.active_index = idx;
            return Ok(existing.id);
        }

        let document = DocumentBuffer::from_file(path)?;

        let replace_initial = self.buffers.len() == 1
            && !self.buffers[0].is_dirty()
            && self.buffers[0].document.path().is_none()
            && self.buffers[0].document.len_chars() == 0;

        if replace_initial {
            let first = &mut self.buffers[0];
            first.document = document;
            first.history = UndoStack::new(self.undo_limit);
            first.scroll_line = 0;
            first.scroll_col = 0;
            first.external_change_pending = false;
            self.active_index = 0;
            return Ok(first.id);
        }

        let id = self.next_id;
        self.next_id += 1;
        self.buffers.push(ManagedBuffer {
            id,
            document,
            history: UndoStack::new(self.undo_limit),
            scroll_line: 0,
            scroll_col: 0,
            external_change_pending: false,
        });
        self.active_index = self.buffers.len() - 1;
        Ok(id)
    }

    pub fn switch_to(&mut self, index: usize) -> bool {
        if index < self.buffers.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    pub fn next_buffer(&mut self) {
        self.active_index = (self.active_index + 1) % self.buffers.len();
    }

    pub fn prev_buffer(&mut self) {
        self.active_index = if self.active_index == 0 {
            self.buffers.len() - 1
        } else {
            self.active_index - 1
        };
    }

    pub fn close_active(&mut self, force: bool) -> CloseResult {
        if !force && self.active().is_dirty() {
            return CloseResult::NeedsConfirmation;
        }

        self.buffers.remove(self.active_index);
        if self.buffers.is_empty() {
            let id = self.next_id;
            self.next_id += 1;
            self.buffers
                .push(ManagedBuffer::new_empty(id, self.undo_limit));
            self.active_index = 0;
        } else {
            self.active_index = self.active_index.min(self.buffers.len() - 1);
        }

        CloseResult::Closed
    }

    pub fn dirty_buffer_ids(&self) -> Vec<BufferId> {
        self.buffers
            .iter()
            .filter(|buf| buf.is_dirty())
            .map(|buf| buf.id)
            .collect()
    }

    pub fn has_dirty_buffers(&self) -> bool {
        self.buffers.iter().any(ManagedBuffer::is_dirty)
    }

    pub fn poll_external_modifications(&mut self) -> Vec<BufferId> {
        let mut pending = Vec::new();
        for buf in &mut self.buffers {
            if buf.document.has_external_modification().unwrap_or(false) {
                buf.external_change_pending = true;
            }
            if buf.external_change_pending {
                pending.push(buf.id);
            }
        }
        pending
    }

    pub fn reload_active_from_disk(&mut self) -> ScriptuiResult<bool> {
        let Some(path) = self.active().document.path().map(Path::to_path_buf) else {
            return Ok(false);
        };

        let document = DocumentBuffer::from_file(&path)?;
        let active = self.active_mut();
        active.document = document;
        active.history.clear();
        active.external_change_pending = false;
        Ok(true)
    }
}

impl Default for BufferManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::time::{Duration, SystemTime};

    use super::*;

    #[test]
    fn open_file_reuses_initial_empty_tab_and_deduplicates_path() {
        let dir = tempfile::tempdir().unwrap();
        let path_a = dir.path().join("a.md");
        let path_b = dir.path().join("b.md");
        fs::write(&path_a, "# Alpha\n").unwrap();
        fs::write(&path_b, "# Beta\n").unwrap();

        let mut mgr = BufferManager::new();
        let id_a = mgr.open_file(&path_a).unwrap();
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.active().display_name(), "a.md");

        let _id_b = mgr.open_file(&path_b).unwrap();
        assert_eq!(mgr.len(), 2);
        assert_eq!(mgr.active_index(), 1);

        let reopened_id = mgr.open_file(&path_a).unwrap();
        assert_eq!(reopened_id, id_a);
        assert_eq!(mgr.len(), 2);
        assert_eq!(mgr.active_index(), 0);
    }

    #[test]
    fn close_active_guards_dirty_buffer_and_allows_force_close() {
        let mut mgr = BufferManager::new();
        mgr.active_mut().document.apply_insert("draft text");

        assert!(mgr.has_dirty_buffers());
        assert_eq!(mgr.close_active(false), CloseResult::NeedsConfirmation);

        assert_eq!(mgr.close_active(true), CloseResult::Closed);
        assert_eq!(mgr.len(), 1);
        assert!(!mgr.active().is_dirty());
    }

    #[test]
    fn external_modification_detection_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.md");
        fs::write(&path, "initial content\n").unwrap();

        let mut mgr = BufferManager::new();
        let id = mgr.open_file(&path).unwrap();
        assert!(mgr.poll_external_modifications().is_empty());

        fs::write(&path, "updated content from disk\n").unwrap();
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() + Duration::from_secs(5))
            .unwrap();

        let modified = mgr.poll_external_modifications();
        assert_eq!(modified, vec![id]);
        assert!(mgr.active().external_change_pending);

        assert!(mgr.reload_active_from_disk().unwrap());
        assert_eq!(mgr.active().document.text(), "updated content from disk\n");
        assert!(!mgr.active().external_change_pending);
    }
}
