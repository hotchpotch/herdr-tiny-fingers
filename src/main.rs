use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use herdr_tiny_fingers::app::{App, Outcome};
use herdr_tiny_fingers::clipboard::copy_to_clipboard;
use herdr_tiny_fingers::config::load_pattern_settings;
use herdr_tiny_fingers::herdr_client::{context_focused_pane_id, SocketClient};
use herdr_tiny_fingers::patterns::Matcher;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log_state(&format!("error: {err:#}"));
            eprintln!("herdr-tiny-fingers: {err:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let socket_path = std::env::var_os("HERDR_SOCKET_PATH")
        .context("HERDR_SOCKET_PATH is not set; open this through the Herdr plugin action")?;
    let pane_id = context_focused_pane_id()
        .context("HERDR_PLUGIN_CONTEXT_JSON did not include focused_pane_id")?;
    let mut client = SocketClient::connect(Path::new(&socket_path))?;
    let text = client.read_visible_pane(&pane_id)?;
    let pane_width = match client.visible_pane_width(&pane_id) {
        Ok(width) => Some(visible_wrap_width(width)),
        Err(err) => {
            log_state(&format!("pane_width_unavailable: {err:#}"));
            None
        }
    };
    let config_dir = std::env::var_os("HERDR_PLUGIN_CONFIG_DIR");
    let pattern_settings = load_pattern_settings(config_dir.as_deref().map(Path::new))?;
    let copy_toast = pattern_settings.copy_toast;
    let direct_paste = pattern_settings.direct_paste;
    let custom_pattern_count = pattern_settings.custom_patterns.len();
    let enabled_builtin_pattern_count = pattern_settings
        .enabled_builtin_patterns
        .as_ref()
        .map(Vec::len)
        .unwrap_or(0);
    let matcher = Matcher::with_builtin_patterns(
        pattern_settings.enabled_builtin_patterns.as_deref(),
        pattern_settings.custom_patterns,
    )?;
    let mut app = match pane_width {
        Some(width) => {
            App::from_text_with_theme_and_pane_width(&text, &matcher, pattern_settings.theme, width)
        }
        None => App::from_text_with_theme(&text, &matcher, pattern_settings.theme),
    };
    log_state(&format!(
        "start pane_id={pane_id} lines={} targets={} pane_width={pane_width:?} custom_patterns={custom_pattern_count} enabled_builtin_patterns={enabled_builtin_pattern_count}",
        app.lines.len(),
        app.targets.len()
    ));

    let outcome = {
        let _restore = TerminalRestore;
        let mut terminal = ratatui::init();
        loop {
            terminal.draw(|frame| herdr_tiny_fingers::ui::draw(frame, &app))?;
            match event::read()? {
                Event::Key(key) => {
                    if let Some(ch) = key_to_char(key) {
                        match app.handle_char(ch) {
                            Outcome::Continue => {}
                            other => break other,
                        }
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    };
    log_state(&format!("outcome={outcome:?}"));

    if let Outcome::Copy(text) = outcome {
        if direct_paste {
            client.send_text(&pane_id, &text)?;
        } else {
            copy_to_clipboard(&text)?;
            if copy_toast {
                match client.show_notification(&copy_notification_title(&text)) {
                    Ok(result) if !result.shown => {
                        log_state(&format!("notification_not_shown reason={}", result.reason));
                    }
                    Ok(_) => {}
                    Err(err) => {
                        log_state(&format!("notification_error: {err:#}"));
                    }
                }
            }
        }
    }
    Ok(())
}

fn copy_notification_title(text: &str) -> String {
    let mut chars = text.chars();
    let mut preview = chars.by_ref().take(15).collect::<String>();
    if chars.next().is_some() {
        preview.push_str("...");
    }
    format!("Copied: {preview}")
}

fn visible_wrap_width(layout_width: usize) -> usize {
    // Herdr's pane rectangle includes the terminal's wrap-pending right edge,
    // while `pane.read` moves that character to the following visible row.
    if layout_width > 1 {
        layout_width - 1
    } else {
        layout_width
    }
}

fn log_state(message: &str) {
    let Some(dir) = std::env::var_os("HERDR_PLUGIN_STATE_DIR") else {
        return;
    };
    let path = Path::new(&dir).join("herdr-tiny-fingers.log");
    let line = format!("{message}\n");
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
}

struct TerminalRestore;

impl Drop for TerminalRestore {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn key_to_char(key: KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Tab | KeyCode::BackTab => return Some('\t'),
        KeyCode::Char('\t') => return Some('\t'),
        _ => {}
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('C') => return Some('\u{3}'),
            KeyCode::Char('i') | KeyCode::Char('I') => return Some('\t'),
            _ => return None,
        }
    }
    match key.code {
        KeyCode::Esc => Some('\u{1b}'),
        KeyCode::Backspace => Some('\u{7f}'),
        KeyCode::Char(ch) => Some(ch),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_notification_title_includes_short_text() {
        assert_eq!(copy_notification_title("README.md"), "Copied: README.md");
    }

    #[test]
    fn copy_notification_title_does_not_truncate_fifteen_characters() {
        assert_eq!(
            copy_notification_title("123456789012345"),
            "Copied: 123456789012345"
        );
    }

    #[test]
    fn copy_notification_title_truncates_after_fifteen_characters() {
        assert_eq!(
            copy_notification_title("1234567890123456"),
            "Copied: 123456789012345..."
        );
    }

    #[test]
    fn copy_notification_title_truncates_by_characters() {
        assert_eq!(
            copy_notification_title("あいうえおかきくけこさしすせそた"),
            "Copied: あいうえおかきくけこさしすせそ..."
        );
    }

    #[test]
    fn visible_wrap_width_excludes_the_terminal_right_edge() {
        assert_eq!(visible_wrap_width(118), 117);
        assert_eq!(visible_wrap_width(1), 1);
    }

    #[test]
    fn converts_tab_key_to_tab_character() {
        assert_eq!(
            key_to_char(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            Some('\t')
        );
    }

    #[test]
    fn converts_control_i_to_tab_character() {
        assert_eq!(
            key_to_char(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::CONTROL)),
            Some('\t')
        );
    }
}
