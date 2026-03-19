# Codebase Concerns

**Analysis Date:** 2026-03-19

## Tech Debt

**VDF Parser Fragility:**
- Issue: The VDF/ACF file parser in `src/core/steam.rs` uses a simplistic line-by-line approach with manual brace tracking and string splitting. This is error-prone for malformed files and edge cases.
- Files: `src/core/steam.rs` (lines 44-82)
- Impact: Parser may silently fail on ACF files with unexpected formatting, whitespace, or quoting variations. The `set_betakey_in_acf` function depends on exact formatting assumptions and could produce invalid output if the original file doesn't match expectations.
- Fix approach: Replace with a proper VDF parser library or implement more robust state machine parsing that handles escaping, inline comments, and structural variations.

**Backup Restoration Overwrites Without Confirmation:**
- Issue: The `restore_backup` function in `src/core/backup.rs` (lines 114-142) directly overwrites existing saves and blueprints without warning or prompting the user.
- Files: `src/core/backup.rs` (lines 114-142), `src/commands/backup.rs` (lines 60-82)
- Impact: User could lose recent saves by accidentally restoring an old backup. No undo mechanism exists once files are overwritten.
- Fix approach: Add interactive confirmation for restore operations, implement versioning or temporary backup of current saves before restoration, or add a dry-run mode.

**Process Detection Using Process Name String Matching:**
- Issue: Steam detection in `src/core/steam.rs` (lines 175-181) uses substring matching on process name (`"steam"` and `"steamvr"`). This is fragile and could match unrelated processes.
- Files: `src/core/steam.rs` (lines 175-181)
- Impact: False positives if other processes contain "steam" in their name. Could prevent switching when Steam is not actually running, or allow switching when it is.
- Fix approach: Use platform-specific process IDs or check for specific executable paths/names more precisely.

**No Validation of Profile Links:**
- Issue: In `src/core/profiles.rs` (lines 108-127), the `link_profile` function validates that a SMM profile exists, but the `activate_profile_for_branch` function (lines 130-161) doesn't verify that the linked profile is still valid before writing to `installations.json`.
- Files: `src/core/profiles.rs` (lines 130-161)
- Impact: If a profile is deleted from SMM after being linked, the tool will set an invalid profile reference in installations.json, potentially breaking SMM.
- Fix approach: Add re-validation before activation, or handle SMM configuration errors more gracefully.

**Hardcoded Paths and IDs:**
- Issue: Steam app ID, manifest extensions, and path components are hardcoded throughout the codebase.
- Files: `src/core/steam.rs` (line 8), `src/core/saves.rs` (line 6), `src/core/backup.rs` (lines 21-32)
- Impact: Makes it difficult to add support for other games or configurations. Any change to Steam's structure or Satisfactory's app ID requires code changes.
- Fix approach: Externalize configuration to a config file or environment variables.

## Security Considerations

**Backup Directory Permissions:**
- Risk: Backup files contain save game data and blueprints which may contain user's personal creations/designs. Backups are stored in user data directories with default OS permissions.
- Files: `src/core/backup.rs` (lines 21-32)
- Current mitigation: Uses standard user data directories which are typically user-only readable on modern OS.
- Recommendations: Document the backup storage location, consider encrypting backups, or warn users about backup security implications.

**No Validation of Restored File Paths:**
- Risk: Restore operation copies files without validating their paths. Malformed backup manifests could theoretically cause path traversal issues.
- Files: `src/core/backup.rs` (lines 125-141)
- Current mitigation: Limited by filesystem permissions and the fact that backups are created by the tool itself.
- Recommendations: Validate all file paths before restoring, ensure no `..` components or absolute paths in restored files.

**SMM Installation Path in Config:**
- Risk: The tool reads and writes to SMM's `installations.json` using the Satisfactory install path as a key (string). If paths contain special characters or are user-controlled, this could cause issues.
- Files: `src/core/profiles.rs` (lines 138-161)
- Current mitigation: Paths come from parsing Steam manifest, not user input.
- Recommendations: Normalize path separators and handle edge cases in path serialization.

## Performance Bottlenecks

**Process List Scan on Every Branch Check:**
- Problem: `is_steam_running()` creates a full system process list on every call. During switch operations, this happens once, but status/monitoring could call this repeatedly.
- Files: `src/core/steam.rs` (lines 175-181)
- Cause: Using `sysinfo::System::new_all()` which is a full system scan.
- Improvement path: Cache the system info for a few seconds, or use lightweight process enumeration specific to Steam.

**Recursive Blueprint Directory Traversal:**
- Problem: Blueprint backup recursively traverses entire directory structure to count files. With large blueprint collections, this could be slow.
- Files: `src/core/saves.rs` (lines 134-147)
- Cause: Separate `collect_blueprints` function traverses entire tree just to count files.
- Improvement path: Count files during copy instead of in separate traversal, or cache blueprint count.

**No Progress Indication for Large Backups:**
- Problem: Large save files or blueprint collections could take time to copy, with no user feedback.
- Files: `src/core/backup.rs` (lines 49-69), `src/core/saves.rs` (lines 134-147)
- Cause: Simple file copy loop without progress reporting.
- Improvement path: Add progress bar or status updates for file operations.

## Fragile Areas

**ACF File Modification:**
- Files: `src/core/steam.rs` (lines 206-262)
- Why fragile: The `set_betakey_in_acf` function parses and reconstructs the entire ACF file based on exact formatting assumptions. A single deviation in whitespace, escaping, or structure could corrupt the file. The function handles both `UserConfig` and `MountedConfig` sections but doesn't validate that it successfully updated both.
- Safe modification: Add validation that both sections were updated before returning. Test extensively against real Steam ACF files with various formatting. Consider adding a pre-modification backup.
- Test coverage: Basic tests exist in `src/core/steam.rs` (lines 281-332) but only test the `set_betakey_in_acf` function with a sample. No integration tests of the full switch flow exist.

