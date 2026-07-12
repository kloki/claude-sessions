use chrono::{DateTime, Utc};

use crate::session::{self, Session};

fn format_age(dt: Option<DateTime<Utc>>) -> String {
    let Some(dt) = dt else {
        return "-".to_string();
    };
    let dur = Utc::now() - dt;
    if dur.num_hours() >= 1 {
        format!("{}h{}m ago", dur.num_hours(), dur.num_minutes() % 60)
    } else if dur.num_minutes() >= 1 {
        format!("{}m ago", dur.num_minutes())
    } else {
        "just now".to_string()
    }
}

fn display_project(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() && path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    }
}

pub fn format_ps(sessions: &[Session], show_id: bool, max_name_width: Option<usize>) -> String {
    if sessions.is_empty() {
        return "No active sessions".to_string();
    }

    let groups = session::grouped_sessions(sessions);

    let mut name_width = sessions
        .iter()
        .map(|s| s.display_name().len())
        .max()
        .unwrap_or(4)
        .max(4);
    if let Some(max) = max_name_width {
        name_width = name_width.min(max);
    }
    let status_width = sessions
        .iter()
        .map(|s| s.status_display().len())
        .max()
        .unwrap_or(6)
        .max(6);

    let mut lines = Vec::new();

    for (i, (project, group)) in groups.iter().enumerate() {
        if i > 0 {
            lines.push(String::new());
        }

        let header = match project {
            Some(path) => display_project(path),
            None => "Unknown".to_string(),
        };
        lines.push(header);

        if show_id {
            lines.push(format!(
                "  {:<status_width$}  {:<name_width$}  {:<36}  {:>10}  {:>10}",
                "STATE", "NAME", "SESSION", "STARTED", "UPDATED",
            ));
        } else {
            lines.push(format!(
                "  {:<status_width$}  {:<name_width$}  {:>10}  {:>10}",
                "STATE", "NAME", "STARTED", "UPDATED",
            ));
        }

        for s in group {
            let name = s.display_name();
            let name = if name.len() > name_width {
                &name[..name_width]
            } else {
                name
            };
            if show_id {
                lines.push(format!(
                    "  {:<status_width$}  {:<name_width$}  {:<36}  {:>10}  {:>10}",
                    s.status_display(),
                    name,
                    s.session_id,
                    format_age(s.started_at()),
                    format_age(s.updated_at()),
                ));
            } else {
                lines.push(format!(
                    "  {:<status_width$}  {:<name_width$}  {:>10}  {:>10}",
                    s.status_display(),
                    name,
                    format_age(s.started_at()),
                    format_age(s.updated_at()),
                ));
            }
        }
    }
    lines.join("\n")
}

pub fn ps_human() -> anyhow::Result<()> {
    let sessions = session::load_sessions();
    println!("{}", format_ps(&sessions, true, None));
    Ok(())
}

#[derive(serde::Serialize)]
struct JsonSession {
    pid: u32,
    id: String,
    name: String,
    state: String,
    /// Raw status from Claude Code: busy | shell | idle | waiting.
    status: String,
    waiting_for: Option<String>,
    cwd: Option<String>,
    started_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

pub fn ps_json() -> anyhow::Result<()> {
    let sessions = session::load_sessions();

    let out: Vec<JsonSession> = sessions
        .iter()
        .map(|s| JsonSession {
            pid: s.pid,
            id: s.session_id.clone(),
            name: s.display_name().to_string(),
            state: s.state().label().to_string(),
            status: s.status.clone(),
            waiting_for: s.waiting_for.clone(),
            cwd: s.cwd.clone(),
            started_at: s.started_at(),
            updated_at: s.updated_at(),
        })
        .collect();

    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}
