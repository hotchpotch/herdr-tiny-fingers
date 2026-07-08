use std::io::Write;

use anyhow::{Context, Result};
use base64::Engine;

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    write_osc52(std::io::stdout(), text.as_bytes())
}

pub fn write_osc52(mut writer: impl Write, bytes: &[u8]) -> Result<()> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    write!(writer, "\x1b]52;c;{encoded}\x07")
        .context("failed to write OSC 52 clipboard sequence")?;
    writer
        .flush()
        .context("failed to flush OSC 52 clipboard sequence")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_osc52_uses_bel_terminated_clipboard_sequence() {
        let mut out = Vec::new();
        write_osc52(&mut out, b"hello").unwrap();
        assert_eq!(out, b"\x1b]52;c;aGVsbG8=\x07");
    }
}
