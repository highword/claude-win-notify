# Phase 3: Notification Type Detection - Context

**Gathered:** 2026-05-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Detect and differentiate 4 notification types (Task Complete, Permission Request, Question, Error) with distinct visuals (Hero Image) and sounds (escalating urgency). Builds on Phase 2's basic hook-to-toast pipeline by adding type classification logic and per-type Toast templates.

</domain>

<decisions>
## Implementation Decisions

### Notification Type Detection Logic
- **D-01:** Detection is purely field-based (no JSONL transcript reading). All classification uses stdin JSON fields only.
- **D-02:** Stop hook classification priority (highest to lowest): Error > Question > Task Complete
- **D-03:** Error detection is CONSERVATIVE — only matches Claude Code system-level error patterns: "API rate limit", "session limit", "context window exceeded", "API error". Never matches user-mentioned errors in task descriptions.
- **D-04:** Question detection: `last_assistant_message` last line ends with `?` character (after trimming whitespace). Zero IO cost, ~97% coverage.
- **D-05:** Permission Request detection: `notification_type == "permission_prompt"`. Fallback: if `notification_type` field is missing (known bug #11964), match `message` containing "permission" keyword.
- **D-06:** Notification hook with any `notification_type` other than "permission_prompt" → classified as Question (covers "idle_prompt", "elicitation_dialog", etc.)
- **D-07:** Unknown hook events (not Stop, not Notification) → silent exit(0) as before (no change from Phase 2)

### Visual Differentiation — Hero Image
- **D-08:** Each notification type has a dedicated PNG Hero Image displayed at top of Toast via `<image placement="hero" src="file:///..."/>` in Toast XML
- **D-09:** Images are compiled into the exe via `include_bytes!()` and extracted to `%LOCALAPPDATA%\claude-win-notify\assets\` on first run. Subsequent runs check existence and skip extraction.
- **D-10:** Hero Image recommended size: 364×180 pixels, PNG format. Expected total size increase: ~50-80KB (well within 15MB limit).
- **D-11:** Toast text layout unchanged from Phase 2: Title = "Claude Code", Body = type-specific message, Attribution = project name. Hero Image is additive.

### Sound Differentiation — Escalating Urgency
- **D-12:** Sound mapping per notification type:
  - Task Complete → `ms-winsoundevent:Notification.Default` (light, non-intrusive)
  - Permission Request → `ms-winsoundevent:Notification.Reminder` (attention-grabbing)
  - Question → `ms-winsoundevent:Notification.IM` (conversational)
  - Error → `ms-winsoundevent:Notification.Looping.Alarm` (single play, urgent)

### Event Classification Boundaries
- **D-13:** Stop + `stop_hook_active == true` → no notification (infinite loop prevention, unchanged)
- **D-14:** Stop + system error keywords in `last_assistant_message` → Error (highest priority)
- **D-15:** Stop + `last_assistant_message` ends with `?` → Question (second priority)
- **D-16:** Stop + none of the above → Task Complete (default)
- **D-17:** Notification + `notification_type == "permission_prompt"` (or message contains "permission" when field missing) → Permission Request
- **D-18:** Notification + any other `notification_type` or fallback → Question
- **D-19:** `notification_type` field may be missing due to Claude Code bug #11964 — always fallback to message content matching

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Foundation
- `.planning/PROJECT.md` — Core value, constraints, tech decisions
- `.planning/REQUIREMENTS.md` — NOTIF-02 through NOTIF-05, TOAST-03, TOAST-04, TECH-05 are Phase 3 requirements
- `.planning/ROADMAP.md` §Phase 3 — Success criteria (5 items)

### Phase 2 Outputs (build upon)
- `src/hook.rs` — Current `handle_stop()` and `handle_notification()` dispatch, `HookInput` struct
- `src/toast.rs` — Current `show_toast(title, body, attribution)` function, Toast XML template
- `src/error.rs` — `AppError` enum (already has Windows error variant)
- `.planning/phases/02-hook-toast/02-CONTEXT.md` — Phase 2 decisions D-05 through D-12

### External References
- Claude Code hooks issue #11964 — `notification_type` field sometimes missing (bug, has repro)
- Claude Code hooks issue #10168 — No dedicated `UserInputRequired` hook exists yet
- Windows Toast documentation — Hero Image placement, audio src values

</canonical_refs>

<code_context>
## Existing Code Insights

### Integration Points (Phase 2 code to extend)
- `src/hook.rs:33-37` — match on `hook_event_name`: add classification logic within `handle_stop()` and `handle_notification()`
- `src/hook.rs:40-51` — `handle_stop()`: currently always shows "✔ Task Complete". Needs Error/Question classification before deciding Toast content.
- `src/hook.rs:53-62` — `handle_notification()`: currently uses generic body. Needs Permission vs Question differentiation.
- `src/toast.rs:15` — `show_toast(title, body, attribution)`: signature needs extension for Hero Image path and sound selection.

### Reusable Patterns
- `include_bytes!()` for asset embedding (standard Rust pattern)
- `std::fs::write()` for asset extraction (same pattern as log directory creation in `src/log.rs`)
- Toast XML template already uses format!() — add `<image>` and modify `<audio>` dynamically

### New Modules Expected
- `src/assets.rs` — Asset extraction logic (include_bytes + write on first run)
- `src/notification.rs` — Notification type enum + classification logic (extracted from hook.rs)

</code_context>

<specifics>
## Specific Ideas

- System error patterns to match (conservative list): `"API rate limit"`, `"rate limit exceeded"`, `"session limit"`, `"context window"`, `"API error"`, `"authentication failed"`
- Question mark detection: `input.last_assistant_message.as_deref().map(|m| m.trim().ends_with('?')).unwrap_or(false)`
- Hero Images should be simple, high-contrast designs that work at small Toast sizes (364×180)
- Consider a `NotificationType` enum: `TaskComplete`, `PermissionRequest`, `Question`, `Error` — used throughout for dispatch

</specifics>

<deferred>
## Deferred Ideas

- JSONL transcript reading for deeper Question detection — deferred until Claude Code provides better hook signals
- Custom notification sounds (WAV files) — deferred to Phase 7 (Configuration & Branding)
- Notification type icons/logos — deferred to Phase 7 (needs custom AUMID for app icon override)

</deferred>

---

*Phase: 3-Notification Type Detection*
*Context gathered: 2026-05-20*
