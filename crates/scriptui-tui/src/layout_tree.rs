use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use scriptui_core::{OverlayMode, PreviewMode, RgbColor, StyledSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRegions {
    pub tab_bar: Rect,
    pub editor: Option<Rect>,
    pub preview_edge_tab: Option<Rect>,
    pub split_divider: Option<Rect>,
    pub preview: Option<Rect>,
    pub find_bar: Option<Rect>,
    pub status_bar: Rect,
}

pub fn compute_screen_regions(
    area: Rect,
    preview_mode: PreviewMode,
    overlay: OverlayMode,
) -> ScreenRegions {
    let find_height = match overlay {
        OverlayMode::Find => 1,
        OverlayMode::Replace => 2,
        _ => 0,
    };

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(find_height),
            Constraint::Length(1),
        ])
        .split(area);

    let tab_bar = vertical[0];
    let main_area = vertical[1];
    let find_bar = if find_height > 0 {
        Some(vertical[2])
    } else {
        None
    };
    let status_bar = vertical[3];

    let (editor, preview_edge_tab, split_divider, preview) = match preview_mode {
        PreviewMode::Collapsed => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(main_area);
            (Some(chunks[0]), Some(chunks[1]), None, None)
        }
        PreviewMode::Split(pct) => {
            let p = pct.clamp(20, 80);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(100 - p),
                    Constraint::Length(1),
                    Constraint::Percentage(p),
                ])
                .split(main_area);
            (Some(chunks[0]), None, Some(chunks[1]), Some(chunks[2]))
        }
        PreviewMode::Expanded => (None, None, None, Some(main_area)),
    };

    ScreenRegions {
        tab_bar,
        editor,
        preview_edge_tab,
        split_divider,
        preview,
        find_bar,
        status_bar,
    }
}

pub fn rgb_to_color(c: RgbColor) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

pub fn segment_to_style(seg: &StyledSegment) -> Style {
    let mut style = Style::default().fg(rgb_to_color(seg.fg));
    if let Some(bg) = seg.bg {
        style = style.bg(rgb_to_color(bg));
    }

    let mut modifiers = Modifier::empty();
    if seg.flags.bold {
        modifiers |= Modifier::BOLD;
    }
    if seg.flags.italic {
        modifiers |= Modifier::ITALIC;
    }
    if seg.flags.underline {
        modifiers |= Modifier::UNDERLINED;
    }
    if seg.flags.dim {
        modifiers |= Modifier::DIM;
    }
    if seg.flags.strikethrough {
        modifiers |= Modifier::CROSSED_OUT;
    }

    if !modifiers.is_empty() {
        style = style.add_modifier(modifiers);
    }

    style
}
