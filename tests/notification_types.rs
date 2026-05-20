//! Integration tests for the full notification classification pipeline and asset extraction.
//!
//! These tests validate all 5 success criteria from the ROADMAP without requiring
//! Windows Toast UI:
//! - SC-1: Task Complete fires on Stop when task finishes successfully
//! - SC-2: Permission Request fires when Claude needs tool approval
//! - SC-3: Question fires when Claude asks a question
//! - SC-4: Error fires on API errors, session limits, abnormal exits
//! - SC-5: Each type has visually distinct styling (validated via body_text output)

use claude_win_notify::hook::HookInput;
use claude_win_notify::notification::{
    body_text, classify_notification, classify_stop, NotificationType,
};

// ─── Helpers ───────────────────────────────────────────────────────────────────

/// Create a minimal HookInput for Stop events.
fn stop_input(last_assistant_message: Option<&str>) -> HookInput {
    HookInput {
        session_id: "test-session".to_string(),
        transcript_path: "/tmp/transcript.jsonl".to_string(),
        cwd: "D:\\Repository\\test-project".to_string(),
        hook_event_name: "Stop".to_string(),
        stop_hook_active: Some(false),
        last_assistant_message: last_assistant_message.map(|s| s.to_string()),
        message: None,
        title: None,
        notification_type: None,
        permission_mode: None,
    }
}

/// Create a minimal HookInput for Notification events.
fn notification_input(notification_type: Option<&str>, message: Option<&str>) -> HookInput {
    HookInput {
        session_id: "test-session".to_string(),
        transcript_path: "/tmp/transcript.jsonl".to_string(),
        cwd: "D:\\Repository\\test-project".to_string(),
        hook_event_name: "Notification".to_string(),
        stop_hook_active: None,
        last_assistant_message: None,
        message: message.map(|s| s.to_string()),
        title: None,
        notification_type: notification_type.map(|s| s.to_string()),
        permission_mode: None,
    }
}

// ─── SC-1: Task Complete fires on Stop when task finishes successfully ─────────

#[test]
fn stop_normal_completion_is_task_complete() {
    let input = stop_input(Some("I've completed the implementation"));
    assert_eq!(classify_stop(&input), NotificationType::TaskComplete);
}

#[test]
fn stop_with_code_output_is_task_complete() {
    let input = stop_input(Some("Here's the updated code:\n```rust\nfn main() {}\n```"));
    assert_eq!(classify_stop(&input), NotificationType::TaskComplete);
}

// ─── SC-2: Permission Request fires when Claude needs tool approval ────────────

#[test]
fn notification_permission_prompt_type() {
    let input = notification_input(Some("permission_prompt"), Some("Bash: rm -rf /tmp/test"));
    assert_eq!(
        classify_notification(&input),
        NotificationType::PermissionRequest
    );
}

#[test]
fn notification_permission_fallback_when_type_missing() {
    // D-19: Bug #11964 — when notification_type is None, check message for "permission"
    let input = notification_input(None, Some("Claude wants permission to run Bash"));
    assert_eq!(
        classify_notification(&input),
        NotificationType::PermissionRequest
    );
}

#[test]
fn notification_permission_case_insensitive() {
    // D-19 fallback uses .to_lowercase() so "PERMISSION" should match
    let input = notification_input(None, Some("PERMISSION needed to access file"));
    assert_eq!(
        classify_notification(&input),
        NotificationType::PermissionRequest
    );
}

// ─── SC-3: Question fires when Claude asks a question ──────────────────────────

#[test]
fn stop_question_ends_with_question_mark() {
    let input = stop_input(Some("Would you like me to proceed?"));
    assert_eq!(classify_stop(&input), NotificationType::Question);
}

#[test]
fn stop_question_multiline_last_line() {
    let input = stop_input(Some(
        "Here's the plan:\n1. Do X\n2. Do Y\nShall I proceed?",
    ));
    assert_eq!(classify_stop(&input), NotificationType::Question);
}

#[test]
fn notification_idle_prompt_is_question() {
    // D-06: idle_prompt notification_type maps to Question
    let input = notification_input(Some("idle_prompt"), Some("Are you still there?"));
    assert_eq!(
        classify_notification(&input),
        NotificationType::Question
    );
}

#[test]
fn notification_unknown_type_is_question() {
    // D-18: any unknown notification_type maps to Question
    let input = notification_input(Some("some_future_type"), Some("Something happened"));
    assert_eq!(
        classify_notification(&input),
        NotificationType::Question
    );
}

// ─── SC-4: Error fires on API errors, session limits, abnormal exits ───────────

#[test]
fn stop_api_rate_limit_error() {
    let input = stop_input(Some("I apologize, but I've hit the API rate limit"));
    assert_eq!(classify_stop(&input), NotificationType::Error);
}

