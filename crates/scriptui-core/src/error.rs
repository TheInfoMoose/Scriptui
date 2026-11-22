use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ScriptuiError {
    #[error("I/O error at {}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("TOML parse error in {}: {source}", path.display())]
    TomlParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid regex '{pattern}': {source}")]
    InvalidRegex {
        pattern: String,
        #[source]
        source: regex::Error,
    },
    #[error("unknown theme: {0}")]
    UnknownTheme(String),
    #[error("char offset {offset} out of bounds for length {len}")]
    OutOfBounds { offset: usize, len: usize },
    #[error("invalid locale identifier: {0}")]
    InvalidLocale(String),
    #[error("spell dictionary error for '{locale}': {reason}")]
    SpellDict { locale: String, reason: String },
}

pub type ScriptuiResult<T> = std::result::Result<T, ScriptuiError>;
