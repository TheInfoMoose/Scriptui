use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use scriptui_core::ScriptuiResult;

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(
    name = "scriptui",
    version,
    about = "Retro-futuristic Ratatui text editor and Markdown IDE"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    #[arg(long)]
    pub theme: Option<String>,
    #[arg(long)]
    pub locale: Option<String>,
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    Dict {
        #[command(subcommand)]
        action: DictCommand,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum DictCommand {
    Import {
        locale: String,
        dic: PathBuf,
        aff: PathBuf,
    },
}

pub fn execute_dict_import(
    locale: &str,
    dic: &Path,
    aff: &Path,
    dict_dir: &Path,
) -> ScriptuiResult<(PathBuf, PathBuf)> {
    scriptui_core::import_locale_files(locale, dic, aff, dict_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_files_theme_and_locale() {
        let cli = Cli::try_parse_from([
            "scriptui",
            "--theme",
            "phosphor_green",
            "--locale",
            "en_GB",
            "notes.md",
            "todo.md",
        ])
        .unwrap();

        assert_eq!(cli.command, None);
        assert_eq!(cli.theme.as_deref(), Some("phosphor_green"));
        assert_eq!(cli.locale.as_deref(), Some("en_GB"));
        assert_eq!(
            cli.files,
            vec![PathBuf::from("notes.md"), PathBuf::from("todo.md")]
        );
    }

    #[test]
    fn cli_parses_dict_import_subcommand_and_executes() {
        let cli = Cli::try_parse_from([
            "scriptui",
            "dict",
            "import",
            "fr_FR",
            "/tmp/fr.dic",
            "/tmp/fr.aff",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Some(Command::Dict {
                action: DictCommand::Import {
                    locale: "fr_FR".to_string(),
                    dic: PathBuf::from("/tmp/fr.dic"),
                    aff: PathBuf::from("/tmp/fr.aff"),
                },
            })
        );
        assert_eq!(cli.theme, None);
        assert_eq!(cli.locale, None);
        assert!(cli.files.is_empty());

        let src_dir = tempfile::tempdir().unwrap();
        let dic_src = src_dir.path().join("fr.dic");
        let aff_src = src_dir.path().join("fr.aff");
        std::fs::write(&dic_src, "1\nbonjour\n").unwrap();
        std::fs::write(&aff_src, "SET UTF-8\n").unwrap();

        let dict_dir = tempfile::tempdir().unwrap();
        let (dic_dest, aff_dest) =
            execute_dict_import("fr_FR", &dic_src, &aff_src, dict_dir.path()).unwrap();

        assert!(dic_dest.exists());
        assert!(aff_dest.exists());
        assert_eq!(dic_dest, dict_dir.path().join("fr_FR.dic"));
        assert_eq!(aff_dest, dict_dir.path().join("fr_FR.aff"));
    }
}
