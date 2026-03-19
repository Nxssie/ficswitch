# ficswitch

> A CLI tool for Satisfactory players who run both stable and experimental branches.

[![Release](https://img.shields.io/github/v/release/Nxssie/ficswitch?style=flat-square&color=orange)](https://github.com/Nxssie/ficswitch/releases/latest)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-b7410e?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux-blue?style=flat-square&logo=linux)](https://github.com/Nxssie/ficswitch/releases/latest)

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

```
ficswitch switch stable

→ Switching from experimental to stable
✓ Saves synced to profile 'experimental' (15 files)
📦 Creating backup of saves...
✓ Backup created: 20260319_130118 (15 saves)
⚙ Restoring stable from cache...
✓ Restored 1182 files from cache
✓ SMM profile activated: stable

✓ Done! Launch Satisfactory directly — no Steam download needed.
```

---

## Features

| | |
|---|---|
| **Instant branch switching** | Hardlink restore instead of Steam download |
| **Automatic save backups** | Snapshot before every switch |
| **SMM profile integration** | Each branch maps to its own mod profile |
| **Mod deployment** | Extracts SML + mods from the SMM download cache before caching |
| **Cache management** | Create, inspect, and clear per-branch caches |

---

## Installation

### Prebuilt binary

Download the latest binary from [Releases](https://github.com/Nxssie/ficswitch/releases/latest) and place it somewhere in your `$PATH`.

### From source

Requires Rust 2021+.

```sh
git clone https://github.com/Nxssie/ficswitch
cd ficswitch
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

### First time setup

```sh
# 1. Link your SMM profiles to each branch
ficswitch profile link stable my-stable-mods
ficswitch profile link experimental experimental-mods

# 2. Cache the current branch's game files
ficswitch cache create

# 3. Switch to the other branch — Steam downloads it the first time
ficswitch switch experimental

# 4. Cache it too
ficswitch cache create
```

From now on, every switch is instant.

### Day-to-day

```sh
ficswitch switch stable       # instant, no download needed
ficswitch switch experimental # instant, no download needed
ficswitch status              # show current state
ficswitch cache status        # show cached branches
ficswitch backup list         # list all save backups
```

### Reference

**Cache**
```sh
ficswitch cache create          # cache the current branch's game files
ficswitch cache status          # show cached branches and file counts
ficswitch cache clear <branch>  # clear cache for a branch
```

**Backups**
```sh
ficswitch backup create                 # create a manual backup
ficswitch backup list                   # list all backups
ficswitch backup restore <id>           # restore a backup by ID
```

**Profiles**
```sh
ficswitch profile link <name> <branch>  # link an SMM profile to a branch
ficswitch profile show                  # show current mappings
ficswitch profile list                  # list available SMM profiles
```

---

## Platform

Linux (Steam/Proton). Paths are resolved automatically for the standard Steam layout.
Windows support is not planned.

---

## Why Rust

Fast, single binary, no runtime. The hardlink pass over ~1200 game files needs to be quick — it runs on every switch.
