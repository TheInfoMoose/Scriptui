pub mod app;
pub mod chrome;
pub mod cli;
pub mod layout_tree;
pub mod mouse;
pub mod preview_pane;
pub mod terminal;
pub mod viewport;

pub use app::{FrameState, TuiApp};
pub use chrome::{
    render_preview_edge_tab, render_split_divider, render_status_bar, render_tab_bar, TabHitbox,
};
pub use cli::{execute_dict_import, Cli, Command, DictCommand};
pub use layout_tree::{compute_screen_regions, rgb_to_color, segment_to_style, ScreenRegions};
pub use mouse::{MouseContext, MouseRouter};
pub use preview_pane::{compute_synced_preview_scroll, render_preview_pane, PreviewPaneState};
pub use terminal::{
    crossterm_key_to_chord, encode_osc52_clipboard, write_osc52_clipboard, TerminalGuard,
};
pub use viewport::{
    compute_gutter_width, ensure_cursor_visible, render_editor_viewport, ViewportMetrics,
};
