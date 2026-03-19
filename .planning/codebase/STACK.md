# Technology Stack

**Analysis Date:** 2026-03-19

## Languages

**Primary:**
- Rust 2021 Edition - Full CLI application implementation

## Runtime

**Environment:**
- Rust compiler (via Cargo)

**Package Manager:**
- Cargo (built-in with Rust)
- Lockfile: `Cargo.lock` (present)

## Frameworks

**Core:**
- clap 4 with derive macros - CLI argument parsing and command structure

**Serialization:**
- serde 1 with derive - Serialization/deserialization framework
- serde_json 1 - JSON serialization for backup manifests

**Error Handling:**
- anyhow 1 - Flexible error handling with context
- thiserror 1 - Derive macros for error types

**Utility:**
- chrono 0.4 with serde - Timestamp generation and serialization
- colored 2 - Colored terminal output for CLI display
- sysinfo 0.30 - System process detection and monitoring
- dirs 5 - Cross-platform directory resolution (home, data, config)

## Key Dependencies

**Critical:**
- clap 4 - Enables CLI interface with structured subcommands and flags
- anyhow 1 - Provides error context and Result handling throughout codebase
- serde/serde_json 1 - Enables backup manifest serialization to JSON files

**Infrastructure:**
- sysinfo 0.30 - Detects running Steam process before branch switching
- dirs 5 - Resolves platform-specific paths for Steam, saves, and backups
- chrono 0.4 - Generates timestamps for backup identification (format: YYYYMMDD_HHMMSS)
- colored 2 - Provides visual feedback in terminal (green/yellow/red status indicators)

## Configuration

**Environment:**
- Platform detection via `cfg!(target_os = "windows")` and `cfg!(target_os = "linux")`
- Windows uses `ProgramFiles(x86)` and `ProgramFiles` environment variables for Steam detection
- Linux uses standard XDG-compliant home directory paths

**Build:**
- `Cargo.toml` - Package manifest with dependency specifications
- `Cargo.lock` - Locked dependency versions for reproducible builds
- No custom build script (`build.rs`)

## Platform Requirements

**Development:**
- Rust toolchain with Cargo
- Targets: Windows (x86_64), Linux (x86_64)
- Edition: 2021

**Production:**
- Windows: Steam installation in `Program Files (x86)\Steam` or `Program Files\Steam`
- Linux: Steam installation in standard locations (`.steam/steam`, `.local/share/Steam`, Flatpak paths)
- Read/write access to Satisfactory appmanifest file
- Access to Satisfactory save directory
- Process monitoring capability for Steam detection

---

*Stack analysis: 2026-03-19*
