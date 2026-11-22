use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{ScriptuiError, ScriptuiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemePalette {
    // Chrome & editor base
    pub background: RgbColor,
    pub surface: RgbColor,
    pub foreground: RgbColor,
    pub muted: RgbColor,
    pub accent: RgbColor,
    pub border_default: RgbColor,
    pub border_focused: RgbColor,
    pub gutter_fg: RgbColor,
    pub gutter_active_fg: RgbColor,
    pub selection_bg: RgbColor,
    pub cursor_bg: RgbColor,
    pub cursor_fg: RgbColor,
    pub search_match_bg: RgbColor,
    pub search_active_bg: RgbColor,

    // GFM semantic differentials
    pub md_h1: RgbColor,
    pub md_h2: RgbColor,
    pub md_h3: RgbColor,
    pub md_emphasis: RgbColor,
    pub md_strong: RgbColor,
    pub md_strikethrough: RgbColor,
    pub md_link: RgbColor,
    pub md_code_inline: RgbColor,
    pub md_code_block: RgbColor,
    pub md_blockquote: RgbColor,
    pub md_list_marker: RgbColor,
    pub md_table_border: RgbColor,
    pub md_hr: RgbColor,
    pub md_alert_note: RgbColor,
    pub md_alert_tip: RgbColor,
    pub md_alert_warning: RgbColor,
    pub md_alert_caution: RgbColor,

    // Diagnostics
    pub spell_error: RgbColor,
    pub status_ok: RgbColor,
    pub status_warn: RgbColor,
    pub status_error: RgbColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub palette: ThemePalette,
}

pub const DEFAULT_CRT_AMBER_TOML: &str = r#"
name = "crt_amber"

[palette]
background = { r = 10, g = 10, b = 8 }
surface = { r = 20, g = 18, b = 14 }
foreground = { r = 255, g = 176, b = 0 }
muted = { r = 153, g = 107, b = 20 }
accent = { r = 255, g = 196, b = 61 }
border_default = { r = 120, g = 82, b = 16 }
border_focused = { r = 255, g = 209, b = 102 }
gutter_fg = { r = 132, g = 92, b = 18 }
gutter_active_fg = { r = 255, g = 209, b = 102 }
selection_bg = { r = 74, g = 52, b = 12 }
cursor_bg = { r = 255, g = 196, b = 61 }
cursor_fg = { r = 10, g = 10, b = 8 }
search_match_bg = { r = 92, g = 66, b = 14 }
search_active_bg = { r = 168, g = 112, b = 18 }

md_h1 = { r = 255, g = 218, b = 121 }
md_h2 = { r = 255, g = 196, b = 61 }
md_h3 = { r = 235, g = 164, b = 36 }
md_emphasis = { r = 255, g = 210, b = 104 }
md_strong = { r = 255, g = 228, b = 148 }
md_strikethrough = { r = 140, g = 98, b = 26 }
md_link = { r = 94, g = 212, b = 212 }
md_code_inline = { r = 94, g = 212, b = 212 }
md_code_block = { r = 132, g = 222, b = 198 }
md_blockquote = { r = 186, g = 132, b = 32 }
md_list_marker = { r = 255, g = 196, b = 61 }
md_table_border = { r = 120, g = 82, b = 16 }
md_hr = { r = 120, g = 82, b = 16 }
md_alert_note = { r = 94, g = 212, b = 212 }
md_alert_tip = { r = 64, g = 224, b = 112 }
md_alert_warning = { r = 255, g = 159, b = 28 }
md_alert_caution = { r = 255, g = 82, b = 82 }

spell_error = { r = 255, g = 82, b = 82 }
status_ok = { r = 64, g = 224, b = 112 }
status_warn = { r = 255, g = 159, b = 28 }
status_error = { r = 255, g = 82, b = 82 }
"#;

pub const PHOSPHOR_GREEN_TOML: &str = r#"
name = "phosphor_green"

[palette]
background = { r = 8, g = 12, b = 9 }
surface = { r = 14, g = 22, b = 16 }
foreground = { r = 64, g = 240, b = 116 }
muted = { r = 38, g = 138, b = 68 }
accent = { r = 118, g = 255, b = 158 }
border_default = { r = 30, g = 104, b = 52 }
border_focused = { r = 128, g = 255, b = 166 }
gutter_fg = { r = 36, g = 120, b = 60 }
gutter_active_fg = { r = 128, g = 255, b = 166 }
selection_bg = { r = 22, g = 68, b = 36 }
cursor_bg = { r = 118, g = 255, b = 158 }
cursor_fg = { r = 8, g = 12, b = 9 }
search_match_bg = { r = 28, g = 84, b = 44 }
search_active_bg = { r = 48, g = 146, b = 76 }

md_h1 = { r = 176, g = 255, b = 196 }
md_h2 = { r = 118, g = 255, b = 158 }
md_h3 = { r = 82, g = 226, b = 126 }
md_emphasis = { r = 142, g = 250, b = 172 }
md_strong = { r = 198, g = 255, b = 212 }
md_strikethrough = { r = 46, g = 124, b = 68 }
md_link = { r = 92, g = 228, b = 228 }
md_code_inline = { r = 92, g = 228, b = 228 }
md_code_block = { r = 128, g = 236, b = 204 }
md_blockquote = { r = 56, g = 172, b = 90 }
md_list_marker = { r = 118, g = 255, b = 158 }
md_table_border = { r = 30, g = 104, b = 52 }
md_hr = { r = 30, g = 104, b = 52 }
md_alert_note = { r = 92, g = 228, b = 228 }
md_alert_tip = { r = 118, g = 255, b = 158 }
md_alert_warning = { r = 255, g = 196, b = 61 }
md_alert_caution = { r = 255, g = 88, b = 88 }

spell_error = { r = 255, g = 88, b = 88 }
status_ok = { r = 64, g = 240, b = 116 }
status_warn = { r = 255, g = 196, b = 61 }
status_error = { r = 255, g = 88, b = 88 }
"#;

pub const ICE_CYAN_TOML: &str = r#"
name = "ice_cyan"

[palette]
background = { r = 8, g = 12, b = 16 }
surface = { r = 14, g = 22, b = 28 }
foreground = { r = 112, g = 228, b = 255 }
muted = { r = 58, g = 132, b = 156 }
accent = { r = 166, g = 242, b = 255 }
border_default = { r = 40, g = 96, b = 118 }
border_focused = { r = 166, g = 242, b = 255 }
gutter_fg = { r = 52, g = 116, b = 138 }
gutter_active_fg = { r = 166, g = 242, b = 255 }
selection_bg = { r = 24, g = 62, b = 82 }
cursor_bg = { r = 166, g = 242, b = 255 }
cursor_fg = { r = 8, g = 12, b = 16 }
search_match_bg = { r = 32, g = 78, b = 102 }
search_active_bg = { r = 54, g = 132, b = 168 }

md_h1 = { r = 210, g = 248, b = 255 }
md_h2 = { r = 166, g = 242, b = 255 }
md_h3 = { r = 124, g = 214, b = 242 }
md_emphasis = { r = 176, g = 236, b = 255 }
md_strong = { r = 224, g = 250, b = 255 }
md_strikethrough = { r = 64, g = 116, b = 136 }
md_link = { r = 130, g = 255, b = 214 }
md_code_inline = { r = 130, g = 255, b = 214 }
md_code_block = { r = 154, g = 236, b = 228 }
md_blockquote = { r = 82, g = 166, b = 192 }
md_list_marker = { r = 166, g = 242, b = 255 }
md_table_border = { r = 40, g = 96, b = 118 }
md_hr = { r = 40, g = 96, b = 118 }
md_alert_note = { r = 112, g = 228, b = 255 }
md_alert_tip = { r = 82, g = 236, b = 152 }
md_alert_warning = { r = 255, g = 190, b = 72 }
md_alert_caution = { r = 255, g = 92, b = 102 }

spell_error = { r = 255, g = 92, b = 102 }
status_ok = { r = 82, g = 236, b = 152 }
status_warn = { r = 255, g = 190, b = 72 }
status_error = { r = 255, g = 92, b = 102 }
"#;

pub const VECTOR_MAGENTA_TOML: &str = r#"
name = "vector_magenta"

[palette]
background = { r = 14, g = 9, b = 16 }
surface = { r = 24, g = 16, b = 28 }
foreground = { r = 242, g = 132, b = 255 }
muted = { r = 144, g = 74, b = 156 }
accent = { r = 255, g = 176, b = 255 }
border_default = { r = 108, g = 52, b = 118 }
border_focused = { r = 255, g = 186, b = 255 }
gutter_fg = { r = 128, g = 64, b = 140 }
gutter_active_fg = { r = 255, g = 186, b = 255 }
selection_bg = { r = 68, g = 30, b = 78 }
cursor_bg = { r = 255, g = 176, b = 255 }
cursor_fg = { r = 14, g = 9, b = 16 }
search_match_bg = { r = 84, g = 38, b = 96 }
search_active_bg = { r = 148, g = 64, b = 168 }

md_h1 = { r = 255, g = 212, b = 255 }
md_h2 = { r = 255, g = 176, b = 255 }
md_h3 = { r = 226, g = 128, b = 242 }
md_emphasis = { r = 250, g = 184, b = 255 }
md_strong = { r = 255, g = 228, b = 255 }
md_strikethrough = { r = 132, g = 72, b = 142 }
md_link = { r = 104, g = 232, b = 255 }
md_code_inline = { r = 104, g = 232, b = 255 }
md_code_block = { r = 144, g = 224, b = 248 }
md_blockquote = { r = 182, g = 98, b = 196 }
md_list_marker = { r = 255, g = 176, b = 255 }
md_table_border = { r = 108, g = 52, b = 118 }
md_hr = { r = 108, g = 52, b = 118 }
md_alert_note = { r = 104, g = 232, b = 255 }
md_alert_tip = { r = 92, g = 238, b = 148 }
md_alert_warning = { r = 255, g = 184, b = 64 }
md_alert_caution = { r = 255, g = 86, b = 108 }

spell_error = { r = 255, g = 86, b = 108 }
status_ok = { r = 92, g = 238, b = 148 }
status_warn = { r = 255, g = 184, b = 64 }
status_error = { r = 255, g = 86, b = 108 }
"#;

pub const HIGH_CONTRAST_MONO_TOML: &str = r#"
name = "high_contrast_mono"

[palette]
background = { r = 0, g = 0, b = 0 }
surface = { r = 18, g = 18, b = 18 }
foreground = { r = 245, g = 245, b = 245 }
muted = { r = 148, g = 148, b = 148 }
accent = { r = 255, g = 255, b = 255 }
border_default = { r = 112, g = 112, b = 112 }
border_focused = { r = 255, g = 255, b = 255 }
gutter_fg = { r = 132, g = 132, b = 132 }
gutter_active_fg = { r = 255, g = 255, b = 255 }
selection_bg = { r = 64, g = 64, b = 64 }
cursor_bg = { r = 255, g = 255, b = 255 }
cursor_fg = { r = 0, g = 0, b = 0 }
search_match_bg = { r = 82, g = 82, b = 82 }
search_active_bg = { r = 140, g = 140, b = 140 }

md_h1 = { r = 255, g = 255, b = 255 }
md_h2 = { r = 235, g = 235, b = 235 }
md_h3 = { r = 215, g = 215, b = 215 }
md_emphasis = { r = 230, g = 230, b = 230 }
md_strong = { r = 255, g = 255, b = 255 }
md_strikethrough = { r = 128, g = 128, b = 128 }
md_link = { r = 220, g = 220, b = 220 }
md_code_inline = { r = 210, g = 210, b = 210 }
md_code_block = { r = 200, g = 200, b = 200 }
md_blockquote = { r = 176, g = 176, b = 176 }
md_list_marker = { r = 255, g = 255, b = 255 }
md_table_border = { r = 112, g = 112, b = 112 }
md_hr = { r = 112, g = 112, b = 112 }
md_alert_note = { r = 220, g = 220, b = 220 }
md_alert_tip = { r = 200, g = 255, b = 200 }
md_alert_warning = { r = 255, g = 224, b = 128 }
md_alert_caution = { r = 255, g = 128, b = 128 }

spell_error = { r = 255, g = 128, b = 128 }
status_ok = { r = 200, g = 255, b = 200 }
status_warn = { r = 255, g = 224, b = 128 }
status_error = { r = 255, g = 128, b = 128 }
"#;

const BUILTIN_THEME_NAMES: &[&str] = &[
    "crt_amber",
    "phosphor_green",
    "ice_cyan",
    "vector_magenta",
    "high_contrast_mono",
];

pub fn builtin_theme_names() -> &'static [&'static str] {
    BUILTIN_THEME_NAMES
}

