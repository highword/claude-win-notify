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
const ERROR_PATTERNS: &[&str] = &[
    "api rate limit",
    "rate limit exceeded",
    "session limit",
    "context window",
    "api error",
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
