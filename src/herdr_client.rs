use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

#[derive(Debug)]
pub struct SocketClient {
    socket_path: PathBuf,
    next_id: u64,
}

impl SocketClient {
    pub fn connect(socket_path: &Path) -> Result<Self> {
        UnixStream::connect(socket_path).with_context(|| {
            format!(
                "cannot connect to Herdr API socket at {}",
                socket_path.display()
            )
        })?;
        Ok(Self {
            socket_path: socket_path.to_path_buf(),
            next_id: 1,
        })
    }

    pub fn read_visible_pane(&mut self, pane_id: &str) -> Result<String> {
        let result = self.call(
            "pane.read",
            json!({
                "pane_id": pane_id,
                "source": "visible",
                "format": "text",
                "strip_ansi": true
            }),
        )?;
        let actual_type = result["type"].as_str().unwrap_or("<missing>");
        if actual_type != "pane_read" {
            bail!("expected pane_read result, got {actual_type}");
        }
        Ok(result["read"]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.to_string();
        self.next_id += 1;

        let mut reader = BufReader::new(UnixStream::connect(&self.socket_path)?);
        let request = json!({"id": id, "method": method, "params": params}).to_string();
        let stream = reader.get_mut();
        stream.write_all(request.as_bytes())?;
        stream.write_all(b"\n")?;

        let mut response = String::new();
        if reader.read_line(&mut response)? == 0 {
            bail!("Herdr closed the API connection before answering {method}");
        }

        let envelope: Value = serde_json::from_str(&response)
            .with_context(|| format!("Herdr returned invalid JSON for {method}"))?;
        if let Some(error) = envelope.get("error") {
            let code = error["code"].as_str().unwrap_or("unknown_error");
            let message = error["message"].as_str().unwrap_or("no message");
            bail!("Herdr API error {code}: {message}");
        }
        if envelope["id"].as_str() != Some(&id) {
            bail!("Herdr response id did not match request id {id}");
        }
        envelope
            .get("result")
            .cloned()
            .context("Herdr response has neither result nor error")
    }
}

pub fn context_focused_pane_id() -> Option<String> {
    let context = std::env::var("HERDR_PLUGIN_CONTEXT_JSON").ok()?;
    let context: Value = serde_json::from_str(&context).ok()?;
    context
        .get("focused_pane_id")?
        .as_str()
        .map(ToString::to_string)
}
