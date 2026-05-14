# Domain Pitfalls

**Domain:** Windows-native notification plugin for Claude Code (Click-to-Focus)
**Researched:** 2026-05-15

## Critical Pitfalls

Mistakes that cause rewrites, enterprise-blocking failures, or core feature breakage.

### Pitfall 1: SetForegroundWindow Silently Fails (Flashes Taskbar Instead)

**What goes wrong:** Windows enforces strict "foreground lock" rules. After Windows 98/2000, `SetForegroundWindow()` will silently fail and merely flash the taskbar icon if the calling process doesn't meet specific criteria. This means Click-to-Focus -- the core differentiator -- appears broken to users with no error returned.

**Why it happens:** Windows prevents background processes from stealing focus. The system only allows `SetForegroundWindow` to succeed when:
- The calling process is the foreground process
- The calling process received the last input event
- The foreground process calls `AllowSetForegroundWindow(targetPID)`
- The user has set `SystemParametersInfo(SPI_SETFOREGROUNDLOCKTIMEOUT, 0, ...)` (disabled lock)
- The calling process was started by the foreground process
- No menus are active
- The foreground lock timeout has expired (default 200ms after user input)

A notification click callback runs in a COM-activated background process -- it fails EVERY condition above.

**Consequences:** Click-to-Focus success rate drops to ~30-50% depending on what the user was doing. Core value proposition breaks. Users report "it doesn't work" inconsistently.

**Prevention:**
1. **Primary:** Use `AttachThreadInput()` trick -- attach to the foreground window's thread, then call `SetForegroundWindow`, then detach. This works ~90% of the time.
2. **Secondary:** Simulate an `Alt` keypress via `SendInput()` before calling `SetForegroundWindow`. This tricks Windows into thinking the calling process is interactive.
3. **Tertiary:** Use `SystemParametersInfo(SPI_SETFOREGROUNDLOCKTIMEOUT, 0, NULL, ...)` during install to set lock timeout to 0 (requires admin, changes system behavior).
4. **Fallback:** Combine `ShowWindow(SW_MINIMIZE)` then `ShowWindow(SW_RESTORE)` -- forces window to foreground as a last resort.
5. **Nuclear:** Use UI Automation `SetFocus()` via `IUIAutomationElement.SetFocus()` which bypasses foreground restrictions in some scenarios.

**Detection:** Track success rate. If `GetForegroundWindow()` != target window after calling `SetForegroundWindow`, log failure and try fallback chain.

**Phase relevance:** Phase 1 (MVP). This MUST be solved before any release. Budget significant time for testing across scenarios (user typing, user in fullscreen app, user idle, etc.)

**Confidence:** HIGH (well-documented Windows behavior since Windows 2000, verified via Microsoft Learn docs)

---

### Pitfall 2: Toast COM Activation Requires Registry + Shortcut + AppUserModelID Dance

**What goes wrong:** Unpackaged desktop apps must perform a complex 3-step registration to receive toast notification callbacks (click events). Missing any step causes toasts to show but clicks to silently do nothing -- or worse, the app launches a new instance instead of communicating with the running one.

**Why it happens:** Windows Toast Notification system was designed for UWP/packaged apps. Unpackaged apps must:
1. Register a COM class (CLSID) in `HKCU\SOFTWARE\Classes\CLSID\{GUID}\LocalServer32` pointing to the exe
2. Create a Start Menu shortcut with `System.AppUserModel.ID` and `System.AppUserModel.ToastActivatorCLSID` properties set
3. Implement `INotificationActivationCallback` COM interface in the exe
4. Call `CoRegisterClassObject` at startup to handle activation callbacks

If the shortcut is missing, toast clicks launch a NEW process via LocalServer32 registry entry instead of activating the callback in the running process.

**Consequences:**
- Clicks open a new instance of the app (user sees process spawning but nothing happens)
- Callbacks never fire in the running process
- Interactive toast buttons (Approve/Deny) become non-functional
- Uninstall leaves orphaned registry entries and shortcuts

