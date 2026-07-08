# Repository Guidelines

This repository contains `herdr-tiny-fingers`, a minimal Herdr plugin inspired
by `tmux-fingers`. The plugin opens an overlay pane, reads the previously
focused Herdr pane via `pane.read` with `source = "visible"`, overlays hint
labels on built-in and user-configured regex matches, and copies selected
matches by emitting an OSC 52 clipboard sequence for Herdr to forward.

## Project Shape

- `herdr-plugin.toml` is the Herdr plugin manifest. Keep the plugin id,
  action command, pane command, and binary name in sync.
- `Cargo.toml` defines both the crate and the release binary as
  `herdr-tiny-fingers`.
- `src/patterns.rs` ports the built-in tmux-fingers regex patterns. Patterns
  with a named `match` capture must copy only that capture. Custom user
  patterns and enabled built-in pattern lists are loaded from
  `$HERDR_PLUGIN_CONFIG_DIR/config.toml`.
- `src/theme.rs` owns the default TUI theme and user-configurable color parsing.
  Keep color names and `#RRGGBB` handling covered by unit tests.
- `src/app.rs` owns interaction state, including hint input and multi-select.
  `Tab` enters multi-select mode and a second `Tab` copies selected matches.
- `src/main.rs` is the TUI entry point and should stay thin; keep matching,
  hinting, theming, clipboard, and socket logic in the library modules.

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

Use this command to find the user-editable plugin config directory:

```bash
herdr plugin config-dir hotchpotch.herdr-tiny-fingers
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
  port; alternate actions and jump mode are out of scope unless explicitly
  requested.
- Support user configuration through `$HERDR_PLUGIN_CONFIG_DIR/config.toml`;
  avoid reading or modifying the host Herdr config except when documenting or
  manually testing the user's keybinding.
- The active plugin id is `hotchpotch.herdr-tiny-fingers`. If local testing
  appears to run stale code, check for an older linked plugin id such as
  `hotchpotch.herdr-finger` and verify the user's `prefix+f` binding points at
  `hotchpotch.herdr-tiny-fingers.open`.
- Matching may ignore pane line breaks for wrap-friendly patterns, but rendering
  must keep the original visible pane lines and avoid changing line widths.
- Do not commit `target/`, runtime logs, or local editor files.
