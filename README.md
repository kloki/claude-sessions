# claude-sessions

It's `ps` for your Claude Code sessions. Track what your agents are up to with a terminal command, a live TUI, or a Waybar widget.

## Commands

| Command                | Description                                              |
| ---------------------- | -------------------------------------------------------- |
| `ps`                   | List active sessions in terminal-friendly format         |
| `watch`                | Live-updating TUI that monitors session changes          |
| `waybar`               | Output Waybar-compatible JSON                            |
| `json`                 | Output sessions as a JSON array                          |
| `process-hook`         | Process a hook event from Claude (reads stdin)           |
| `process-notification` | Send a desktop notification via `notify-send` for a hook |
| `clear`                | Clear all session state                                  |

# Install

## Binaries

Check [Releases](https://github.com/kloki/claude-sessions/releases) for binaries and installers

# Configure

claude-sessionss works by processing the hooks provided by claude code.

## Claude hooks

Add to your Claude Code `settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-hook" }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-hook" }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-hook" }
        ]
      }
    ],
    "Notification": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-hook" },
          {
            "type": "command",
            "command": "claude-sessions process-notification"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-hook" }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-hook" }
        ]
      }
    ],
    "PreToolUse": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-hook" }
        ]
      }
    ]
  }
}
```

## Waybar widget

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
