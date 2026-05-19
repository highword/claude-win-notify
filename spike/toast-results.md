# Toast Validation Results

## C# NativeAOT
- Build: PASS
- NativeAOT publish: PASS
- Toast displayed: PASS
- Binary size: 3.22 MB
- Notes: Requires `vswhere.exe` in PATH for NativeAOT linking (VS Build Tools dependency at compile time only). No runtime warnings. Single-file exe with no .NET DLLs alongside.

## Rust
- Build: PASS
- Release build: PASS
- Toast displayed: PASS
- Binary size: 0.14 MB
- Notes: Clean build with zero warnings. 12s compile time with all dependencies. Single-file exe with no external DLL dependencies.

## AUMID Used
- PowerShell AUMID for both (avoids shortcut requirement in spike): `{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe`
- Custom AUMID registration deferred to Phase 2

## Comparison

| Metric | C# NativeAOT | Rust | Winner |
|--------|--------------|------|--------|
| Binary size | 3.22 MB | 0.14 MB | Rust (23x smaller) |
| Build time | ~15s publish | ~12s release | Rust |
| Toast display | PASS | PASS | Tie |
| Self-contained | Yes (single exe) | Yes (single exe) | Tie |
| Compile dependency | VS Build Tools + .NET 9 SDK | Rust toolchain | Rust (simpler) |

## Conclusion

Both stacks successfully display Windows Toast notifications via WinRT API using NativeAOT/native compilation. Integration point #1 (Toast) is validated for both candidates.
