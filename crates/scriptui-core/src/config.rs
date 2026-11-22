use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{ScriptuiError, ScriptuiResult};
use crate::indent::IndentConfig;

fn default_theme_name() -> String {
    "crt_amber".to_string()
}

fn default_locale() -> String {
    "en_US".to_string()
}

fn default_true() -> bool {
    true
}

fn default_preview_split() -> u16 {
    50
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_theme_name")]
    pub theme: String,
    #[serde(default = "default_locale")]
    pub spell_locale: String,
    #[serde(default = "default_true")]
    pub spell_enabled: bool,
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    #[serde(default = "default_true")]
    pub soft_wrap: bool,
    #[serde(default)]
    pub indent: IndentConfig,
    #[serde(default = "default_preview_split")]
    pub preview_split_percent: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
            spell_locale: default_locale(),
            spell_enabled: default_true(),
            line_numbers: default_true(),
            soft_wrap: default_true(),
            indent: IndentConfig::default(),
            preview_split_percent: default_preview_split(),
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &Path) -> ScriptuiResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|source| ScriptuiError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&content).map_err(|source| ScriptuiError::TomlParse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn load_or_default(path: &Path) -> ScriptuiResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        Self::load_from_file(path)
    }

    pub fn default_config_dir() -> PathBuf {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
            return PathBuf::from(xdg).join("scriptui");
        }
        if let Some(home) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
            return PathBuf::from(home).join(".config").join("scriptui");
        }
        PathBuf::from(".config/scriptui")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_and_partial_toml_override() {
        let cfg: AppConfig =
            toml::from_str("theme = \"ice_cyan\"\nspell_locale = \"en_GB\"\n").unwrap();
        assert_eq!(cfg.theme, "ice_cyan");
        assert_eq!(cfg.spell_locale, "en_GB");
        assert!(cfg.line_numbers);
        assert_eq!(cfg.preview_split_percent, 50);
        assert!(cfg.spell_enabled);
        assert!(cfg.soft_wrap);
        assert_eq!(cfg.indent, IndentConfig::default());
    }
}
