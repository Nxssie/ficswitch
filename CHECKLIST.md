# ficswitch v1.0 Release Checklist

Pre-release manual smoke tests. Check each item off before tagging v1.0.
Build first: `cargo build --release` then use `./target/release/ficswitch` (Linux) or `.\target\release\ficswitch.exe` (Windows).

## Automated Tests (run first)

- [ ] `~/.cargo/bin/cargo test` — all tests green
- [ ] `~/.cargo/bin/cargo build --release` — zero warnings, zero errors

## CMD-01: status command (requires live Steam + Satisfactory)

Test on each target platform.

- [ ] Run: `ficswitch status`
- [ ] PASS if: output shows "Current branch: stable" or "Current branch: experimental" (not an error)
- [ ] PASS if: output shows "Steam: running" or "Steam: not running"
- [ ] PASS if: output shows "Save directory:" with a valid path (not "Save directory: Could not find...")
- [ ] PASS if: output shows "Profile mappings:" section (may be "none configured")
- [ ] FAIL if: any panic, backtrace, or "thread 'main' panicked" appears

## PLAT-01: Windows Steam detection

Requires: Windows machine with Steam installed in default location (Program Files (x86)\Steam or Program Files\Steam).

- [ ] Run: `ficswitch status`
- [ ] PASS if: "Manifest:" line shows a path containing `steamapps\appmanifest_526870.acf`
- [ ] PASS if: "Current branch:" is shown (stable or experimental)
- [ ] FAIL if: "Satisfactory not found" error when Satisfactory IS installed

## PLAT-02: Linux Steam detection

Requires: Linux machine with native Steam installed (not Flatpak).

- [ ] Run: `ficswitch status`
- [ ] PASS if: "Manifest:" path is under `~/.local/share/Steam/` or `~/.steam/steam/`
- [ ] PASS if: branch is correctly detected

Requires: Linux machine with Flatpak Steam installed.

- [ ] Run: `ficswitch status`
- [ ] PASS if: "Manifest:" path is under `~/.var/app/com.valvesoftware.Steam/`

## PLAT-03: Windows save directory

Requires: Windows machine with Satisfactory installed and at least one save file.

- [ ] Run: `ficswitch status`
- [ ] PASS if: "Save directory:" shows a path containing `AppData\Local\FactoryGame\Saved\SaveGames`
- [ ] PASS if: "Save files:" count is > 0

## PLAT-04: Linux Proton save directory (native Steam)

Requires: Linux machine with native Steam + Satisfactory via Proton + at least one save.

- [ ] Run: `ficswitch status`
- [ ] PASS if: "Save directory:" shows a path containing `.local/share/Steam/steamapps/compatdata/526870/`
- [ ] PASS if: "Save files:" count is > 0

## PLAT-05: Flatpak Proton save directory fallback

Requires: Linux machine with Flatpak Steam + Satisfactory via Proton + at least one save.

- [ ] Run: `ficswitch status`
- [ ] PASS if: "Save directory:" shows a path containing `.var/app/com.valvesoftware.Steam/`
- [ ] PASS if: "Save files:" count is > 0

## ERR-01: Switch while Steam running (any platform)

Requires: Steam running in background.

- [ ] Start Steam
- [ ] Run: `ficswitch switch experimental` (or stable if already on experimental)
- [ ] PASS if: output contains "Steam is currently running. Please close Steam before switching branches."
- [ ] PASS if: process exits non-zero (no branch switch occurs)
- [ ] FAIL if: panic or switch proceeds while Steam is running

## ERR-02: Satisfactory not installed (any platform)

Requires: Machine without Satisfactory installed, OR pointing to a non-existent Steam library.

- [ ] Run: `ficswitch status`
- [ ] PASS if: output contains "Satisfactory not found:" with a human-readable message
- [ ] FAIL if: panic or empty output

## ERR-03: Profile link with no SMM (any platform)

Requires: Machine without SMM installed (no `~/.local/share/ficsit/profiles.json` on Linux, no `%APPDATA%\ficsit\profiles.json` on Windows).

- [ ] Run: `ficswitch profile link MyProfile stable`
- [ ] PASS if: output contains a warning about SMM having no profiles
- [ ] PASS if: process exits 0 (non-fatal)
- [ ] FAIL if: process exits non-zero or shows a backtrace

## Full Integration: switch command (PLAT-01..05 prerequisite must pass first)

Requires: Live Steam + Satisfactory NOT running, SMM installed with at least one profile.

- [ ] Run: `ficswitch profile link <your-profile-name> experimental`
- [ ] Confirm: exits 0, shows "Linked profile..."
- [ ] Run: `ficswitch switch experimental`
- [ ] PASS if: "Branch set to experimental" appears
- [ ] PASS if: Backup was created (shown in output)
- [ ] PASS if: SMM profile activated (shown in output)
- [ ] PASS if: `ficswitch status` now shows "Current branch: experimental"
- [ ] Switch back: `ficswitch switch stable` — confirm symmetric behavior

## Sign-Off

- [ ] All automated tests green on CI/dev machine
- [ ] All PLAT checks completed on both Windows and Linux
- [ ] All ERR checks completed
- [ ] Full integration test completed
- Tester: ________________  Date: ________________
