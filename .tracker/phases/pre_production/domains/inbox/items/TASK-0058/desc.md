## Problem Description
In crates/scriptui-tui/tests/tui_integration.rs:1199-1208, cli_startup_opens_multiple_files_and_overrides_config passes locale: Some("en_US"), which equals AppConfig::default().spell_locale and cannot catch TuiApp::from_cli ignoring cli.locale. Exercise a non-default locale override.

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
