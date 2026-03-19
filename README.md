# ficswitch

A CLI tool for Satisfactory players who run both stable and experimental branches.
Switches branches instantly from the terminal — no Steam re-downloads, saves backed up automatically, mods in sync.

---

## The problem

Switching between Satisfactory's stable and experimental branches via Steam means:

- Waiting for Steam to download gigabytes of game files every time
- Manually swapping SMM mod profiles
- Hoping your saves don't get mixed up between versions

ficswitch solves all three.

---

## How it works

ficswitch uses **hardlinks** to cache each branch's game files locally.
Switching branches copies nothing — it just rewires which files the game dir points to.
The first cache creation is slow (Steam download + hardlink pass). Every switch after that is instant.

Before caching, it also deploys your SMM mods from the local download cache into the game directory, so mods are included in the snapshot and don't need to be re-fetched.

---

## Features

- **Instant branch switching** — hardlink restore instead of Steam download
- **Automatic save backups** before every switch
- **SMM profile integration** — each branch maps to its own mod profile
- **Mod deployment** — extracts SML + mods from the SMM download cache before caching
- **Cache management** — create, inspect, and clear per-branch caches

---

## Installation

Requires Rust 2021+.

```sh
git clone https://github.com/nxssie/ficswitcher
cd ficswitcher
cargo install --path .
```

---

## Usage

```
ficswitch <COMMAND>

Commands:
  status   Show current branch, saves, and profile mappings
  switch   Switch to a different branch
  backup   Manage save backups
  profile  Manage SMM profile-branch associations
  cache    Manage local branch game file cache
```

### Typical workflow

**First time setup — cache both branches:**

```sh
# On stable
ficswitch cache create

# Switch to experimental (Steam downloads it), then cache it too
ficswitch switch experimental
ficswitch cache create
```

**Day-to-day switching:**

```sh
ficswitch switch stable
# → backup created, 1183 files restored from cache, SMM profile activated
# → launch Satisfactory directly, no Steam download needed
```

**Check what's going on:**

```sh
ficswitch status
ficswitch cache status
ficswitch backup list
```

### Branch cache

```sh
ficswitch cache create   # Cache the current branch's game files
ficswitch cache status   # Show cached branches and file counts
ficswitch cache clear    # Clear the cache for a branch
```

### Save backups

Backups are created automatically on every switch. You can also manage them manually:

```sh
ficswitch backup create
ficswitch backup list
ficswitch backup restore <id>
```

### SMM profile mapping

ficswitch maps each branch to an SMM profile. When you switch branches, the matching profile is activated automatically.

```sh
ficswitch profile set stable my-stable-mods
ficswitch profile set experimental experimental-mods
```

---

## Platform

Linux (Steam/Proton). Paths are resolved automatically for the standard Steam layout.

---

## Why Rust

Fast, single binary, no runtime. The hardlink pass over ~1200 game files needs to be quick — it runs on every switch.
