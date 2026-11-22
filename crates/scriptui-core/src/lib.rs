pub mod buffer;
pub mod error;
pub mod history;
pub mod indent;
pub mod search;
pub mod selection;

pub use buffer::{BufferEdit, DocumentBuffer};
pub use error::{ScriptuiError, ScriptuiResult};
pub use history::{EditTransaction, UndoStack};
pub use indent::{
    compute_enter_indent, dedent_lines, indent_lines, EnterOutcome, IndentConfig, IndentStyle,
};
pub use search::{
    find_all, next_match, prev_match, replace_all, select_all_occurrences, SearchMatch, SearchQuery,
};
pub use selection::{Position, Selection, SelectionSet};
