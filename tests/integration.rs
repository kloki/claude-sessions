use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::{Value, json};
use tempfile::TempDir;

/// A PID that is essentially guaranteed not to be running, so its registry file
/// is treated as stale. Liveness is only enforced on Linux (via /proc).
const DEAD_PID: u32 = 4_000_000_000;

fn cmd(config_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("claude-sessions").unwrap();
    cmd.env("CLAUDE_CONFIG_DIR", config_dir);
    cmd
}

/// Write a session registry file the way Claude Code does, at
/// `<config>/sessions/<pid>.json`. Defaults to the current (alive) process pid.
fn write_session(config_dir: &Path, session_id: &str, status: &str, extra: Value) {
    let dir = config_dir.join("sessions");
    fs::create_dir_all(&dir).unwrap();

    let mut entry = json!({
        "pid": std::process::id(),
        "sessionId": session_id,
        "kind": "interactive",
        "entrypoint": "cli",
        "status": status,
        "startedAt": 1_700_000_000_000i64,
        "statusUpdatedAt": 1_700_000_000_000i64,
        "updatedAt": 1_700_000_000_000i64,
    });
    if let Value::Object(fields) = extra {
        for (k, v) in fields {
            entry[k] = v;
        }
    }

    let path = dir.join(format!("{}.json", entry["pid"]));
    fs::write(path, entry.to_string()).unwrap();
}

fn ps_json(config_dir: &Path) -> Value {
    let out = cmd(config_dir)
        .args(["ps", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

fn ps_human(config_dir: &Path) -> String {
    let out = cmd(config_dir)
        .arg("ps")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).unwrap()
}

fn waybar_output(config_dir: &Path) -> Value {
    let out = cmd(config_dir)
        .arg("waybar")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).unwrap()
}

#[test]
fn ps_json_empty_when_no_registry() {
    let dir = TempDir::new().unwrap();
    let sessions = ps_json(dir.path());
    assert_eq!(sessions.as_array().unwrap().len(), 0);
}

#[test]
fn ps_json_reports_state_from_status() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "busy-sess", "busy", Value::Null);

    let sessions = ps_json(dir.path());
    let arr = sessions.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["status"], "busy");
    assert_eq!(arr[0]["state"], "Working");
}

#[test]
fn ps_json_idle_state() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "idle-sess", "idle", Value::Null);
    let arr = ps_json(dir.path());
    assert_eq!(arr[0]["state"], "Idle");
}

#[test]
fn ps_json_waiting_state_carries_reason() {
    let dir = TempDir::new().unwrap();
    write_session(
        dir.path(),
        "wait-sess",
        "waiting",
        json!({ "waitingFor": "permission prompt" }),
    );
    let arr = ps_json(dir.path());
    assert_eq!(arr[0]["state"], "Needs input");
    assert_eq!(arr[0]["waiting_for"], "permission prompt");
}

#[test]
fn ps_json_shell_counts_as_working() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "shell-sess", "shell", Value::Null);
    let arr = ps_json(dir.path());
    assert_eq!(arr[0]["state"], "Working");
}

#[cfg(target_os = "linux")]
#[test]
fn dead_sessions_are_filtered() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "dead-sess", "idle", json!({ "pid": DEAD_PID }));
    let arr = ps_json(dir.path());
    assert_eq!(arr.as_array().unwrap().len(), 0);
}

#[test]
fn ps_human_shows_project_and_status() {
    let dir = TempDir::new().unwrap();
    write_session(
        dir.path(),
        "sess-1",
        "waiting",
        json!({ "cwd": "/home/user/repos/myproject", "waitingFor": "input needed" }),
    );

    let ps = ps_human(dir.path());
    assert!(ps.contains("/home/user/repos/myproject"), "ps was: {ps}");
    assert!(ps.contains("Needs input"), "ps was: {ps}");
    assert!(ps.contains("input needed"), "ps was: {ps}");
}

#[test]
fn ps_human_uses_name_then_cwd_then_id() {
    let dir = TempDir::new().unwrap();
    // Explicit name wins.
    write_session(dir.path(), "sess-1", "idle", json!({ "name": "my-label" }));
    let ps = ps_human(dir.path());
    assert!(ps.contains("my-label"), "ps was: {ps}");
}

#[test]
fn ps_human_unknown_group_without_cwd() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "sess-1", "idle", Value::Null);
    let ps = ps_human(dir.path());
    assert!(ps.contains("Unknown"), "ps was: {ps}");
}

#[test]
fn waybar_empty_registry() {
    let dir = TempDir::new().unwrap();
    let out = waybar_output(dir.path());
    assert_eq!(out["text"], "0");
    assert_eq!(out["tooltip"], "No active sessions");
    assert_eq!(out["class"], "claude-empty");
}

#[test]
fn waybar_counts_and_classes_by_status() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "sess-1", "busy", Value::Null);
    let out = waybar_output(dir.path());
    assert_eq!(out["text"], "1");
    assert_eq!(out["class"], "claude-active");
}

#[test]
fn waybar_class_waiting_takes_priority() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "sess-1", "waiting", Value::Null);
    let out = waybar_output(dir.path());
    assert_eq!(out["class"], "claude-waiting");
}

#[test]
fn waybar_class_idle() {
    let dir = TempDir::new().unwrap();
    write_session(dir.path(), "sess-1", "idle", Value::Null);
    let out = waybar_output(dir.path());
    assert_eq!(out["class"], "claude-idle");
}

#[test]
fn ps_json_ignores_unparseable_files() {
    let dir = TempDir::new().unwrap();
    let sessions_dir = dir.path().join("sessions");
    fs::create_dir_all(&sessions_dir).unwrap();
    fs::write(sessions_dir.join("garbage.json"), "not json").unwrap();
    fs::write(sessions_dir.join("note.txt"), "ignored").unwrap();
    write_session(dir.path(), "sess-1", "busy", Value::Null);

    let arr = ps_json(dir.path());
    assert_eq!(arr.as_array().unwrap().len(), 1);
}

#[test]
fn process_notification_succeeds() {
    let dir = TempDir::new().unwrap();
    let input = json!({
        "session_id": "notif-sess",
        "message": "Task complete",
        "cwd": "/home/user/integration-test",
    });
    // notify-send may not exist in CI; the command .ok()s that call and still succeeds.
    cmd(dir.path())
        .arg("process-notification")
        .write_stdin(input.to_string())
        .assert()
        .success();
}

#[test]
fn process_notification_fallback_when_no_message() {
    let dir = TempDir::new().unwrap();
    let input = json!({ "session_id": "notif-sess-2", "cwd": "/tmp/integration-test" });
    cmd(dir.path())
        .arg("process-notification")
        .write_stdin(input.to_string())
        .assert()
        .success();
}

#[test]
fn process_notification_resolves_custom_title() {
    let dir = TempDir::new().unwrap();
    let session_id = "notif-transcript";
    let transcript = dir.path().join(format!("{session_id}.jsonl"));
    let entry = json!({
        "type": "custom-title",
        "customTitle": "integration-label",
        "sessionId": session_id,
    });
    fs::write(&transcript, format!("{entry}\n")).unwrap();

    let input = json!({
        "session_id": session_id,
        "message": "Done!",
        "transcript_path": transcript.to_str().unwrap(),
    });
    cmd(dir.path())
        .arg("process-notification")
        .write_stdin(input.to_string())
        .assert()
        .success();
}

#[test]
fn process_notification_rejects_invalid_json() {
    let dir = TempDir::new().unwrap();
    cmd(dir.path())
        .arg("process-notification")
        .write_stdin("not json")
        .assert()
        .failure();
}
