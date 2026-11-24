use crate::buffer::{BufferEdit, DocumentBuffer};
use crate::error::ScriptuiResult;
use crate::selection::SelectionSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditTransaction {
    pub edits: Vec<BufferEdit>,
    pub selection_before: SelectionSet,
    pub selection_after: SelectionSet,
}

#[derive(Debug, Clone)]
pub struct UndoStack {
    undo_entries: Vec<EditTransaction>,
    redo_entries: Vec<EditTransaction>,
    max_entries: usize,
    saved_depth: Option<usize>,
}

impl UndoStack {
    pub fn new(max_entries: usize) -> Self {
        Self {
            undo_entries: Vec::new(),
            redo_entries: Vec::new(),
            max_entries: max_entries.max(1),
            saved_depth: Some(0),
        }
    }

    pub fn mark_saved(&mut self) {
        self.saved_depth = Some(self.undo_entries.len());
    }

    pub fn is_at_saved_state(&self) -> bool {
        self.saved_depth == Some(self.undo_entries.len())
    }

    pub fn record(&mut self, tx: EditTransaction) {
        if tx.edits.is_empty() {
            return;
        }
        if let Some(d) = self.saved_depth {
            if d > self.undo_entries.len() {
                self.saved_depth = None;
            }
        }
        self.undo_entries.push(tx);
        self.redo_entries.clear();
        if self.undo_entries.len() > self.max_entries {
            self.undo_entries.remove(0);
            self.saved_depth = match self.saved_depth {
                Some(d) if d > 0 => Some(d - 1),
                _ => None,
            };
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_entries.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_entries.clear();
        self.redo_entries.clear();
        self.saved_depth = Some(0);
    }

    pub fn undo(&mut self, buf: &mut DocumentBuffer) -> ScriptuiResult<bool> {
        let Some(tx) = self.undo_entries.pop() else {
            return Ok(false);
        };

        for edit in tx.edits.iter().rev() {
            let inserted_len = edit.inserted.chars().count();
            buf.apply_raw_edit(edit.start, edit.start + inserted_len, &edit.removed)?;
        }

        buf.set_selection(tx.selection_before.clone());
        self.redo_entries.push(tx);
        buf.set_dirty(!self.is_at_saved_state());
        Ok(true)
    }

    pub fn redo(&mut self, buf: &mut DocumentBuffer) -> ScriptuiResult<bool> {
        let Some(tx) = self.redo_entries.pop() else {
            return Ok(false);
        };

        for edit in &tx.edits {
            buf.apply_raw_edit(edit.start, edit.end, &edit.inserted)?;
        }

        buf.set_selection(tx.selection_after.clone());
        self.undo_entries.push(tx);
        buf.set_dirty(!self.is_at_saved_state());
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::Selection;

    fn apply_and_record(buf: &mut DocumentBuffer, stack: &mut UndoStack, text: &str) {
        let selection_before = buf.selection().clone();
        let edits = buf.apply_insert(text);
        let selection_after = buf.selection().clone();
        stack.record(EditTransaction {
            edits,
            selection_before,
            selection_after,
        });
    }

    #[test]
    fn undo_redo_multi_cursor_transaction() {
        let mut buf = DocumentBuffer::from_str_content("alpha beta alpha");
        let selection_before =
            SelectionSet::from_ranges(vec![Selection::new(0, 5), Selection::new(11, 16)]);
        buf.set_selection(selection_before.clone());

        let edits = buf.apply_insert("gamma");
        let selection_after = buf.selection().clone();

        let mut stack = UndoStack::new(64);
        stack.record(EditTransaction {
            edits,
            selection_before: selection_before.clone(),
            selection_after,
        });

        assert!(stack.undo(&mut buf).expect("undo should succeed"));
        assert_eq!(buf.text(), "alpha beta alpha");
        assert_eq!(buf.selection(), &selection_before);

        assert!(stack.redo(&mut buf).expect("redo should succeed"));
        assert_eq!(buf.text(), "gamma beta gamma");
    }

    #[test]
    fn new_record_clears_redo_stack() {
        let mut buf = DocumentBuffer::empty();
        let mut stack = UndoStack::new(64);

        apply_and_record(&mut buf, &mut stack, "one ");
        apply_and_record(&mut buf, &mut stack, "two ");

        assert!(stack.undo(&mut buf).expect("undo should succeed"));
        assert!(stack.can_redo());

        apply_and_record(&mut buf, &mut stack, "three");
        assert!(!stack.can_redo());
    }

    #[test]
    fn max_entries_evicts_oldest() {
        let mut buf = DocumentBuffer::empty();
        let mut stack = UndoStack::new(2);

        apply_and_record(&mut buf, &mut stack, "a");
        apply_and_record(&mut buf, &mut stack, "b");
        apply_and_record(&mut buf, &mut stack, "c");

        assert!(stack.undo(&mut buf).expect("first undo should succeed"));
        assert!(stack.undo(&mut buf).expect("second undo should succeed"));
        assert!(!stack
            .undo(&mut buf)
            .expect("third undo should return false"));
    }

    #[test]
    fn undo_back_to_initial_or_saved_state_clears_dirty_flag() {
        let mut buf = DocumentBuffer::empty();
        let mut stack = UndoStack::new(64);
        assert!(!buf.is_dirty());
        assert!(stack.is_at_saved_state());

        apply_and_record(&mut buf, &mut stack, "hello");
        assert!(buf.is_dirty());
        assert!(!stack.is_at_saved_state());

        assert!(stack.undo(&mut buf).expect("undo should succeed"));
        assert!(!buf.is_dirty());
        assert!(stack.is_at_saved_state());

        assert!(stack.redo(&mut buf).expect("redo should succeed"));
        assert!(buf.is_dirty());
        assert!(!stack.is_at_saved_state());

        stack.mark_saved();
        buf.set_dirty(false);
        assert!(stack.is_at_saved_state());

        apply_and_record(&mut buf, &mut stack, " world");
        assert!(buf.is_dirty());
        assert!(!stack.is_at_saved_state());

        assert!(stack.undo(&mut buf).expect("undo to saved should succeed"));
        assert!(!buf.is_dirty());
        assert!(stack.is_at_saved_state());

        // Undoing past the saved point and recording a new edit invalidates the saved point.
        assert!(stack
            .undo(&mut buf)
            .expect("undo past saved should succeed"));
        assert!(buf.is_dirty());
        assert!(!stack.is_at_saved_state());

        apply_and_record(&mut buf, &mut stack, "branch");
        assert!(buf.is_dirty());
        assert!(!stack.is_at_saved_state());
    }
}
