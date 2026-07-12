# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust CLI tool (`claude-sessions`) that surfaces running Claude Code sessions,
primarily as a Waybar widget. It reads Claude Code's **own** session registry —
`~/.claude/sessions/<pid>.json` (honoring `CLAUDE_CONFIG_DIR`) — directly. It
keeps no state of its own and installs no state-tracking hooks; sessions whose
process is gone are filtered out via `/proc` on Linux.

## Build & test

```bash
cargo build
cargo test            # integration tests in tests/integration.rs
cargo test <name>     # run a single test by name substring
```

Requires Rust edition 2024. No linter or formatter is enforced in CI.

## Architecture

- **`src/main.rs`** — clap CLI with subcommands (`ps [--format human|json]`, `waybar`, `process-notification`, `completions`).
- **`src/session.rs`** — Core domain: the `Session` struct is deserialized straight from a registry file. `SessionState` (Active/Idle/WaitingForInput) is derived from Claude's `status` field (`busy`/`shell` → Active, `idle` → Idle, `waiting` → WaitingForInput). `load_sessions()` reads the registry dir and drops dead/unparseable entries; `grouped_sessions()` groups by cwd.
- **`src/output.rs`** — Terminal table (`ps_human`) and JSON (`ps_json`) output. `format_ps()` is shared between `ps` and the `waybar` tooltip.
- **`src/waybar.rs`** — Waybar JSON output with CSS class priority: waiting > idle > active > empty.
- **`src/hooks.rs`** — `process-notification` only: reads a Notification hook event from stdin and emits a readable, per-session-deduplicated `notify-send` bubble.

## Registry file schema (owned by Claude Code, read-only here)

Fields consumed: `pid`, `sessionId`, `cwd`, `name`, `status` (one of
`busy`/`shell`/`idle`/`waiting`), `waitingFor` (reason string, only when
`waiting`), `startedAt`, `statusUpdatedAt` (all epoch millis). Other fields
(`kind`, `entrypoint`, `sock`, `jobId`, …) exist but are currently ignored.

## Key patterns

- **No custom state** — every command is a pure read of Claude's registry, so output always reflects reality with no drift.
- **Liveness filtering** — a session is shown only if `/proc/<pid>` exists (Linux); non-Linux builds assume alive.
- Session names resolve in priority order: registry `name` → cwd last component → truncated session id. `process-notification` additionally prefers a user-set transcript custom-title.
- Integration tests set `CLAUDE_CONFIG_DIR` to a temp dir and write registry files using the test process's own (alive) PID.
