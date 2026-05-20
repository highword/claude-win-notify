# Roadmap: claude-win-notify

## Overview

From tech spike to shipping installer: validate the Windows-native stack (C# NativeAOT vs Rust), build the hook-to-toast pipeline, prove Click-to-Focus as the core differentiator at both window and tab level, add throttling/configuration for production readiness, then package everything into a one-click PowerShell installer. Every phase delivers an observable, testable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Tech Spike** - Validate C# NativeAOT vs Rust for Toast + Protocol activation + SetForegroundWindow ✓ 2026-05-20
- [x] **Phase 2: Hook & Toast Foundation** - Hook reads stdin JSON and displays a basic Windows Toast notification ✓ 2026-05-20
- [ ] **Phase 3: Notification Type Detection** - All 4 notification types detected and displayed with distinct visuals and sounds
- [ ] **Phase 4: Click-to-Focus Window Level** - Toast click activates the correct terminal window via Protocol activation
- [ ] **Phase 5: Click-to-Focus Tab Level** - Toast click switches to the correct tab in Windows Terminal and Warp
- [ ] **Phase 6: Throttling & Deduplication** - Anti-spam mechanisms prevent notification bombardment
- [ ] **Phase 7: Configuration & Branding** - User-customizable behavior and branded notification identity
- [ ] **Phase 8: Installation & Distribution** - PowerShell one-liner installs everything automatically

## Phase Details

### Phase 1: Tech Spike
**Goal**: Determine the production tech stack by validating critical integration points that could fail
**Depends on**: Nothing (first phase)
**Requirements**: TECH-01
**Success Criteria** (what must be TRUE):
  1. A NativeAOT-compiled exe can display a Windows Toast notification via WinRT API
  2. Protocol activation (`claude-notify://`) launches the exe and receives URI arguments
  3. SetForegroundWindow (with fallback chain) successfully brings a window to foreground from a protocol-activated process
  4. Decision documented: chosen stack with evidence from spike results
**Plans**:
  - **Wave 1:** 01-A (Environment Setup & Project Scaffolding)
  - **Wave 2:** 01-B (Toast Notification Validation), 01-C (Protocol Activation Validation) *(blocked on Wave 1 completion)*
  - **Wave 3:** 01-D (SetForegroundWindow Validation) *(blocked on Waves 1+2)*
  - **Wave 4:** 01-E (Comparison Report & Stack Decision) *(blocked on all prior waves)*

### Phase 2: Hook & Toast Foundation
**Goal**: The basic pipeline works end-to-end: Claude Code hook triggers exe, exe reads stdin, exe shows a Toast
**Depends on**: Phase 1
**Requirements**: NOTIF-01, TOAST-01, TOAST-02, TECH-03, TECH-04, INST-06, INST-08
**Success Criteria** (what must be TRUE):
  1. Running the exe with Claude Code hook stdin JSON correctly parses session_id, transcript_path, cwd, and hook_event_name
  2. A Windows native Toast notification appears within 500ms of hook execution
  3. The exe runs on Windows 10 1903+ and Windows 11 without errors
  4. CJK characters in file paths and project names display correctly in the Toast
  5. The compiled binary is a single exe under 15MB with zero runtime dependencies
**Plans**: 3 plans
Plans:
- [ ] 02-01-PLAN.md — Project scaffolding (Cargo.toml, modules, CLI skeleton, logging)
- [ ] 02-02-PLAN.md — Hook stdin parsing + Toast notification display (core pipeline)
- [ ] 02-03-PLAN.md — Integration tests, CI workflow, release binary verification

### Phase 3: Notification Type Detection
**Goal**: Users receive contextually appropriate notifications for all 4 event types with distinct appearance
**Depends on**: Phase 2
**Requirements**: NOTIF-02, NOTIF-03, NOTIF-04, NOTIF-05, TOAST-03, TOAST-04, TECH-05
**Success Criteria** (what must be TRUE):
  1. Task Complete notification fires on Stop hook when task finishes successfully
  2. Permission Request notification fires when Claude needs tool approval (permission_prompt detected)
  3. Question notification fires when Claude asks the user a question (? ending or AskUserQuestion)
  4. Error notification fires on API errors, session limits, or abnormal exits
  5. Each notification type has visually distinct styling, different sound, and shows project name from cwd
**Plans**: TBD

### Phase 4: Click-to-Focus Window Level
**Goal**: Clicking a Toast notification brings the correct terminal window to the foreground - the core differentiator works
**Depends on**: Phase 3
**Requirements**: FOCUS-01, FOCUS-02, FOCUS-05, FOCUS-06, TECH-02
**Success Criteria** (what must be TRUE):
  1. Clicking a Toast activates the protocol handler (`claude-notify://focus?session=...&pid=...`) and brings the terminal window to foreground
  2. If the terminal window is minimized, it restores and then activates
  3. The exe auto-detects whether the user is running Warp, Windows Terminal, or another terminal
  4. Window-level Click-to-Focus succeeds > 95% of the time across normal usage scenarios
**Plans**: TBD
**UI hint**: yes

### Phase 5: Click-to-Focus Tab Level
**Goal**: Click-to-Focus navigates to the exact tab where Claude Code is running, not just the window
**Depends on**: Phase 4
**Requirements**: FOCUS-03, FOCUS-04
**Success Criteria** (what must be TRUE):
  1. In Windows Terminal, clicking the Toast switches to the correct tab (via UI Automation)
  2. In Warp, clicking the Toast switches to the correct tab (via UI Automation or available API)
  3. Tab switching works even when multiple Claude Code sessions run in different tabs
**Plans**: TBD
**UI hint**: yes

### Phase 6: Throttling & Deduplication
**Goal**: Users are not bombarded with notifications during rapid Claude Code activity
**Depends on**: Phase 3
**Requirements**: NOTIF-06, NOTIF-07
**Success Criteria** (what must be TRUE):
  1. Same session does not produce repeat notifications within the cooldown window (default 5 seconds)
  2. Identical events do not trigger duplicate notifications (file-lock based dedup with 2-second TTL)
  3. Legitimate new events after the cooldown period still trigger notifications normally
**Plans**: TBD

### Phase 7: Configuration & Branding
**Goal**: Users can customize notification behavior and notifications display with proper brand identity
**Depends on**: Phase 6
**Requirements**: NOTIF-08, TOAST-05
**Success Criteria** (what must be TRUE):
  1. User can enable/disable individual notification types via config file
  2. User can adjust cooldown duration and sound preferences via config file
  3. Notifications display with custom icon and "Claude Code Notifications" app name (AUMID registered)
**Plans**: TBD

### Phase 8: Installation & Distribution
**Goal**: New users go from zero to working notifications in under 30 seconds with a single PowerShell command
**Depends on**: Phase 7
**Requirements**: INST-01, INST-02, INST-03, INST-04, INST-05, INST-07
**Success Criteria** (what must be TRUE):
  1. `irm https://url | iex` downloads the exe, places it in PATH, and completes all setup
  2. Install script injects hooks.json configuration without overwriting existing user hooks
  3. Install script registers the `claude-notify://` protocol URI scheme
  4. Install script creates Start Menu shortcut (required for AUMID)
  5. Uninstall command cleanly removes all registry entries, shortcuts, and hooks.json modifications
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

Note: Phase 6 depends on Phase 3 (not Phase 5), so Phases 5 and 6 could theoretically parallelize, but sequential execution is simpler.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Tech Spike | 1/5 | In Progress | - |
| 2. Hook & Toast Foundation | 3/3 | Complete | 2026-05-20 |
| 3. Notification Type Detection | 0/TBD | Not started | - |
| 4. Click-to-Focus Window Level | 0/TBD | Not started | - |
| 5. Click-to-Focus Tab Level | 0/TBD | Not started | - |
| 6. Throttling & Deduplication | 0/TBD | Not started | - |
| 7. Configuration & Branding | 0/TBD | Not started | - |
| 8. Installation & Distribution | 0/TBD | Not started | - |
