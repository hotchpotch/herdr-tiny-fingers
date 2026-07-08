# Repository Guidelines

This repository contains `herdr-tiny-fingers`, a minimal Herdr plugin inspired
by `tmux-fingers`. The plugin opens an overlay pane, reads the previously
focused Herdr pane via `pane.read` with `source = "visible"`, overlays hint
labels on built-in regex matches, and copies the selected match by emitting an
OSC 52 clipboard sequence for Herdr to forward.

## Project Shape

- `herdr-plugin.toml` is the Herdr plugin manifest. Keep the plugin id,
  action command, pane command, and binary name in sync.
- `Cargo.toml` defines both the crate and the release binary as
  `herdr-tiny-fingers`.
- `src/patterns.rs` ports the built-in tmux-fingers regex patterns. Patterns
  with a named `match` capture must copy only that capture. Custom user
  patterns are loaded from `$HERDR_PLUGIN_CONFIG_DIR/config.toml`.
- `src/main.rs` is the TUI entry point and should stay thin; keep matching,
  hinting, clipboard, and socket logic in the library modules.

## Development Process

Use TDD for behavior changes. Add or update failing unit tests that describe
the intended behavior first, then implement the smallest change that makes
those tests pass. Keep tests focused on pure modules where possible before
checking the Herdr integration manually.

## Development Commands

Run these before committing:

```bash
cargo fmt -- --check
cargo test
cargo build --release --locked
cargo clippy --all-targets -- -D warnings
```

For local Herdr testing:

```bash
cargo build --release --locked
herdr plugin link .
herdr server reload-config
herdr plugin action invoke hotchpotch.herdr-tiny-fingers.open
```

Herdr keybindings live in the user's Herdr config, not in the plugin manifest.
The recommended binding is:

```toml
[[keys.command]]
key = "prefix+f"
type = "plugin_action"
command = "hotchpotch.herdr-tiny-fingers.open"
description = "fingers mode"
```

## Implementation Notes

- Prefer Herdr's socket API over shelling out to `herdr` from the running TUI.
- Clipboard writes should use OSC 52, not platform clipboard commands. Herdr
  already forwards OSC 52 clipboard writes from plugin panes to the foreground
  client.
- Keep the implementation intentionally small. This is not a full tmux-fingers
  port; alternate actions, multi-select, jump mode, and style configuration are
  out of scope unless explicitly requested.
- Matching may ignore pane line breaks for wrap-friendly patterns, but rendering
  must keep the original visible pane lines and avoid changing line widths.
- Do not commit `target/`, runtime logs, or local editor files.
