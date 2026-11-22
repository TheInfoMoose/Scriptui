use std::process::ExitCode;

use clap::Parser;
use scriptui_core::AppConfig;
use scriptui_tui::{execute_dict_import, Cli, Command, DictCommand, TerminalGuard, TuiApp};

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Some(Command::Dict {
        action: DictCommand::Import { locale, dic, aff },
    }) = cli.command
    {
        let dict_dir = AppConfig::default_config_dir().join("dicts");
        match execute_dict_import(&locale, &dic, &aff, &dict_dir) {
            Ok((dic_dest, aff_dest)) => {
                println!(
                    "Imported dictionary for '{locale}' to {} and {}",
                    dic_dest.display(),
                    aff_dest.display()
                );
                return ExitCode::SUCCESS;
            }
            Err(err) => {
                eprintln!("Failed to import dictionary for '{locale}': {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    let mut app = match TuiApp::from_cli(&cli) {
        Ok(app) => app,
        Err(err) => {
            eprintln!("Failed to initialize ScripTUI: {err}");
            return ExitCode::FAILURE;
        }
    };

    let mut guard = match TerminalGuard::enter() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to enter terminal UI mode: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = app.run_event_loop(&mut guard) {
        eprintln!("ScripTUI exited with error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
