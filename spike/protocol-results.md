# Protocol Activation Validation Results

**Date:** 2026-05-20
**Plan:** 01-C (Protocol Activation Validation)
**Environment:** Windows 11 Pro 10.0.26200

## Summary

Both C# NativeAOT and Rust successfully implement protocol activation via `claude-notify://` URI scheme.

## Registry Registration

| Item | Result |
|------|--------|
| Script `spike/register-protocol.ps1` | PASS |
| HKCU:\Software\Classes\claude-notify created | PASS |
| "URL Protocol" property set | PASS |
| shell\open\command with `--focus "%1"` | PASS |
| Re-registration (switch exe target) | PASS |

## C# NativeAOT Stack

| Test | Result |
|------|--------|
| Build (`dotnet publish -c Release -r win-x64`) | PASS |
| Direct invocation: `--focus "claude-notify://focus?session=test123&pid=999"` | PASS |
| Session extraction: "test123" | PASS |
| PID extraction: "999" | PASS |
| URL-encoded CJK decoding (`%E6%B5%8B%E8%AF%95` -> decoded correctly) | PASS |
| Protocol activation via `Start-Process` | PASS |
| Binary size (with protocol parsing) | 3.3 MB |

**Output:**
```
PROTOCOL ACTIVATED:
  Full URI: claude-notify://focus?session=test123&pid=999
  Session: test123
  PID: 999
SUCCESS: Protocol activation parsed correctly (C#)
```

## Rust Stack

| Test | Result |
|------|--------|
| Build (`cargo build --release`) | PASS |
| Direct invocation: `--focus "claude-notify://focus?session=test123&pid=999"` | PASS |
| Session extraction: "test123" | PASS |
| PID extraction: "999" | PASS |
| URL-encoded CJK decoding (`%E6%B5%8B%E8%AF%95` -> `测试`) | PASS |
| Protocol activation via `Start-Process` | PASS |
| Binary size (with url crate + protocol parsing) | 381 KB |

**Output:**
```
PROTOCOL ACTIVATED:
  Full URI: claude-notify://focus?session=test123&pid=999
  Session: test123
  PID: 999
SUCCESS: Protocol activation parsed correctly (Rust)
```

## E2E Protocol Activation Flow

| Test | C# | Rust |
|------|-----|------|
| Register protocol -> Start-Process URI -> Exe launches | PASS | PASS |
| Exe receives full URI as argument | PASS | PASS |
| Query parameters parsed correctly | PASS | PASS |
| CJK characters decoded from URL encoding | PASS | PASS |

## Notes

- Protocol activation via `Start-Process` launches a new process window (console window flashes briefly)
- In production, the exe will use the parsed PID to focus the target window instead of printing to console
- Both stacks handle the URI identically from a parsing perspective
- Rust binary is ~9x smaller (381 KB vs 3.3 MB) even with the `url` crate added
- The `url` crate adds ICU normalizer dependencies but the binary remains compact
- C# uses manual query string parsing (NativeAOT-safe, avoids System.Web dependency)

## Conclusion

**Integration Point #2 VALIDATED for both stacks.** Protocol activation works end-to-end:
Registry -> URI invocation -> Exe launch -> Argument parsing -> Parameter extraction.
