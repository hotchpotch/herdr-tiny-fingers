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
    let config_dir = std::env::var_os("HERDR_PLUGIN_CONFIG_DIR");
    let pattern_settings = load_pattern_settings(config_dir.as_deref().map(Path::new))?;
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
    let mut app = App::from_text_with_theme(&text, &matcher, pattern_settings.theme);
    log_state(&format!(
        "start pane_id={pane_id} lines={} targets={} custom_patterns={custom_pattern_count} enabled_builtin_patterns={enabled_builtin_pattern_count}",
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
        copy_to_clipboard(&text)?;
    }
    Ok(())
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
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('C') => return Some('\u{3}'),
            _ => return None,
        }
    }
    match key.code {
        KeyCode::Esc => Some('\u{1b}'),
        KeyCode::Backspace => Some('\u{7f}'),
        KeyCode::Tab => Some('\t'),
        KeyCode::Char(ch) => Some(ch),
        _ => None,
    }
}
