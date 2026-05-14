# Feature Landscape

**Domain:** Claude Code notification tool (Windows-first)
**Researched:** 2026-05-15
**Overall confidence:** MEDIUM-HIGH (Microsoft docs verified; competitive intelligence from training data)

## Competitive Landscape Summary

### claude-notifications-go (617+ Stars, primary competitor)

The dominant player in this space. Features include:

| Feature | Details |
|---------|---------|
| Notification Types | 6 types: task complete, permission request, question asked, error/panic, sub-agent spawned, notification test |
| Detection Logic | JSONL transcript parsing + stdin JSON hook events |
| Platform Support | macOS (full), Linux (full), Windows (partial - no Click-to-Focus) |
| Audio | Custom sound playback per notification type |
| Webhook | Outgoing HTTP webhook for remote notifications |
| Cooldown/Dedup | Configurable cooldown period between notifications |
| Installation | `go install` + manual hooks.json injection |
| Click-to-Focus | macOS only (AppleScript). Linux partial (wmctrl). **Windows: explicitly unsupported** |
| Window Detection | Terminal PID tracking via process tree |
| Notification Content | Static templates per type (no AI summarization) |

### Other Tools

| Tool | Approach | Weakness |
|------|----------|----------|
| ClaudePulse | Python-based file watcher, polls JSONL | Heavy runtime (Python), no click-to-focus |
| claude-shadow | Node.js daemon, webhooks only | No native notifications |
| cc-paw | Browser extension overlay | Not terminal-native, Chrome dependency |
| claude-notify | Shell script wrapper | Fragile, no Windows support |

### Key Gaps in the Ecosystem (Our Opportunity)

1. **No tool delivers Click-to-Focus on Windows** - explicitly marked as unsupported
2. **No tool uses Windows Toast interactive features** (buttons, text input, progress bars)
3. **No tool provides AI-powered notification summaries** - all use static templates
4. **No tool handles multi-agent notification aggregation**
5. **Installation is painful everywhere** - requires manual hooks.json editing
6. **No tool is enterprise-ready** (code signing, policy compliance)

---

## Table Stakes

Features users expect. Missing = product feels incomplete compared to claude-notifications-go.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Task Complete notification | Core use case - "Claude is done, come back" | Low | Most requested feature by far |
| Permission Request notification | Security-critical - Claude needs approval to proceed | Low | Must be high-priority/urgent scenario |
| Error/Panic notification | User needs to know something failed | Low | Should use distinct visual treatment |
| Question notification | Claude is asking something, waiting for input | Low | Similar urgency to permission request |
| Click-to-Focus (window level) | **THE** differentiator but immediately table stakes once promised | High | SetForegroundWindow + ForegroundLockTimeout bypass |
| Cooldown/dedup | Without this, notification spam makes tool unusable | Low | Simple time-window dedup per notification type |
| Hooks system integration | The only supported way to receive Claude Code events | Medium | hooks.json configuration for PreToolUse/PostToolUse/Notification |
| JSONL transcript detection | Backup/enrichment method for notification intelligence | Medium | Watch ~/.claude/projects/*/sessions/*/transcript.jsonl |
| Sound/audio alert | Users expect audible cue alongside visual notification | Low | Windows system sounds or custom WAV |
| Configuration file | Users need to customize behavior | Low | TOML or JSON in ~/.config/claude-win-notify/ |
| One-command installation | Competitor pain point - manual hooks.json is confusing | Medium | PowerShell one-liner that downloads + configures |

---

## Differentiators

