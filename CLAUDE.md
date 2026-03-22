# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust CLI tool (`claude-sessions`) that tracks Claude Code sessions via hooks and exposes a Waybar widget. It reads hook events from stdin, maintains session state in `~/.claude_sessions` (JSON), and outputs status for Waybar, terminal (`ps`), or JSON consumers.

## Build & test

```bash
cargo build
cargo test            # all tests (integration tests in tests/integration.rs)
cargo test <name>     # run a single test by name substring
```

Requires Rust edition 2024. No linter or formatter is enforced in CI.

## Architecture

- **`src/main.rs`** — CLI entrypoint using clap with subcommands (`process-hook`, `process-notification`, `ps`, `waybar`, `json`, `watch`, `clear`, `completions`)
- **`src/session.rs`** — Core domain: `Session`, `SessionState` (Active/Idle/WaitingForInput), `SessionStore` (HashMap-backed, persisted as JSON via atomic write). Stale sessions (>24h) are cleaned on read. Sessions are grouped by project path.
- **`src/hooks.rs`** — Reads Claude hook JSON from stdin, maps hook events to state transitions, upserts sessions. Handles `process-notification` (desktop notifications via `notify-send`).
- **`src/output.rs`** — Terminal table formatting (`ps`) and JSON output. `format_ps()` is shared between `ps` and `waybar` tooltip.
- **`src/waybar.rs`** — Waybar JSON output with CSS class priority: waiting > idle > active > empty.
- **`src/watch.rs`** — Live TUI using crossterm + notify file watcher on the session state file.

## Key patterns

- Hook processing uses an **upsert pattern** — sessions are created on any event if they don't exist, making the system resilient to missed `SessionStart` events.
- State file uses **atomic write** (write to `.tmp`, then rename).
- Integration tests override `HOME` env var to use a temp directory, keeping tests isolated.
- Session names resolve in priority order: transcript custom-title > cwd last component > truncated session ID.
