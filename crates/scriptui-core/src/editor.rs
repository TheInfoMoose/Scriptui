use std::path::{Path, PathBuf};

use crate::buffers::{BufferManager, CloseResult};
use crate::config::AppConfig;
use crate::error::{ScriptuiError, ScriptuiResult};
use crate::history::EditTransaction;
use crate::indent::{compute_enter_indent, dedent_lines, indent_lines, EnterOutcome, IndentStyle};
use crate::keymap::{Action, Keymap};
use crate::markdown::{parse_gfm, GfmDocument};
use crate::preview::{build_preview, PreviewDocument};
use crate::search::{
    find_all, next_match, prev_match, select_all_occurrences, SearchMatch, SearchQuery,
};
use crate::selection::{Position, Selection, SelectionSet};
use crate::spell::{SpellDiagnostic, SpellEngine};
use crate::theme::{default_theme, resolve_theme, Theme};

const PAGE_MOVE_LINES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorMove {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    Collapsed,
    Split(u16),
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMode {
    None,
    Find,
    Replace,
    CommandPalette,
    SpellQuickFix,
    ConfirmQuit,
    ConfirmCloseBuffer,
    OpenFilePrompt,
    SaveAsPrompt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindReplaceState {
    pub query: SearchQuery,
    pub replacement: String,
    pub matches: Vec<SearchMatch>,
    pub active_match: Option<usize>,
}

impl Default for FindReplaceState {
    fn default() -> Self {
        Self {
            query: SearchQuery::plain(""),
            replacement: String::new(),
            matches: Vec::new(),
            active_match: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteState {
    pub query: String,
    pub selected_index: usize,
    pub commands: Vec<CommandEntry>,
}

impl CommandPaletteState {
    pub fn default_commands() -> Vec<CommandEntry> {
        vec![
            CommandEntry {
                id: "file.save",
                label: "Save",
                action: Action::Save,
            },
            CommandEntry {
                id: "buffer.new",
                label: "New Buffer",
                action: Action::NewBuffer,
            },
            CommandEntry {
                id: "buffer.close",
                label: "Close Buffer",
                action: Action::CloseBuffer,
            },
            CommandEntry {
                id: "search.find",
                label: "Find",
                action: Action::OpenFind,
            },
            CommandEntry {
                id: "search.replace",
                label: "Replace",
                action: Action::OpenReplace,
            },
            CommandEntry {
                id: "search.select_all_occurrences",
                label: "Select All Occurrences",
                action: Action::SelectAllOccurrences,
            },
            CommandEntry {
                id: "edit.undo",
                label: "Undo",
                action: Action::Undo,
            },
            CommandEntry {
                id: "edit.redo",
                label: "Redo",
                action: Action::Redo,
            },
            CommandEntry {
                id: "view.toggle_preview",
                label: "Toggle Preview",
                action: Action::TogglePreview,
            },
            CommandEntry {
                id: "view.expand_preview",
                label: "Expand Preview",
                action: Action::ExpandPreview,
            },
            CommandEntry {
                id: "view.toggle_soft_wrap",
                label: "Toggle Soft Wrap",
                action: Action::ToggleSoftWrap,
            },
            CommandEntry {
                id: "view.toggle_line_numbers",
                label: "Toggle Line Numbers",
                action: Action::ToggleLineNumbers,
            },
            CommandEntry {
                id: "spell.quick_fix",
                label: "Spellcheck Quick Fix",
                action: Action::OpenSpellQuickFix,
            },
            CommandEntry {
                id: "app.quit",
                label: "Quit",
                action: Action::Quit,
            },
        ]
    }

    pub fn filtered(&self) -> Vec<&CommandEntry> {
        let trimmed = self.query.trim();
        if trimmed.is_empty() {
            return self.commands.iter().collect();
        }

        let needle = trimmed.to_lowercase();
        let mut substring_matches = Vec::new();
        let mut subsequence_matches = Vec::new();

        for entry in &self.commands {
            let label_lower = entry.label.to_lowercase();
            let id_lower = entry.id.to_lowercase();
            if label_lower.contains(&needle) || id_lower.contains(&needle) {
                substring_matches.push(entry);
            } else if is_ordered_subsequence(&needle, &label_lower)
                || is_ordered_subsequence(&needle, &id_lower)
            {
                subsequence_matches.push(entry);
            }
        }

        substring_matches.extend(subsequence_matches);
        substring_matches
    }
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            query: String::new(),
            selected_index: 0,
            commands: Self::default_commands(),
        }
    }
}

fn is_ordered_subsequence(needle: &str, haystack: &str) -> bool {
    let mut haystack_chars = haystack.chars();
    for target in needle.chars() {
        if !haystack_chars.any(|ch| ch == target) {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellQuickFixState {
    pub diagnostic: SpellDiagnostic,
    pub suggestions: Vec<String>,
    pub selected_index: usize,
}

#[derive(Debug, Clone)]
struct AnalysisCache {
    doc_id: u64,
    buffer_index: usize,
    doc_revision: u64,
    spell_revision: u64,
    gfm: GfmDocument,
    spell_diagnostics: Vec<SpellDiagnostic>,
}

pub struct EditorState {
    pub config: AppConfig,
    pub theme: Theme,
    pub keymap: Keymap,
    pub buffers: BufferManager,
    pub preview_mode: PreviewMode,
    pub overlay: OverlayMode,
    pub find_replace: FindReplaceState,
    pub palette: CommandPaletteState,
    pub spell_popup: Option<SpellQuickFixState>,
    pub spell_engine: Option<SpellEngine>,
    pub file_prompt_input: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    spell_revision: u64,
    analysis_cache: Option<AnalysisCache>,
    analysis_recomputes: u64,
}

impl EditorState {
    pub fn new(config: AppConfig) -> ScriptuiResult<Self> {
        let theme = match resolve_theme(&config.theme, &[]) {
            Ok(theme) => theme,
            Err(_) => default_theme()?,
        };
        let spell_engine = if config.spell_enabled {
            SpellEngine::default_en_us().ok()
        } else {
            None
        };

        Ok(Self {
            config,
            theme,
            keymap: Keymap::default_cua(),
            buffers: BufferManager::new(),
            preview_mode: PreviewMode::Collapsed,
            overlay: OverlayMode::None,
            find_replace: FindReplaceState::default(),
            palette: CommandPaletteState::default(),
            spell_popup: None,
            spell_engine,
            file_prompt_input: String::new(),
            status_message: None,
            should_quit: false,
            spell_revision: 0,
            analysis_cache: None,
            analysis_recomputes: 0,
        })
    }

    pub fn active_analysis(&mut self) -> (&GfmDocument, &[SpellDiagnostic]) {
        let doc_id = self.buffers.active().document.doc_id();
        let buffer_index = self.buffers.active_index();
        let doc_revision = self.buffers.active().document.revision();
        let spell_revision = self.spell_revision;

        let is_fresh = self.analysis_cache.as_ref().is_some_and(|cache| {
            cache.doc_id == doc_id
                && cache.buffer_index == buffer_index
                && cache.doc_revision == doc_revision
                && cache.spell_revision == spell_revision
        });

        if !is_fresh {
            self.analysis_cache = None;
        }

        let cache = self.analysis_cache.get_or_insert_with(|| {
            let text = self.buffers.active().document.text();
            let gfm = parse_gfm(&text);
            let spell_diagnostics = self
                .spell_engine
                .as_ref()
                .map(|e| e.check_document(&text, &gfm.spell_skip_byte_ranges))
                .unwrap_or_default();
            self.analysis_recomputes += 1;
            AnalysisCache {
                doc_id,
                buffer_index,
                doc_revision,
                spell_revision,
                gfm,
                spell_diagnostics,
            }
        });

        (&cache.gfm, &cache.spell_diagnostics)
    }

    pub fn invalidate_analysis_cache(&mut self) {
        self.analysis_cache = None;
    }

    pub fn open_file(&mut self, path: &Path) -> ScriptuiResult<crate::buffers::BufferId> {
        let id = self.buffers.open_file(path)?;
        self.find_replace.matches.clear();
        self.find_replace.active_match = None;
        self.analysis_cache = None;
        Ok(id)
    }

    pub fn analysis_recompute_count(&self) -> u64 {
        self.analysis_recomputes
    }

    pub fn insert_text(&mut self, text: &str) {
        let active = self.buffers.active_mut();
        let selection_before = active.document.selection().clone();
        let edits = active.document.apply_insert(text);
        let selection_after = active.document.selection().clone();
        active.history.record(EditTransaction {
            edits,
            selection_before,
            selection_after,
        });
    }

    pub fn update_find_matches(&mut self) -> ScriptuiResult<usize> {
        let matches = find_all(&self.buffers.active().document, &self.find_replace.query)?;
        let count = matches.len();
        self.find_replace.active_match = if matches.is_empty() { None } else { Some(0) };
        self.find_replace.matches = matches;
        Ok(count)
    }

    pub fn open_spell_quick_fix_at_cursor(&mut self) -> bool {
        if self.spell_engine.is_none() {
            return false;
        }

        let pos = {
            let doc_buf = &self.buffers.active().document;
            doc_buf.char_to_position(doc_buf.selection().primary().head)
        };
        let found = {
            let (_, diagnostics) = self.active_analysis();
            diagnostics
                .iter()
                .find(|diag| {
                    diag.line == pos.line && pos.col >= diag.col_start && pos.col <= diag.col_end
                })
                .cloned()
        };

        if let (Some(diag), Some(engine)) = (found, &self.spell_engine) {
            let suggestions = engine.suggest(&diag.word, 5);
            self.spell_popup = Some(SpellQuickFixState {
                diagnostic: diag,
                suggestions,
                selected_index: 0,
            });
            self.overlay = OverlayMode::SpellQuickFix;
            true
        } else {
            false
        }
    }

    pub fn apply_selected_spell_fix(&mut self) -> ScriptuiResult<bool> {
        let Some(popup) = &self.spell_popup else {
            return Ok(false);
        };
        let Some(replacement) = popup.suggestions.get(popup.selected_index).cloned() else {
            return Ok(false);
        };
        let diagnostic = popup.diagnostic.clone();

        let active = self.buffers.active_mut();
        let selection_before = active.document.selection().clone();
        let start = active.document.position_to_char(Position {
            line: diagnostic.line,
            col: diagnostic.col_start,
        });
        let end = active.document.position_to_char(Position {
            line: diagnostic.line,
            col: diagnostic.col_end,
        });
        let edit = active.document.apply_raw_edit(start, end, &replacement)?;
        let new_caret = start + replacement.chars().count();
        active
            .document
            .set_selection(SelectionSet::point(new_caret));
        let selection_after = active.document.selection().clone();
        active.history.record(EditTransaction {
            edits: vec![edit],
            selection_before,
            selection_after,
        });

        self.spell_popup = None;
        self.overlay = OverlayMode::None;
        Ok(true)
    }

    pub fn switch_theme(&mut self, name: &str, search_dirs: &[PathBuf]) -> ScriptuiResult<()> {
        let theme = resolve_theme(name, search_dirs)?;
        self.theme = theme;
        self.config.theme = name.to_string();
        Ok(())
    }

    pub fn switch_spell_locale(
        &mut self,
        locale: &str,
        search_dirs: &[PathBuf],
    ) -> ScriptuiResult<()> {
        let engine = SpellEngine::load_locale(locale, search_dirs)?;
        self.spell_engine = Some(engine);
        self.config.spell_locale = locale.to_string();
        self.spell_revision = self.spell_revision.wrapping_add(1);
        Ok(())
    }

    pub fn add_spell_popup_word_to_personal_dict(&mut self) -> bool {
        let (Some(popup), Some(engine)) = (&self.spell_popup, &mut self.spell_engine) else {
            return false;
        };
        engine.personal_mut().add(&popup.diagnostic.word);
        self.spell_popup = None;
        self.overlay = OverlayMode::None;
        self.spell_revision = self.spell_revision.wrapping_add(1);
        true
    }

    pub fn ignore_spell_popup_word(&mut self) -> bool {
        let (Some(popup), Some(engine)) = (&self.spell_popup, &mut self.spell_engine) else {
            return false;
        };
        engine.ignore_word(&popup.diagnostic.word);
        self.spell_popup = None;
        self.overlay = OverlayMode::None;
        self.spell_revision = self.spell_revision.wrapping_add(1);
        true
    }

    pub fn replace_active_match(&mut self) -> ScriptuiResult<bool> {
        if self.find_replace.query.pattern.is_empty() {
            return Ok(false);
        }
        let needs_refresh = match self.find_replace.active_match {
            Some(idx) => idx >= self.find_replace.matches.len(),
            None => true,
        };
        if needs_refresh {
            self.update_find_matches()?;
        }
        let Some(idx) = self.find_replace.active_match else {
            return Ok(false);
        };
        let Some(m) = self.find_replace.matches.get(idx).copied() else {
            return Ok(false);
        };

        let replacement = self.find_replace.replacement.clone();
        let active = self.buffers.active_mut();
        let selection_before = active.document.selection().clone();
        let edit = active
            .document
            .apply_raw_edit(m.start, m.end, &replacement)?;
        let new_caret = m.start + replacement.chars().count();
        active
            .document
            .set_selection(SelectionSet::point(new_caret));
        let selection_after = active.document.selection().clone();
        active.history.record(EditTransaction {
            edits: vec![edit],
            selection_before,
            selection_after,
        });

        self.update_find_matches()?;
        if !self.find_replace.matches.is_empty() {
            self.find_replace.active_match = self
                .find_replace
                .matches
                .iter()
                .position(|next| next.start >= new_caret)
                .or(Some(0));
        }
        Ok(true)
    }

    pub fn replace_all_matches(&mut self) -> ScriptuiResult<usize> {
        let query = self.find_replace.query.clone();
        let replacement = self.find_replace.replacement.clone();
        let count = {
            let active = self.buffers.active_mut();
            let selection_before = active.document.selection().clone();
            let edits = crate::search::replace_all(&mut active.document, &query, &replacement)?;
            let count = edits.len();
            if !edits.is_empty() {
                let selection_after = active.document.selection().clone();
                active.history.record(EditTransaction {
                    edits,
                    selection_before,
                    selection_after,
                });
            }
            count
        };
        if count > 0 {
            self.update_find_matches()?;
        }
        Ok(count)
    }

    pub fn toggle_indent_style(&mut self) {
        self.config.indent.style = match self.config.indent.style {
            IndentStyle::Spaces(4) => IndentStyle::Spaces(2),
            IndentStyle::Spaces(2) => IndentStyle::Tabs,
            _ => IndentStyle::Spaces(4),
        };
    }

    pub fn confirm_dirty_action(&mut self, force: bool) {
        if !force {
            self.overlay = OverlayMode::None;
            return;
        }
        match self.overlay {
            OverlayMode::ConfirmQuit => {
                self.should_quit = true;
                self.overlay = OverlayMode::None;
            }
            OverlayMode::ConfirmCloseBuffer => {
                let _ = self.buffers.close_active(true);
                self.find_replace.matches.clear();
                self.find_replace.active_match = None;
                self.analysis_cache = None;
                self.overlay = OverlayMode::None;
            }
            _ => {}
        }
    }

    pub fn build_active_preview(&self, width: usize) -> PreviewDocument {
        build_preview(&self.buffers.active().document.text(), &self.theme, width)
    }

    pub fn selected_text(&self) -> String {
        let doc = &self.buffers.active().document;
        let text = doc.text();
        doc.selection()
            .ranges()
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| {
                text.chars()
                    .skip(s.start())
                    .take(s.end() - s.start())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn dispatch(&mut self, action: Action) -> ScriptuiResult<()> {
        match action {
            Action::NewBuffer => {
                self.buffers.new_buffer();
                self.find_replace.matches.clear();
                self.find_replace.active_match = None;
                self.analysis_cache = None;
            }
            Action::CloseBuffer => match self.buffers.close_active(false) {
                CloseResult::Closed => {
                    self.find_replace.matches.clear();
                    self.find_replace.active_match = None;
                    self.analysis_cache = None;
                }
                CloseResult::NeedsConfirmation => {
                    self.overlay = OverlayMode::ConfirmCloseBuffer;
                }
            },
            Action::NextBuffer => {
                self.buffers.next_buffer();
                self.find_replace.matches.clear();
                self.find_replace.active_match = None;
                self.analysis_cache = None;
            }
            Action::PrevBuffer => {
                self.buffers.prev_buffer();
                self.find_replace.matches.clear();
                self.find_replace.active_match = None;
                self.analysis_cache = None;
            }
            Action::Quit => {
                if self.buffers.has_dirty_buffers() {
                    self.overlay = OverlayMode::ConfirmQuit;
                } else {
                    self.should_quit = true;
                }
            }
            Action::Save => {
                if self.buffers.active().document.path().is_none() {
                    self.file_prompt_input.clear();
                    self.overlay = OverlayMode::SaveAsPrompt;
                } else {
                    self.buffers.active_mut().document.save_atomic()?;
                    self.status_message = Some("Saved".to_string());
                }
            }
            Action::SaveAs => {
                self.file_prompt_input = self
                    .buffers
                    .active()
                    .document
                    .path()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.overlay = OverlayMode::SaveAsPrompt;
            }
            Action::OpenFile => {
                self.file_prompt_input.clear();
                self.overlay = OverlayMode::OpenFilePrompt;
            }
            Action::Undo => {
                let active = self.buffers.active_mut();
                active.history.undo(&mut active.document)?;
            }
            Action::Redo => {
                let active = self.buffers.active_mut();
                active.history.redo(&mut active.document)?;
            }
            Action::OpenFind => {
                self.overlay = OverlayMode::Find;
            }
            Action::OpenReplace => {
                self.overlay = OverlayMode::Replace;
            }
            Action::FindNext => {
                self.jump_to_match(true)?;
            }
            Action::FindPrev => {
                self.jump_to_match(false)?;
            }
            Action::SelectAllOccurrences => {
                let query = self.find_replace.query.clone();
                select_all_occurrences(&mut self.buffers.active_mut().document, &query)?;
            }
            Action::OpenCommandPalette => {
                self.palette.query.clear();
                self.palette.selected_index = 0;
                self.overlay = OverlayMode::CommandPalette;
            }
            Action::OpenSpellQuickFix => {
                self.open_spell_quick_fix_at_cursor();
            }
            Action::TogglePreview => {
                let split = self.config.preview_split_percent.clamp(20, 80);
                self.preview_mode = match self.preview_mode {
                    PreviewMode::Collapsed => PreviewMode::Split(split),
                    PreviewMode::Split(_) | PreviewMode::Expanded => PreviewMode::Collapsed,
                };
            }
            Action::ExpandPreview => {
                let split = self.config.preview_split_percent.clamp(20, 80);
                self.preview_mode = match self.preview_mode {
                    PreviewMode::Expanded => PreviewMode::Split(split),
                    PreviewMode::Collapsed | PreviewMode::Split(_) => PreviewMode::Expanded,
                };
            }
            Action::ToggleSoftWrap => {
                self.config.soft_wrap = !self.config.soft_wrap;
            }
            Action::ToggleLineNumbers => {
                self.config.line_numbers = !self.config.line_numbers;
            }
            Action::InsertNewline => {
                self.handle_insert_newline()?;
            }
            Action::Backspace => {
                self.handle_delete(true);
            }
            Action::Delete => {
                self.handle_delete(false);
            }
            Action::Indent => {
                self.handle_indent(true)?;
            }
            Action::Dedent => {
                self.handle_indent(false)?;
            }
            Action::SelectAll => {
                let doc = &mut self.buffers.active_mut().document;
                let len_chars = doc.len_chars();
                let sel = if len_chars == 0 {
                    SelectionSet::point(0)
                } else {
                    SelectionSet::from_ranges(vec![Selection::new(0, len_chars)])
                };
                doc.set_selection(sel);
            }
            Action::Cut => {
                let has_selection = self
                    .buffers
                    .active()
                    .document
                    .selection()
                    .ranges()
                    .iter()
                    .any(|sel| !sel.is_empty());
                if has_selection {
                    self.handle_delete(true);
                }
            }
            Action::Copy | Action::Paste => {}
            Action::MoveLeft => self.move_primary_selection(CursorMove::Left, false),
            Action::MoveRight => self.move_primary_selection(CursorMove::Right, false),
            Action::MoveUp => self.move_primary_selection(CursorMove::Up, false),
            Action::MoveDown => self.move_primary_selection(CursorMove::Down, false),
            Action::MoveHome => self.move_primary_selection(CursorMove::Home, false),
            Action::MoveEnd => self.move_primary_selection(CursorMove::End, false),
            Action::PageUp => self.move_primary_selection(CursorMove::PageUp, false),
            Action::PageDown => self.move_primary_selection(CursorMove::PageDown, false),
            Action::SelectLeft => self.move_primary_selection(CursorMove::Left, true),
            Action::SelectRight => self.move_primary_selection(CursorMove::Right, true),
            Action::SelectUp => self.move_primary_selection(CursorMove::Up, true),
            Action::SelectDown => self.move_primary_selection(CursorMove::Down, true),
            Action::SelectHome => self.move_primary_selection(CursorMove::Home, true),
            Action::SelectEnd => self.move_primary_selection(CursorMove::End, true),
            Action::Escape => {
                if self.overlay != OverlayMode::None {
                    self.overlay = OverlayMode::None;
                    self.spell_popup = None;
                } else {
                    let mut sel = self.buffers.active().document.selection().clone();
                    sel.collapse_to_primary();
                    self.buffers.active_mut().document.set_selection(sel);
                }
            }
        }
        Ok(())
    }

    fn jump_to_match(&mut self, forward: bool) -> ScriptuiResult<()> {
        if self.find_replace.query.pattern.is_empty() {
            self.find_replace.matches.clear();
            self.find_replace.active_match = None;
            return Ok(());
        } else {
            self.update_find_matches()?;
        }

        let primary = self.buffers.active().document.selection().primary();
        let found = if forward {
            next_match(&self.find_replace.matches, primary.end())
        } else {
            prev_match(&self.find_replace.matches, primary.start())
        };

        if let Some(m) = found {
            self.find_replace.active_match = self
                .find_replace
                .matches
                .iter()
                .position(|candidate| *candidate == m);
            self.buffers
                .active_mut()
                .document
                .set_selection(SelectionSet::from_ranges(vec![Selection::new(
                    m.start, m.end,
                )]));
        }
        Ok(())
    }

    fn handle_insert_newline(&mut self) -> ScriptuiResult<()> {
        let (pos, current_line, in_code_block) = {
            let doc = &self.buffers.active().document;
            let text = doc.text();
            let pos = doc.char_to_position(doc.selection().primary().head);
            let gfm = parse_gfm(&text);
            let in_code_block = gfm.is_line_in_code_block(pos.line).is_some();
            let current_line = doc.line_text(pos.line).ok_or(ScriptuiError::OutOfBounds {
                offset: pos.line,
                len: doc.len_lines(),
            })?;
            (pos, current_line, in_code_block)
        };

        match compute_enter_indent(&current_line, in_code_block, self.config.indent) {
            EnterOutcome::Insert(text) => {
                self.insert_text(&text);
            }
            EnterOutcome::ClearCurrentLine => {
                let active = self.buffers.active_mut();
                let selection_before = active.document.selection().clone();
                let line_start = active.document.position_to_char(Position {
                    line: pos.line,
                    col: 0,
                });
                let line_end = line_start + current_line.chars().count();
                let edit = active.document.apply_raw_edit(line_start, line_end, "")?;
                active
                    .document
                    .set_selection(SelectionSet::point(line_start));
                let selection_after = active.document.selection().clone();
                active.history.record(EditTransaction {
                    edits: vec![edit],
                    selection_before,
                    selection_after,
                });
            }
        }
        Ok(())
    }

    fn handle_delete(&mut self, backward: bool) {
        let active = self.buffers.active_mut();
        let selection_before = active.document.selection().clone();
        let edits = active.document.apply_delete(backward);
        let selection_after = active.document.selection().clone();
        active.history.record(EditTransaction {
            edits,
            selection_before,
            selection_after,
        });
    }

    fn handle_indent(&mut self, indent: bool) -> ScriptuiResult<()> {
        let indent_cfg = self.config.indent;
        let active = self.buffers.active_mut();
        let selection_before = active.document.selection().clone();
        let primary = selection_before.primary();
        let start_pos = active.document.char_to_position(primary.start());
        let end_pos = active.document.char_to_position(primary.end());
        let start_line = start_pos.line;
        let end_line = if !primary.is_empty() && end_pos.col == 0 && end_pos.line > start_line {
            end_pos.line - 1
        } else {
            end_pos.line
        };

        let edits = if indent {
            indent_lines(&mut active.document, start_line, end_line, indent_cfg)?
        } else {
            dedent_lines(&mut active.document, start_line, end_line, indent_cfg)?
        };

        let max_len = active.document.len_chars() as isize;
        let shift_offset = |offset: usize| -> usize {
            let delta: isize = edits
                .iter()
                .filter(|edit| edit.start <= offset)
                .map(|edit| {
                    let inserted_len = edit.inserted.chars().count() as isize;
                    let deleted_len = (edit.end - edit.start) as isize;
                    if offset < edit.end {
                        inserted_len - (offset - edit.start) as isize
                    } else {
                        inserted_len - deleted_len
                    }
                })
                .sum();
            (offset as isize + delta).clamp(0, max_len) as usize
        };
        let new_anchor = shift_offset(primary.anchor);
        let new_head = shift_offset(primary.head);
        active
            .document
            .set_selection(SelectionSet::from_ranges(vec![Selection::new(
                new_anchor, new_head,
            )]));

        let selection_after = active.document.selection().clone();
        active.history.record(EditTransaction {
            edits,
            selection_before,
            selection_after,
        });
        Ok(())
    }

    fn move_primary_selection(&mut self, motion: CursorMove, extend: bool) {
        let doc = &mut self.buffers.active_mut().document;
        let primary = doc.selection().primary();
        let len_chars = doc.len_chars();
        let len_lines = doc.len_lines();
        let head_pos = doc.char_to_position(primary.head);

        let new_head = match motion {
            CursorMove::Left => primary.head.saturating_sub(1),
            CursorMove::Right => (primary.head + 1).min(len_chars),
            CursorMove::Up => {
                if head_pos.line > 0 {
                    doc.position_to_char(Position {
                        line: head_pos.line - 1,
                        col: head_pos.col,
                    })
                } else {
                    0
                }
            }
            CursorMove::Down => {
                if head_pos.line + 1 < len_lines {
                    doc.position_to_char(Position {
                        line: head_pos.line + 1,
                        col: head_pos.col,
                    })
                } else {
                    len_chars
                }
            }
            CursorMove::Home => doc.position_to_char(Position {
                line: head_pos.line,
                col: 0,
            }),
            CursorMove::End => {
                let line_len = doc
                    .line_text(head_pos.line)
                    .map_or(0, |line| line.chars().count());
                doc.position_to_char(Position {
                    line: head_pos.line,
                    col: line_len,
                })
            }
            CursorMove::PageUp => {
                let target_line = head_pos.line.saturating_sub(PAGE_MOVE_LINES);
                doc.position_to_char(Position {
                    line: target_line,
                    col: head_pos.col,
                })
            }
            CursorMove::PageDown => {
                let target_line =
                    (head_pos.line + PAGE_MOVE_LINES).min(len_lines.saturating_sub(1));
                doc.position_to_char(Position {
                    line: target_line,
                    col: head_pos.col,
                })
            }
        };

        let next_selection = if extend {
            Selection::new(primary.anchor, new_head)
        } else {
            Selection::point(new_head)
        };
        doc.set_selection(SelectionSet::from_ranges(vec![next_selection]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_state_dispatch_edit_undo_redo_and_smart_enter() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("- item");
        editor.dispatch(Action::InsertNewline).unwrap();
        assert_eq!(editor.buffers.active().document.text(), "- item\n- ");

        editor.dispatch(Action::Undo).unwrap();
        assert_eq!(editor.buffers.active().document.text(), "- item");

        editor.dispatch(Action::Redo).unwrap();
        assert_eq!(editor.buffers.active().document.text(), "- item\n- ");
    }

    #[test]
    fn editor_state_find_select_all_and_command_palette_fuzzy_filter() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("foo bar foo");
        editor.find_replace.query = SearchQuery::plain("foo");

        editor.dispatch(Action::SelectAllOccurrences).unwrap();
        assert_eq!(editor.buffers.active().document.selection().len(), 2);

        editor.palette.query = "prv".to_string();
        let labels: Vec<&str> = editor
            .palette
            .filtered()
            .into_iter()
            .map(|entry| entry.label)
            .collect();
        assert!(labels.contains(&"Toggle Preview"));
        assert!(labels.contains(&"Expand Preview"));
    }

    #[test]
    fn editor_state_preview_toggle_dirty_quit_guard_and_spell_quick_fix() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        assert_eq!(editor.preview_mode, PreviewMode::Collapsed);

        editor.dispatch(Action::TogglePreview).unwrap();
        assert_eq!(editor.preview_mode, PreviewMode::Split(50));

        editor.dispatch(Action::TogglePreview).unwrap();
        assert_eq!(editor.preview_mode, PreviewMode::Collapsed);

        editor.dispatch(Action::ExpandPreview).unwrap();
        assert_eq!(editor.preview_mode, PreviewMode::Expanded);

        editor.insert_text("hello wrld");
        editor.dispatch(Action::Quit).unwrap();
        assert_eq!(editor.overlay, OverlayMode::ConfirmQuit);
        assert!(!editor.should_quit);

        let caret = editor
            .buffers
            .active()
            .document
            .position_to_char(Position { line: 0, col: 7 });
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(caret));

        assert!(editor.open_spell_quick_fix_at_cursor());
        assert_eq!(editor.overlay, OverlayMode::SpellQuickFix);
        let suggestion = editor
            .spell_popup
            .as_ref()
            .and_then(|popup| popup.suggestions.first())
            .cloned()
            .expect("spell engine should provide a suggestion for 'wrld'");

        assert!(editor.apply_selected_spell_fix().unwrap());
        assert_eq!(editor.overlay, OverlayMode::None);
        assert!(editor.spell_popup.is_none());
        assert_eq!(
            editor.buffers.active().document.text(),
            format!("hello {suggestion}")
        );
    }

    #[test]
    fn find_next_refreshes_stale_matches_after_edit_and_buffer_switch() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("alpha beta");
        editor.find_replace.query = SearchQuery::plain("alpha");
        assert_eq!(editor.update_find_matches().unwrap(), 1);

        // Prepend text so the original match offset (0..5) becomes stale.
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(0));
        editor.insert_text("prefix ");
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(0));

        editor.dispatch(Action::FindNext).unwrap();
        assert_eq!(editor.selected_text(), "alpha");
        let sel = editor.buffers.active().document.selection().primary();
        assert_eq!((sel.start(), sel.end()), (7, 12));

        // Switch to a second buffer containing "alpha" at a different offset.
        editor.dispatch(Action::NewBuffer).unwrap();
        assert!(editor.find_replace.matches.is_empty());
        assert_eq!(editor.find_replace.active_match, None);

        editor.insert_text("zz alpha");
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(0));

        editor.dispatch(Action::FindNext).unwrap();
        assert_eq!(editor.selected_text(), "alpha");
        let sel2 = editor.buffers.active().document.selection().primary();
        assert_eq!((sel2.start(), sel2.end()), (3, 8));
    }

    #[test]
    fn indent_shifts_selection_and_excludes_trailing_col_zero_line() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("one\ntwo\nthree");

        let start = editor
            .buffers
            .active()
            .document
            .position_to_char(Position { line: 0, col: 1 });
        let end = editor
            .buffers
            .active()
            .document
            .position_to_char(Position { line: 2, col: 0 });
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::from_ranges(vec![Selection::new(start, end)]));

        editor.dispatch(Action::Indent).unwrap();

        assert_eq!(
            editor.buffers.active().document.text(),
            "    one\n    two\nthree"
        );
        let primary = editor.buffers.active().document.selection().primary();
        assert_eq!(primary.anchor, start + 4);
        assert_eq!(primary.head, end + 8);

        editor.dispatch(Action::SelectAll).unwrap();
        assert_eq!(editor.selected_text(), "    one\n    two\nthree");
    }

    #[test]
    fn dedent_inside_leading_whitespace_clamps_caret_to_line_start() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("one\n    two");

        let offset = editor
            .buffers
            .active()
            .document
            .position_to_char(Position { line: 1, col: 2 });
        assert_eq!(offset, 6);
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(offset));

        editor.dispatch(Action::Dedent).unwrap();

        assert_eq!(editor.buffers.active().document.text(), "one\ntwo");
        let new_offset = editor.buffers.active().document.selection().primary().head;
        assert_eq!(new_offset, 4);
        assert_eq!(
            editor
                .buffers
                .active()
                .document
                .char_to_position(new_offset),
            Position { line: 1, col: 0 }
        );
    }

    #[test]
    fn replace_active_and_all_and_spell_dict_helpers() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("foo bar foo baz foo");
        editor.find_replace.query = SearchQuery::plain("foo");
        editor.find_replace.replacement = "qux".to_string();

        assert!(editor.replace_active_match().unwrap());
        assert_eq!(
            editor.buffers.active().document.text(),
            "qux bar foo baz foo"
        );
        assert_eq!(
            editor.buffers.active().document.selection().primary().head,
            3
        );

        let replaced = editor.replace_all_matches().unwrap();
        assert_eq!(replaced, 2);
        assert_eq!(
            editor.buffers.active().document.text(),
            "qux bar qux baz qux"
        );

        // Undo reverts replace_all_matches as a single transaction.
        editor.dispatch(Action::Undo).unwrap();
        assert_eq!(
            editor.buffers.active().document.text(),
            "qux bar foo baz foo"
        );

        // Spell personal dict and ignore helpers.
        editor.dispatch(Action::NewBuffer).unwrap();
        editor.insert_text("hello wrld zzxxqq");
        let pos_wrld = editor
            .buffers
            .active()
            .document
            .position_to_char(Position { line: 0, col: 7 });
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(pos_wrld));
        assert!(editor.open_spell_quick_fix_at_cursor());
        assert!(editor.add_spell_popup_word_to_personal_dict());
        assert_eq!(editor.overlay, OverlayMode::None);
        assert!(editor.spell_popup.is_none());
        assert!(!editor.open_spell_quick_fix_at_cursor());

        let pos_zz = editor
            .buffers
            .active()
            .document
            .position_to_char(Position { line: 0, col: 12 });
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(pos_zz));
        assert!(editor.open_spell_quick_fix_at_cursor());
        assert!(editor.ignore_spell_popup_word());
        assert_eq!(editor.overlay, OverlayMode::None);
        assert!(editor.spell_popup.is_none());
        assert!(!editor.open_spell_quick_fix_at_cursor());

        // Indent style cycling: Spaces(4) -> Spaces(2) -> Tabs -> Spaces(4).
        assert_eq!(editor.config.indent.style, IndentStyle::Spaces(4));
        editor.toggle_indent_style();
        assert_eq!(editor.config.indent.style, IndentStyle::Spaces(2));
        editor.toggle_indent_style();
        assert_eq!(editor.config.indent.style, IndentStyle::Tabs);
        editor.toggle_indent_style();
        assert_eq!(editor.config.indent.style, IndentStyle::Spaces(4));

        // Confirm dirty actions (cancel, close buffer, quit).
        let buf_count = editor.buffers.len();
        editor.dispatch(Action::CloseBuffer).unwrap();
        assert_eq!(editor.overlay, OverlayMode::ConfirmCloseBuffer);
        editor.confirm_dirty_action(false);
        assert_eq!(editor.overlay, OverlayMode::None);
        assert_eq!(editor.buffers.len(), buf_count);

        editor.dispatch(Action::CloseBuffer).unwrap();
        assert_eq!(editor.overlay, OverlayMode::ConfirmCloseBuffer);
        editor.confirm_dirty_action(true);
        assert_eq!(editor.overlay, OverlayMode::None);
        assert_eq!(editor.buffers.len(), buf_count - 1);

        editor.dispatch(Action::Quit).unwrap();
        assert_eq!(editor.overlay, OverlayMode::ConfirmQuit);
        assert!(!editor.should_quit);
        editor.confirm_dirty_action(true);
        assert_eq!(editor.overlay, OverlayMode::None);
        assert!(editor.should_quit);
    }

    #[test]
    fn replace_active_match_advances_past_overlapping_replacement() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("foo bar foo baz foo");
        editor.find_replace.query = SearchQuery::plain("foo");
        editor.find_replace.replacement = "foobar".to_string();

        assert!(editor.replace_active_match().unwrap());
        assert_eq!(
            editor.buffers.active().document.text(),
            "foobar bar foo baz foo"
        );
        assert_eq!(editor.find_replace.active_match, Some(1));
        assert_eq!(
            editor.find_replace.matches[1],
            SearchMatch { start: 11, end: 14 }
        );

        assert!(editor.replace_active_match().unwrap());
        assert_eq!(
            editor.buffers.active().document.text(),
            "foobar bar foobar baz foo"
        );
        assert_eq!(editor.find_replace.active_match, Some(2));
        assert_eq!(
            editor.find_replace.matches[2],
            SearchMatch { start: 22, end: 25 }
        );
    }

    #[test]
    fn active_analysis_caches_across_unchanged_revisions_and_invalidates_on_edit_or_spell_change() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();
        editor.insert_text("hello wrld");

        let (_, diags_first) = editor.active_analysis();
        assert_eq!(diags_first.len(), 1);
        assert_eq!(editor.analysis_recompute_count(), 1);

        let (_, diags_second) = editor.active_analysis();
        assert_eq!(diags_second.len(), 1);
        assert_eq!(editor.analysis_recompute_count(), 1);

        editor.insert_text(" more");
        let _ = editor.active_analysis();
        assert_eq!(editor.analysis_recompute_count(), 2);

        let caret = editor
            .buffers
            .active()
            .document
            .position_to_char(Position { line: 0, col: 7 });
        editor
            .buffers
            .active_mut()
            .document
            .set_selection(SelectionSet::point(caret));
        assert!(editor.open_spell_quick_fix_at_cursor());
        assert_eq!(editor.analysis_recompute_count(), 2);

        assert!(editor.ignore_spell_popup_word());
        let (_, diags_after_ignore) = editor.active_analysis();
        assert!(diags_after_ignore.is_empty());
        assert_eq!(editor.analysis_recompute_count(), 3);
    }

    #[test]
    fn active_analysis_recomputes_when_open_file_replaces_initial_empty_buffer() {
        let mut editor = EditorState::new(AppConfig::default()).unwrap();

        let (_, initial_diags) = editor.active_analysis();
        assert!(initial_diags.is_empty());
        assert_eq!(editor.analysis_recompute_count(), 1);

        let dir = tempfile::tempdir().unwrap();
        let temp_path = dir.path().join("note.md");
        std::fs::write(&temp_path, "# Heading\nhello wrld\n").unwrap();

        editor.open_file(&temp_path).unwrap();
        assert_eq!(editor.buffers.active_index(), 0);
        assert_eq!(editor.buffers.active().document.revision(), 0);

        let (gfm, diags_after_open) = editor.active_analysis();
        assert!(!gfm.spans.is_empty());
        assert!(diags_after_open.iter().any(|diag| diag.word == "wrld"));
        assert_eq!(editor.analysis_recompute_count(), 2);

        std::fs::write(&temp_path, "clean text\n").unwrap();
        assert!(editor.buffers.reload_active_from_disk().unwrap());

        let (_, diags_after_reload) = editor.active_analysis();
        assert!(diags_after_reload.is_empty());
        assert_eq!(editor.analysis_recompute_count(), 3);
    }
}