Features that set product apart. Not expected, but create competitive advantage.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Click-to-Focus (tab level) | Jump to exact tab in Windows Terminal/Warp, not just the window | Very High | Uses `wt.exe focus-tab -t {index}` for WT; UI Automation for Warp |
| Interactive Toast buttons | Approve/Deny permission directly FROM the notification | High | Windows Toast supports up to 5 buttons with activation callbacks |
| Quick-reply text input | Type response to Claude's question in the toast itself | High | Toast TextBox input + background activation handler |
| AI-powered notification summary | "Refactored auth module" not "Task complete" | Medium | Parse recent JSONL context, generate meaningful 1-liner with local heuristics |
| Progress bar in notification | Show multi-step task progress (e.g., "3/7 files processed") | Medium | Toast ProgressBar with data binding for live updates |
| Notification aggregation | Merge 5 sub-agent completions into 1 grouped notification | Medium | Toast Collections/Headers + time-window batching |
| Zero-dependency single exe | No Go/Python/Node runtime needed | Medium | NativeAOT or Rust produces truly standalone binary |
| PowerShell one-liner install | `irm url | iex` - 30-second setup, great for README | Low | Script downloads exe + patches hooks.json automatically |
| Enterprise code signing | Works in Device Guard / Intune environments | Medium | Requires certificate + signing pipeline (cost ~$200/yr) |
| MIT License | Enterprise-friendly vs competitor GPL-3.0 | None | Legal differentiator for corporate adoption |
| Multi-terminal support | Works across WT, Warp, VS Code terminal, ConEmu | High | Each terminal has different focus/tab mechanisms |
| Notification history/log | Review missed notifications after returning to desk | Low | SQLite or JSON append log with simple viewer |
| Mobile push (Bark/Pushover) | Long-running tasks - get notified on phone | Medium | HTTP POST to external service, opt-in feature |

---

## Anti-Features

Features to explicitly NOT build. These add complexity without proportional value.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Cross-platform support (macOS/Linux) | claude-notifications-go already dominates there; dilutes "Windows-first" positioning | Stay Windows-only, collaborate/reference claude-notifications-go for other platforms |
| Web dashboard / status UI | Maintenance burden, not the core value, adds attack surface | Notification history via simple log file or Notification Center |
| TTS / voice synthesis | Gimmicky, accessibility concerns, high complexity | Use distinct Windows system sounds per notification type |
| Windows Widget integration | Win11-only, API unstable (WidgetProvider is WinUI3-only), small user base | Rely on Toast notifications which work everywhere |
| Always-on system tray icon | Background process feels heavyweight, users dislike tray clutter | Run as hooks-triggered process that exits after notification delivery |
| Custom notification UI (WPF popup) | Reinventing the wheel, inconsistent with OS UX, breaks accessibility | Use native Windows Toast which integrates with Focus Assist, Do Not Disturb, etc. |
| Plugin/extension system | Over-engineering for v1, delays ship date | Expose configuration and webhooks instead |
| Slack/Teams/Discord integration | Scope creep, each platform has different auth/API | Provide webhook output that users can connect to Zapier/n8n |
| Auto-update mechanism | Complex, security implications, not needed for v1 | Ship via winget/scoop which handle updates |
| Monitoring multiple Claude instances | Adds state management complexity | Support single active instance, document multi-instance via separate configs |

---

## Feature Dependencies

```
Hooks Integration ──┬──> Task Complete Notification
                    ├──> Permission Request Notification  
                    ├──> Error Notification
                    └──> Question Notification

JSONL Transcript Parsing ──> AI Summary Generation
                         ──> Progress Detection

Click-to-Focus (Window) ──> Click-to-Focus (Tab)
                              └──> Terminal Process Association

Toast Notification System ──┬──> Interactive Buttons (Approve/Deny)
                            ├──> Text Input (Quick Reply)
                            ├──> Progress Bar
                            └──> Notification Aggregation (Collections)

One-command Install ──> hooks.json Auto-configuration
                   ──> PATH Registration

Code Signing ──> Enterprise Deployment
             ──> winget/scoop Publishing
```

---

## Windows Toast Capabilities (Verified from Microsoft Docs)

Based on official Windows App SDK documentation (updated 2026-04-21):

### Supported Interactive Elements
- **Up to 5 buttons** per notification (including context menu items)
- **Text input box** (quick reply pattern)
- **Selection/combo box** (dropdown menu)
- **Progress bar** with live data binding (updateable without re-sending notification)
- **Custom audio** (ms-appx, ms-appdata, or ms-winsoundevent URIs)
- **Hero image** (364x180 at 100% scale)
- **App logo override** with circle crop
- **Custom timestamps**
- **Notification headers** for grouping
- **Button styles**: Success (green), Critical (red) - Win11+
- **Tooltips** on buttons - Win11+

### Activation Scenarios
- **Reminder**: Stays on screen until dismissed, reminder sound
- **Alarm**: Loops audio, stays on screen
- **IncomingCall**: Full pre-expanded display, ringtone loop
- **Urgent**: Breaks through Focus Assist / Do Not Disturb

### Key Limitations
- **Not supported in elevated (admin) processes** - must run non-elevated
- Max 3 text lines (1 title + 2 body)
- Max 5 buttons total
- Images max 3MB (1MB on metered connections)
- Requires MSIX packaging OR COM server registration for activation handling
- Non-packaged apps need AUMID (App User Model ID) registration

