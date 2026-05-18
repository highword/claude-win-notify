---
phase: 1
plan_id: 01-A
title: "Environment Setup & Project Scaffolding"
wave: 1
depends_on: []
files_modified:
  - spike/csharp/CSharpSpike.csproj
  - spike/csharp/Program.cs
  - spike/rust/Cargo.toml
  - spike/rust/src/main.rs
  - spike/README.md
requirements_addressed: [TECH-01]
autonomous: true
must_haves:
  goal: "Both .NET 9 SDK and Rust toolchain installed; skeleton projects compile and run"
  truths:
    - ".NET 9 SDK installed and dotnet --version returns 9.x"
    - "Rust stable toolchain installed and cargo --version returns valid version"
    - "C# spike project compiles with NativeAOT enabled (dotnet publish -c Release succeeds)"
    - "Rust spike project compiles (cargo build --release succeeds)"
---

# Plan 01-A: Environment Setup & Project Scaffolding

## Objective

Install required toolchains (.NET 9 SDK, Rust) and create skeleton projects for both spikes that compile successfully, proving the build pipeline works before adding Windows API calls.

## Tasks

<task id="A1">
<title>Install .NET 9 SDK</title>
<read_first>
- .planning/phases/01-tech-spike/01-CONTEXT.md (D-15: toolchains need installation)
</read_first>
<action>
Install .NET 9 SDK via winget:
```powershell
winget install Microsoft.DotNet.SDK.9
```
Verify installation: `dotnet --version` should output `9.0.x`.
If winget fails, download from https://dotnet.microsoft.com/download/dotnet/9.0
</action>
<acceptance_criteria>
- `dotnet --version` outputs a version starting with `9.`
- `dotnet new console --help` runs without error
</acceptance_criteria>
</task>

<task id="A2">
<title>Install Rust toolchain</title>
<read_first>
- .planning/phases/01-tech-spike/01-CONTEXT.md (D-15: toolchains need installation)
</read_first>
<action>
Install Rust via rustup:
```powershell
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile "$env:TEMP\rustup-init.exe"
& "$env:TEMP\rustup-init.exe" -y --default-toolchain stable-x86_64-pc-windows-msvc
```
Refresh PATH (restart terminal or run `$env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","User") + ";" + [System.Environment]::GetEnvironmentVariable("PATH","Machine")`).
Verify: `rustc --version` and `cargo --version` both output valid versions.
</action>
<acceptance_criteria>
- `rustc --version` outputs a version string
- `cargo --version` outputs a version string
- Target `stable-x86_64-pc-windows-msvc` is installed (`rustup show` lists it)
</acceptance_criteria>
</task>

<task id="A3">
<title>Create C# NativeAOT spike project</title>
<read_first>
- .planning/research/STACK.md (lines 211-229: .csproj configuration for NativeAOT)
- .planning/phases/01-tech-spike/01-RESEARCH.md (NativeAOT constraints section)
</read_first>
<action>
Create directory `spike/csharp/` and scaffold:

```powershell
mkdir spike/csharp
cd spike/csharp
dotnet new console -n CSharpSpike --framework net9.0
```

Replace the generated `.csproj` with:
```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net9.0-windows10.0.19041.0</TargetFramework>
    <RuntimeIdentifier>win-x64</RuntimeIdentifier>
    <PublishAot>true</PublishAot>
    <PublishTrimmed>true</PublishTrimmed>
    <SelfContained>true</SelfContained>
    <InvariantGlobalization>true</InvariantGlobalization>
    <IlcOptimizationPreference>Size</IlcOptimizationPreference>
    <IsAotCompatible>true</IsAotCompatible>
    <CsWinRTAotOptimizerEnabled>true</CsWinRTAotOptimizerEnabled>
    <WindowsSdkPackageVersion>10.0.19041.41</WindowsSdkPackageVersion>
  </PropertyGroup>
</Project>
```

Replace `Program.cs` with a minimal "Hello Spike" that prints to console. Verify `dotnet publish -c Release -r win-x64` produces an exe in `bin/Release/net9.0-windows10.0.19041.0/win-x64/publish/`.
</action>
<acceptance_criteria>
- File `spike/csharp/CSharpSpike.csproj` exists and contains `<PublishAot>true</PublishAot>`
- File `spike/csharp/Program.cs` exists
- `dotnet publish -c Release -r win-x64` in `spike/csharp/` completes without error
- A `.exe` file exists in the publish output directory
</acceptance_criteria>
</task>

<task id="A4">
<title>Create Rust spike project</title>
<read_first>
- .planning/research/STACK.md (lines 49-78: Rust + windows-rs assessment)
- .planning/phases/01-tech-spike/01-RESEARCH.md (Rust approach section)
</read_first>
<action>
Create directory `spike/rust/` and scaffold:

```powershell
cd spike
cargo new rust-spike --name rust_spike
Rename-Item rust-spike rust
```

Edit `spike/rust/Cargo.toml` to add windows-rs dependency:
```toml
[package]
name = "rust-spike"
version = "0.1.0"
edition = "2021"

[dependencies.windows]
version = "0.62"
features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
    "UI_Notifications",
    "Data_Xml_Dom",
]
```

Replace `src/main.rs` with a minimal "Hello Spike" that prints to console. Verify `cargo build --release` succeeds.
</action>
<acceptance_criteria>
- File `spike/rust/Cargo.toml` contains `windows` dependency with version `"0.62"`
- File `spike/rust/src/main.rs` exists
- `cargo build --release` in `spike/rust/` completes without error
- A `.exe` file exists in `spike/rust/target/release/`
</acceptance_criteria>
</task>

<task id="A5">
<title>Create spike README</title>
<read_first>
- .planning/phases/01-tech-spike/01-CONTEXT.md (D-07: spike is throwaway, D-09: lives in spike/ dir)
</read_first>
<action>
Create `spike/README.md` with:
```markdown
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
```
</action>
<acceptance_criteria>
- File `spike/README.md` exists and contains "Toast notification via WinRT"
- File `spike/README.md` contains instructions for running both spikes
</acceptance_criteria>
</task>

## Verification

```powershell
# All checks must pass
Test-Path spike/csharp/CSharpSpike.csproj   # True
Test-Path spike/rust/Cargo.toml              # True
Test-Path spike/README.md                    # True
dotnet --version                             # 9.x
cargo --version                              # Valid
```
