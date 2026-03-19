use crate::session::{SessionState, SessionStore};

#[derive(serde::Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}

fn waybar_class(store: &SessionStore) -> &'static str {
    if store
        .sessions
        .values()
        .any(|s| s.state == SessionState::WaitingForInput)
    {
        "claude-waiting"
    } else if store
        .sessions
        .values()
        .any(|s| s.state == SessionState::Idle)
    {
        "claude-idle"
    } else if !store.sessions.is_empty() {
        "claude-active"
    } else {
        "claude-empty"
    }
}

pub fn waybar() -> anyhow::Result<()> {
    let store = SessionStore::load_and_cleanup()?;

    let count = store.sessions.len();
    let tooltip = store
        .sorted_sessions()
        .iter()
        .map(|(id, s)| format!("{}: {}", s.state.label(), s.display_name(id)))
        .collect::<Vec<_>>()
        .join("\n");

    let output = WaybarOutput {
        text: count.to_string(),
        tooltip,
        class: waybar_class(&store).to_string(),
    };

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
