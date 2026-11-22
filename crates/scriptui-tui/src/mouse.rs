use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use scriptui_core::{
    Action, EditorState, PreviewMode, ScriptuiResult, Selection, SelectionSet, WrappedLayout,
};

use crate::chrome::TabHitbox;
use crate::layout_tree::ScreenRegions;
use crate::viewport::ViewportMetrics;

const SCROLL_STEP_LINES: usize = 3;

#[derive(Debug, Clone)]
pub struct MouseContext<'a> {
    pub regions: ScreenRegions,
    pub tab_hitboxes: &'a [TabHitbox],
    pub viewport: Option<ViewportMetrics>,
    pub wrapped_layout: Option<&'a WrappedLayout>,
    pub total_area: Rect,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MouseRouter {
    dragging_selection_anchor: Option<usize>,
    dragging_split_divider: bool,
}

impl MouseRouter {
    pub fn handle_mouse(
        &mut self,
        event: MouseEvent,
        state: &mut EditorState,
        ctx: &MouseContext<'_>,
    ) -> ScriptuiResult<bool> {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(hb) = ctx
                    .tab_hitboxes
                    .iter()
                    .find(|hb| rect_contains(hb.rect, event.column, event.row))
                {
                    state.buffers.switch_to(hb.buffer_index);
                    state.find_replace.matches.clear();
                    state.find_replace.active_match = None;
                    return Ok(true);
                }

                if let Some(edge_tab) = ctx.regions.preview_edge_tab {
                    if rect_contains(edge_tab, event.column, event.row) {
                        state.dispatch(Action::TogglePreview)?;
                        return Ok(true);
                    }
                }

                if let Some(divider) = ctx.regions.split_divider {
                    if rect_contains(divider, event.column, event.row) {
                        self.dragging_split_divider = true;
                        return Ok(true);
                    }
                }

                if let (Some(viewport), Some(wrapped_layout)) = (ctx.viewport, ctx.wrapped_layout) {
                    if rect_contains(viewport.text_area, event.column, event.row) {
                        let offset = resolve_char_offset(
                            state,
                            viewport.text_area,
                            wrapped_layout,
                            event.column,
                            event.row,
                        );
                        state
                            .buffers
                            .active_mut()
                            .document
                            .set_selection(SelectionSet::point(offset));
                        self.dragging_selection_anchor = Some(offset);
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging_split_divider && ctx.total_area.width > 0 {
                    let right_edge = ctx.total_area.x.saturating_add(ctx.total_area.width);
                    let preview_pct = (right_edge.saturating_sub(event.column) as u32 * 100
                        / ctx.total_area.width as u32)
                        .clamp(20, 80) as u16;
                    state.config.preview_split_percent = preview_pct;
                    state.preview_mode = PreviewMode::Split(preview_pct);
                    return Ok(true);
                }

                if let Some(anchor) = self.dragging_selection_anchor {
                    if let (Some(viewport), Some(wrapped_layout)) =
                        (ctx.viewport, ctx.wrapped_layout)
                    {
                        if viewport.text_area.width == 0 || viewport.text_area.height == 0 {
                            return Ok(false);
                        }
                        let max_col = viewport
                            .text_area
                            .x
                            .saturating_add(viewport.text_area.width.saturating_sub(1));
                        let max_row = viewport
                            .text_area
                            .y
                            .saturating_add(viewport.text_area.height.saturating_sub(1));
                        let clamped_col = event.column.clamp(viewport.text_area.x, max_col);
                        let clamped_row = event.row.clamp(viewport.text_area.y, max_row);
                        let head = resolve_char_offset(
                            state,
                            viewport.text_area,
                            wrapped_layout,
                            clamped_col,
                            clamped_row,
                        );
                        state.buffers.active_mut().document.set_selection(
                            SelectionSet::from_ranges(vec![Selection::new(anchor, head)]),
                        );
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.dragging_selection_anchor = None;
                self.dragging_split_divider = false;
                Ok(false)
            }
            MouseEventKind::Down(MouseButton::Right) => {
                if let (Some(viewport), Some(wrapped_layout)) = (ctx.viewport, ctx.wrapped_layout) {
                    if rect_contains(viewport.text_area, event.column, event.row) {
                        let offset = resolve_char_offset(
                            state,
                            viewport.text_area,
                            wrapped_layout,
                            event.column,
                            event.row,
                        );
                        state
                            .buffers
                            .active_mut()
                            .document
                            .set_selection(SelectionSet::point(offset));
                        state.open_spell_quick_fix_at_cursor();
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            MouseEventKind::ScrollUp => {
                let max_visual_line = max_visual_line_index(state, ctx.wrapped_layout);
                let active = state.buffers.active_mut();
                active.scroll_line = active
                    .scroll_line
                    .saturating_sub(SCROLL_STEP_LINES)
                    .min(max_visual_line);
                Ok(true)
            }
            MouseEventKind::ScrollDown => {
                let max_visual_line = max_visual_line_index(state, ctx.wrapped_layout);
                let active = state.buffers.active_mut();
                active.scroll_line = active
                    .scroll_line
                    .saturating_add(SCROLL_STEP_LINES)
                    .min(max_visual_line);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn rect_contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x
        && col < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn resolve_char_offset(
    state: &EditorState,
    text_area: Rect,
    wrapped_layout: &WrappedLayout,
    col: u16,
    row: u16,
) -> usize {
    let active = state.buffers.active();
    let visual_row = active
        .scroll_line
        .saturating_add(row.saturating_sub(text_area.y) as usize);
    let visual_col = active
        .scroll_col
        .saturating_add(col.saturating_sub(text_area.x) as usize);
    let pos = wrapped_layout.visual_to_logical(visual_row, visual_col);
    active.document.position_to_char(pos)
}

fn max_visual_line_index(state: &EditorState, wrapped_layout: Option<&WrappedLayout>) -> usize {
    match wrapped_layout {
        Some(layout) => layout.len_visual_lines().saturating_sub(1),
        None => state
            .buffers
            .active()
            .document
            .len_lines()
            .saturating_sub(1),
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;
    use scriptui_core::{
        compute_wrapped_layout, AppConfig, EditorState, OverlayMode, PreviewMode, Selection,
    };

    use super::{MouseContext, MouseRouter};
    use crate::chrome::TabHitbox;
    use crate::layout_tree::compute_screen_regions;
    use crate::viewport::ViewportMetrics;

    fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn mouse_click_drag_scroll_tab_and_divider_routing() {
        let config = AppConfig {
            spell_enabled: true,
            ..AppConfig::default()
        };
        let mut state = EditorState::new(config).expect("editor state should initialize");
        state.insert_text("hello wrld\nline 2\nline 3\nline 4\nline 5\nline 6");
        state.buffers.new_buffer();
        state.insert_text("second buffer");
        state.buffers.switch_to(0);

        let total_area = Rect::new(0, 0, 100, 24);
        let regions = compute_screen_regions(total_area, state.preview_mode, state.overlay);
        let editor_area = regions.editor.expect("editor region should exist");
        let text_area = Rect::new(
            editor_area.x + 4,
            editor_area.y,
            editor_area.width.saturating_sub(4),
            editor_area.height,
        );
        let viewport = ViewportMetrics {
            area: editor_area,
            gutter_width: 4,
            text_area,
        };
        let layout = compute_wrapped_layout(
            &state.buffers.active().document,
            text_area.width as usize,
            false,
            4,
        );
        let hitboxes = vec![
            TabHitbox {
                rect: Rect::new(0, 0, 12, 1),
                buffer_index: 0,
            },
            TabHitbox {
                rect: Rect::new(12, 0, 12, 1),
                buffer_index: 1,
            },
        ];

        let mut router = MouseRouter::default();
        let ctx = MouseContext {
            regions,
            tab_hitboxes: &hitboxes,
            viewport: Some(viewport),
            wrapped_layout: Some(&layout),
            total_area,
        };

        // 1. Left click cursor placement at (row 0, col 2) -> char offset 2 ("he|llo wrld")
        let handled = router
            .handle_mouse(
                mouse_event(
                    MouseEventKind::Down(MouseButton::Left),
                    text_area.x + 2,
                    text_area.y,
                ),
                &mut state,
                &ctx,
            )
            .expect("left click should succeed");
        assert!(handled);
        assert_eq!(
            state.buffers.active().document.selection().primary(),
            Selection::point(2)
        );

        // 2. Left drag selection range creation from offset 2 to offset 5
        let handled = router
            .handle_mouse(
                mouse_event(
                    MouseEventKind::Drag(MouseButton::Left),
                    text_area.x + 5,
                    text_area.y,
                ),
                &mut state,
                &ctx,
            )
            .expect("left drag should succeed");
        assert!(handled);
        assert_eq!(
            state.buffers.active().document.selection().primary(),
            Selection::new(2, 5)
        );

        let handled = router
            .handle_mouse(
                mouse_event(
                    MouseEventKind::Up(MouseButton::Left),
                    text_area.x + 5,
                    text_area.y,
                ),
                &mut state,
                &ctx,
            )
            .expect("left up should succeed");
        assert!(!handled);

        // 3. Scroll wheel down then up
        let handled = router
            .handle_mouse(
                mouse_event(MouseEventKind::ScrollDown, text_area.x, text_area.y),
                &mut state,
                &ctx,
            )
            .expect("scroll down should succeed");
        assert!(handled);
        assert_eq!(state.buffers.active().scroll_line, 3);

        let handled = router
            .handle_mouse(
                mouse_event(MouseEventKind::ScrollUp, text_area.x, text_area.y),
                &mut state,
                &ctx,
            )
            .expect("scroll up should succeed");
        assert!(handled);
        assert_eq!(state.buffers.active().scroll_line, 0);

        // 4. Right-click on misspelled word "wrld" (col 7) opens OverlayMode::SpellQuickFix
        let handled = router
            .handle_mouse(
                mouse_event(
                    MouseEventKind::Down(MouseButton::Right),
                    text_area.x + 7,
                    text_area.y,
                ),
                &mut state,
                &ctx,
            )
            .expect("right click should succeed");
        assert!(handled);
        assert_eq!(state.overlay, OverlayMode::SpellQuickFix);
        assert!(state.spell_popup.is_some());

        // 5. Tab hitbox switch to buffer 1 and back to buffer 0
        let handled = router
            .handle_mouse(
                mouse_event(MouseEventKind::Down(MouseButton::Left), 14, 0),
                &mut state,
                &ctx,
            )
            .expect("tab click should succeed");
        assert!(handled);
        assert_eq!(state.buffers.active_index(), 1);

        let handled = router
            .handle_mouse(
                mouse_event(MouseEventKind::Down(MouseButton::Left), 2, 0),
                &mut state,
                &ctx,
            )
            .expect("tab click back should succeed");
        assert!(handled);
        assert_eq!(state.buffers.active_index(), 0);

        // 6. Preview edge-tab toggle (Collapsed -> Split)
        let edge_tab = regions
            .preview_edge_tab
            .expect("preview edge tab should exist when collapsed");
        assert_eq!(state.preview_mode, PreviewMode::Collapsed);
        let handled = router
            .handle_mouse(
                mouse_event(
                    MouseEventKind::Down(MouseButton::Left),
                    edge_tab.x,
                    edge_tab.y,
                ),
                &mut state,
                &ctx,
            )
            .expect("preview edge-tab click should succeed");
        assert!(handled);
        assert!(matches!(state.preview_mode, PreviewMode::Split(_)));

        // 7. Split divider drag updating PreviewMode::Split(pct)
        let split_regions = compute_screen_regions(total_area, state.preview_mode, state.overlay);
        let divider = split_regions
            .split_divider
            .expect("split divider should exist in split mode");
        let split_ctx = MouseContext {
            regions: split_regions,
            tab_hitboxes: &hitboxes,
            viewport: Some(viewport),
            wrapped_layout: Some(&layout),
            total_area,
        };

        let handled = router
            .handle_mouse(
                mouse_event(
                    MouseEventKind::Down(MouseButton::Left),
                    divider.x,
                    divider.y,
                ),
                &mut state,
                &split_ctx,
            )
            .expect("divider press should succeed");
        assert!(handled);

        let handled = router
            .handle_mouse(
                mouse_event(MouseEventKind::Drag(MouseButton::Left), 60, divider.y),
                &mut state,
                &split_ctx,
            )
            .expect("divider drag should succeed");
        assert!(handled);
        assert_eq!(state.preview_mode, PreviewMode::Split(40));
        assert_eq!(state.config.preview_split_percent, 40);
    }
}
