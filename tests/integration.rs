use std::io::Write;
use std::process::{Command, Stdio};

fn cargo_bin() -> std::path::PathBuf {
    let status = Command::new("cargo")
        .args(["build"])
        .status()
        .expect("cargo build failed");
    assert!(status.success());

    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("debug");
    path.push("claude-win-notify.exe");
    path
}

#[test]
fn hook_with_valid_stop_json_exits_zero() {
    let bin = cargo_bin();
    let mut child = Command::new(&bin)
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(br#"{"session_id":"test","transcript_path":"C:\\test.jsonl","cwd":"D:\\Repository\\test-project","hook_event_name":"Stop","stop_hook_active":false}"#).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "Expected exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn hook_with_notification_json_exits_zero() {
    let bin = cargo_bin();
    let mut child = Command::new(&bin)
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    let stdin = child.stdin.as_mut().unwrap();
    let json = r#"{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"D:\\项目\\测试","hook_event_name":"Notification","message":"Permission needed","title":"Bash","notification_type":"permission_prompt"}"#;
    stdin.write_all(json.as_bytes()).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn hook_with_unknown_event_exits_zero_silently() {
    let bin = cargo_bin();
    let mut child = Command::new(&bin)
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(br#"{"session_id":"x","transcript_path":"y","cwd":"z","hook_event_name":"PreToolUse","tool_name":"Bash"}"#).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn hook_with_malformed_json_exits_zero() {
    let bin = cargo_bin();
    let mut child = Command::new(&bin)
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(b"this is not json at all").unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "Malformed JSON should exit 0 (per D-06)"
    );
}

#[test]
fn hook_with_null_stdin_exits_zero() {
    let bin = cargo_bin();
    let output = Command::new(&bin)
        .arg("hook")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run");
    // With null stdin (not TTY), reads empty and exits 0
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn version_flag_prints_version() {
    let bin = cargo_bin();
    let output = Command::new(&bin)
        .arg("--version")
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("claude-win-notify"),
        "Version output should contain binary name, got: {}",
        stdout
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn hook_stop_active_true_no_toast() {
    let bin = cargo_bin();
    let mut child = Command::new(&bin)
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(br#"{"session_id":"test","transcript_path":"C:\\test.jsonl","cwd":"D:\\test","hook_event_name":"Stop","stop_hook_active":true}"#).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stop_hook_active=true should exit 0 without showing Toast"
    );
}
