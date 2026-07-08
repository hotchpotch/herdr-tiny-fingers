# herdr-tiny-fingers

`herdr-tiny-fingers` is a
[`tmux-fingers`](https://github.com/Morantron/tmux-fingers)-like Herdr plugin.
It finds useful text already visible in the focused Herdr pane, puts short hint
labels on each match, and lets you copy a match by typing its hint.

This is intentionally not a full `tmux-fingers` port. It implements the subset
of `tmux-fingers` that is sufficient for my Herdr workflow.

https://github.com/user-attachments/assets/b8e78007-4dda-45f3-ab40-ca82647c7614


## Features

- Detects file paths, Git SHAs, numbers, IP addresses, UUIDs, URLs, Kubernetes
  names, Git status paths, and diff paths.



- Opens as an overlay pane on top of the focused Herdr pane.
- Copies selected text to the clipboard.
- Supports multi-select with `Tab`: enter multi mode, select hints, then press
  `Tab` again to copy selections joined by newlines.
- Supports custom Rust regex patterns in the plugin config.
- Ignores pane line breaks for wrap-friendly matches, so wrapped URLs can copy
  without the inserted terminal newline.
- Supports configurable hint, match, selected-match, and status colors.
- Can show an optional Herdr toast with the copied text, truncated after 15
  characters.

## Quickstart

Install the plugin from GitHub:

```bash
herdr plugin install hotchpotch/herdr-tiny-fingers
```

Add a keybinding to your Herdr config:

```toml
[[keys.command]]
key = "prefix+f"
type = "plugin_action"
command = "hotchpotch.herdr-tiny-fingers.open"
description = "fingers mode"
```

Reload Herdr:

```bash
herdr server reload-config
```

Then press `prefix+f` in Herdr to open fingers mode. Type a hint to copy the
matched text.

## Copy Toast

Herdr notification toasts are controlled by the user's Herdr config, not the
plugin config. To show the copied text after selection, enable Herdr's in-app
toast delivery:

```toml
[ui.toast]
delivery = "herdr"
```

Reload the Herdr config after changing it:

```bash
herdr server reload-config
```

If `delivery` is omitted or set to `off`, copying still works but the copied
text toast is not shown.

## Patterns

The default enabled patterns are ported from tmux-fingers built-ins:

- `ip`
- `uuid`
- `sha`
- `digit`
- `url`
- `path`
- `hex`
- `kubernetes`
- `kubernetes-pod`
- `git-status`
- `git-status-branch`
- `diff`

Patterns with a `match` capture copy only that capture, matching tmux-fingers'
behavior for git status and diff output.

Pane line breaks are ignored for wrap-friendly patterns, so a URL or IP address
split by terminal wrapping can still be selected and copied without the inserted
line break.

## Custom Patterns

Add Rust regular expressions to the plugin config file:

```bash
herdr plugin config-dir hotchpotch.herdr-tiny-fingers
```

```toml
# $HERDR_PLUGIN_CONFIG_DIR/config.toml
enabled_builtin_patterns = ["url", "sha", "git-status", "git-status-branch"]

[style]
hint_fg = "black"
hint_bg = "yellow"
match_fg = "yellow"
selected_hint_fg = "white"
selected_hint_bg = "magenta"
selected_match_fg = "white"
selected_match_bg = "magenta"
status_fg = "black"
status_bg = "gray"

[[patterns]]
name = "ticket"
regex = "PROJ-[0-9]+"

[[patterns]]
name = "env"
regex = "env=(?P<match>[a-z0-9_-]+)"
```

Multiple `[[patterns]]` entries are supported. If a pattern defines a named
`match` capture, only that capture is copied; otherwise the full regex match is
copied. Custom patterns also ignore pane line breaks while matching.

Omit `enabled_builtin_patterns` to enable all built-in patterns. Set it to a
list of built-in pattern names to reduce noisy matches on busy panes.

Style colors are optional. Supported names are `black`, `red`, `green`,
`yellow`, `blue`, `magenta`, `cyan`, `gray`, `dark-gray`, `light-red`,
`light-green`, `light-yellow`, `light-blue`, `light-magenta`, `light-cyan`,
and `white`; `#RRGGBB` values are also accepted. `match_bg` can be omitted to
highlight matches with foreground color only.

## Development

```bash
cargo test
cargo build --release --locked
herdr plugin link .
```

## Technical Notes

The plugin opens an overlay pane, reads the previously focused pane through
Herdr's socket API with `pane.read` and `source = "visible"`, redraws the
visible text, and overlays hint labels on detected matches.

Clipboard writes use OSC 52 so Herdr can forward the copied text from the
plugin pane to the foreground client. Copy notifications use Herdr's
`notification.show` socket API and follow the user's `[ui.toast]` settings.

## Acknowledgements

This project is inspired by and references the ideas and implementation of
[tmux-fingers](https://github.com/Morantron/tmux-fingers). Many thanks to its
author and contributors for the excellent original tool.

## Author

Yuichi Tateno ([@hotchpotch](https://github.com/hotchpotch))

## License

MIT
