pub mod app;
pub mod chrome;
pub mod cli;
pub mod find_bar;
pub mod layout_tree;
pub mod modals;
pub mod mouse;
pub mod palette;
pub mod preview_pane;
pub mod terminal;
pub mod viewport;

pub use app::{FrameState, TuiApp};
pub use chrome::{
    render_preview_edge_tab, render_split_divider, render_status_bar, render_tab_bar, TabHitbox,
};
pub use cli::{execute_dict_import, Cli, Command, DictCommand};
pub use find_bar::{handle_find_bar_key, render_find_bar, FindBarState, FindFocus};
pub use layout_tree::{compute_screen_regions, rgb_to_color, segment_to_style, ScreenRegions};
pub use modals::{
    centered_rect, handle_confirm_modal_key, handle_file_prompt_key, handle_spell_quick_fix_key,
    render_confirm_modal, render_file_prompt, render_spell_quick_fix,
};
pub use mouse::{MouseContext, MouseRouter};
pub use palette::{
    build_palette_entries, discover_installed_locales, filter_palette_entries, handle_palette_key,
    render_command_palette, PaletteEntry, PaletteItemKind,
};
pub use preview_pane::{compute_synced_preview_scroll, render_preview_pane, PreviewPaneState};
pub use terminal::{
    crossterm_key_to_chord, encode_osc52_clipboard, write_osc52_clipboard, TerminalGuard,
};
pub use viewport::{
    compute_gutter_width, ensure_cursor_visible, render_editor_viewport, ViewportMetrics,
};
