use crate::hook::HookInput;

/// The four notification types with escalating urgency levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    TaskComplete,
    PermissionRequest,
    Question,
    Error,
}

/// Conservative set of system-level error patterns.
/// Only matches Claude Code infrastructure errors, never user-mentioned errors.
/// Patterns are chosen to avoid false positives on user task descriptions (D-03).
const ERROR_PATTERNS: &[&str] = &[
    "api rate limit",
    "rate limit exceeded",
    "session limit",
    "context window",
    "api error:",
    "api error occurred",
    "authentication failed",
];

impl NotificationType {
    /// Display title for the Toast notification.
    pub fn title(&self) -> &'static str {
        match self {
            NotificationType::TaskComplete => "\u{2714} Task Complete",
            NotificationType::PermissionRequest => "\u{1F510} Permission Required",
            NotificationType::Question => "\u{2753} Question",
            NotificationType::Error => "\u{26A0} Error",
        }
    }

    /// Windows system sound event URI for this notification type.
    pub fn audio_src(&self) -> &'static str {
        match self {
            NotificationType::TaskComplete => "ms-winsoundevent:Notification.Default",
            NotificationType::PermissionRequest => "ms-winsoundevent:Notification.Reminder",
            NotificationType::Question => "ms-winsoundevent:Notification.IM",
            NotificationType::Error => "ms-winsoundevent:Notification.Looping.Alarm",
        }
    }

    /// Filename of the hero image asset for this notification type.
    pub fn hero_filename(&self) -> &'static str {
        match self {
            NotificationType::TaskComplete => "hero-task-complete.png",
            NotificationType::PermissionRequest => "hero-permission.png",
            NotificationType::Question => "hero-question.png",
            NotificationType::Error => "hero-error.png",
        }
    }

    /// Whether the audio should loop. Always false (Error uses Looping.Alarm but single-play).
    pub fn audio_loop(&self) -> bool {
        false
    }
}

/// Classify a Stop hook event into a notification type.
///
/// Priority order (D-02): Error > Question > TaskComplete
pub fn classify_stop(input: &HookInput) -> NotificationType {
    if let Some(ref msg) = input.last_assistant_message {
        let lower = msg.to_lowercase();

        // Error detection (D-03, D-14): check against conservative pattern list
        for pattern in ERROR_PATTERNS {
            if lower.contains(pattern) {
                return NotificationType::Error;
            }
        }

        // Question detection (D-04, D-15): last_assistant_message ends with '?'
        if msg.trim().ends_with('?') {
            return NotificationType::Question;
        }
    }

    // Default (D-16): TaskComplete
    NotificationType::TaskComplete
}

/// Classify a Notification hook event into a notification type.
///
/// D-17: permission_prompt -> PermissionRequest
/// D-18: any other notification_type -> Question
/// D-19: fallback when notification_type is None - check message for "permission"
pub fn classify_notification(input: &HookInput) -> NotificationType {
    match input.notification_type.as_deref() {
        Some("permission_prompt") => NotificationType::PermissionRequest,
        Some(_) => NotificationType::Question,
        None => {
            // D-19: Bug #11964 fallback — check message for "permission" keyword
            if let Some(ref message) = input.message {
                if message.to_lowercase().contains("permission") {
                    return NotificationType::PermissionRequest;
                }
            }
            NotificationType::Question
        }
    }
}