**SMM Configuration Mutations:**
- Files: `src/core/profiles.rs` (lines 138-161)
- Why fragile: The function reads SMM's `installations.json`, modifies it, and writes it back without any locking or transactional guarantees. If SMM is reading/writing simultaneously, corruption could occur.
- Safe modification: Implement file locking or atomic writes (write to temp file, rename). Consider reading the file again before writing to detect concurrent modifications.
- Test coverage: No tests for profile activation. No tests for concurrent access scenarios.

**Directory Creation Race Conditions:**
- Files: `src/core/backup.rs` (lines 41-47), `src/core/backup.rs` (lines 145-146)
- Why fragile: Multiple calls to `fs::create_dir_all` with the same path could race in rare scenarios, though Rust's error handling should catch this.
- Safe modification: Validate directory existence after creation, or use more defensive patterns for concurrent operations.
- Test coverage: No tests for concurrent backup creation.

## Scaling Limits

**Backup Storage:**
- Current capacity: Unlimited storage (just filesystem constraints)
- Limit: User's disk space will be consumed by backups. With large save files (100MB+ for late-game saves) and many backups, storage could become an issue quickly.
- Scaling path: Add backup size tracking, implement backup retention policies, add deletion functionality, or add compression.

**SMM Profiles:**
- Current capacity: Limited by SMM's ability to store profiles (not documented)
- Limit: With many profiles and mod counts, reading/writing `profiles.json` and `installations.json` could become slow.
- Scaling path: Implement caching or lazy loading of profile data.

## Test Coverage Gaps

**Branch Switching Integration:**
- What's not tested: The full flow from `commands::switch::run()` through manifest modification, backup creation, and profile activation is not tested end-to-end.
- Files: `src/commands/switch.rs`, integration with `src/core/steam.rs`, `src/core/backup.rs`, `src/core/profiles.rs`
- Risk: Regressions in the complete workflow could only be caught by manual testing.
- Priority: High - This is the primary feature.

**SMM Profile Operations:**
- What's not tested: `profiles::activate_profile_for_branch()` is not tested. The interaction with SMM configuration files is only unit-tested for reading, not writing.
- Files: `src/core/profiles.rs` (lines 130-161)
- Risk: Profile activation could silently fail or corrupt SMM configuration.
- Priority: High - Could break SMM for users.

**Error Scenarios:**
- What's not tested: Most error paths (missing directories, permission errors, malformed config files) are not tested.
- Files: Across all core modules
- Risk: Error messages may be unclear, recovery may not be graceful, or edge cases may crash instead of providing helpful feedback.
- Priority: Medium - Affects user experience but less critical than happy path.

**Platform-Specific Paths:**
- What's not tested: Windows and Linux path detection are not unit-tested. Tests would need to run on both platforms.
- Files: `src/core/steam.rs` (lines 100-136), `src/core/saves.rs` (lines 33-69)
- Risk: Path detection could fail on new configurations or OS updates.
- Priority: Medium - Blocking on platform, hard to test in CI.

**Save Header Parsing:**
- What's not tested: The test in `src/core/saves.rs` (lines 159-178) manually creates a test file, but doesn't test edge cases like truncated files or corrupted headers.
- Files: `src/core/saves.rs` (lines 15-30)
- Risk: Malformed save files could panic instead of providing helpful error messages.
- Priority: Low - Unlikely scenario, already has basic test.

## Known Issues

**Backup Restore Without Branch Awareness:**
- Symptoms: User can restore a backup from the stable branch while on experimental branch, potentially causing compatibility issues.
- Files: `src/commands/backup.rs` (lines 60-82), `src/core/backup.rs` (lines 114-142)
- Cause: The `restore_backup` function doesn't check if the backup's recorded branch matches the current branch.
- Workaround: User should manually verify they're switching to the correct branch before restoring.
- Fix: Compare backup branch with current branch and warn user.

**Steam Running Check is Unreliable:**
- Symptoms: User may be unable to switch branches despite Steam being closed, or may get false warnings.
- Files: `src/core/steam.rs` (lines 175-181)
- Cause: Process name string matching is imprecise and platform-dependent.
- Workaround: Close all Steam-related processes and ensure they're not in the process list.
- Fix: Use more reliable detection method (lock files, port binding, platform APIs).

**No Handling for SMM Not Installed:**
- Symptoms: Switching branch fails with unclear error when SMM is not installed and profile linking is attempted.
- Files: `src/commands/switch.rs` (lines 78-112)
- Cause: Errors from SMM operations are caught and printed but don't prevent the branch switch from being reported as successful.
- Workaround: Ignore profile warnings if SMM not used.
- Fix: Make profile operations truly optional or provide clearer error messages.

## Recommendations

**Priority 1 (Critical):**
1. Add integration tests for the full branch switching workflow
2. Implement validation and error handling for SMM profile activation
3. Add confirmation/dry-run for backup restoration

**Priority 2 (Important):**
1. Replace simplistic VDF parser with robust implementation
2. Improve Steam process detection reliability
3. Add unit tests for error scenarios

**Priority 3 (Nice to Have):**
1. Add progress indicators for file operations
2. Implement backup retention/cleanup policies
3. Externalize configuration and magic numbers

---

*Concerns audit: 2026-03-19*