pub fn default_theme() -> ScriptuiResult<Theme> {
    parse_builtin_theme("crt_amber", DEFAULT_CRT_AMBER_TOML)
}

pub fn load_theme_file(path: &Path) -> ScriptuiResult<Theme> {
    let content = std::fs::read_to_string(path).map_err(|source| ScriptuiError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&content).map_err(|source| ScriptuiError::TomlParse {
        path: path.to_path_buf(),
        source,
    })
}

pub fn resolve_theme(name: &str, search_dirs: &[PathBuf]) -> ScriptuiResult<Theme> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(ScriptuiError::UnknownTheme(name.to_string()));
    }

    for dir in search_dirs {
        let candidate = dir.join(format!("{name}.toml"));
        if candidate.is_file() {
            return load_theme_file(&candidate);
        }
    }

    match name {
        "crt_amber" => parse_builtin_theme(name, DEFAULT_CRT_AMBER_TOML),
        "phosphor_green" => parse_builtin_theme(name, PHOSPHOR_GREEN_TOML),
        "ice_cyan" => parse_builtin_theme(name, ICE_CYAN_TOML),
        "vector_magenta" => parse_builtin_theme(name, VECTOR_MAGENTA_TOML),
        "high_contrast_mono" => parse_builtin_theme(name, HIGH_CONTRAST_MONO_TOML),
        _ => Err(ScriptuiError::UnknownTheme(name.to_string())),
    }
}

