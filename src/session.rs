use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    Active,
    Idle,
    WaitingForInput,
}

impl SessionState {
    pub fn label(&self) -> &'static str {
        match self {
            SessionState::Active => "[Thinking]   ",
            SessionState::Idle => "[Idle]       ",
            SessionState::WaitingForInput => "[Needs input]",
        }
    }
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Active => write!(f, "Thinking"),
            SessionState::Idle => write!(f, "Idle"),
            SessionState::WaitingForInput => write!(f, "Waiting For Input"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub state: SessionState,
    pub name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionStore {
    pub sessions: HashMap<String, Session>,
}

fn state_file_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".claude_sessions")
}

const ID_DISPLAY_LEN: usize = 8;

impl Session {
    pub fn display_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.name.as_deref().unwrap_or_else(|| {
            if id.len() > ID_DISPLAY_LEN {
                &id[..ID_DISPLAY_LEN]
            } else {
                id
            }
        })
    }
}

impl SessionStore {
    pub fn sorted_sessions(&self) -> Vec<(&str, &Session)> {
        let mut v: Vec<(&str, &Session)> = self
            .sessions
            .iter()
            .map(|(id, s)| (id.as_str(), s))
            .collect();
        v.sort_by_key(|(id, s)| s.display_name(id));
        v
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = state_file_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path).context("reading session store")?;
        serde_json::from_str(&contents).context("parsing session store")
    }

    pub fn load_and_cleanup() -> anyhow::Result<Self> {
        let mut store = Self::load()?;
        store.cleanup_stale();
        store.save()?;
        Ok(store)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = state_file_path();
        let tmp = path.with_extension("tmp");
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&tmp, &contents)?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn clear() -> anyhow::Result<()> {
        let path = state_file_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Get or create a session (upsert pattern for resilience to missed SessionStart)
    pub fn upsert(&mut self, session_id: &str) -> &mut Session {
        let now = Utc::now();
        self.sessions
            .entry(session_id.to_string())
            .or_insert(Session {
                state: SessionState::Idle,
                name: None,
                started_at: now,
                updated_at: now,
            })
    }

    /// Remove sessions older than 24 hours based on updated_at
    pub fn cleanup_stale(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        self.sessions.retain(|_, s| s.updated_at > cutoff);
    }
}

pub fn read_custom_title(transcript_path: &str) -> Option<String> {
    let content = fs::read_to_string(transcript_path).ok()?;
    content
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .rfind(|v| v["type"] == "custom-title")
        .and_then(|v| v["customTitle"].as_str().map(str::to_string))
}
