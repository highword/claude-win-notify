# Architecture Patterns

**Domain:** Windows-native Claude Code notification plugin (CLI tool extension)
**Researched:** 2026-05-15

## Recommended Architecture

### High-Level Overview

```
+------------------+       stdin JSON        +-------------------+
|   Claude Code    | ----------------------> |  Hook Entrypoint  |
|   (Hook Events)  |                         |  (single exe)     |
+------------------+                         +--------+----------+
                                                      |
                                                      v
                                             +--------+----------+
                                             |   Event Router    |
                                             |  (Stop/Notif/etc) |
                                             +--------+----------+
                                                      |
                              +-------------+---------+---------+-------------+
                              |             |                   |             |
                              v             v                   v             v
                    +---------+---+ +-------+-----+  +---------+---+ +-------+------+
                    | Transcript  | |   Dedup     |  |   State     | |   Config     |
                    | Analyzer    | |   Manager   |  |   Manager   | |   Loader     |
                    | (JSONL)     | | (lock file) |  | (cooldown)  | | (JSON/TOML)  |
                    +------+------+ +------+------+  +------+------+ +--------------+
                           |               |                |
                           v               v                v
                    +------+----------------------------------------------+
                    |                  Notification Engine                  |
                    |  +------------+  +------------+  +--------------+   |
                    |  | Toast Send |  | Summary Gen|  | Click-Focus  |   |
                    |  | (WinRT)    |  | (text trim)|  | (Win32 API)  |   |
                    |  +------------+  +------------+  +--------------+   |
                    +-----------------------------------------------------+
```

### Component Boundaries

| Component | Responsibility | Communicates With | Build Phase |
|-----------|---------------|-------------------|-------------|
| **Hook Entrypoint** (main) | Parse stdin JSON, route to handler | Event Router, Config Loader | Phase 1 |
| **Event Router** | Dispatch by hook_event_name, orchestrate pipeline | All downstream components | Phase 1 |
| **Config Loader** | Load/validate config from disk, provide defaults | All components (read-only) | Phase 1 |
| **Transcript Analyzer** | Parse JSONL, run state machine to detect notification type | Event Router (called by) | Phase 2 |
| **Dedup Manager** | Two-phase lock to prevent duplicate notifications | Event Router (called by), filesystem | Phase 2 |
| **State Manager** | Per-session cooldown tracking, state persistence | Event Router (called by), filesystem | Phase 2 |
| **Toast Sender** | Construct XML, register AUMID, send WinRT toast | Notification Engine | Phase 1 |
| **Summary Generator** | Extract meaningful text from transcript for notification body | Notification Engine | Phase 2 |
| **Click-to-Focus** | Find terminal window, activate it, switch tab | Toast Sender (callback) | Phase 3 |
| **Webhook Sender** | HTTP POST to Slack/Discord/custom (async, fire-and-forget) | Notification Engine | Phase 4+ |

## Data Flow

### Primary Flow: Hook Event to Notification

```
1. Claude Code fires hook event (Stop, Notification, SubagentStop)
2. Claude Code spawns our exe, pipes JSON to stdin:
   {
     "session_id": "abc123",
     "transcript_path": "/path/to/transcript.jsonl",
     "cwd": "/current/working/dir",
     "permission_mode": "ask",
     "hook_event_name": "Stop",
     "reason": "Task appears complete"
   }
3. Hook Entrypoint reads stdin, deserializes JSON
4. Config Loader provides settings (cooldown, sound prefs, etc.)
5. Event Router determines flow based on hook_event_name:
   - "Stop" -> analyze transcript for task type
   - "Notification" -> direct notification (Claude's own notification event)
   - "SubagentStop" -> may aggregate or notify with agent context
6. Dedup Manager Phase 1: check lock file age < 2s -> if exists, exit
7. Transcript Analyzer: parse JSONL from transcript_path
   - Read last N lines (tail-read, not full load)
   - State machine determines: task_complete | question | api_error | review_complete
8. Dedup Manager Phase 2: acquire lock
9. State Manager: check cooldown -> if within cooldown, exit
10. Summary Generator: extract last assistant message, clean/truncate
11. Toast Sender: construct notification XML, show via WinRT
12. State Manager: update last notification timestamp
13. Process exits (< 500ms total)
```

