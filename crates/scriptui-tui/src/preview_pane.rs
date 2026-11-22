use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use scriptui_core::{EditorState, PreviewDocument};

use crate::layout_tree::{rgb_to_color, segment_to_style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewPaneState {
    pub scroll_line: usize,
    pub total_lines: usize,
    pub sync_enabled: bool,
}

impl Default for PreviewPaneState {
    fn default() -> Self {
        Self {
            scroll_line: 0,
            total_lines: 0,
            sync_enabled: true,
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

    let preview_doc = state.build_active_preview(inner_width);
    pane_state.total_lines = preview_doc.lines.len();

    if pane_state.sync_enabled {
        pane_state.scroll_line =
            compute_synced_preview_scroll(&preview_doc, editor_top_source_line, inner_height);
    } else {
        pane_state.scroll_line = pane_state
            .scroll_line
            .min(pane_state.total_lines.saturating_sub(1));
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
    use scriptui_core::{AppConfig, DocumentBuffer, EditorState};

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
}
