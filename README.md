# herdr-tiny-fingers

`tmux-fingers` style copy hints for Herdr.

This plugin opens an overlay pane, reads the previously focused pane with
`pane.read --source visible`, redraws that screen, and labels matches with
letter hints. Type the hint to copy the matched text through Herdr's OSC 52
clipboard forwarding.

## Keybinding

Herdr plugin manifests do not install keybindings, so bind the action in your
Herdr config:

```toml
[[keys.command]]
key = "prefix+f"
type = "plugin_action"
command = "hotchpotch.herdr-tiny-fingers.open"
description = "fingers mode"
```

This is the Herdr spelling for tmux-style `prefix + f`.

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

## Development

```bash
cargo test
cargo build --release --locked
herdr plugin link .
```