/// Generate the body text for a Toast notification based on type and input.
pub fn body_text(ntype: NotificationType, input: &HookInput) -> String {
    match ntype {
        NotificationType::TaskComplete => "\u{2714} Task Complete".to_string(),
        NotificationType::PermissionRequest => {
            input.message.as_deref().unwrap_or("Permission needed").to_string()
        }
        NotificationType::Question => {
            if let Some(ref msg) = input.last_assistant_message {
                let last_line = msg.lines().last().unwrap_or("").trim();
                if last_line.is_empty() {
                    "Claude has a question".to_string()
                } else if last_line.len() > 200 {
                    format!("{}...", &last_line[..197])
                } else {
                    last_line.to_string()
                }
            } else {
                "Claude has a question".to_string()
            }
        }
        NotificationType::Error => {
            if let Some(ref msg) = input.last_assistant_message {
                let trimmed = msg.trim();
                if trimmed.is_empty() {
                    "An error occurred".to_string()
                } else if trimmed.len() > 200 {
                    format!("{}...", &trimmed[..197])
                } else {
                    trimmed.to_string()
                }
            } else {
                "An error occurred".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookInput;

    /// Helper to create a minimal HookInput for Stop events.
    fn stop_input(last_assistant_message: Option<&str>) -> HookInput {
        HookInput {
            session_id: "test".to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            cwd: "/tmp/project".to_string(),
            hook_event_name: "Stop".to_string(),
            stop_hook_active: Some(false),
            last_assistant_message: last_assistant_message.map(|s| s.to_string()),
            message: None,
            title: None,
            notification_type: None,
            permission_mode: None,
        }
    }

    /// Helper to create a minimal HookInput for Notification events.
    fn notification_input(
        notification_type: Option<&str>,
        message: Option<&str>,
    ) -> HookInput {
        HookInput {
            session_id: "test".to_string(),
            transcript_path: "/tmp/t.jsonl".to_string(),
            cwd: "/tmp/project".to_string(),
            hook_event_name: "Notification".to_string(),
            stop_hook_active: None,
            last_assistant_message: None,
            message: message.map(|s| s.to_string()),
            title: None,
            notification_type: notification_type.map(|s| s.to_string()),
            permission_mode: None,
        }
    }

    #[test]
    fn stop_default_is_task_complete() {
        let input = stop_input(Some("All done, files updated successfully."));
        assert_eq!(classify_stop(&input), NotificationType::TaskComplete);
    }

    #[test]
    fn stop_with_api_rate_limit_is_error() {
        let input = stop_input(Some("Hit the API rate limit, please wait."));
        assert_eq!(classify_stop(&input), NotificationType::Error);
    }

    #[test]
    fn stop_with_session_limit_is_error() {
        let input = stop_input(Some("Reached the session limit for today."));
        assert_eq!(classify_stop(&input), NotificationType::Error);
    }

    #[test]
    fn stop_with_context_window_is_error() {
        let input = stop_input(Some("The context window exceeded maximum length."));
        assert_eq!(classify_stop(&input), NotificationType::Error);
    }

    #[test]
    fn stop_error_case_insensitive() {
        let input = stop_input(Some("api Rate Limit reached"));
        assert_eq!(classify_stop(&input), NotificationType::Error);
    }

    #[test]
    fn stop_user_mentions_error_not_classified() {
        // D-03: User task descriptions mentioning "error" should NOT trigger Error classification.
        // "api error" pattern refined to "api error:" / "api error occurred" to avoid false positives.
        let input = stop_input(Some("Fixed the API error handling"));
        assert_eq!(classify_stop(&input), NotificationType::TaskComplete);
    }

    #[test]
    fn stop_with_question_mark_is_question() {
        let input = stop_input(Some("Would you like me to continue?"));
        assert_eq!(classify_stop(&input), NotificationType::Question);
    }

    #[test]
    fn stop_question_with_trailing_whitespace() {
        let input = stop_input(Some("Should I continue? \n"));
        assert_eq!(classify_stop(&input), NotificationType::Question);
    }

    #[test]
    fn stop_error_takes_priority_over_question() {
        // D-02: Error > Question priority
        let input = stop_input(Some("API rate limit exceeded. Should I retry?"));
        assert_eq!(classify_stop(&input), NotificationType::Error);
    }

    #[test]
    fn stop_none_message_is_task_complete() {
        let input = stop_input(None);
        assert_eq!(classify_stop(&input), NotificationType::TaskComplete);
    }

    #[test]
    fn notification_permission_prompt() {
        let input = notification_input(Some("permission_prompt"), Some("Bash: rm -rf"));
        assert_eq!(classify_notification(&input), NotificationType::PermissionRequest);
    }

    #[test]
    fn notification_missing_type_with_permission_message() {
        // D-19: Bug #11964 fallback
        let input = notification_input(None, Some("Tool needs permission to write"));
        assert_eq!(classify_notification(&input), NotificationType::PermissionRequest);
    }

    #[test]
    fn notification_other_type_is_question() {
        // D-06/D-18: any notification_type other than "permission_prompt" -> Question
        let input = notification_input(Some("idle_prompt"), Some("Are you still there?"));
        assert_eq!(classify_notification(&input), NotificationType::Question);
    }

    #[test]
    fn notification_no_type_no_permission_is_question() {
        let input = notification_input(None, Some("Hello"));
        assert_eq!(classify_notification(&input), NotificationType::Question);
    }
}