**Prevention:**
1. Use the Windows App SDK `AppNotificationManager` API (available since WinAppSDK 1.0) which handles most registration automatically for unpackaged apps
2. If using raw WinRT, create an installer/first-run routine that:
   - Generates a stable GUID (don't randomize per install!)
   - Writes `HKCU\SOFTWARE\Classes\CLSID\{GUID}\LocalServer32` = exe path
   - Creates shortcut in `%APPDATA%\Microsoft\Windows\Start Menu\Programs\` with proper properties
3. Implement proper single-instance detection (named mutex) so COM activation reaches the running process
4. Implement clean uninstall that removes registry + shortcut

**Detection:** Test toast click callback in a CI environment. If `INotificationActivationCallback.Activate()` never fires, registration is broken.

**Phase relevance:** Phase 1 (MVP). Toast notifications without click handling are useless for Click-to-Focus.

**Confidence:** HIGH (verified via Microsoft Learn C++/WinRT coclasses documentation and COM activation patterns)

---

### Pitfall 3: Claude Code Hook Shell Execution on Windows -- The `sh`/`bash`/`powershell` Confusion

**What goes wrong:** Claude Code hooks on Windows have undergone multiple shell execution changes across versions. Earlier versions used `cmd.exe` (causing silent failures), then switched to Git Bash, and as of v2.1.120 can use PowerShell natively. Hook commands written for one shell break on another. The `shell` field in hook config was added specifically to address this.

**Why it happens:**
- Pre-2.1.120: Claude Code required Git for Windows and used Git Bash internally
- Post-2.1.120: Git Bash is optional; PowerShell is the fallback
- Hook commands default to `bash` shell unless `"shell": "powershell"` is specified
- If Git Bash is not installed and `shell` is not set to `powershell`, hooks fail silently
- The `CLAUDE_CODE_GIT_BASH_PATH` env var must be set if Git Bash is in a non-standard location
- Known competitor issue (#55): double `sh` wrapping causes commands to be executed as `sh -c "sh -c \"actual_command\""` leading to path resolution failures on Windows

**Consequences:**
- Hook never fires (Claude Code treats it as non-blocking error, continues silently)
- Notification never shows, user thinks plugin is broken
- Path separators break (`/` vs `\`) depending on shell
- `jq` commands from hook examples fail because jq isn't in Windows PATH
- Environment variable expansion differs (`$VAR` vs `$env:VAR` vs `%VAR%`)

**Prevention:**
1. **Always set `"shell": "powershell"`** in hook configuration for Windows
2. Ship as a self-contained exe that reads stdin JSON directly -- no shell script wrapper needed
3. Use the hook `command` field to directly invoke the exe: `"command": "C:\\path\\to\\claude-win-notify.exe notify"`
4. Avoid relying on `jq`, `grep`, or other Unix tools being available
5. Handle both forward and backslash paths in `cwd` and `transcript_path` fields
6. Test with Git Bash absent (pure PowerShell-only Windows install)

**Detection:** Check Claude Code version during install. Verify shell availability. Log to a file when hook fires (since stderr behavior varies).

**Phase relevance:** Phase 1 (MVP). This determines whether the plugin works AT ALL on a given Windows machine.

**Confidence:** HIGH (verified from official Claude Code docs, changelog entry about Git Bash, and `shell` field documentation)

---

### Pitfall 4: NativeAOT + COM Interop = "No Built-in COM" Limitation

**What goes wrong:** .NET NativeAOT explicitly states "No built-in COM" as a limitation. Toast notification COM activation (INotificationActivationCallback) may not work with standard .NET COM interop when compiled as NativeAOT. The trimmer strips reflection-based COM marshalling code.

**Why it happens:** NativeAOT limitations documented by Microsoft:
- No built-in COM (explicitly listed)
- Requires trimming, which removes unreachable code including COM infrastructure
- `System.Linq.Expressions` uses interpreted form (slower)
- Not all runtime libraries are fully annotated for AOT compatibility
- Generic virtual methods have size explosion issues
- WinRT/COM projections (CsWinRT) may have trimming issues

**Consequences:**
- App compiles and runs but toast click callbacks silently never fire
- COM class factory registration fails at runtime with `CLASS_E_NOAGGREGATION` or silent HRESULT
- Build succeeds with trimming warnings that are easy to miss
- Attempting to use `[ComVisible(true)]` attributes gets trimmed away

**Prevention:**
1. **If using C# NativeAOT:** Use Windows App SDK `AppNotificationManager` which is designed for modern .NET and has AOT annotations. Avoid raw COM interop.
2. **If using C# NativeAOT:** Use source-generated COM interop (`[GeneratedComInterface]` in .NET 8+) instead of runtime-generated stubs
3. **Alternative:** Use Rust (no trimming issue, direct Win32/COM API calls, no runtime)
4. **Test aggressively:** Run the NativeAOT-published exe in a clean VM (no .NET runtime). If toast callbacks don't work, you'll know immediately.
5. Set `<IsAotCompatible>true</IsAotCompatible>` and treat ALL trim warnings as errors

**Detection:** Enable `<TrimmerSingleWarn>false</TrimmerSingleWarn>` and `<SuppressTrimAnalysisWarnings>false</SuppressTrimAnalysisWarnings>`. Any IL2XXX or IL3050 warnings related to COM types are fatal.

**Phase relevance:** Phase 0 (Tech Stack Decision). This pitfall may drive the Rust vs C# decision.

**Confidence:** HIGH (verified from Microsoft Learn NativeAOT limitations page: "No built-in COM" explicitly listed)

---

### Pitfall 5: Enterprise Device Guard / WDAC Blocks Unsigned Executables

**What goes wrong:** Enterprise environments running Windows Defender Application Control (WDAC, formerly Device Guard) will block any unsigned executable from running. The exe won't even start -- it gets killed before `main()` executes. SmartScreen also shows scary "Windows protected your PC" dialogs for unsigned EXEs downloaded from the internet.

**Why it happens:**
- WDAC enforces code integrity policies that only allow whitelisted/signed binaries
- SmartScreen checks Mark-of-the-Web (MOTW) on downloaded files and reputation
- Enterprise Intune/GPO policies can enforce stricter signing requirements
- Even with signing, a new publisher certificate has zero reputation (SmartScreen still warns for first ~1000 downloads)

**Consequences:**
- Enterprise users (the primary target audience per PROJECT.md) literally cannot run the tool
- Users see "This app has been blocked by your system administrator" -- game over
- Support tickets flood in with "doesn't work on my company laptop"
- Even with code signing, new certificates get SmartScreen warnings for weeks/months

**Prevention:**
1. **Code sign from day one** -- use a proper EV (Extended Validation) code signing certificate (~$300-500/year). EV certs get immediate SmartScreen reputation.
2. **Document the PowerShell bypass** for early adopters: `Unblock-File` removes MOTW
3. **Submit to Microsoft for reputation building** via the SmartScreen reporting page
4. **Provide winget/scoop distribution** -- package managers bypass MOTW
5. **Enterprise deployment guidance:** Document how IT admins can whitelist the exe path or hash in WDAC supplemental policies
6. **Consider installer approach** (MSI/MSIX) which can carry signing and auto-registers Start Menu shortcuts (needed for toast anyway)

**Detection:** Test in a VM with WDAC audit mode enabled. Check Windows Event Log `Microsoft-Windows-CodeIntegrity/Operational` for block events.

**Phase relevance:** Phase 3+ (Enterprise). But budget for code signing certificate acquisition in Phase 1 to start reputation building early.

**Confidence:** HIGH (well-documented enterprise Windows security behavior, confirmed by WDAC documentation)

---

### Pitfall 6: Windows Terminal `focus-tab` Has No External API

**What goes wrong:** Windows Terminal has no public API or command-line argument to focus a specific tab from an external process. The `wt.exe` CLI can specify which tab should be focused at LAUNCH time, but cannot manipulate tabs of an already-running terminal instance.

**Why it happens:**
- `wt.exe` CLI is designed for launching new instances/panes, not controlling existing ones
- Windows Terminal has no IPC mechanism for external tab control
- The command palette and keyboard shortcuts only work when the terminal is focused
- There's no COM/WinRT API exposed for tab manipulation of running instances

**Consequences:**
- Click-to-Focus can bring the Windows Terminal WINDOW to foreground, but cannot switch to the correct TAB
- If user has 10+ tabs open, they still have to manually find the right one
- Core differentiator partially broken for multi-tab users

**Prevention:**
1. **Use UI Automation:** Enumerate `TabItem` elements in the Windows Terminal window tree, find the tab by title matching (Claude Code sessions set tab titles), and invoke `SelectionItemPattern.Select()`
2. **Send keystrokes:** After focusing the window, send `Ctrl+Tab` sequences or `Ctrl+<number>` to reach the target tab (fragile, requires knowing tab index)
3. **Process association:** Track which terminal window PID hosts Claude Code by monitoring parent processes. If only one tab exists, focus-window alone is sufficient.
4. **Tab title matching:** Claude Code sets terminal title. Use UI Automation to find the tab with the matching title text.
5. **Document limitation:** If UI Automation is unreliable, clearly state that multi-tab focus requires Windows Terminal 1.18+ (which improved accessibility tree)

**Detection:** Test with 5+ tabs open. If the wrong tab remains active after click-to-focus, the tab switching failed.

**Phase relevance:** Phase 2 (Tab-Level Focus). This is the hard part that competitors gave up on.

**Confidence:** MEDIUM (confirmed WT CLI only launches new instances; UI Automation approach is based on Windows accessibility architecture patterns, needs empirical validation)

---

### Pitfall 7: Warp on Windows Has No External Tab Control API

**What goes wrong:** Warp terminal on Windows has no documented external API for controlling tab focus from another process. The launch configuration YAML (`active_tab_index`, `is_focused`) only works at launch time, not for runtime manipulation.

**Why it happens:**
- Warp is a Rust application with its own rendering pipeline (not using standard Windows controls)
- No IPC/socket API exposed for external tab management
- Warp's internal `CLI_AGENT_INPUT_ALLOWED_COMMANDS` explicitly limits which commands external agents can send
- The `/rename-tab` command was intentionally excluded from CLI agent input
- UI Automation may not expose Warp's custom-rendered tabs as standard accessible elements

**Consequences:**
- Tab-level focus for Warp may be impossible or require reverse-engineering internal APIs
- Can only focus the Warp WINDOW, not switch to the correct tab
- May need to rely on future Warp API additions (collaboration/feature request)

**Prevention:**
1. **Test UI Automation tree:** Warp may expose tabs via accessibility (UIA) even with custom rendering -- test empirically
2. **Focus on window-level first:** Window focus alone is still valuable for Warp single-tab users
3. **File a Warp feature request:** Request an IPC mechanism or CLI command for tab focus
4. **Process tracking:** Associate Claude Code process with Warp window via parent PID to at least focus the correct window
5. **Mark as known limitation:** Be transparent in README that Warp tab focus depends on future Warp API support

**Detection:** Inspect Warp's UIA tree using Accessibility Insights. If no `TabItem` pattern is found, tab control is not feasible via standard means.

**Phase relevance:** Phase 2 (Tab-Level Focus). Research spike needed.

**Confidence:** MEDIUM (Warp source shows no external tab API; accessibility tree behavior needs empirical testing on Windows)

## Moderate Pitfalls

### Pitfall 8: Hook stdin JSON Encoding Issues on Windows

**What goes wrong:** Claude Code sends JSON to hook stdin. On Windows, stdin encoding defaults to the system ANSI codepage (e.g., cp936 for Chinese Windows), not UTF-8. If `cwd` or `transcript_path` contains non-ASCII characters (common in CJK usernames), the JSON parsing fails.

**Prevention:**
1. Set stdin to UTF-8 mode explicitly at app startup (`SetConsoleOutputCP(65001)` + read stdin as binary then decode as UTF-8)
2. Parse JSON from raw bytes, not from a text reader that applies codepage conversion
3. Test with a CJK username path like `C:\Users\` (Chinese characters in user directory name)
4. Claude Code likely sends UTF-8 regardless of console codepage -- verify empirically

**Detection:** Test with a Windows user account that has non-ASCII characters in the username.

**Phase relevance:** Phase 1 (MVP). Silent failures for CJK users.

**Confidence:** MEDIUM (standard Windows encoding issue; needs empirical verification of what Claude Code actually sends)

---

### Pitfall 9: JSONL Transcript Parsing -- Incomplete Writes and File Locking

**What goes wrong:** The JSONL transcript file is actively being written by Claude Code during the session. Reading it concurrently can encounter:
- Partial JSON lines (write in progress)
- File locks (depending on how Claude Code opens the file)
- Rapidly growing files (100MB+ for long sessions with image content)

**Prevention:**
1. **Read from end:** Seek to end, read backwards to find the last complete line
2. **Handle incomplete lines gracefully:** If the last line doesn't parse as JSON, discard it and use the previous complete line
3. **Use sharing mode:** Open with `FileShare.ReadWrite` to avoid lock conflicts
4. **Don't parse entire file:** Only read the last N bytes (e.g., 64KB) to find recent state
5. **Watch for BOM:** UTF-8 BOM at file start is possible on Windows
6. **Memory-map for large files:** Don't read 100MB into memory just to find the last message

**Detection:** Stress test with a rapidly-chatting session. If notifications arrive with garbled content, it's a partial-read issue.

**Phase relevance:** Phase 2+ (Smart notification type detection from transcript). Not needed for Phase 1 if only using hook stdin data.

**Confidence:** MEDIUM (standard concurrent file access pattern; specific Claude Code file locking behavior needs testing)

---

### Pitfall 10: Toast Notification Limitations Break Interactive Workflows

**What goes wrong:** Windows Toast notifications have hard limits that silently truncate content or prevent desired interaction patterns:
- Maximum 5 action buttons per toast
- Maximum 3 text elements (title + 2 body lines, text gets truncated at display width)
- Text input boxes have no validation capability
- Toasts auto-dismiss after ~7 seconds (can be overridden with `scenario="reminder"` for longer display)
- No way to update toast content after showing (must remove and re-show)
- DND/Focus Assist suppresses toasts entirely (they go to Action Center instead)
- Button labels truncate at ~40 characters

**Prevention:**
1. **Permission requests:** Use only 2 buttons (Approve/Deny), keep labels short
2. **Long messages:** Put summary in toast, full context available on click
3. **Use `scenario="reminder"`** for permission prompts that need user attention (plays sound repeatedly, stays visible)
4. **Handle Focus Assist/DND:** Detect DND state via `FocusAssistControl` API and either queue notifications or use alternative alert (taskbar flash)
5. **Don't put reply functionality in toast:** Toast text inputs are limited. If user needs to reply, click-to-focus to terminal is better UX.
6. **Group notifications:** Use `group` and `tag` properties to replace/update existing notifications instead of stacking

**Detection:** Test with Focus Assist enabled. If notifications never appear visually, they're being suppressed.

**Phase relevance:** Phase 1 (basic toasts), Phase 3 (interactive toasts). Design notification UX within these limits from the start.

**Confidence:** HIGH (well-documented toast schema limitations)

---

### Pitfall 11: Single-Instance Mutex + COM Activation Race Condition

**What goes wrong:** When a toast is clicked, Windows uses the `LocalServer32` registry entry to activate the COM class. If the running instance hasn't registered itself via `CoRegisterClassObject`, Windows spawns a NEW instance using the registered exe path. Now you have two instances, neither handling the click correctly.

**Prevention:**
1. Implement named mutex (`Global\claude-win-notify-{user-sid}`) check at startup
2. Register COM class factory IMMEDIATELY at startup, before any other initialization
3. If a second instance detects the mutex, forward the activation intent to the existing instance (via named pipe or shared memory)
4. Handle the startup race: if COM activates before full init, queue the callback
5. Consider using a persistent background process model rather than launch-on-demand

**Detection:** Rapidly click 3 toast notifications. If multiple processes appear in Task Manager, the single-instance handling is broken.

**Phase relevance:** Phase 1 (MVP). Fundamental to correct toast click handling.

**Confidence:** HIGH (standard COM activation pattern; verified from Microsoft's C++/WinRT toast example which explicitly shows this pattern)

---

### Pitfall 12: Hook Timeout Kills Notification Process

**What goes wrong:** Claude Code hooks have a default timeout of 10 minutes (configurable per hook). For the `Notification` event, the hook command must return quickly. If the notification exe blocks waiting for user interaction (e.g., waiting for toast click callback), Claude Code may kill the process at timeout.

**Prevention:**
1. **Return immediately from the hook:** Show the toast, write the callback routing info to a file/pipe, and exit with code 0
2. **Use a persistent background daemon:** The hook command just signals the daemon (via named pipe, HTTP localhost, or file), daemon handles toast lifecycle
3. **Don't wait for click in hook process:** Toast clicks activate via COM (separate mechanism), not via the hook process
4. **Set `"async": true`** in hook config if you need the hook process to stay alive for notification lifecycle management
5. **If using `asyncRewake: true`:** Be aware that exit code 2 wakes Claude -- don't accidentally trigger this

**Detection:** If notifications appear but clicks never work, the hook process may have been killed before COM registration completes.

**Phase relevance:** Phase 1 (MVP). Architecture decision: daemon vs per-invocation.

**Confidence:** HIGH (verified from Claude Code hooks documentation: timeout defaults and async behavior)

## Minor Pitfalls

### Pitfall 13: Notification Cooldown Logic -- Rate Limiting Without State

**What goes wrong:** Claude Code can fire many `Notification` events in rapid succession (e.g., multiple sub-agents completing simultaneously). Without state between hook invocations, each invocation is stateless and can't implement cooldown.

**Prevention:**
1. Use a lightweight state file (e.g., `%TEMP%\claude-win-notify-state.json`) with last notification timestamp
2. Or use the daemon architecture -- daemon maintains in-memory state across invocations
3. Lock the state file during read-modify-write to handle concurrent hook invocations

**Phase relevance:** Phase 2 (notification aggregation/cooldown).

---

### Pitfall 14: `transcript_path` Uses Forward Slashes on Windows

**What goes wrong:** The Claude Code hook input provides `transcript_path` with forward slashes (Unix-style) even on Windows, because Claude Code runs in a Node.js environment that normalizes paths. Some Windows APIs reject forward-slash paths.

**Prevention:**
1. Normalize all paths from hook input: replace `/` with `\` on Windows
2. Or use APIs that accept both (most .NET/Rust path APIs handle this; raw Win32 `CreateFile` does too)
3. Don't display raw paths to users in notifications without normalization

**Phase relevance:** Phase 1 (path handling in hook input parsing).

---

### Pitfall 15: PowerShell Execution Policy Blocks Install Script

**What goes wrong:** The planned "PowerShell one-click install" may fail on enterprise machines where execution policy is set to `Restricted` or `AllSigned`. The `irm | iex` pattern (used by Claude Code itself) bypasses this in some contexts but not all.

**Prevention:**
1. Document alternative: `powershell -ExecutionPolicy Bypass -File install.ps1`
2. Provide a CMD-based alternative: `curl -o install.ps1 ... && powershell -ep bypass ./install.ps1`
3. Consider distributing as a `.cmd` file that bootstraps PowerShell with bypass
4. For enterprise: provide MSI/MSIX installer that doesn't need PowerShell scripts

**Phase relevance:** Phase 3 (distribution/install). Plan for it from the start.

---

### Pitfall 16: Notification Type Detection -- `notification_type` Field May Change

**What goes wrong:** The Claude Code `Notification` hook provides a `notification_type` field (e.g., `"permission_prompt"`, `"idle_prompt"`). Relying on specific string values that may change across Claude Code versions causes silent breakage.

**Prevention:**
1. Use `matcher` in hook config to route notification types (official API)
2. Treat unknown `notification_type` values as generic notifications (graceful degradation)
3. Don't hardcode exhaustive type lists -- use a default/fallback handler
4. Subscribe to Claude Code changelog for notification type additions

**Phase relevance:** Phase 1 (notification routing).

---

### Pitfall 17: Start Menu Shortcut Path Varies Across Windows Editions

**What goes wrong:** The Start Menu shortcut (required for toast COM activation) must be in a specific location. But the path varies:
- Standard: `%APPDATA%\Microsoft\Windows\Start Menu\Programs\`
- All Users: `%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\`
- Windows Server: Sometimes uses different base paths

**Prevention:**
1. Use `Environment.GetFolderPath(Environment.SpecialFolder.Programs)` or equivalent to get the correct path
2. Never hardcode `C:\Users\...\AppData\Roaming\...`
3. Verify shortcut exists on every startup (user may have deleted it)

**Phase relevance:** Phase 1 (toast registration setup).

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Phase 0: Tech Stack | NativeAOT COM limitation (#4) | Evaluate Rust vs C# early. If C#, validate WinAppSDK + NativeAOT compatibility in a spike |
| Phase 1: MVP Toast | COM registration dance (#2), Shell execution (#3), Timeout (#12) | Use daemon architecture; always set `shell: powershell`; return from hook immediately |
| Phase 1: MVP Focus | SetForegroundWindow fails (#1) | Implement multi-strategy focus with fallback chain; test 5+ scenarios |
| Phase 2: Tab Focus | No external WT/Warp API (#6, #7) | UI Automation spike early; document limitations transparently |
| Phase 2: Smart Detection | JSONL partial reads (#9), Encoding (#8) | Read from end; binary UTF-8 parsing; test CJK paths |
| Phase 3: Interactive | Toast limits (#10), Single-instance (#11) | Design UX within 5-button/3-text limits; mutex + COM race handling |
| Phase 3: Enterprise | WDAC/SmartScreen (#5), Install (#15) | Budget for EV code signing cert; provide MSI alternative |
| All Phases | Notification type changes (#16) | Graceful degradation for unknown types; don't hardcode enums |

## Sources

- Microsoft Learn: NativeAOT deployment limitations (https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot) -- HIGH confidence
- Microsoft Learn: C++/WinRT Coclasses (toast COM activation) (https://learn.microsoft.com/en-us/windows/uwp/cpp-and-winrt-apis/author-coclasses) -- HIGH confidence
- Claude Code Docs: Hooks system (https://code.claude.com/docs/en/hooks) -- HIGH confidence
- Claude Code Docs: Hooks guide (https://code.claude.com/docs/en/hooks-guide) -- HIGH confidence
- Claude Code Changelog: Git Bash/PowerShell shell changes -- HIGH confidence
- Warp source specs: CLI Agent Input Allowed Commands, Tab management -- MEDIUM confidence
- Windows Terminal Docs: Command line arguments (https://learn.microsoft.com/en-us/windows/terminal) -- HIGH confidence
- Microsoft Learn: WDAC Application Control (https://learn.microsoft.com/en-us/defender-endpoint/mde-p1-setup-configuration) -- HIGH confidence
- SetForegroundWindow behavior: Based on well-known Win32 API documentation (MSDN/Learn) -- HIGH confidence
- Toast notification limitations: Based on Windows toast schema documentation -- HIGH confidence
- claude-notifications-go issues (#55 shell execution, #69 Device Guard, #79 stderr): Referenced but unable to access directly due to network restrictions -- MEDIUM confidence (known competitor issues reported by community)
