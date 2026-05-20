# Phase 2: Hook & Toast Foundation - Research

**Researched:** 2026-05-20
**Phase:** 02 — Hook & Toast Foundation
**Requirements:** NOTIF-01, TOAST-01, TOAST-02, TECH-03, TECH-04, INST-06, INST-08

## 1. Claude Code Hooks Stdin JSON Schema

### Stop Hook Input (Primary Target)

```json
{
  "session_id": "abc123",
  "transcript_path": "~/.claude/projects/.../00893aaf-19fa-41d2-8238-13269b9b3ca0.jsonl",
  "cwd": "/Users/...",
  "permission_mode": "default",
  "hook_event_name": "Stop",
  "stop_hook_active": true,
  "last_assistant_message": "I've completed the refactoring. Here's a summary..."
}
```

**Fields:**
- `session_id` (string) — unique session identifier
- `transcript_path` (string) — path to JSONL transcript file
- `cwd` (string) — current working directory (may contain CJK characters)
- `permission_mode` (string) — permission mode setting
- `hook_event_name` (string) — event type identifier ("Stop", "Notification", etc.)
- `stop_hook_active` (boolean) — true if Claude is already continuing from a stop hook (prevent infinite loops)
- `last_assistant_message` (string) — final response text from Claude

### Notification Hook Input (Secondary Target)

```json
{
  "session_id": "abc123",
  "transcript_path": "/Users/.../.claude/projects/.../00893aaf.jsonl",
  "cwd": "/Users/...",
  "hook_event_name": "Notification",
  "message": "Claude needs your permission to use Bash",
  "title": "Permission needed",
  "notification_type": "permission_prompt"
}
```

**Fields (additional):**
- `message` (string) — notification message text
- `title` (string) — notification title
- `notification_type` (string) — type discriminator (e.g., "permission_prompt")

**Important:** Notification hooks cannot block or modify notifications. They are fire-and-forget side effects.

### Common Fields (All Hook Events)

All hook events share: `session_id`, `transcript_path`, `cwd`, `hook_event_name`.
Additional fields vary by event type.

### hooks.json Configuration Format

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-win-notify hook"
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-win-notify hook"
          }
        ]
      }
    ]
  }
}
```

**Key facts:**
- Configuration lives in `~/.claude/settings.json` (global) or `.claude/settings.json` (project)
- `matcher` field is optional — empty string matches all events of that type
- `type: "command"` runs a shell command
- `async: true` can be added to run hook in background without blocking Claude
- Hook receives JSON via stdin
- Exit code 0 = success (no effect on Claude); Exit code 2 = special behavior (Stop hook: force continue)

## 2. Rust Project Architecture

### Dependencies (Cargo.toml)

```toml
[package]
name = "claude-win-notify"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dependencies.windows]
version = "0.62"
features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_System_Threading",
    "UI_Notifications",
    "Data_Xml_Dom",
]

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Module Layout

```
src/
├── main.rs          # Entry point: clap CLI → dispatch
├── lib.rs           # Public API for integration tests
├── cli.rs           # clap derive structs (Commands enum)
├── hook.rs          # Stdin JSON parsing + event routing
├── toast.rs         # Toast notification display
├── error.rs         # Error types (thiserror or manual)
└── log.rs           # Simple file-append logging
```

### CLI Design (clap derive)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claude-win-notify", version, about = "Windows notifications for Claude Code")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read hook JSON from stdin and show Toast notification
    Hook,
    /// Focus a terminal window (Protocol activation)
    Focus {
        /// Protocol URI (claude-notify://focus?session=...&pid=...)
        uri: String,
    },
}
```

### Stdin JSON Parsing (serde)

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HookInput {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub hook_event_name: String,
    // Optional fields (present in some events)
    pub stop_hook_active: Option<bool>,
    pub last_assistant_message: Option<String>,
    pub message: Option<String>,
    pub title: Option<String>,
    pub notification_type: Option<String>,
}
```

**Strategy:** Use `#[serde(default)]` for optional fields. Unknown fields are ignored by default with serde (no `deny_unknown_fields`). This makes the parser forward-compatible with new Claude Code hook fields.

## 3. Toast Notification Implementation

### Toast XML Template

```xml
<toast>
  <visual>
    <binding template="ToastGeneric">
      <text>Claude Code</text>
      <text>✔ Task Complete</text>
      <text placement="attribution">claude-win-notify</text>
    </binding>
  </visual>
  <audio src="ms-winsoundevent:Notification.Default"/>
</toast>
```

**Notes:**
- `<text>` elements: 1st = title, 2nd = body, 3rd with `placement="attribution"` = attribution line
- Attribution text shows the project name (from cwd last component)
- CJK characters work directly in Toast XML — Windows handles Unicode natively
- Special XML characters in project names (`<`, `>`, `&`, `"`, `'`) MUST be escaped

### AUMID (Application User Model ID)

Phase 2 borrows PowerShell's AUMID:
```
{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe
```

This avoids needing a Start Menu shortcut for now (Phase 7 creates proper AUMID).

### WinRT API Usage (windows-rs 0.62)

```rust
use windows::core::*;
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

fn show_toast(title: &str, body: &str, attribution: &str) -> Result<()> {
    let aumid = "...";
    let xml_string = format!(r#"<toast>
      <visual>
        <binding template="ToastGeneric">
          <text>{}</text>
          <text>{}</text>
          <text placement="attribution">{}</text>
        </binding>
      </visual>
      <audio src="ms-winsoundevent:Notification.Default"/>
    </toast>"#, escape_xml(title), escape_xml(body), escape_xml(attribution));

    let toast_xml = XmlDocument::new()?;
    toast_xml.LoadXml(&HSTRING::from(xml_string))?;
    let toast = ToastNotification::CreateToastNotification(&toast_xml)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(aumid))?;
    notifier.Show(&toast)?;
    Ok(())
}
```

