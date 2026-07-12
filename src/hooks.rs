use std::io::Read;

use serde::Deserialize;

use crate::session::{self, dir_name, read_custom_title};

#[derive(Deserialize)]
struct NotificationInput {
    session_id: String,
    message: Option<String>,
    cwd: Option<String>,
    transcript_path: Option<String>,
}

/// Resolve the friendliest name for a session: a user-set custom title wins,
/// then the name Claude derived in its registry, then the cwd's last component,
/// then a truncated session id.
fn resolve_name(notif: &NotificationInput) -> String {
    if let Some(title) = notif.transcript_path.as_deref().and_then(read_custom_title) {
        return title;
    }
    if let Some(name) = session::load_sessions()
        .into_iter()
        .find(|s| s.session_id == notif.session_id)
        .and_then(|s| s.name)
        .filter(|n| !n.is_empty())
    {
        return name;
    }
    if let Some(base) = notif.cwd.as_deref().and_then(dir_name) {
        return base.to_string();
    }
    notif.session_id[..notif.session_id.len().min(8)].to_string()
}

pub fn process_notification() -> anyhow::Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let notif: NotificationInput = serde_json::from_str(&input)?;

    let name = resolve_name(&notif);
    let summary = format!("Claude · {name}");
    let body = notif
        .message
        .as_deref()
        .filter(|m| !m.is_empty())
        .unwrap_or("Waiting for your input");

    // Collapse repeated notifications for the same session into one bubble
    // instead of stacking, so the tray stays readable.
    let dedup_key = format!(
        "x-canonical-private-synchronous:claude-{}",
        notif.session_id
    );

    std::process::Command::new("notify-send")
        .args(["-a", "Claude Code", "-u", "normal", "-h"])
        .arg(format!("string:{dedup_key}"))
        .arg(&summary)
        .arg(body)
        .status()
        .ok();

    Ok(())
}
