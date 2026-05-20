# Phase 3: Notification Type Detection — Research

**Researched:** 2026-05-20
**Status:** Complete

## 1. Windows Toast XML — Hero Image

### Syntax
```xml
<toast>
  <visual>
    <binding template="ToastGeneric">
      <text>Title</text>
      <text>Body</text>
      <text placement="attribution">Project Name</text>
      <image placement="hero" src="file:///C:/path/to/image.png"/>
    </binding>
  </visual>
  <audio src="ms-winsoundevent:Notification.Default"/>
</toast>
```

### Key findings
- `<image placement="hero">` must use `file:///` URI scheme for local files (NOT plain file path)
- Hero images display at the TOP of the toast, full-width (364×180 recommended by MS docs)
- Image format: PNG recommended; JPEG works. Max file size not documented but < 200KB is safe.
- If the image file doesn't exist, the toast still displays without it (graceful degradation)
- The `<image>` element goes inside `<binding template="ToastGeneric">` alongside `<text>` elements
- Order within `<binding>` doesn't matter — `placement="hero"` determines position

### Integration with current code
Current `toast.rs` uses `format!()` to build XML string. Adding hero image is trivial — conditionally insert `<image placement="hero" src="file:///{path}"/>` into the binding block.

## 2. Toast Audio — System Sound Events

### Available `ms-winsoundevent` values (confirmed working)
| Sound ID | Character |
|----------|-----------|
| `Notification.Default` | Light, standard notification |
| `Notification.IM` | Instant message tone (conversational) |
| `Notification.Reminder` | Calendar reminder (attention-grabbing) |
| `Notification.Looping.Alarm` | Alarm sound (urgent, plays once unless loop="true") |
| `Notification.SMS` | SMS received tone |
| `Notification.Mail` | Email received tone |

### Syntax
```xml
<audio src="ms-winsoundevent:Notification.Reminder"/>
```

For single-play alarm (no looping):
```xml
<audio src="ms-winsoundevent:Notification.Looping.Alarm" loop="false"/>
```

### CONTEXT.md D-12 mapping (validated):
- Task Complete → `Notification.Default` ✓ (light)
- Permission Request → `Notification.Reminder` ✓ (attention-grabbing)
- Question → `Notification.IM` ✓ (conversational)
- Error → `Notification.Looping.Alarm` with `loop="false"` ✓ (urgent, single play)

## 3. Rust Asset Embedding Pattern

### `include_bytes!()` approach
```rust
const HERO_TASK_COMPLETE: &[u8] = include_bytes!("../assets/hero-task-complete.png");
const HERO_PERMISSION: &[u8] = include_bytes!("../assets/hero-permission.png");
const HERO_QUESTION: &[u8] = include_bytes!("../assets/hero-question.png");
const HERO_ERROR: &[u8] = include_bytes!("../assets/hero-error.png");
```

### Extraction pattern
```rust
use std::fs;
use std::path::PathBuf;

fn assets_dir() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| std::env::var("APPDATA").unwrap_or_default());
    PathBuf::from(local_app_data).join("claude-win-notify").join("assets")
}

fn ensure_asset(filename: &str, data: &[u8]) -> PathBuf {
    let dir = assets_dir();
    let path = dir.join(filename);
    if !path.exists() {
        fs::create_dir_all(&dir).ok();
        fs::write(&path, data).ok();
    }
    path
}
```

### Key considerations
- `include_bytes!()` is zero-cost at runtime (data baked into binary's .rodata section)
- Binary size impact: 4 PNG files × ~15-20KB each = ~60-80KB total. Negligible vs 15MB limit.
- Extraction happens once on first run (subsequent runs check file existence only — near-zero overhead)
- If extraction fails (permissions issue), toast shows without hero image (graceful degradation matches D-09)

## 4. Notification Type Classification Architecture

### Current flow (Phase 2)
```
stdin JSON → parse HookInput → match hook_event_name → show_toast(title, body, attribution)
```

### Proposed flow (Phase 3)
```
stdin JSON → parse HookInput → classify(input) → NotificationType enum
    → show_typed_toast(type, body, project_name)
        → resolve hero image path
        → select audio src
        → build XML with hero + audio
        → show toast
```

### NotificationType enum design
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    TaskComplete,
    PermissionRequest,
    Question,
    Error,
}

impl NotificationType {
    pub fn title(&self) -> &'static str {
        match self {
            Self::TaskComplete => "✔ Task Complete",
            Self::PermissionRequest => "🔐 Permission Required",
            Self::Question => "❓ Question",
            Self::Error => "⚠ Error",
        }
    }

    pub fn audio_src(&self) -> &'static str {
        match self {
            Self::TaskComplete => "ms-winsoundevent:Notification.Default",
            Self::PermissionRequest => "ms-winsoundevent:Notification.Reminder",
            Self::Question => "ms-winsoundevent:Notification.IM",
            Self::Error => "ms-winsoundevent:Notification.Looping.Alarm",
        }
    }

    pub fn hero_filename(&self) -> &'static str {
        match self {
            Self::TaskComplete => "hero-task-complete.png",
            Self::PermissionRequest => "hero-permission.png",
            Self::Question => "hero-question.png",
            Self::Error => "hero-error.png",
        }
    }
}
```

### Classification logic (from CONTEXT.md decisions D-01 to D-19)
```rust
fn classify_stop(input: &HookInput) -> NotificationType {
    // D-13: Stop + stop_hook_active → no notification (handled before calling this)
    
    // D-14: Error detection (highest priority)
    if let Some(msg) = &input.last_assistant_message {
        let lower = msg.to_lowercase();
        if ERROR_PATTERNS.iter().any(|p| lower.contains(p)) {
            return NotificationType::Error;
        }
    }

    // D-15: Question detection (second priority)  
    if let Some(msg) = &input.last_assistant_message {
        if msg.trim().ends_with('?') {
            return NotificationType::Question;
        }
    }

    // D-16: Default
    NotificationType::TaskComplete
}

