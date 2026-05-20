use serde::Deserialize;

use crate::notification::{classify_stop, classify_notification, body_text};

#[derive(Deserialize, Debug)]
pub struct HookInput {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub hook_event_name: String,
    pub stop_hook_active: Option<bool>,
    pub last_assistant_message: Option<String>,
    pub message: Option<String>,
    pub title: Option<String>,
    pub notification_type: Option<String>,
    pub permission_mode: Option<String>,
}

pub fn handle_hook() -> Result<(), crate::error::AppError> {
    let input = std::io::read_to_string(std::io::stdin())?;

    if input.trim().is_empty() {
        crate::log::log_error("Empty stdin received");
        return Ok(());
    }

    let hook_input: HookInput = match serde_json::from_str(&input) {
        Ok(parsed) => parsed,
        Err(e) => {
            crate::log::log_error(&format!("JSON parse error: {}", e));
            return Ok(());
        }
    };

    match hook_input.hook_event_name.as_str() {
        "Stop" => handle_stop(&hook_input),
        "Notification" => handle_notification(&hook_input),
        _ => Ok(()),
    }
}

fn handle_stop(input: &HookInput) -> Result<(), crate::error::AppError> {
    if input.stop_hook_active == Some(true) {
        return Ok(());
    }

    let ntype = classify_stop(input);
    let project_name = extract_project_name(&input.cwd);
    let body = body_text(ntype, input);

    crate::toast::show_typed_toast(ntype, &body, &project_name)
}

fn handle_notification(input: &HookInput) -> Result<(), crate::error::AppError> {
    let ntype = classify_notification(input);
    let project_name = extract_project_name(&input.cwd);
    let body = body_text(ntype, input);

    crate::toast::show_typed_toast(ntype, &body, &project_name)
}

fn extract_project_name(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stop_hook_all_fields() {
        let json = r#"{"session_id":"abc123","transcript_path":"C:\\Users\\test\\.claude\\transcript.jsonl","cwd":"D:\\Repository\\claude-win-notify","hook_event_name":"Stop","stop_hook_active":false,"last_assistant_message":"Done","permission_mode":"default"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "abc123");
        assert_eq!(input.hook_event_name, "Stop");
        assert_eq!(input.cwd, r"D:\Repository\claude-win-notify");
        assert_eq!(input.stop_hook_active, Some(false));
        assert_eq!(input.last_assistant_message.as_deref(), Some("Done"));
    }

    #[test]
    fn parse_notification_hook() {
        let json = r#"{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"/home/user/project","hook_event_name":"Notification","message":"Permission needed","title":"Bash","notification_type":"permission_prompt"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.hook_event_name, "Notification");
        assert_eq!(input.notification_type.as_deref(), Some("permission_prompt"));
        assert_eq!(input.message.as_deref(), Some("Permission needed"));
    }

    #[test]
    fn parse_cjk_path() {
        let json = r#"{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"D:\\项目\\我的测试","hook_event_name":"Stop"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.cwd, "D:\\项目\\我的测试");
    }

    #[test]
    fn parse_unknown_event_succeeds() {
        let json = r#"{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"/tmp","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":"{}"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.hook_event_name, "PreToolUse");
    }

    #[test]
    fn parse_minimal_fields() {
        let json = r#"{"session_id":"x","transcript_path":"y","cwd":"z","hook_event_name":"Stop"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "x");
        assert!(input.stop_hook_active.is_none());
        assert!(input.message.is_none());
    }

    #[test]
    fn extract_project_name_windows_path() {
        let cwd = r"D:\Repository\claude-win-notify";
        let name = std::path::Path::new(cwd)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        assert_eq!(name, "claude-win-notify");
    }

    #[test]
    fn extract_project_name_cjk() {
        let cwd = r"D:\项目\测试项目";
        let name = std::path::Path::new(cwd)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        assert_eq!(name, "测试项目");
    }
}
