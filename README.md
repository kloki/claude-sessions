# claude-sessions

A Claude Code session tracker module for [waybar](https://github.com/Alexays/Waybar) that works for me.

It reads Claude Code's own session registry (`~/.claude/sessions/<pid>.json`)
directly — no hooks, no custom state file, nothing to keep in sync. Sessions
whose process is no longer running are ignored automatically.

# Install

## Binaries

Check [Releases](https://github.com/kloki/claude-sessions/releases) for binaries and installers

# Commands

| Command                | Description                                                    |
| ---------------------- | -------------------------------------------------------------- |
| `ps`                   | List active sessions in a terminal-friendly table              |
| `ps --format json`     | List active sessions as a JSON array                           |
| `waybar`               | Output Waybar-compatible JSON                                  |
| `process-notification` | Send a desktop notification via `notify-send` for a hook event |
| `completions <shell>`  | Generate shell completions                                     |

Session state is derived from Claude Code's own status field:

| Claude status    | Shown as        | Waybar class     |
| ---------------- | --------------- | ---------------- |
| `busy` / `shell` | `Working`       | `claude-active`  |
| `idle`           | `Idle`          | `claude-idle`    |
| `waiting`        | `Needs input`\* | `claude-waiting` |

\* When Claude is waiting, the reason it reports (`permission prompt`,
`input needed`, `sandbox request`, …) is shown alongside.

# Notifications (optional)

`process-notification` is the only piece that uses a hook. It turns a
`Notification` hook event into a readable `notify-send` bubble, deduplicated per
session. Add to your Claude Code `settings.json`:

```json
{
  "hooks": {
    "Notification": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "claude-sessions process-notification"
          }
        ]
      }
    ]
  }
}
```

# Waybar

Add this to your `config.jsonc`

```json
{
  "custom/claude-sessions": {
    "exec": "~/.cargo/bin/claude-sessions waybar",
    "return-type": "json",
    "interval": 5
  }
}
```

### Styling

The module sets a CSS class based on the state of your sessions. Add to your `style.css`:

```css
#custom-claude-sessions {
  /* default styles */
}

#custom-claude-sessions.claude-idle {
  color: #888888;
}

#custom-claude-sessions.claude-active {
  color: #89b4fa; /* Claude is thinking */
}

#custom-claude-sessions.claude-waiting {
  color: #f38ba8; /* Claude is waiting for your input */
}

#custom-claude-sessions.claude-empty {
  color: #f38ba8; /* Claude is waiting for your input */
}
```
