# Architecture

**Analysis Date:** 2026-03-19

## Pattern Overview

**Overall:** Layered CLI architecture with clear separation between command handlers, core business logic, and system integration.

**Key Characteristics:**
- Clean layering: commands → core modules → system I/O
- Functional error handling using `anyhow::Result` throughout
- Cross-platform path resolution (Windows/Linux with Steam/Proton support)
- Atomic file operations for safe branch switching
- Configuration file abstraction for user preferences and system state

## Layers

**Command Layer:**
- Purpose: Parse user input, route to appropriate handlers, present output
- Location: `src/commands/`
- Contains: Four command modules (status, switch, backup, profile) with public `run()` or action functions
- Depends on: Core modules for business logic, `colored` crate for terminal output
- Used by: `main.rs` which dispatches Clap command enum variants

**Core Logic Layer:**
- Purpose: Implement domain-specific operations for Satisfactory branch management
- Location: `src/core/`
- Contains: Steam integration, save file handling, backup orchestration, mod profile management
- Depends on: System I/O (filesystem, process detection), serialization (`serde`)
- Used by: Command layer and other core modules

**Utilities Layer:**
- Purpose: Provide foundational operations for cross-cutting concerns
- Within core modules: VDF parsing in `steam.rs`, save header parsing in `saves.rs`, JSON configuration I/O in `profiles.rs`
- Patterns: Private helper functions (`parse_vdf_flat`, `set_betakey_in_acf`, `copy_dir_recursive`, `read_json_file`)

**Configuration Layer:**
- Purpose: Placeholder for future configuration management
- Location: `src/config/mod.rs`
- Currently: Empty (commented placeholder)

## Data Flow

**Branch Switch Workflow:**

1. User invokes `satis-switcher switch <branch>`
2. `commands::switch::run()` receives target branch string
3. Validates target branch using `steam::Branch::from_str()`
4. Locates game manifest: `steam::find_manifest()` → searches Steam directories
5. Detects current branch: `steam::detect_branch()` → parses appmanifest ACF file
6. Checks preconditions: Steam must not be running (`steam::is_steam_running()`)
7. Creates backup: `backup::create_backup()` → copies saves and blueprints
8. Modifies manifest: `steam::switch_branch()` → atomic write of betakey value
9. Activates SMM profile: `profiles::activate_profile_for_branch()` → updates installations.json
10. Returns control to user with status messages

**Status Display Workflow:**

1. User invokes `satis-switcher status`
2. `commands::status::run()` queries multiple subsystems in sequence
3. Manifest detection → branch detection → install directory resolution
4. Steam process status check
5. Save directory discovery → list and parse save file headers
6. Profile mappings lookup and display
7. Colored output formatted for readability

**Backup Create/Restore Workflow:**

1. User invokes `satis-switcher backup create` or `satis-switcher backup restore <id>`
2. `commands::backup::create()` calls `backup_core::create_backup()`:
   - Generates timestamp-based backup ID (YYYYMMDD_HHMMSS)
   - Creates backup directory hierarchy
   - Copies .sav files from current save directory
   - Recursively copies blueprints folder
   - Writes manifest.json with metadata
3. `commands::backup::restore()` calls `backup_core::restore_backup()`:
   - Validates backup exists
   - Copies saves back to save directory
   - Recursively restores blueprints

**State Management:**

- **Runtime state**: Passed through function parameters (paths, branch selections)
- **Persistent state**: JSON files in platform-specific data directories:
  - `satis-switcher/backups/` - backup storage with manifest.json per backup
  - `satis-switcher/branch_profiles.json` - branch-to-profile mappings
  - `ficsit/profiles.json` - SMM profile definitions (read-only)
  - `ficsit/installations.json` - SMM installation configurations (modified for activation)
- **System state**: Detected from files (appmanifest ACF, Steam directory structure)

## Key Abstractions

**Branch:**
- Purpose: Represent Satisfactory release channels (stable vs experimental)
- Examples: `src/core/steam.rs` lines 10-40
- Pattern: Enum with `from_str()` constructor, `betakey()` method for ACF representation, `Display` for CLI output

**SaveHeader:**
- Purpose: Encapsulate version metadata extracted from binary save files
- Examples: `src/core/saves.rs` lines 8-13
- Pattern: Struct with three i32 fields (header_version, save_version, build_version)

**BackupManifest:**
- Purpose: Describe a backup with metadata for restoration and listing
- Examples: `src/core/backup.rs` lines 10-18
- Pattern: Serializable struct with `serde`, includes timestamp, branch info, counts

**VDF/ACF Configuration:**
- Purpose: Parse Steam's flat VDF format for appmanifest files
- Examples: `src/core/steam.rs` lines 44-82, 206-262
- Pattern: Custom parser handling nested sections and key-value extraction

**SMM Profiles:**
- Purpose: Represent mod configurations from Satisfactory Mod Manager
- Examples: `src/core/profiles.rs` lines 18-36
- Pattern: Serializable structs mapping profile names to mod entries with enabled status

## Entry Points

**Main Entry:**
- Location: `src/main.rs`
- Triggers: Binary invocation with `satis-switcher <command> [args]`
- Responsibilities:
  - Parse CLI arguments using Clap derive macros
  - Route to appropriate command handler
  - Return result (exits with non-zero on error via anyhow)

**Command Entry Points:**
- `commands::status::run()` - Diagnostic display
- `commands::switch::run(&branch, no_backup)` - Primary workflow
- `commands::backup::create/list/restore()` - Backup management
- `commands::profile::list/link/show()` - SMM profile management

## Error Handling

**Strategy:** Comprehensive error propagation using `anyhow::Result<T>` with context

**Patterns:**
- `with_context()` for enriching errors with operation context (file paths, operations)
- `anyhow!()` for creating custom errors with messages
- `ok_or_else()` for converting Option to Result with dynamic error messages
- Silent graceful degradation in status display (warnings don't halt execution)
- Fatal errors in switch/backup operations propagate to CLI for user notification

**Example from `src/core/steam.rs` lines 85-98:**
```rust
pub fn detect_branch(manifest_path: &Path) -> Result<Branch> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
    // ... parsing logic ...
    Branch::from_str(betakey)
}
```

## Cross-Cutting Concerns

**Logging:** Uses colored output via `colored` crate for visual distinction:
- Status messages with emoji prefixes (✓, ⚠, ℹ, →, 📦, ⚙)
- Error output in red, warnings in yellow, info in blue/cyan
- Implemented in command layer, not centralized

**Validation:**
- Branch string validation: `steam::Branch::from_str()` rejects invalid strings
- Profile existence check: `profiles::link_profile()` verifies against SMM data
- File existence checks before read/write operations

**Authentication:**
- No explicit auth layer; relies on OS file permissions
- Backup/save restoration assumes write access to Satisfactory directories
- SMM profile activation modifies files owned by SMM

**Platform Abstraction:**
- `cfg!(target_os = "windows")` guards at discovery points
- `dirs` crate provides standard data/config directory resolution
- Proton path detection for Linux Steam compatibility (`.local/share/Steam/steamapps/compatdata/`)
- Flatpak Steam path as fallback

---

*Architecture analysis: 2026-03-19*
