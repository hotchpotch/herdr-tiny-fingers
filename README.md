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
- Copies selected text to the clipboard, or sends it directly to the focused
  pane when direct paste is enabled.
- Supports multi-select with `Tab`: enter multi mode, select hints, then press
  `Tab` again to copy selections joined by newlines. When `direct_paste` is
  enabled, multi-select values are joined with spaces and line breaks are never
  sent to the original pane.
- Supports custom Rust regex patterns in the plugin config.
- Ignores terminal-wrap line breaks for wrap-friendly matches, including long
  paths, so they copy without the inserted terminal newline.
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

## Direct Paste

Direct paste is disabled by default. Set `direct_paste = true` in the plugin
config to send the selected text directly to the original focused pane instead
of writing it to the clipboard. This sends literal text without pressing Enter;
`copy_toast` is ignored in this mode. For multi-select, selected values are
joined with a single space, and carriage returns or line feeds are replaced with
spaces so they cannot submit text to the target shell.

## Copy Toast

Copy toasts are disabled by default. To show the copied text after selection,
enable `copy_toast` in the plugin config:

```bash
herdr plugin config-dir hotchpotch.herdr-tiny-fingers
```

```toml
# $HERDR_PLUGIN_CONFIG_DIR/config.toml
copy_toast = true
direct_paste = false
```

The toast is sent through Herdr's notification system, so Herdr's in-app toast
delivery must also be enabled in the user's Herdr config:

```toml
[ui.toast]
delivery = "herdr"
```

Reload the Herdr config after changing it:

```bash
herdr server reload-config
```

If `copy_toast` is omitted or set to `false`, copying works without showing a
plugin toast. If Herdr's `delivery` is omitted or set to `off`, the plugin can
request a toast but Herdr will not display it.

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

Terminal-wrap line breaks are ignored for wrap-friendly patterns. The plugin
uses the focused pane's actual display width, so URLs, IP addresses, and long
paths split by terminal wrapping can still be selected and copied without the
inserted line break. An indented file name after a wrapped directory path is
also joined by the built-in `path` pattern. URL and other patterns retain that
indentation as a boundary, so an indented `foo.html` is not accidentally added
to a preceding URL.

## Custom Patterns

Add Rust regular expressions to the plugin config file:

```bash
herdr plugin config-dir hotchpotch.herdr-tiny-fingers
```

```toml
# $HERDR_PLUGIN_CONFIG_DIR/config.toml
copy_toast = false
direct_paste = false

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
copied. Custom patterns ignore pane line breaks while matching by default. Set
`ignore_line_breaks = false` for a pattern that should be evaluated separately
on each visible row, such as a line-anchored table pattern.

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