### Relevant for Our Use Cases
| Claude Code Event | Toast Design |
|-------------------|-------------|
| Task Complete | Standard notification + "Focus Terminal" button |
| Permission Request | **Urgent** scenario + "Approve" / "Deny" buttons (green/red) |
| Question | Text input box + "Send" button for quick reply |
| Error | Standard notification with error icon + "Focus Terminal" button |
| Sub-agent spawned | Silent/grouped, no popup (use collection headers) |
| Progress | Progress bar with data binding, updated live |

---

## Windows Terminal Focus Capabilities (Verified from Microsoft Docs)

The `wt.exe` CLI provides:
- `wt -w {window-id} focus-tab -t {tab-index}` - Focus specific tab in specific window
- `--window, -w` accepts: integer ID, window name, `0` (most recent), `-1` (new window)
- Can target existing windows by name or ID without creating new ones

**Critical insight**: We can use `wt -w 0 focus-tab -t {N}` to switch to the correct tab if we can determine which tab Claude Code is running in. This requires:
1. Mapping Claude Code process PID to terminal tab
2. Determining tab index from process tree

For Warp: Warp on Windows uses a different mechanism - likely requires UI Automation or Warp's own CLI/API (needs further research in implementation phase).

---

## MVP Recommendation

### Phase 1: Core (Ship in < 2 weeks)
1. **Hooks integration** - Read Claude Code hook events
2. **4 notification types** - Complete, Permission, Question, Error
3. **Click-to-Focus (window level)** - SetForegroundWindow
4. **Sound alerts** - Distinct sounds per type
5. **PowerShell one-liner install** - Download + configure hooks.json

### Phase 2: Differentiation (Next 2-4 weeks)
1. **Click-to-Focus (tab level)** - Windows Terminal `focus-tab`
2. **Interactive buttons** - Approve/Deny on permission notifications
3. **AI summary** - Meaningful notification descriptions
4. **Cooldown/dedup** - Prevent notification fatigue

### Phase 3: Polish (4-8 weeks)
1. **Quick-reply text input** - Answer Claude from toast
2. **Progress bar** - Live task progress
3. **Notification aggregation** - Group sub-agent notifications
4. **Enterprise code signing**
5. **winget/scoop distribution**

### Defer to Later
- Mobile push (nice-to-have, not core)
- Multi-terminal support beyond WT + Warp (diminishing returns)
- Notification history viewer (Notification Center serves this role)

---

## Installation Experience Comparison

| Tool | Install Method | Pain Points |
|------|---------------|-------------|
| claude-notifications-go | `go install` + manual hooks.json edit | Requires Go toolchain; hooks.json path is confusing; no auto-detection |
| ClaudePulse | `pip install` + config file | Python dependency; config file location varies by OS |
| Our target | `irm https://url/install.ps1 \| iex` | Zero prerequisites; auto-detects Claude Code location; patches hooks.json |

**Key insight from user feedback**: The #1 complaint about existing tools is installation friction. A 30-second install experience is a massive differentiator.

---

## Sources

- Microsoft Learn: App Notifications Overview (updated 2026-05-08) - https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/
- Microsoft Learn: App Notification Content (updated 2026-04-21) - https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/app-notifications-content
- Microsoft Learn: App Notification Progress Bar (updated 2026-04-21) - https://learn.microsoft.com/en-us/windows/apps/develop/notifications/app-notifications/app-notifications-progress-bar
- Microsoft Learn: Windows Terminal Command Line Arguments (updated 2025-11-12) - https://learn.microsoft.com/en-us/windows/terminal/command-line-arguments
- claude-notifications-go GitHub repository (617+ stars) - https://github.com/nicobailey/claude-notifications-go
- Claude Code Hooks documentation - https://docs.anthropic.com/en/docs/claude-code/hooks

**Confidence notes:**
- Windows Toast capabilities: HIGH (verified from official Microsoft docs dated 2026)
- Windows Terminal focus-tab: HIGH (verified from official Microsoft docs)
- Competitor feature sets: MEDIUM (from training data, GitHub inaccessible from network)
- User pain points: MEDIUM (from training data, Reddit/GitHub issues inaccessible)
- Warp Windows tab focus: LOW (needs implementation-phase research)
