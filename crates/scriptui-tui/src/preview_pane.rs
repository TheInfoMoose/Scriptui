use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use scriptui_core::{EditorState, PreviewDocument};

use crate::layout_tree::{rgb_to_color, segment_to_style};

#[derive(Debug, Clone)]
struct CachedPreview {
    doc_id: u64,
    doc_revision: u64,
    theme_name: String,
    inner_width: usize,
    preview_doc: PreviewDocument,
}

#[derive(Debug, Clone)]
pub struct PreviewPaneState {
    pub scroll_line: usize,
    pub total_lines: usize,
    pub viewport_height: usize,
    pub sync_enabled: bool,
    cache: Option<CachedPreview>,
    rebuild_count: u64,
}

impl PreviewPaneState {
    pub fn rebuild_count(&self) -> u64 {
        self.rebuild_count
    }
}

impl Default for PreviewPaneState {
    fn default() -> Self {
        Self {
            scroll_line: 0,
            total_lines: 0,
            viewport_height: 0,
            sync_enabled: true,
            cache: None,
            rebuild_count: 0,
        }
    }
}

pub fn compute_synced_preview_scroll(
    preview_doc: &PreviewDocument,
    editor_top_source_line: usize,
    viewport_height: usize,
) -> usize {
    let target = preview_doc.source_line_to_preview_line(editor_top_source_line);
    let max_scroll = preview_doc
        .lines
        .len()
        .saturating_sub(viewport_height.max(1));
    target.min(max_scroll)
}

