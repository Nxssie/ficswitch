# ficswitcher (satis-switcher)

## What This Is

A Rust CLI tool for Satisfactory players who run both the stable and experimental branches. It handles the full context-switch: modifying the Steam manifest to change branch, activating the matching Satisfactory Mod Manager profile, and creating a timestamped backup of saves and blueprints before any change. Targets Windows and Linux (Steam/Proton), intended for public release.

## Core Value

Safe, one-command switching between Satisfactory branches with saves backed up and the right mod profile active — so players never lose progress or break their mod setup.

## Requirements

### Validated

- Branch switching between stable and experimental via Steam appmanifest ACF modification — existing
- Steam process detection to block switching while Steam is running — existing
- Timestamped save/blueprint backups created automatically before each switch — existing
- Backup list, create, and restore commands — existing
- SMM (Satisfactory Mod Manager) profile linking per branch — existing
- SMM profile activation on switch via installations.json mutation — existing
- Status command showing current branch, saves, and linked profiles — existing
- Cross-platform path resolution (Windows Steam, Linux Steam, Proton, Flatpak) — existing
- Save file header parsing for version metadata display — existing

### Active

- [ ] Codebase compiles without errors on stable Rust toolchain
- [ ] Binary runs correctly on Windows (Steam install)
- [ ] Binary runs correctly on Linux (Steam + Proton paths)
- [ ] All existing commands behave as documented (switch, status, backup, profile)

### Out of Scope

- GUI / TUI — CLI-only for v1
- Support for games other than Satisfactory — hardcoded app ID is intentional for now
- Backup encryption or compression — filesystem storage is sufficient for v1
- Automatic backup retention/cleanup — manual management acceptable for v1
- Real-time progress bars for file operations — silent copy acceptable for v1

## Context

- Binary name: `satis-switcher`; repo directory: `ficswitcher`
- Written in Rust 2021 edition, uses Cargo
- Key crates: clap 4 (CLI), anyhow (errors), serde_json (config/backup manifests), sysinfo (Steam detection), dirs 5 (platform paths), chrono (timestamps), colored (terminal output)
- Codebase map available at `.planning/codebase/` — generated 2026-03-19
- Known fragile areas: VDF/ACF parser, SMM installations.json mutation, process detection by name substring
- Build errors are the current blocker; their exact nature needs investigation

## Constraints

- **Tech stack**: Rust — no runtime changes; must stay Rust CLI
- **Compatibility**: Must support Windows + Linux (Steam native and Proton)
- **Distribution**: Public release — binary must be self-contained, no runtime dependencies beyond the OS
- **SMM integration**: Reads/writes SMM config files directly (no official API); must not corrupt SMM state

## Key Decisions

| Decision | Rationale | Outcome |
|---|---|---|
| Rust CLI (not GUI) | Fast, self-contained binary; no runtime deps | — Pending |
| Direct ACF mutation | Steam has no API for branch switching | — Pending |
| Direct SMM file mutation | SMM has no plugin/extension API | — Pending |
| Backup before every switch | Protects saves against branch incompatibility | — Pending |

---
*Last updated: 2026-03-19 after initialization*
