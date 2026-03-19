# Coding Conventions

**Analysis Date:** 2026-03-19

## Naming Patterns

**Files:**
- Module files use snake_case: `steam.rs`, `backup.rs`, `profiles.rs`, `saves.rs`
- Entry point is `main.rs`
- Module groups organized by subdirectory: `src/core/`, `src/commands/`, `src/config/`

**Functions:**
- Public functions use snake_case: `find_manifest()`, `detect_branch()`, `create_backup()`, `parse_vdf_flat()`
- Private helper functions also use snake_case: `find_steam_dir()`, `copy_dir_recursive()`, `find_steam_id_dir()`
- Command handlers follow verb_object pattern: `create()`, `list()`, `restore()`, `link()`, `show()`

**Variables:**
- Local variables use snake_case: `backup_base`, `manifest_path`, `target_branch`, `save_dir`
- Constants use SCREAMING_SNAKE_CASE: `SATISFACTORY_APP_ID`, `SAMPLE_ACF`

**Types and Structs:**
- Public structs use PascalCase: `Branch`, `BackupManifest`, `SaveHeader`, `SmmProfiles`, `SmmInstallations`, `BranchProfiles`, `ModEntry`, `InstallationConfig`
- Enum variants use PascalCase: `Stable`, `Experimental`

## Code Style

**Formatting:**
- Standard Rust formatting applied (consistent spacing, indentation)
- 4-space indentation throughout
- Line length varies but generally stays readable
- Empty lines used for logical separation between code blocks

**Linting:**
- Default clippy lints assumed (no explicit configuration)
- No explicit linting configuration files present
- Code follows standard Rust idioms and conventions

## Import Organization

**Order:**
1. Standard library imports (`use std::...`)
2. External crate imports (anyhow, serde, chrono, colored, sysinfo, dirs)
3. Internal crate imports (`use crate::...`)
4. Module declarations (`mod ...`)

**Example from `src/core/steam.rs`:**
```rust
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::System;
```

**Path Aliases:**
- No path aliases configured
- Explicit relative paths used: `crate::core::`, `super::`

## Error Handling

**Patterns:**
- All fallible functions return `Result<T>` where T is the success type
- anyhow crate used for error propagation and context
- `?` operator used for early returns on error
- Custom error messages added with `.with_context()` for file operations:
  ```rust
  fs::read_to_string(manifest_path)
      .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?
  ```
- `anyhow!()` macro used for constructing errors inline:
  ```rust
  Err(anyhow!("Could not find Satisfactory appmanifest (AppID {})", SATISFACTORY_APP_ID))
  ```
- Graceful degradation in UI layer (`src/commands/`) with optional logging instead of panics

**Error Recovery:**
- `match` expressions used for conditional error handling in commands
- Warnings printed to stderr for non-fatal errors: `println!("{} {}", "⚠".yellow(), e)`
- Operations continue despite errors where possible (e.g., backup failures don't halt branch switch)

## Logging

**Framework:** Console output via `println!()` macro

**Patterns:**
- Status messages prefixed with Unicode symbols: `"→".cyan()`, `"📦".cyan()`, `"✓".green()`, `"⚠".yellow()`, `"ℹ".blue()`
- Colored output using `colored` crate: `.green()`, `.red()`, `.yellow()`, `.cyan()`, `.bold()`, `.dimmed()`
- Three-level output:
  - Success: `"✓".green()`
  - Warning: `"⚠".yellow()`
  - Info: `"ℹ".blue()`
  - Action: `"→".cyan()`, `"📦".cyan()`, `"⚙".cyan()`

**Usage example from `src/commands/switch.rs`:**
```rust
println!("{} Switching from {} to {}", "→".cyan(), current_branch.to_string().bold(), target_branch.to_string().bold());
```

## Comments

**When to Comment:**
- Module-level documentation comments used for public functions and types
- Inline comments explain complex logic or file format details
- Comments describe the "why" rather than the "what"

**Examples:**
- `/// Parse a flat VDF/ACF file into key-value pairs.`
- `/// This is a simplistic parser that works for appmanifest ACF files.`
- `// Steam status` - section headers
- `// Save info` - section headers
- `// Format: 3x int32 little-endian` - data format explanations

**Documentation Comments:**
- Doc comments use `///` for items
- No rustdoc format enforced but documentation is present for public APIs
- Example from `src/core/steam.rs`:
  ```rust
  /// Detect the current branch from the appmanifest file.
  pub fn detect_branch(manifest_path: &Path) -> Result<Branch> { ... }
  ```

## Function Design

**Size:**
- Functions range from 3 lines (getters) to ~50 lines (complex parsers)
- Most functions stay under 30 lines
- Single responsibility principle followed

**Parameters:**
- Functions take ownership where mutation is needed
- References (`&`) used for read-only access
- Path parameters use `&Path` for maximum compatibility
- Optional parameters use `Option<T>`: `label: Option<&str>`

**Return Values:**
- All fallible operations return `Result<T>`
- Success type is the primary return, not a tuple
- Option<T> used for nullable values: `Ok(None)` when a profile isn't linked

## Module Design

**Exports:**
- Public items explicitly marked with `pub`
- Command functions declared `pub fn` in `src/commands/*`
- Core logic functions declared `pub fn` in `src/core/*`
- Private helper functions (no `pub` keyword)

**Barrel Files:**
- `src/core/mod.rs` exports all submodules:
  ```rust
  pub mod steam;
  pub mod saves;
  pub mod backup;
  pub mod profiles;
  ```
- `src/commands/mod.rs` exports command submodules:
  ```rust
  pub mod status;
  pub mod switch;
  pub mod backup;
  pub mod profile;
  ```
- Main `src/main.rs` imports via crate namespace: `crate::core::`, `commands::`

## Trait Implementations

**Standard Traits:**
- `Debug` derived via `#[derive(Debug)]` on most structs
- `Clone` derived where needed: `#[derive(Clone)]`
- `PartialEq` derived on enums for comparisons: `#[derive(PartialEq)]`
- `Serialize`/`Deserialize` from serde for JSON serialization: `#[derive(Serialize, Deserialize)]`

**Custom Traits:**
- `Display` implemented manually for `Branch` enum to control string representation
- Example from `src/core/steam.rs`:
  ```rust
  impl fmt::Display for Branch {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          match self {
              Branch::Stable => write!(f, "stable"),
              Branch::Experimental => write!(f, "experimental"),
          }
      }
  }
  ```

---

*Convention analysis: 2026-03-19*