pub fn render_preview_pane(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    pane_state: &mut PreviewPaneState,
    editor_top_source_line: usize,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    let inner_height = area.height.saturating_sub(2).max(1) as usize;

    let active_doc = &state.buffers.active().document;
    let doc_id = active_doc.doc_id();
    let doc_revision = active_doc.revision();
    let theme_name = state.theme.name.as_str();

    let cache_hit = pane_state.cache.as_ref().is_some_and(|cached| {
        cached.doc_id == doc_id
            && cached.doc_revision == doc_revision
            && cached.theme_name == theme_name
            && cached.inner_width == inner_width
    });

    if !cache_hit {
        let preview_doc = state.build_active_preview(inner_width);
        pane_state.cache = Some(CachedPreview {
            doc_id,
            doc_revision,
            theme_name: theme_name.to_string(),
            inner_width,
            preview_doc,
        });
        pane_state.rebuild_count = pane_state.rebuild_count.saturating_add(1);
    }

    let Some(cached) = pane_state.cache.as_ref() else {
        return;
    };
    let preview_doc = &cached.preview_doc;
    pane_state.total_lines = preview_doc.lines.len();
    pane_state.viewport_height = inner_height;

    if pane_state.sync_enabled {
        pane_state.scroll_line =
            compute_synced_preview_scroll(preview_doc, editor_top_source_line, inner_height);
    } else {
        pane_state.scroll_line = pane_state
            .scroll_line
            .min(pane_state.total_lines.saturating_sub(inner_height.max(1)));
    }

    let palette = &state.theme.palette;
    let bg_color = rgb_to_color(palette.background);

    let start = pane_state.scroll_line.min(preview_doc.lines.len());
    let end = start
        .saturating_add(inner_height)
        .min(preview_doc.lines.len());

    let visible_lines: Vec<Line<'static>> = preview_doc.lines[start..end]
        .iter()
        .map(|preview_line| {
            let spans: Vec<Span<'static>> = preview_line
                .segments
                .iter()
                .map(|seg| {
                    let mut style = segment_to_style(seg);
                    if seg.bg.is_none() {
                        style = style.bg(bg_color);
                    }
                    Span::styled(seg.text.clone(), style)
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    let title = if pane_state.sync_enabled {
        " Preview [SYNC] "
    } else {
        " Preview "
    };
    let preview_style = Style::default()
        .fg(rgb_to_color(palette.foreground))
        .bg(bg_color);
    let border_style = Style::default()
        .fg(rgb_to_color(palette.border_default))
        .bg(bg_color);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
        .style(preview_style);
    let widget = Paragraph::new(visible_lines)
        .block(block)
        .style(preview_style);
    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use scriptui_core::{
        builtin_theme_names, resolve_theme, AppConfig, DocumentBuffer, EditorState,
    };

    use super::*;

    #[test]
    fn preview_pane_renders_styled_gfm_and_syncs_scroll() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        let md = concat!(
            "# Preview Heading\n",
            "\n",
            "- [x] Done\n",
            "\n",
            "> [!WARNING]\n",
            "> Watch out\n",
            "\n",
            "| Col A | Col B |\n",
            "| ----- | ----- |\n",
            "| 1     | 2     |\n",
            "\n",
            "## Tail Section\n",
            "\n",
            "More content below for scroll sync verification.\n",
        );
        state.buffers.active_mut().document = DocumentBuffer::from_str_content(md);

        let backend = TestBackend::new(60, 14);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut pane_state = PreviewPaneState::default();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_preview_pane(frame, area, &state, &mut pane_state, 0);
            })
            .expect("draw preview pane");

        let buf = terminal.backend().buffer();
        let heading_cell = &buf[(1, 1)];
        assert_eq!(
            heading_cell.fg,
            rgb_to_color(state.theme.palette.md_h1),
            "expected heading cell fg to match md_h1"
        );

        let mut rendered = String::new();
        for y in 0..14u16 {
            for x in 0..60u16 {
                rendered.push_str(buf[(x, y)].symbol());
            }
            rendered.push('\n');
        }

        assert!(
            rendered.contains("Preview [SYNC]"),
            "expected SYNC title in:\n{rendered}"
        );
        assert!(
            rendered.contains("[x]") && rendered.contains("Done"),
            "expected task list marker and label in:\n{rendered}"
        );
        assert!(
            rendered.contains("[WARNING]"),
            "expected [WARNING] alert label in:\n{rendered}"
        );
        assert!(
            rendered.contains('┌') && rendered.contains('│'),
            "expected table box-drawing characters in:\n{rendered}"
        );

        let preview_doc = state.build_active_preview(58);
        let scroll_top = compute_synced_preview_scroll(&preview_doc, 0, 6);
        let scroll_mid = compute_synced_preview_scroll(&preview_doc, 7, 6);
        let scroll_tail = compute_synced_preview_scroll(&preview_doc, 11, 6);
        assert!(
            scroll_mid > scroll_top,
            "expected scroll_mid ({scroll_mid}) > scroll_top ({scroll_top})"
        );
        assert!(
            scroll_tail >= scroll_mid,
            "expected scroll_tail ({scroll_tail}) >= scroll_mid ({scroll_mid})"
        );
    }

    #[test]
    fn render_preview_pane_caches_until_doc_theme_or_width_changes() {
        let mut state = EditorState::new(AppConfig::default()).expect("default editor state");
        let md = concat!(
            "# Cached Preview\n",
            "\n",
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\n",
            "Line 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\n",
            "\n",
            "## Bottom Section\n",
            "\n",
            "Trailing paragraph for scroll sync.\n",
        );
        state.buffers.active_mut().document = DocumentBuffer::from_str_content(md);

        let mut terminal = Terminal::new(TestBackend::new(60, 16)).expect("test terminal");
        let mut pane_state = PreviewPaneState::default();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 60, 16);
                render_preview_pane(frame, area, &state, &mut pane_state, 0);
            })
            .expect("first draw");
        assert_eq!(pane_state.rebuild_count(), 1);
        let initial_scroll = pane_state.scroll_line;

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 60, 16);
                render_preview_pane(frame, area, &state, &mut pane_state, 12);
            })
            .expect("second draw with different scroll line");
        assert_eq!(pane_state.rebuild_count(), 1);
        assert!(pane_state.scroll_line > initial_scroll);

        state.insert_text("!");
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 60, 16);
                render_preview_pane(frame, area, &state, &mut pane_state, 12);
            })
            .expect("draw after document edit");
        assert_eq!(pane_state.rebuild_count(), 2);

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 40, 16);
                render_preview_pane(frame, area, &state, &mut pane_state, 12);
            })
            .expect("draw after width change");
        assert_eq!(pane_state.rebuild_count(), 3);

        let alternate_theme_name = builtin_theme_names()
            .iter()
            .copied()
            .find(|name| *name != state.theme.name)
            .expect("at least one alternate builtin theme");
        state.theme =
            resolve_theme(alternate_theme_name, &[]).expect("resolve alternate builtin theme");
        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 40, 16);
                render_preview_pane(frame, area, &state, &mut pane_state, 12);
            })
            .expect("draw after theme change");
        assert_eq!(pane_state.rebuild_count(), 4);

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 40, 16);
                render_preview_pane(frame, area, &state, &mut pane_state, 0);
            })
            .expect("repeat draw at (40, 16)");
        assert_eq!(pane_state.rebuild_count(), 4);
    }
}
