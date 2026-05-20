# Phase 2: Hook & Toast Foundation - Context

**Gathered:** 2026-05-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the basic end-to-end pipeline: Claude Code hook triggers the exe, exe reads stdin JSON, exe parses fields, exe displays a Windows Toast notification. This phase delivers a working foundation — not yet differentiated by notification type (Phase 3) or click-to-focus (Phase 4).

</domain>

<decisions>
## Implementation Decisions

### Project Structure & Architecture
- **D-01:** Single crate with module-per-file structure (src/main.rs + src/lib.rs + src/hook.rs + src/toast.rs + src/error.rs etc.) — standard Rust CLI pattern for <5K LOC projects
- **D-02:** CLI interface uses clap with subcommands: `claude-win-notify hook` (stdin→Toast), `claude-win-notify focus <uri>` (Phase 4), `claude-win-notify --version`
- **D-03:** Executable name: `claude-win-notify` (matches repo name)
- **D-04:** Spike code (`spike/`) is discarded — Phase 2 starts fresh with proper architecture (carried from D-07 Phase 1)

### Hook stdin Parsing
- **D-05:** Phase 2 handles only Stop and Notification hook events; all other events trigger silent exit(0)
- **D-06:** Parse failure strategy: exit(0) + write error to local log file. Never return non-zero exit code (don't disturb Claude Code workflow)
- **D-07:** Required fields from stdin JSON: session_id, transcript_path, cwd, hook_event_name

### Toast Notification Content & Style
- **D-08:** Toast content layout: Title="Claude Code", Body="✔ Task Complete" (or event description), Attribution=project name (last directory component from cwd)
- **D-09:** Toast click action in Phase 2: none (dismiss only). Protocol activation added in Phase 4
- **D-10:** AUMID: borrow PowerShell's for now (same as spike). Custom AUMID is Phase 7 (TOAST-05)

### Development & Testing Strategy
- **D-11:** Three-layer testing: (1) unit tests for JSON parsing logic, (2) stdin pipe manual validation (`echo '...' | claude-win-notify hook`), (3) real Claude Code dogfooding via hooks.json
- **D-12:** Release profile: extreme compression — strip symbols + LTO + opt-level=z + codegen-units=1. Spike achieved 0.38MB; target with clap+serde is 1-2MB (well under 15MB limit)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Foundation
- `.planning/PROJECT.md` — Core value, constraints, key decisions
- `.planning/REQUIREMENTS.md` — NOTIF-01, TOAST-01, TOAST-02, TECH-03, TECH-04, INST-06, INST-08 are Phase 2 requirements
- `.planning/ROADMAP.md` §Phase 2 — Success criteria (5 items)

### Phase 1 Spike Results (reference, not reuse)
- `spike/rust/Cargo.toml` — Validated dependencies: windows 0.62 (features list), url 2
- `spike/rust/src/main.rs` — Validated patterns: Toast via WinRT XmlDocument, SetForegroundWindow fallback chain, Protocol URI parsing
- `spike/RESULTS.md` — Binary size (0.38MB), startup time (8.9ms), all integration points PASS

### Claude Code Hooks
- Claude Code hooks documentation (external) — stdin JSON schema, hook event types, hooks.json configuration format

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Spike Toast pattern (`spike/rust/src/main.rs:197-220`): WinRT ToastNotificationManager + XmlDocument approach validated — reimplement with proper error handling
- Spike Cargo.toml: windows crate feature flags validated (Win32_Foundation, UI_Notifications, Data_Xml_Dom)

### Established Patterns
- Fire-and-forget execution model: exe starts, does work, exits. No daemon, no persistent state.
- windows-rs WinRT bindings for Toast (not raw COM)

### Integration Points
- Claude Code hooks system: stdin JSON pipe → exe. Configured via `~/.claude/settings.json` hooks field
- Windows Toast notification system: WinRT API via windows-rs crate
- Filesystem: log file for errors (location TBD by planner — likely `%LOCALAPPDATA%\claude-win-notify\logs\`)

</code_context>

<specifics>
## Specific Ideas

- Project name extraction: last path component of `cwd` field (e.g., `D:\Repository\claude-win-notify` → "claude-win-notify")
- Log rotation or size cap needed to prevent unbounded growth (planner decides mechanism)
- serde + serde_json for stdin JSON deserialization (standard Rust approach)
- clap derive macros for CLI definition (idiomatic, compile-time validated)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 2-Hook & Toast Foundation*
*Context gathered: 2026-05-20*
