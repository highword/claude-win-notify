# CLAUDE.md

## Project: claude-win-notify

Windows-First Claude Code notification plugin with Click-to-Focus.

## Quick Reference

- **Planning:** `.planning/` directory contains all project docs
- **Roadmap:** `.planning/ROADMAP.md` — 8 phases, 32 requirements
- **State:** `.planning/STATE.md` — current phase and progress
- **Config:** `.planning/config.json` — workflow preferences

## GSD Workflow

This project uses the GSD (Get Shit Done) workflow:

1. `/gsd-discuss-phase N` — gather context and clarify approach
2. `/gsd-plan-phase N` — create executable plan
3. `/gsd-execute-phase N` — implement the plan
4. `/gsd-transition` — verify and advance to next phase

## Current State

Phase 1: Tech Spike — validate C# NativeAOT vs Rust

## Conventions

- **Language:** Code and commits in English; user communication in Chinese (unless asked otherwise)
- **Commits:** Conventional commits (`feat:`, `fix:`, `docs:`, `chore:`)
- **Tech stack:** TBD (Phase 1 spike will determine)
- **Target:** Windows 10 1903+ / Windows 11
- **License:** MIT

## Branching Strategy

- **Phase 1 (Tech Spike):** Work directly on `main`
- **Phase 2+:** Create a branch per phase (e.g., `phase/02-hook-toast`, `phase/03-notification-types`), merge back to `main` via PR when phase completes
- Branch naming: `phase/{NN}-{short-slug}`
