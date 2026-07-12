use std::{collections::BTreeMap, fs, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Active,
    Idle,
    WaitingForInput,
}

impl SessionState {
    pub fn label(&self) -> &'static str {
        match self {
            SessionState::Active => "Working",
            SessionState::Idle => "Idle",
            SessionState::WaitingForInput => "Needs input",
        }
    }

    /// Derive the state from Claude Code's own session status field.
    /// Claude writes one of: "busy", "shell", "idle", "waiting".
    fn from_status(status: &str) -> Self {
        match status {
            "waiting" => SessionState::WaitingForInput,
            "idle" => SessionState::Idle,
            // "busy", "shell", and anything unknown are treated as working.
            _ => SessionState::Active,
        }
    }
}

/// A live Claude Code session, deserialized directly from its registry file at
/// `~/.claude/sessions/<pid>.json`. Claude Code owns this file — we never write it.
#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    pub pid: u32,
    #[serde(rename = "sessionId", default)]
    pub session_id: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: String,
    /// Only present when `status == "waiting"`, e.g. "permission prompt",
    /// "input needed", "sandbox request".
    #[serde(rename = "waitingFor", default)]
    pub waiting_for: Option<String>,
    #[serde(rename = "startedAt", default)]
    pub started_at_ms: Option<i64>,
    #[serde(rename = "statusUpdatedAt", default)]
    pub status_updated_at_ms: Option<i64>,
}

const ID_DISPLAY_LEN: usize = 8;

pub type GroupedSessions<'a> = Vec<(Option<&'a str>, Vec<&'a Session>)>;

impl Session {
    pub fn state(&self) -> SessionState {
        SessionState::from_status(&self.status)
    }

    /// Human status, with the waiting reason appended when Claude provides one.
    pub fn status_display(&self) -> String {
        match (self.state(), self.waiting_for.as_deref()) {
            (SessionState::WaitingForInput, Some(reason)) => {
                format!("{} ({reason})", SessionState::WaitingForInput.label())
            }
            (state, _) => state.label().to_string(),
        }
    }

    pub fn display_name(&self) -> &str {
        if let Some(name) = self.name.as_deref().filter(|n| !n.is_empty()) {
            return name;
        }
        if let Some(base) = self.cwd.as_deref().and_then(dir_name) {
            return base;
        }
        if self.session_id.len() > ID_DISPLAY_LEN {
            &self.session_id[..ID_DISPLAY_LEN]
        } else if !self.session_id.is_empty() {
            &self.session_id
        } else {
            "?"
        }
    }

    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at_ms.and_then(DateTime::from_timestamp_millis)
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.status_updated_at_ms
            .and_then(DateTime::from_timestamp_millis)
    }

    /// Whether the owning process is still running. Claude removes its registry
    /// file on clean exit, but a crash can leave a stale one behind.
    pub fn is_alive(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/proc")
                .join(self.pid.to_string())
                .exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            true
        }
    }
}

/// `~/.claude/sessions`, honoring `CLAUDE_CONFIG_DIR` when set.
pub fn sessions_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        return PathBuf::from(dir).join("sessions");
    }
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".claude").join("sessions")
}

/// Load every live session from the registry directory. Files that fail to
/// parse, or whose owning process is gone, are skipped.
pub fn load_sessions() -> Vec<Session> {
    let dir = sessions_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut sessions: Vec<Session> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json"))
        .filter_map(|p| fs::read_to_string(&p).ok())
        .filter_map(|c| serde_json::from_str::<Session>(&c).ok())
        .filter(Session::is_alive)
        .collect();

    sessions.sort_by(|a, b| a.display_name().cmp(b.display_name()));
    sessions
}

/// Group sessions by their working directory, sorted alphabetically. Sessions
/// without a cwd land in a trailing "Unknown" group.
pub fn grouped_sessions(sessions: &[Session]) -> GroupedSessions<'_> {
    let mut groups: BTreeMap<Option<&str>, Vec<&Session>> = BTreeMap::new();
    for s in sessions {
        groups.entry(s.cwd.as_deref()).or_default().push(s);
    }
    for group in groups.values_mut() {
        group.sort_by(|a, b| a.display_name().cmp(b.display_name()));
    }

    let mut result: Vec<_> = groups.into_iter().collect();
    result.sort_by(|(a, _), (b, _)| match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => a.cmp(b),
    });
    result
}

pub fn dir_name(path: &str) -> Option<&str> {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| !n.is_empty())
}

/// Read the newest user-set custom title from a session transcript, if any.
pub fn read_custom_title(transcript_path: &str) -> Option<String> {
    let content = fs::read_to_string(transcript_path).ok()?;
    content
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .rfind(|v| v["type"] == "custom-title")
        .and_then(|v| v["customTitle"].as_str().map(str::to_string))
}
