# Codebase Structure

**Analysis Date:** 2026-03-19

## Directory Layout

```
ficswitcher/
├── src/                      # Rust source code
│   ├── main.rs              # CLI entry point, command routing
│   ├── commands/            # User-facing command handlers
│   │   ├── mod.rs           # Module exports
│   │   ├── status.rs        # Display system status
│   │   ├── switch.rs        # Branch switching orchestration
│   │   ├── backup.rs        # Backup management UI
│   │   └── profile.rs       # SMM profile management UI
│   ├── core/                # Domain logic and integrations
│   │   ├── mod.rs           # Module exports
│   │   ├── steam.rs         # Steam directory/manifest operations
│   │   ├── saves.rs         # Satisfactory save file handling
│   │   ├── backup.rs        # Backup creation and restoration
│   │   └── profiles.rs      # SMM profile and configuration I/O
│   └── config/              # Configuration management
│       └── mod.rs           # Placeholder for future config
├── Cargo.toml               # Package manifest and dependencies
├── Cargo.lock               # Locked dependency versions
├── .gitignore               # Rust/Cargo build artifacts
└── .planning/               # GSD planning documents
    └── codebase/            # Architecture and structure analysis
```

## Directory Purposes

**`src/`:**
- Purpose: All Rust source code
- Contains: Main entry point, command handlers, core business logic
- Key files: `main.rs` orchestrates Clap parsing and command dispatch

**`src/commands/`:**
- Purpose: User-facing CLI command implementations
- Contains: Four public modules exporting handler functions
- Key files:
  - `status.rs`: Diagnostic display, no mutations
  - `switch.rs`: Primary workflow with precondition checks and side effects
  - `backup.rs`: Backup creation/listing/restoration
  - `profile.rs`: SMM profile linking and listing
- Pattern: Each module exports functions matching command structure (some return Result<()>, some take parameters)

**`src/core/`:**
- Purpose: Domain-specific business logic, platform integration, persistence
- Contains: Four modules for distinct concerns
- Key files:
  - `steam.rs`: 283 lines - Manifest parsing, branch detection, VDF file handling, process detection
  - `saves.rs`: 180 lines - Save file discovery, header parsing, blueprint listing
  - `backup.rs`: 164 lines - Backup creation, restoration, manifest management
  - `profiles.rs`: 175 lines - SMM profile reading, branch-profile association, configuration I/O
- Responsibility: Encapsulates all filesystem I/O, system calls, and file format parsing

**`src/config/`:**
- Purpose: Placeholder for future configuration system
- Contains: Empty module stub
- Status: Not yet implemented (comment placeholder only)

**`.planning/`:**
- Purpose: GSD codebase analysis documents
- Contains: Architecture and structure documentation
- Generated: By GSD tooling, not part of source tree

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point, defines CLI structure with Clap derive macros (lines 1-102)
  - Declares Commands enum with Status, Switch, Backup, Profile variants
  - Declares nested BackupAction and ProfileAction enums
  - Routes each variant to handler functions
  - Executes with error handling via `anyhow::Result`

**Configuration:**
- `Cargo.toml`: Package metadata and dependency declarations
  - Edition: 2021
  - Dependencies: clap, serde, serde_json, anyhow, thiserror, chrono, colored, sysinfo, dirs

**Core Logic:**
- `src/core/steam.rs`: Platform-independent Steam/Satisfactory integration
  - Branch enum and parsing (lines 10-40)
  - VDF file parser (lines 44-82)
  - Manifest detection and reading (lines 139-172)
  - Branch switching with atomic writes (lines 183-203)
  - Process detection (lines 174-181)

- `src/core/saves.rs`: Save file discovery and metadata extraction
  - Save directory location (platform-specific, lines 32-69)
  - Save file listing (lines 105-123)
  - Blueprint recursive discovery (lines 125-152)
  - Binary header parsing: 3x i32 little-endian (lines 15-30)

- `src/core/backup.rs`: Backup lifecycle management
  - BackupManifest struct (lines 10-18)
  - Backup root directory resolution (lines 20-33)
  - Backup creation with manifest (lines 35-86)
  - Backup restoration with file copying (lines 113-142)
  - Recursive directory copying (lines 144-163)

- `src/core/profiles.rs`: SMM integration and configuration
  - SMM profile structures: SmmProfiles, SmmInstallations (lines 18-36)
  - Branch-profile mapping storage (lines 38-43)
  - Configuration directory detection (lines 45-70)
  - Profile linking and activation (lines 108-161)
  - JSON file I/O helpers (lines 163-174)

**Testing:**
- Embedded unit tests in core modules:
  - `src/core/steam.rs` lines 280-332: VDF parsing, Branch parsing, ACF modification
  - `src/core/saves.rs` lines 154-179: Save header parsing with temp file creation

## Naming Conventions

**Files:**
- `mod.rs`: Module declaration files (one per directory)
- `<domain>.rs`: Single responsibility files (steam, saves, backup, profiles)
- No test files: Tests embedded in implementation files using `#[cfg(test)]` modules

**Directories:**
- Plural for grouping related modules: `commands/`, `core/`
- Descriptive names mapping to business domain: `steam`, `saves`, `backup`, `profiles`

**Functions:**
- Public functions: `snake_case` with descriptive verbs (find_manifest, detect_branch, list_saves, create_backup)
- Private functions: Same convention, used within modules (parse_vdf_flat, set_betakey_in_acf, copy_dir_recursive, read_json_file)
- Handler exports: match command structure (run, create, list, restore, link, show)

**Types:**
- Enums: PascalCase (Branch, BackupAction, ProfileAction, Commands)
- Structs: PascalCase (SaveHeader, BackupManifest, SmmProfiles, InstallationConfig)
- Generic/complex types: Descriptive (e.g., HashMap<String, String> for VDF flat representation)

**Constants:**
- SCREAMING_SNAKE_CASE: SATISFACTORY_APP_ID = "526870"

## Where to Add New Code

**New Feature:**
- If it's a new command: Create new file in `src/commands/<command_name>.rs`, export from `mod.rs`, add variant to Commands enum in `main.rs`
- If it's cross-cutting: Add to appropriate core module or create new core module following existing patterns
- Primary code: `src/commands/<name>.rs` or `src/core/<domain>.rs`
- Tests: Embedded in same file using `#[cfg(test)] mod tests { ... }`

**New Component/Module:**
- Implementation: `src/core/<component>.rs` (or `src/commands/` if user-facing)
- Export from `src/core/mod.rs` or `src/commands/mod.rs` using `pub mod <component>;`
- Follow existing error handling: Use `anyhow::Result<T>` for all fallible operations
- Use `with_context()` to enrich errors with operation details

**Utilities:**
- Shared helpers within a module: Define as private functions in that module (pattern: `copy_dir_recursive`, `parse_vdf_flat`)
- Shared across modules: Define in appropriate core module and export publicly if needed
- No separate utils directory; keep focused modules together

**Platform-Specific Code:**
- Gate with `cfg!(target_os = "...")` at discovery points
- Use `dirs` crate for path resolution rather than hardcoding
- Document Proton/Flatpak paths alongside standard Windows/Linux paths

## Special Directories

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (.gitignore entry)

**`.planning/codebase/`:**
- Purpose: GSD analysis and planning documents
- Generated: Yes (by GSD tooling)
- Committed: Yes (part of planning artifacts)

---

*Structure analysis: 2026-03-19*