**XML Escaping (critical for CJK paths with special chars):**
```rust
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}
```

## 4. Logging Strategy

For a fire-and-forget CLI, heavy logging frameworks (tracing, env_logger) are overkill. Simple file-append is sufficient:

```rust
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub fn log_error(msg: &str) {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude-win-notify")
        .join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_file = log_dir.join("error.log");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&log_file) {
        let _ = writeln!(f, "[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), msg);
    }
}
```

**Log location:** `%LOCALAPPDATA%\claude-win-notify\logs\error.log`

**Consideration:** `chrono` adds ~200KB. Alternative: use `std::time::SystemTime` with manual formatting to avoid the dependency.

**Log rotation:** Simple size check — if file > 1MB, truncate to last 500KB on startup. No external dependency needed.

## 5. Binary Size Estimate

| Component | Estimated Size |
|-----------|---------------|
| Rust std (stripped) | ~300 KB |
| windows-rs (Toast + Win32) | ~200 KB |
| clap (derive) | ~300 KB |
| serde + serde_json | ~200 KB |
| Application code | ~50 KB |
| **Total (with LTO + strip + opt-z)** | **~1.0-1.5 MB** |

Well under the 15MB requirement. The spike was 0.38MB without clap/serde.

**`panic = "abort"`** saves ~100KB by removing unwinding machinery.

## 6. Windows Compatibility (TECH-03)

### Windows 10 1903+ (Build 18362)
- WinRT Toast API (`Windows.UI.Notifications`) available since Windows 8.1
- `ToastGeneric` template available since Windows 10 Anniversary Update (1607)
- Attribution text in Toast available since Windows 10 Creators Update (1703)
- All target features are safe on 1903+

### Potential Issues
- No issues expected with WinRT Toast on supported versions
- `SetForegroundWindow` behavior unchanged across target range (Phase 4)
- Windows 11 uses the same WinRT Toast API (backward compatible)

## 7. CJK/Unicode Handling (TECH-04)

- **Stdin:** Rust reads stdin as raw bytes; `serde_json` handles UTF-8 natively
- **Toast XML:** Windows Toast renders Unicode correctly. Only need XML entity escaping
- **File paths in transcript_path/cwd:** May use forward slashes (from WSL) or backslashes. Use `std::path::PathBuf` for path manipulation
- **Project name extraction:** `Path::new(cwd).file_name()` handles Unicode correctly

## 8. Testing Approach

### Unit Tests (in-module)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stop_hook() {
        let json = r#"{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"D:\\项目\\测试","hook_event_name":"Stop","stop_hook_active":false}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.hook_event_name, "Stop");
        assert_eq!(input.cwd, "D:\\项目\\测试");
    }

    #[test]
    fn parse_notification_hook() {
        let json = r#"{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"/home/user/project","hook_event_name":"Notification","message":"Permission needed","title":"Bash","notification_type":"permission_prompt"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.notification_type, Some("permission_prompt".to_string()));
    }

    #[test]
    fn unknown_event_is_ok() {
        let json = r#"{"session_id":"abc","transcript_path":"/tmp/t.jsonl","cwd":"/tmp","hook_event_name":"PreToolUse","tool_name":"Bash"}"#;
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.hook_event_name, "PreToolUse");
    }
}
```

### Integration Tests (tests/)
```rust
// tests/integration.rs
use std::process::Command;

#[test]
fn hook_with_valid_stop_json() {
    let output = Command::new("cargo")
        .args(["run", "--", "hook"])
        .stdin(/* pipe JSON */)
        .output()
        .expect("failed to run");
    assert_eq!(output.status.code(), Some(0));
}
```

### stdin Pipe Manual Testing
```powershell
echo '{"session_id":"test","transcript_path":"C:\\Users\\test\\.claude\\transcript.jsonl","cwd":"D:\\Repository\\claude-win-notify","hook_event_name":"Stop","stop_hook_active":false,"last_assistant_message":"Done"}' | cargo run -- hook
```

## 9. Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Toast fails silently on some Windows configs | Low | Log error, exit 0 gracefully |
| Large binary with clap+serde | Low | LTO+strip keeps it under 2MB |
| stdin read hangs if no input | Medium | Set read timeout or detect empty stdin |
| XML injection via project name | Medium | escape_xml() function |
| Permission mode field missing in older Claude Code | Low | Use Option<String> for non-essential fields |

### stdin Hang Mitigation
If the exe is invoked without stdin (e.g., user runs it manually), `stdin().read_to_string()` will hang. Solution:
- Check if stdin is a TTY: if TTY, print help and exit
- Or: set a short timeout (1 second) for stdin read

```rust
use std::io::IsTerminal;
if std::io::stdin().is_terminal() {
    eprintln!("Error: claude-win-notify hook expects JSON input via stdin.");
    eprintln!("This command is meant to be invoked by Claude Code hooks.");
    std::process::exit(1);
}
```

## 10. Validation Architecture

### Dimension 1: Unit Tests
- JSON parsing for all hook event types
- XML escaping correctness
- Project name extraction from various path formats (Windows, Unix, CJK)
- Unknown fields handled gracefully

### Dimension 2: Integration Tests
- End-to-end stdin pipe → exit code verification
- Binary size assertion (< 15MB in CI)
- CJK path handling

### Dimension 3: Manual Validation
- Toast actually appears on screen
- Toast content is correct (title, body, attribution)
- Latency < 500ms (measured with Measure-Command in PowerShell)
- Dogfooding via real hooks.json

---

## RESEARCH COMPLETE