fn parse_builtin_theme(name: &str, raw: &str) -> ScriptuiResult<Theme> {
    toml::from_str(raw).map_err(|source| ScriptuiError::TomlParse {
        path: PathBuf::from(format!("<builtin:{name}>")),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtin_themes_parse_and_crt_amber_matches_suite() {
        for &name in builtin_theme_names() {
            let theme = resolve_theme(name, &[]).unwrap();
            assert_eq!(theme.name, name);
        }

        let default = default_theme().unwrap();
        assert_eq!(default.name, "crt_amber");
        assert_eq!(
            default.palette.foreground,
            RgbColor {
                r: 255,
                g: 176,
                b: 0
            }
        );
        assert_eq!(default.palette.background, RgbColor { r: 10, g: 10, b: 8 });
    }

    #[test]
    fn custom_toml_theme_overrides_builtin() {
        let dir = tempfile::tempdir().unwrap();
        let custom_toml = DEFAULT_CRT_AMBER_TOML.replace(
            "foreground = { r = 255, g = 176, b = 0 }",
            "foreground = { r = 12, g = 34, b = 56 }",
        );
        std::fs::write(dir.path().join("crt_amber.toml"), custom_toml).unwrap();

        let resolved = resolve_theme("crt_amber", &[dir.path().to_path_buf()]).unwrap();
        assert_eq!(
            resolved.palette.foreground,
            RgbColor {
                r: 12,
                g: 34,
                b: 56
            }
        );
    }

    #[test]
    fn resolve_theme_rejects_path_traversal() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("themes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(root.path().join("secret.toml"), DEFAULT_CRT_AMBER_TOML).unwrap();

        let err = resolve_theme("../secret", std::slice::from_ref(&dir)).unwrap_err();
        assert!(matches!(err, ScriptuiError::UnknownTheme(name) if name == "../secret"));

        for invalid in ["", ".", "..", "sub/theme", r"sub\theme", "theme.toml"] {
            let err = resolve_theme(invalid, std::slice::from_ref(&dir)).unwrap_err();
            assert!(
                matches!(err, ScriptuiError::UnknownTheme(_)),
                "expected UnknownTheme for {invalid:?}"
            );
        }
    }
}
