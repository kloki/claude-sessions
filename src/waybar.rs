use crate::{
    output::format_ps,
    session::{self, Session, SessionState},
};

#[derive(serde::Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}

fn waybar_class(sessions: &[Session]) -> &'static str {
    let has = |state: SessionState| sessions.iter().any(|s| s.state() == state);
    if has(SessionState::WaitingForInput) {
        "claude-waiting"
    } else if has(SessionState::Idle) {
        "claude-idle"
    } else if !sessions.is_empty() {
        "claude-active"
    } else {
        "claude-empty"
    }
}

pub fn waybar() -> anyhow::Result<()> {
    let sessions = session::load_sessions();

    let output = WaybarOutput {
        text: sessions.len().to_string(),
        tooltip: format_ps(&sessions, false, Some(12)),
        class: waybar_class(&sessions).to_string(),
    };

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