fn classify_notification(input: &HookInput) -> NotificationType {
    // D-17: Permission Request
    if input.notification_type.as_deref() == Some("permission_prompt") {
        return NotificationType::PermissionRequest;
    }
    // D-19: Fallback for missing notification_type field (bug #11964)
    if let Some(msg) = &input.message {
        if msg.to_lowercase().contains("permission") {
            return NotificationType::PermissionRequest;
        }
    }
    // D-18: Any other notification_type → Question
    NotificationType::Question
}

const ERROR_PATTERNS: &[&str] = &[
    "api rate limit",
    "rate limit exceeded",
    "session limit",
    "context window",
    "api error",
    "authentication failed",
];
```

## 5. Testing Strategy

### Unit-testable without WinRT
The classification logic is pure function (HookInput → NotificationType). All tests can run without Windows Toast system:

```rust
#[test]
fn stop_with_error_pattern_classifies_as_error() {
    let input = HookInput { last_assistant_message: Some("API rate limit hit".into()), .. };
    assert_eq!(classify_stop(&input), NotificationType::Error);
}

#[test]
fn stop_with_question_mark_classifies_as_question() {
    let input = HookInput { last_assistant_message: Some("Should I continue?".into()), .. };
    assert_eq!(classify_stop(&input), NotificationType::Question);
}

#[test]
fn notification_with_permission_prompt_classifies_correctly() {
    let input = HookInput { notification_type: Some("permission_prompt".into()), .. };
    assert_eq!(classify_notification(&input), NotificationType::PermissionRequest);
}
```

### Integration test approach
- Test the full Toast XML generation (string comparison, no WinRT call)
- Test asset extraction to a temp directory
- Mock the toast display for CI (feature-gate actual WinRT calls)

## 6. Module Structure

### New modules
| Module | Responsibility |
|--------|---------------|
| `src/notification.rs` | `NotificationType` enum, classification functions, body text generation |
| `src/assets.rs` | `include_bytes!()` constants, `ensure_asset()`, `assets_dir()` |

### Modified modules
| Module | Changes |
|--------|---------|
| `src/toast.rs` | New `show_typed_toast(type, body, attribution)` with hero + audio params |
| `src/hook.rs` | Replace inline logic with `classify_stop()` / `classify_notification()` calls |
| `src/lib.rs` | Add `pub mod notification;` and `pub mod assets;` |
| `src/main.rs` | No changes needed |

## 7. Hero Image Generation

### Approach
Generate 4 simple PNG files (364×180) at development time:
- `hero-task-complete.png` — Green gradient/checkmark theme
- `hero-permission.png` — Orange/amber lock theme
- `hero-question.png` — Blue question mark theme
- `hero-error.png` — Red warning theme

### Options for generating
1. **Manual creation** — Use any image editor, keep simple geometric designs
2. **Build-time generation** — Use `image` crate in build.rs (adds compile dependency)
3. **Pre-committed assets** — Simplest, store in `assets/` directory (recommended)

**Recommendation:** Option 3 (pre-committed assets). Store in `assets/` directory, reference via `include_bytes!("../assets/...")`. Total size ~60-80KB is negligible.

## 8. Toast XML Template (Phase 3)

Final XML template with all Phase 3 features:
```xml
<toast>
  <visual>
    <binding template="ToastGeneric">
      <text>{title}</text>
      <text>{body}</text>
      <text placement="attribution">{project_name}</text>
      <image placement="hero" src="file:///{hero_image_path}"/>
    </binding>
  </visual>
  <audio src="ms-winsoundevent:{audio_event}"{loop_attr}/>
</toast>
```

Where:
- `{hero_image_path}` uses forward slashes in the file:/// URI (Windows accepts both but forward is canonical)
- `{loop_attr}` is empty string for all types except Error which adds ` loop="false"` (explicit non-looping for alarm sound)

## 9. Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Hero image file not found at runtime | Graceful degradation — toast shows without image |
| %LOCALAPPDATA% not writable | Log warning, continue without hero images |
| Unknown notification_type values from future Claude Code versions | D-18: treat as Question (safe default) |
| Error pattern false positives | D-03: Conservative list only matches system-level patterns |
| Audio src typo causes silent notification | Test all 4 audio values manually on Windows 10 + 11 |

## Validation Architecture

### Test Categories
1. **Unit tests** — Classification logic (all D-01 to D-19 decision boundaries)
2. **Integration tests** — Toast XML generation (string validation)
3. **Asset tests** — File extraction to temp dir, existence checks
4. **Manual smoke tests** — Visual verification of all 4 toast types on Windows

### Coverage targets
- Classification: 100% branch coverage (all decision paths)
- Toast XML: Template correctness for each notification type
- Assets: Extract, check existence, skip re-extraction

---

## RESEARCH COMPLETE