#[test]
fn stop_session_limit_error() {
    let input = stop_input(Some("The session limit has been reached"));
    assert_eq!(classify_stop(&input), NotificationType::Error);
}

#[test]
fn stop_context_window_error() {
    let input = stop_input(Some("context window exceeded, cannot continue"));
    assert_eq!(classify_stop(&input), NotificationType::Error);
}

#[test]
fn stop_authentication_failed_error() {
    let input = stop_input(Some("authentication failed for the API"));
    assert_eq!(classify_stop(&input), NotificationType::Error);
}

// ─── SC-5: Each type has visually distinct styling (via body_text output) ──────

#[test]
fn body_text_task_complete_contains_checkmark() {
    let input = stop_input(Some("All done."));
    let body = body_text(NotificationType::TaskComplete, &input);
    assert!(
        body.contains('\u{2714}'),
        "TaskComplete body should contain checkmark, got: {}",
        body
    );
}

#[test]
fn body_text_permission_uses_message() {
    let input = notification_input(Some("permission_prompt"), Some("Bash: ls -la"));
    let body = body_text(NotificationType::PermissionRequest, &input);
    assert_eq!(body, "Bash: ls -la");
}

#[test]
fn body_text_question_uses_last_message() {
    let mut input = stop_input(Some("Should I continue with this approach?"));
    // For Question, body_text uses the last line of last_assistant_message
    input.last_assistant_message = Some("Should I continue with this approach?".to_string());
    let body = body_text(NotificationType::Question, &input);
    assert_eq!(body, "Should I continue with this approach?");
}

#[test]
fn body_text_error_uses_last_message() {
    let input = stop_input(Some("API rate limit exceeded"));
    let body = body_text(NotificationType::Error, &input);
    assert_eq!(body, "API rate limit exceeded");
}

// ─── Asset Extraction Integration Tests ────────────────────────────────────────

use claude_win_notify::assets::ensure_hero_image_in;
use std::fs;
use std::path::Path;

#[test]
fn asset_extraction_creates_files() {
    let dir = std::env::temp_dir().join("claude-win-notify-test-creates");
    // Clean up from any previous run
    let _ = fs::remove_dir_all(&dir);

    let types = [
        NotificationType::TaskComplete,
        NotificationType::PermissionRequest,
        NotificationType::Question,
        NotificationType::Error,
    ];

    for ntype in &types {
        let result = ensure_hero_image_in(&dir, *ntype);
        assert!(
            result.is_some(),
            "ensure_hero_image_in should succeed for {:?}",
            ntype
        );
        let path = result.unwrap();
        assert!(path.exists(), "File should exist on disk: {:?}", path);
    }

    // Clean up
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn asset_extraction_skips_existing() {
    let dir = std::env::temp_dir().join("claude-win-notify-test-skips");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let ntype = NotificationType::TaskComplete;
    let path = dir.join(ntype.hero_filename());

    // Write a dummy sentinel file
    let sentinel = b"SENTINEL_DATA_DO_NOT_OVERWRITE";
    fs::write(&path, sentinel).unwrap();

    // Call ensure_hero_image_in — should skip because file exists
    let result = ensure_hero_image_in(&dir, ntype);
    assert!(result.is_some());

    // Verify the original content was NOT overwritten
    let content = fs::read(&path).unwrap();
    assert_eq!(
        content, sentinel,
        "Existing file should not be overwritten"
    );

    // Clean up
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn asset_extraction_returns_none_on_invalid_dir() {
    // Use a path that cannot be created (invalid characters on Windows)
    let invalid_dir = Path::new("Z:\\nonexistent\\path\\that\\wont\\exist\\<invalid>");
    let result = ensure_hero_image_in(invalid_dir, NotificationType::TaskComplete);
    assert!(
        result.is_none(),
        "Should return None for invalid directory path"
    );
}

#[test]
fn extracted_files_are_valid_png() {
    let dir = std::env::temp_dir().join("claude-win-notify-test-png");
    let _ = fs::remove_dir_all(&dir);

    let types = [
        NotificationType::TaskComplete,
        NotificationType::PermissionRequest,
        NotificationType::Question,
        NotificationType::Error,
    ];

    let png_magic: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

    for ntype in &types {
        let result = ensure_hero_image_in(&dir, *ntype);
        assert!(result.is_some(), "Should extract for {:?}", ntype);

        let path = result.unwrap();
        let content = fs::read(&path).unwrap();
        assert!(
            content.len() >= 8,
            "PNG file for {:?} should be at least 8 bytes",
            ntype
        );
        assert_eq!(
            &content[..8],
            &png_magic,
            "File for {:?} should have valid PNG magic bytes",
            ntype
        );
    }

    // Clean up
    let _ = fs::remove_dir_all(&dir);
}
