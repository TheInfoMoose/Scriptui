use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyModifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        alt: true,
        shift: false,
    };
    pub const SHIFT: Self = Self {
        ctrl: false,
        alt: false,
        shift: true,
    };
    pub const CTRL_SHIFT: Self = Self {
        ctrl: true,
        alt: false,
        shift: true,
    };
    pub const ALT_SHIFT: Self = Self {
        ctrl: false,
        alt: true,
        shift: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyChord {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn normalized(self) -> Self {
        if self.code == KeyCode::BackTab
            && self.modifiers.shift
            && !self.modifiers.ctrl
            && !self.modifiers.alt
        {
            return Self {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE,
            };
        }
        if let KeyCode::Char(c) = self.code {
            if self.modifiers.ctrl || self.modifiers.alt {
                return Self {
                    code: KeyCode::Char(c.to_ascii_lowercase()),
                    modifiers: self.modifiers,
                };
            }
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // File / Buffer / App
    Save,
    SaveAs,
    OpenFile,
    NewBuffer,
    CloseBuffer,
    NextBuffer,
    PrevBuffer,
    Quit,
    // History
    Undo,
    Redo,
    // Search / Multi-cursor
    OpenFind,
    OpenReplace,
    FindNext,
    FindPrev,
    SelectAllOccurrences,
    // Overlays / View
    OpenCommandPalette,
    OpenSpellQuickFix,
    TogglePreview,
    ExpandPreview,
    ToggleSoftWrap,
    ToggleLineNumbers,
    // Navigation
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveHome,
    MoveEnd,
    PageUp,
    PageDown,
    // Selection
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
    SelectHome,
    SelectEnd,
    // Editing
    InsertNewline,
    Backspace,
    Delete,
    Indent,
    Dedent,
    Copy,
    Cut,
    Paste,
    SelectAll,
    Escape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    bindings: HashMap<KeyChord, Action>,
}

impl Keymap {
    pub fn default_cua() -> Self {
        let mut map = Self {
            bindings: HashMap::new(),
        };

        // File / Buffer / App
        map.bind(
            KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL),
            Action::Save,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL_SHIFT),
            Action::SaveAs,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('o'), KeyModifiers::CTRL),
            Action::OpenFile,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('n'), KeyModifiers::CTRL),
            Action::NewBuffer,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('w'), KeyModifiers::CTRL),
            Action::CloseBuffer,
        );
        map.bind(
            KeyChord::new(KeyCode::PageDown, KeyModifiers::CTRL),
            Action::NextBuffer,
        );
        map.bind(
            KeyChord::new(KeyCode::PageUp, KeyModifiers::CTRL),
            Action::PrevBuffer,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('q'), KeyModifiers::CTRL),
            Action::Quit,
        );

        // History
        map.bind(
            KeyChord::new(KeyCode::Char('z'), KeyModifiers::CTRL),
            Action::Undo,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('y'), KeyModifiers::CTRL),
            Action::Redo,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('z'), KeyModifiers::CTRL_SHIFT),
            Action::Redo,
        );

        // Search / Multi-cursor
        map.bind(
            KeyChord::new(KeyCode::Char('f'), KeyModifiers::CTRL),
            Action::OpenFind,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('h'), KeyModifiers::CTRL),
            Action::OpenReplace,
        );
        map.bind(
            KeyChord::new(KeyCode::F(3), KeyModifiers::NONE),
            Action::FindNext,
        );
        map.bind(
            KeyChord::new(KeyCode::F(3), KeyModifiers::SHIFT),
            Action::FindPrev,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('l'), KeyModifiers::CTRL_SHIFT),
            Action::SelectAllOccurrences,
        );

        // Overlays / View
        map.bind(
            KeyChord::new(KeyCode::Char('p'), KeyModifiers::CTRL),
            Action::OpenCommandPalette,
        );
        map.bind(
            KeyChord::new(KeyCode::F(1), KeyModifiers::NONE),
            Action::OpenCommandPalette,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('.'), KeyModifiers::CTRL),
            Action::OpenSpellQuickFix,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('p'), KeyModifiers::CTRL_SHIFT),
            Action::TogglePreview,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('p'), KeyModifiers::ALT),
            Action::ExpandPreview,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('z'), KeyModifiers::ALT),
            Action::ToggleSoftWrap,
        );

        // Navigation
        map.bind(
            KeyChord::new(KeyCode::Left, KeyModifiers::NONE),
            Action::MoveLeft,
        );
        map.bind(
            KeyChord::new(KeyCode::Right, KeyModifiers::NONE),
            Action::MoveRight,
        );
        map.bind(
            KeyChord::new(KeyCode::Up, KeyModifiers::NONE),
            Action::MoveUp,
        );
        map.bind(
            KeyChord::new(KeyCode::Down, KeyModifiers::NONE),
            Action::MoveDown,
        );
        map.bind(
            KeyChord::new(KeyCode::Home, KeyModifiers::NONE),
            Action::MoveHome,
        );
        map.bind(
            KeyChord::new(KeyCode::End, KeyModifiers::NONE),
            Action::MoveEnd,
        );
        map.bind(
            KeyChord::new(KeyCode::PageUp, KeyModifiers::NONE),
            Action::PageUp,
        );
        map.bind(
            KeyChord::new(KeyCode::PageDown, KeyModifiers::NONE),
            Action::PageDown,
        );

        // Selection
        map.bind(
            KeyChord::new(KeyCode::Left, KeyModifiers::SHIFT),
            Action::SelectLeft,
        );
        map.bind(
            KeyChord::new(KeyCode::Right, KeyModifiers::SHIFT),
            Action::SelectRight,
        );
        map.bind(
            KeyChord::new(KeyCode::Up, KeyModifiers::SHIFT),
            Action::SelectUp,
        );
        map.bind(
            KeyChord::new(KeyCode::Down, KeyModifiers::SHIFT),
            Action::SelectDown,
        );
        map.bind(
            KeyChord::new(KeyCode::Home, KeyModifiers::SHIFT),
            Action::SelectHome,
        );
        map.bind(
            KeyChord::new(KeyCode::End, KeyModifiers::SHIFT),
            Action::SelectEnd,
        );

        // Editing
        map.bind(
            KeyChord::new(KeyCode::Enter, KeyModifiers::NONE),
            Action::InsertNewline,
        );
        map.bind(
            KeyChord::new(KeyCode::Backspace, KeyModifiers::NONE),
            Action::Backspace,
        );
        map.bind(
            KeyChord::new(KeyCode::Delete, KeyModifiers::NONE),
            Action::Delete,
        );
        map.bind(
            KeyChord::new(KeyCode::Tab, KeyModifiers::NONE),
            Action::Indent,
        );
        map.bind(
            KeyChord::new(KeyCode::BackTab, KeyModifiers::NONE),
            Action::Dedent,
        );
        map.bind(
            KeyChord::new(KeyCode::Tab, KeyModifiers::SHIFT),
            Action::Dedent,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('c'), KeyModifiers::CTRL),
            Action::Copy,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('x'), KeyModifiers::CTRL),
            Action::Cut,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('v'), KeyModifiers::CTRL),
            Action::Paste,
        );
        map.bind(
            KeyChord::new(KeyCode::Char('a'), KeyModifiers::CTRL),
            Action::SelectAll,
        );
        map.bind(
            KeyChord::new(KeyCode::Escape, KeyModifiers::NONE),
            Action::Escape,
        );

        map
    }

    pub fn lookup(&self, chord: KeyChord) -> Option<Action> {
        self.bindings.get(&chord.normalized()).copied()
    }

    pub fn bind(&mut self, chord: KeyChord, action: Action) {
        self.bindings.insert(chord.normalized(), action);
    }

    pub fn unbind(&mut self, chord: KeyChord) -> Option<Action> {
        self.bindings.remove(&chord.normalized())
    }

    pub fn bindings_for(&self, action: Action) -> Vec<KeyChord> {
        self.bindings
            .iter()
            .filter_map(|(&chord, &act)| (act == action).then_some(chord))
            .collect()
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::default_cua()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cua_bindings_cover_core_actions() {
        let keymap = Keymap::default_cua();

        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL)),
            Some(Action::Save)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('f'), KeyModifiers::CTRL)),
            Some(Action::OpenFind)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('h'), KeyModifiers::CTRL)),
            Some(Action::OpenReplace)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('l'), KeyModifiers::CTRL_SHIFT)),
            Some(Action::SelectAllOccurrences)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('z'), KeyModifiers::CTRL)),
            Some(Action::Undo)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('y'), KeyModifiers::CTRL)),
            Some(Action::Redo)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('p'), KeyModifiers::CTRL)),
            Some(Action::OpenCommandPalette)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::F(1), KeyModifiers::NONE)),
            Some(Action::OpenCommandPalette)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('.'), KeyModifiers::CTRL)),
            Some(Action::OpenSpellQuickFix)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Right, KeyModifiers::SHIFT)),
            Some(Action::SelectRight)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Tab, KeyModifiers::NONE)),
            Some(Action::Indent)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::BackTab, KeyModifiers::NONE)),
            Some(Action::Dedent)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('p'), KeyModifiers::CTRL_SHIFT)),
            Some(Action::TogglePreview)
        );
        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('z'), KeyModifiers::ALT)),
            Some(Action::ToggleSoftWrap)
        );
    }

    #[test]
    fn lookup_normalizes_ctrl_char_case_and_supports_override() {
        let mut keymap = Keymap::default_cua();

        assert_eq!(
            keymap.lookup(KeyChord::new(KeyCode::Char('S'), KeyModifiers::CTRL)),
            Some(Action::Save)
        );

        let custom = KeyChord::new(KeyCode::Char('k'), KeyModifiers::CTRL);
        keymap.bind(custom, Action::CloseBuffer);
        assert_eq!(keymap.lookup(custom), Some(Action::CloseBuffer));

        assert_eq!(keymap.unbind(custom), Some(Action::CloseBuffer));
        assert_eq!(keymap.lookup(custom), None);
    }

    #[test]
    fn backtab_with_shift_normalizes_to_dedent() {
        assert_eq!(
            Keymap::default_cua().lookup(KeyChord::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(Action::Dedent)
        );
    }
}