### Click-to-Focus Flow (on toast click)

```
1. User clicks toast notification
2. Windows activates our exe via protocol activation (or COM)
   - Protocol: claude-notify://focus?session=abc123&terminal=warp&pid=1234
3. Click-to-Focus module:
   a. Read activation arguments (session_id, terminal type, PID)
   b. FindWindow/EnumWindows to locate terminal window by PID or title
   c. For Windows Terminal: UI Automation to find correct tab
   d. SetForegroundWindow (requires AllowSetForegroundWindow trick)
   e. For tab switching: SendInput or UI Automation SelectionItemPattern
4. Process exits
```

### Configuration File Location

```
%USERPROFILE%\.claude-win-notify\config.toml   (primary, user-level)
%USERPROFILE%\.claude\settings.json            (hooks registration lives here)
```

## Patterns to Follow

### Pattern 1: Fire-and-Forget Hook Execution

**What:** Hook process must start fast, execute, and exit. Claude Code does NOT wait for hook output on Notification events (it's informational).

**When:** Every hook invocation.

**Why:** Claude Code spawns the hook as a child process. Long-running hooks block the agent for Stop events. Target: entire execution < 500ms.

**Implementation:**
```rust
fn main() {
    // 1. Read stdin (blocking, but data is small ~1KB)
    let input: HookInput = serde_json::from_reader(std::io::stdin()).unwrap_or_else(|_| exit(0));
    
    // 2. Load config (cached in memory-mapped file or fast TOML parse)
    let config = Config::load_or_default();
    
    // 3. Quick checks (dedup, cooldown) - exit early if suppressed
    if dedup::is_duplicate(&input.session_id) { return; }
    if state::in_cooldown(&input.session_id, &config) { return; }
    
    // 4. Analyze and notify (the meat)
    let status = analyzer::determine_status(&input, &config);
    let summary = summary::generate(&input.transcript_path, &status);
    toast::send(&config, &status, &summary, &input);
    
    // 5. Update state and exit
    state::record_notification(&input.session_id);
}
```

### Pattern 2: Protocol Activation for Click-to-Focus

**What:** Register a custom URI protocol (`claude-notify://`) so Windows can launch our exe when a toast is clicked, passing arguments.

**When:** User clicks a toast notification.

**Why:** COM activation requires a registered COM server (complex setup, needs registry changes). Protocol activation is simpler: just a registry key pointing to our exe. Works without background services.

**Implementation approach:**
```
Registry entry:
HKCU\Software\Classes\claude-notify\shell\open\command
  (Default) = "C:\path\to\claude-win-notify.exe" --focus "%1"

Toast XML uses protocol activation:
<toast activationType="protocol" launch="claude-notify://focus?session=abc&pid=1234&terminal=wt">
```

**Trade-off vs COM activation:**
- Protocol: simpler install, no COM registration, but spawns new process on click
- COM: can reuse running process, but requires CLSID registration, more complex install
- **Decision: Use protocol activation** -- simpler, matches single-exe goal, install is just a registry write

### Pattern 3: Tail-Read JSONL Transcript

**What:** Only read the last N KB of the transcript file, not the entire thing.

**When:** Analyzing transcript to determine notification type.

**Why:** Transcripts can grow very large (MB+). We only need the last few assistant messages to determine status. Full-load would blow our 500ms budget.

**Implementation:**
```rust
fn read_tail(path: &Path, max_bytes: usize) -> Vec<Message> {
    let file = File::open(path)?;
    let len = file.metadata()?.len() as usize;
    let offset = len.saturating_sub(max_bytes);
    
    file.seek(SeekFrom::Start(offset as u64))?;
    // Skip partial first line if we seeked into middle
    let reader = BufReader::new(file);
    let lines: Vec<_> = reader.lines()
        .skip(if offset > 0 { 1 } else { 0 })  // skip partial line
        .filter_map(|l| serde_json::from_str::<Message>(&l.ok()?).ok())
        .collect();
    lines
}
```

### Pattern 4: AUMID Registration for Branded Notifications

**What:** Register a custom Application User Model ID so notifications show "Claude Code" with our icon instead of "cmd.exe".

**When:** Installation time (one-time setup).

**Why:** Without AUMID, notifications show under "Command Prompt" or "PowerShell" branding. Custom AUMID gives proper app identity + enables Action Center persistence + enables click callbacks.

**Registry structure:**
```
HKCU\Software\Classes\AppUserModelId\ClaudeWinNotify
  DisplayName = "Claude Code Notifications"
  IconUri = "C:\path\to\icon.png"
  IconBackgroundColor = "#CC785C"  (Claude brand color)
```

### Pattern 5: Window Discovery via EnumWindows + Process Tree

**What:** Find the terminal window running the Claude Code session that triggered the notification.

**When:** Click-to-Focus activation.

**Why:** User may have multiple terminal windows open. We need to focus the RIGHT one. The hook input includes `cwd` and we can track the parent process PID.

**Strategy:**
1. Hook saves parent PID (claude process) and terminal window title at notification time
2. On click, use saved PID to walk process tree up to terminal process
3. EnumWindows to find window owned by terminal process
4. SetForegroundWindow (with AllowSetForegroundWindow workaround)

**Tab-level focus (Windows Terminal):**
- Windows Terminal exposes tab titles in its accessibility tree
- Use UI Automation: find TabItem elements, match by working directory or session name
- Call SelectionItemPattern.Select() on the correct tab

**Tab-level focus (Warp):**
- Warp on Windows: research needed on accessibility tree exposure
- Fallback: window-level focus only (still better than nothing)

## Anti-Patterns to Avoid

### Anti-Pattern 1: Background Service / Daemon

**What:** Running a persistent background process that watches for notifications.

**Why bad:** 
- Adds resource consumption even when not needed
- Requires service registration or startup entries (enterprise-hostile)
- Single-exe goal violated (need service installer + manager)
- Claude Code hooks are designed for on-demand invocation, not polling

**Instead:** Pure on-demand execution. Each hook event spawns our exe, it runs < 500ms, exits. No persistent state in memory.

### Anti-Pattern 2: Full Transcript Load

**What:** Reading the entire JSONL transcript file into memory.

**Why bad:** Transcripts can be 10+ MB for long sessions. Parsing all of it is wasteful when we only need the last few messages to determine state.

**Instead:** Tail-read pattern (Pattern 3). Seek to last 32KB, parse only recent messages.

### Anti-Pattern 3: COM Server for Click Activation

**What:** Registering a COM server class for toast activation callbacks.

**Why bad:**
- Requires CLSID registration in registry (more complex install)
- COM activation expects a running process or registered out-of-proc server
- Conflicts with single-exe, zero-dependency goal
- Enterprise environments may restrict COM registration

**Instead:** Protocol activation (Pattern 2). Simpler registry entry, same end result.

### Anti-Pattern 4: Blocking on Webhook Delivery

**What:** Waiting for HTTP webhook response before exiting hook process.

**Why bad:** Network calls add 100-2000ms latency. Hook must exit fast.

**Instead:** For webhooks, spawn a detached process or use non-blocking fire-and-forget (with timeout). Accept that delivery is best-effort from the hook's perspective.

### Anti-Pattern 5: Global Mutex for Dedup

**What:** Using a named Windows mutex for deduplication across processes.

**Why bad:** Mutex requires both processes to be alive simultaneously. Claude Code fires Stop hooks in rapid succession (known bug: fires 2-3 times), and our process starts/exits quickly.

**Instead:** File-based lock with timestamp (Pattern from claude-notifications-go). Check lock file age < 2s = duplicate. Simple, reliable, works across process boundaries.

## Architecture Decisions

### Decision 1: Single Executable, On-Demand Invocation

Claude Code hooks spawn a process for each event. Our exe:
- Starts in ~10ms (NativeAOT or Rust)
- Reads stdin (~1ms, small JSON)
- Does work (~100-300ms: parse transcript, send toast)
- Exits

No daemon. No IPC. No socket. Each invocation is stateless except for:
- Config file on disk (read-only)
- Lock files for dedup (temp dir)
- State files for cooldown (temp dir)

### Decision 2: Protocol Activation over COM

Toast notifications support three activation types:
1. **Foreground** -- only works if app is already running with message loop
2. **Background (COM)** -- requires registered COM class, complex
3. **Protocol** -- launches exe with custom URI scheme, simple registry entry

We use **protocol activation** because:
- One registry key to set up
- Works with single-exe model (exe is launched fresh on click)
- Arguments passed via URI query string
- No background process needed
- Install/uninstall is trivial

### Decision 3: Hybrid AUMID Strategy

- **Phase 1 (MVP):** Use Windows Terminal's AUMID (`Microsoft.WindowsTerminal_8wekyb3d8bbwe!App`) or PowerShell's built-in AUMID for quick notifications. Limits: no custom icon, attribution text workaround.
- **Phase 2:** Register custom AUMID during install for branded notifications. Requires Start Menu shortcut with AppUserModelID property (the Windows-blessed way for non-UWP apps).

### Decision 4: State in Temp Directory

All mutable state lives in `%TEMP%\claude-win-notify\`:
```
%TEMP%\claude-win-notify\
  locks\
    {session_id}.lock          # Dedup lock files (2s TTL)
  state\
    {session_id}.json          # Last notification timestamp, window info
  cache\
    focus-context.json         # Terminal PID + window handle for click-to-focus
```

Why temp:
- Auto-cleaned on reboot (no stale state accumulation)
- No elevated permissions needed
- Fast filesystem access
- Enterprise-friendly (no AppData pollution that syncs via roaming profiles)

### Decision 5: Hook Registration Strategy

Claude Code hooks are defined in `%USERPROFILE%\.claude\settings.json` (global) or `.claude/settings.json` (project). Our installer writes to global settings:

```json
{
  "hooks": {
    "Stop": [
      {
        "matcher": "*",
        "hooks": [{
          "type": "command",
          "command": "claude-win-notify.exe hook",
          "timeout": 5
        }]
      }
    ],
    "Notification": [
      {
        "matcher": "*",
        "hooks": [{
          "type": "command",
          "command": "claude-win-notify.exe hook",
          "timeout": 5
        }]
      }
    ],
    "SubagentStop": [
      {
        "matcher": "*",
        "hooks": [{
          "type": "command",
          "command": "claude-win-notify.exe hook",
          "timeout": 5
        }]
      }
    ]
  }
}
```

The exe is placed in PATH (e.g., `%USERPROFILE%\.claude-win-notify\bin\`) and the installer adds it to user PATH.

## Component Details

### Transcript Analyzer: State Machine

Status detection logic (adapted from claude-notifications-go, validated against their architecture):

```
Priority order (first match wins):
1. Check for "session limit reached" pattern -> StatusSessionLimitReached
2. Check for API error patterns -> StatusAPIError / StatusAPIErrorOverloaded
3. Check for pending tool approval (PreToolUse context) -> StatusPermissionRequest
4. Check last assistant message ends with "?" -> StatusQuestion
5. Check for plan/review mode exit -> StatusReviewComplete / StatusPlanReady
6. Default on Stop event -> StatusTaskComplete
```

Fields from JSONL that matter:
- `role`: "assistant" | "user" | "tool"
- `type`: "text" | "tool_use" | "tool_result"
- `content[].text`: actual message text (for question detection)
- `content[].name`: tool name (for plan mode detection)
- `stop_reason`: why the model stopped

### Toast Sender: XML Template

Windows Toast notifications use XML-based content:

```xml
<toast activationType="protocol" launch="claude-notify://focus?session={id}&amp;pid={pid}">
  <visual>
    <binding template="ToastGeneric">
      <text>Task Complete</text>
      <text>{summary_text}</text>
      <text placement="attribution">Claude Code - {project_name}</text>
    </binding>
  </visual>
  <actions>
    <action content="Focus Terminal" activationType="protocol" 
            arguments="claude-notify://focus?session={id}&amp;pid={pid}"/>
    <action content="Dismiss" activationType="system" arguments="dismiss"/>
  </actions>
  <audio src="ms-winsoundevent:Notification.Default"/>
</toast>
```

### Click-to-Focus: Window Activation Strategy

Windows has strict rules about SetForegroundWindow (to prevent focus stealing). A process can only set foreground if:
1. It IS the foreground process, OR
2. The foreground process has called AllowSetForegroundWindow for our PID, OR
3. The user has interacted with us (e.g., clicked our notification)

**Protocol activation from toast click counts as user interaction** -- Windows grants us foreground rights because the activation came from user clicking the notification. This is why protocol activation is ideal for click-to-focus.

After getting foreground rights:
```rust
// Pseudocode
fn focus_terminal(pid: u32, terminal_type: &str) {
    // 1. Find window by process
    let hwnd = find_window_by_pid(pid);  // EnumWindows + GetWindowThreadProcessId
    
    // 2. Restore if minimized
    if is_iconic(hwnd) { show_window(hwnd, SW_RESTORE); }
    
    // 3. Bring to foreground
    set_foreground_window(hwnd);
    
    // 4. Tab-level focus (terminal-specific)
    match terminal_type {
        "windows_terminal" => focus_wt_tab(hwnd, session_cwd),
        "warp" => focus_warp_tab(hwnd, session_cwd),
        _ => {} // window-level only
    }
}
```

## Build Order (Dependency Graph)

```
Phase 1: Foundation (no dependencies)
  ├── Hook Entrypoint (stdin parse + route)
  ├── Config Loader (TOML parse + defaults)
  └── Toast Sender (basic notification, no actions)
      
Phase 2: Intelligence (depends on Phase 1)
  ├── Transcript Analyzer (JSONL state machine)
  ├── Summary Generator (text extraction)
  ├── Dedup Manager (lock files)
  └── State Manager (cooldown)

Phase 3: Interaction (depends on Phase 1 Toast)
  ├── Protocol Activation handler
  ├── Window Discovery (EnumWindows)
  ├── SetForegroundWindow logic
  └── Tab-level focus (UI Automation)

Phase 4: Polish (depends on Phases 2+3)
  ├── Custom AUMID registration
  ├── Interactive toast actions (Approve/Deny buttons)
  ├── Notification aggregation (multi-subagent)
  └── Webhook sender (Slack/Discord)

Phase 5: Distribution
  ├── PowerShell installer script
  ├── Code signing
  ├── winget/scoop manifests
  └── Auto-updater
```

**Critical path:** Phase 1 -> Phase 3 (Click-to-Focus is the differentiator, should be proven early)

**Parallel work possible:** Phase 2 (intelligence) can develop alongside Phase 3 (interaction) since they share only the Phase 1 foundation.

## Scalability Considerations

| Concern | At 1 user | At 100 users | At 10K users |
|---------|-----------|--------------|--------------|
| Notification storm | Dedup + cooldown | Same | Same (per-machine) |
| Transcript size | Tail-read 32KB | Same | Same |
| Multi-session | Per-session state files | Same | Cleanup on reboot |
| Enterprise deploy | Manual install | PowerShell + GPO | winget/Intune |
| Config management | Local TOML | Same | GPO-managed TOML path |

## Sources

- Claude Code hooks documentation: https://github.com/anthropics/claude-code (via Context7, HIGH confidence)
- claude-notifications-go architecture: https://github.com/777genius/claude-notifications-go/blob/main/docs/ARCHITECTURE.md (via Context7, HIGH confidence)
- Windows Toast notification API (Rust windows crate): https://microsoft.github.io/windows-docs-rs/doc/windows/UI/Notifications/ (via Context7, HIGH confidence)
- Windows Toast AUMID requirements: https://windows-toasts.readthedocs.io/ (via Context7, HIGH confidence)
- Claude Code settings location: `%USERPROFILE%\.claude\settings.json` for global hooks (via Context7 zebbern guide, HIGH confidence)
- Windows SetForegroundWindow restrictions: Win32 API documentation (training data, MEDIUM confidence)
- Protocol activation for toast: Windows documentation pattern (training data, MEDIUM confidence -- needs validation with actual Rust windows crate implementation)
- UI Automation for tab switching: Windows accessibility framework (training data, MEDIUM confidence -- needs Phase 3 spike)
