# External Integrations

**Analysis Date:** 2026-03-19

## APIs & External Services

**Game Platform:**
- Steam (Valve) - Satisfactory distribution and branch management
  - Integration point: Steam appmanifest file parsing and modification
  - App ID: `526870` (Satisfactory)
  - Used for: Detecting and switching between stable/experimental branches

## Data Storage

**Databases:**
- None - CLI application uses local filesystem only

**File Storage:**
- **Local Filesystem Only**
  - Backup storage: `~/.local/share/satis-switcher/backups/` (Linux) or `%APPDATA%\Local\satis-switcher\backups\` (Windows)
  - Config storage: Platform-specific data directories via `dirs` crate
  - Manifest files: JSON format at `.planning/codebase/` locations
  - Save files: Satisfactory save directory (platform-dependent)
  - Blueprints: Satisfactory blueprints subdirectory

**Caching:**
- None detected

## Authentication & Identity

**Auth Provider:**
- None - Application operates locally with file system permissions
- No API authentication required
- Relies on file system access to Steam installation and user saves directory

## Monitoring & Observability

**Error Tracking:**
- None configured

**Logs:**
- Console output only via `println!()` and `colored` crate for terminal display
- No persistent logging to files
- Error messages printed to stdout with colored formatting

## CI/CD & Deployment

**Hosting:**
- Local CLI application (no server/cloud deployment)
- Distributed as compiled binary or via Cargo

**CI Pipeline:**
- None detected

## Environment Configuration

**Required env vars:**
- None explicitly required
- Optional platform-specific vars:
  - Windows: `ProgramFiles(x86)`, `ProgramFiles` (for Steam location fallback)
  - Linux: `HOME` (implicit, used by `dirs` crate)

**Secrets location:**
- Not applicable - No credentials or secrets managed
- File-based: All data in local file system
- No .env files required

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Steam Integration Details

**Manifest File Parsing:**
- Reads and parses `appmanifest_526870.acf` files (custom VDF/ACF flat parser in `src/core/steam.rs`)
- Detects current branch from `UserConfig.betakey` and `MountedConfig.betakey` fields
- Atomic write pattern: writes to temporary file, then renames to avoid corruption

**Branch Detection:**
- Stable branch: `betakey` is empty string or `""`
- Experimental branch: `betakey` is `"experimental"`

**Satisfactory Save File Structure:**
- Saves directory: `Satisfactory/Saved/SaveGames/`
- Blueprint directory: `Satisfactory/Saved/SaveGames/blueprints/`
- Save header parsing for metadata display (header version, save version, build version)

## Process Integration

**Steam Running Detection:**
- Uses `sysinfo` crate to enumerate system processes
- Checks for processes containing "steam" in name (case-insensitive)
- Excludes SteamVR processes (filters names containing "steamvr")
- Prevents branch switching while Steam is actively running

---

*Integration audit: 2026-03-19*
