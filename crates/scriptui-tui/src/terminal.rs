use std::io::{self, Write};
use std::panic;

use crossterm::cursor::Show;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    KeyCode as CrosstermKeyCode, KeyEvent, KeyModifiers as CrosstermKeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scriptui_core::{KeyChord, KeyCode, KeyModifiers};

pub type CrosstermTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub struct TerminalGuard {
    terminal: CrosstermTerminal,
}

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            restore_terminal_best_effort();
            previous_hook(panic_info);
        }));

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(err) = execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        ) {
            restore_terminal_best_effort();
            return Err(err);
        }

        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(err) => {
                restore_terminal_best_effort();
                return Err(err);
            }
        };

        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut CrosstermTerminal {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal_best_effort();
    }
}

fn restore_terminal_best_effort() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste,
        Show
    );
}

pub fn crossterm_key_to_chord(event: KeyEvent) -> Option<KeyChord> {
    let code = match event.code {
        CrosstermKeyCode::Char(c) => KeyCode::Char(c),
        CrosstermKeyCode::Enter => KeyCode::Enter,
        CrosstermKeyCode::Tab => KeyCode::Tab,
        CrosstermKeyCode::BackTab => KeyCode::BackTab,
        CrosstermKeyCode::Backspace => KeyCode::Backspace,
        CrosstermKeyCode::Delete => KeyCode::Delete,
        CrosstermKeyCode::Esc => KeyCode::Escape,
        CrosstermKeyCode::Left => KeyCode::Left,
        CrosstermKeyCode::Right => KeyCode::Right,
        CrosstermKeyCode::Up => KeyCode::Up,
        CrosstermKeyCode::Down => KeyCode::Down,
        CrosstermKeyCode::Home => KeyCode::Home,
        CrosstermKeyCode::End => KeyCode::End,
        CrosstermKeyCode::PageUp => KeyCode::PageUp,
        CrosstermKeyCode::PageDown => KeyCode::PageDown,
        CrosstermKeyCode::F(n) => KeyCode::F(n),
        _ => return None,
    };

    let modifiers = KeyModifiers {
        ctrl: event.modifiers.contains(CrosstermKeyModifiers::CONTROL),
        alt: event.modifiers.contains(CrosstermKeyModifiers::ALT),
        shift: event.modifiers.contains(CrosstermKeyModifiers::SHIFT),
    };

    Some(KeyChord::new(code, modifiers).normalized())
}

const BASE64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn encode_base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        match *chunk {
            [b0, b1, b2] => {
                out.push(BASE64_TABLE[(b0 >> 2) as usize] as char);
                out.push(BASE64_TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
                out.push(BASE64_TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
                out.push(BASE64_TABLE[(b2 & 0x3f) as usize] as char);
            }
            [b0, b1] => {
                out.push(BASE64_TABLE[(b0 >> 2) as usize] as char);
                out.push(BASE64_TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
                out.push(BASE64_TABLE[((b1 & 0x0f) << 2) as usize] as char);
                out.push('=');
            }
            [b0] => {
                out.push(BASE64_TABLE[(b0 >> 2) as usize] as char);
                out.push(BASE64_TABLE[((b0 & 0x03) << 4) as usize] as char);
                out.push('=');
                out.push('=');
            }
            _ => {}
        }
    }
    out
}

pub fn encode_osc52_clipboard(text: &str) -> String {
    let encoded = encode_base64(text.as_bytes());
    format!("\x1b]52;c;{encoded}\x07")
}

pub fn write_osc52_clipboard<W: Write>(writer: &mut W, text: &str) -> io::Result<()> {
    let sequence = encode_osc52_clipboard(text);
    writer.write_all(sequence.as_bytes())?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossterm_key_event_maps_to_normalized_key_chord() {
        let ctrl_s = KeyEvent::new(CrosstermKeyCode::Char('S'), CrosstermKeyModifiers::CONTROL);
        assert_eq!(
            crossterm_key_to_chord(ctrl_s),
            Some(KeyChord::new(KeyCode::Char('s'), KeyModifiers::CTRL))
        );

        let shift_f3 = KeyEvent::new(CrosstermKeyCode::F(3), CrosstermKeyModifiers::SHIFT);
        assert_eq!(
            crossterm_key_to_chord(shift_f3),
            Some(KeyChord::new(KeyCode::F(3), KeyModifiers::SHIFT))
        );

        let back_tab = KeyEvent::new(CrosstermKeyCode::BackTab, CrosstermKeyModifiers::NONE);
        assert_eq!(
            crossterm_key_to_chord(back_tab),
            Some(KeyChord::new(KeyCode::BackTab, KeyModifiers::NONE))
        );
    }

    #[test]
    fn osc52_clipboard_encodes_rfc4648_base64_sequence() {
        assert_eq!(encode_osc52_clipboard("hello"), "\x1b]52;c;aGVsbG8=\x07");
        assert_eq!(encode_osc52_clipboard("a"), "\x1b]52;c;YQ==\x07");

        let mut buffer = Vec::new();
        write_osc52_clipboard(&mut buffer, "hello").expect("write to Vec<u8> should succeed");
        assert_eq!(buffer, b"\x1b]52;c;aGVsbG8=\x07");
    }
}
