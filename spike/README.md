# Tech Spike: C# NativeAOT vs Rust

Throwaway PoC code to validate 3 integration points:
1. Toast notification via WinRT API
2. Protocol activation (`claude-notify://`)
3. SetForegroundWindow (bring terminal to foreground)

## Structure

- `csharp/` — C# .NET 9 NativeAOT spike
- `rust/` — Rust + windows-rs spike
- `RESULTS.md` — Comparison report (generated after spike)

## Running

### C# Spike
```powershell
cd csharp && dotnet publish -c Release -r win-x64
./bin/Release/net9.0-windows10.0.19041.0/win-x64/publish/CSharpSpike.exe
```

### Rust Spike
```powershell
cd rust && cargo build --release
./target/release/rust-spike.exe
```

## Status

This code is intentionally throwaway. Phase 2 starts fresh with proper architecture.
