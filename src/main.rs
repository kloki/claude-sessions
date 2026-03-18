mod session;

use std::io::Read;

use clap::{Parser, Subcommand};
use serde::Deserialize;
use session::{SessionState, SessionStore};

#[derive(Parser)]
#[command(name = "claude-sessions", about = "Track Claude Code sessions")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Process a webhook event from Claude hooks (reads JSON from stdin)
    ProcessWebhook,
    /// Clear all session state
    Clear,
    /// Output Waybar-compatible JSON
    Waybar,
}

#[derive(Deserialize)]
struct HookInput {
    session_id: String,
    hook_event_name: String,
}

fn process_webhook() -> anyhow::Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let hook: HookInput = serde_json::from_str(&input)?;

    let mut store = SessionStore::load()?;

    if hook.hook_event_name == "SessionEnd" {
        store.sessions.remove(&hook.session_id);
    } else {
        let session = store.upsert(&hook.session_id);
        session.updated_at = chrono::Utc::now();
        session.state = match hook.hook_event_name.as_str() {
            "SessionStart" | "UserPromptSubmit" => SessionState::Active,
            "Stop" => SessionState::Idle,
            "Notification" => SessionState::WaitingForInput,
            _ => SessionState::Active,
        };
    }

    store.save()?;
    Ok(())
}

#[derive(serde::Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}

fn waybar() -> anyhow::Result<()> {
    let mut store = SessionStore::load()?;
    store.cleanup_stale();
    store.save()?;

    let count = store.sessions.len();
    let tooltip: String = store
        .sessions
        .iter()
        .map(|(id, s)| {
            let short_id = if id.len() > 8 { &id[..8] } else { id };
            format!("{}: {}", short_id, s.state)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let class = if store
        .sessions
        .values()
        .any(|s| s.state == SessionState::WaitingForInput)
    {
        "claude-waiting"
    } else if store
        .sessions
        .values()
        .any(|s| s.state == SessionState::Active)
    {
        "claude-active"
    } else {
        "claude-idle"
    };

    let output = WaybarOutput {
        text: count.to_string(),
        tooltip,
        class: class.to_string(),
    };

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::ProcessWebhook => process_webhook(),
        Command::Clear => SessionStore::clear().map_err(Into::into),
        Command::Waybar => waybar(),
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
