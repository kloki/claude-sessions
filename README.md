# Claude Sessions Tracker

A Rust CLI tool that tracks Claude Code sessions via hooks and exposes a Waybar widget.

## Install

```sh
cargo install --path .
```

## Claude hooks configuration

Add to your Claude Code `settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-webhook" }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-webhook" }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-webhook" }
        ]
      }
    ],
    "Notification": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-webhook" }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          { "type": "command", "command": "claude-sessions process-webhook" }
        ]
      }
    ]
  }
}
```

## Waybar configuration

Add to your Waybar config:

```json
{
  "custom/claude": {
    "exec": "claude-sessions waybar",
    "return-type": "json",
    "interval": 5
  }
}
```

The widget outputs:

- **text**: count of all sessions
- **tooltip**: each session ID (truncated) and its state
- **class**: `waiting` if any session needs input, `active` if any are active, `idle` otherwise

## CLI commands

- `claude-sessions process-webhook` — reads hook JSON from stdin, updates session state
- `claude-sessions waybar` — outputs Waybar-compatible JSON
- `claude-sessions clear` — removes session state file
