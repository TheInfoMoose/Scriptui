pub mod buffer;
pub mod buffers;
pub mod config;
pub mod editor;
pub mod error;
pub mod highlight;
pub mod history;
pub mod indent;
pub mod keymap;
pub mod layout;
pub mod markdown;
pub mod preview;
pub mod search;
pub mod selection;
pub mod spell;
pub mod theme;

pub use buffer::{BufferEdit, DocumentBuffer};
pub use buffers::{BufferId, BufferManager, CloseResult, ManagedBuffer};
pub use config::AppConfig;
pub use editor::{
    CommandEntry, CommandPaletteState, EditorState, FindReplaceState, OverlayMode, PreviewMode,
    SpellQuickFixState,
};
pub use error::{ScriptuiError, ScriptuiResult};
pub use highlight::{
    highlight_editor_line, style_for_md_token, CodeHighlighter, LineOverlay, StyleFlags,
    StyledSegment,
};
pub use history::{EditTransaction, UndoStack};
pub use indent::{
    compute_enter_indent, dedent_lines, indent_lines, EnterOutcome, IndentConfig, IndentStyle,
};
pub use keymap::{Action, KeyChord, KeyCode, KeyModifiers, Keymap};
pub use layout::{compute_indent_guides, compute_wrapped_layout, VisualLine, WrappedLayout};
pub use markdown::{parse_gfm, CodeBlockRegion, GfmDocument, MdSpan, MdTokenKind};
pub use preview::{build_preview, PreviewDocument, PreviewLine};
pub use search::{
    find_all, next_match, prev_match, replace_all, select_all_occurrences, SearchMatch, SearchQuery,
};
pub use selection::{Position, Selection, SelectionSet};
pub use spell::{
    import_locale_files, validate_locale_name, PersonalDictionary, SpellDiagnostic, SpellEngine,
};
pub use theme::{builtin_theme_names, default_theme, resolve_theme, RgbColor, Theme, ThemePalette};
