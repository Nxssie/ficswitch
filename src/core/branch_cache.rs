use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::steam::Branch;

const SENTINEL: &str = ".cache_complete";

/// Root directory for branch caches.
pub fn cache_root() -> Result<PathBuf> {
    let data_dir = if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Cannot determine local app data directory"))?
            .join("ficswitch")
    } else {
        dirs::data_dir()
            .ok_or_else(|| anyhow!("Cannot determine data directory"))?
            .join("ficswitch")
    };
    Ok(data_dir.join("branch_cache"))
}

fn branch_cache_dir(branch: &Branch) -> Result<PathBuf> {
    Ok(cache_root()?.join(branch.to_string()))
}

/// Returns true if the branch has a complete cache.
pub fn is_cached(branch: &Branch) -> Result<bool> {
    let dir = branch_cache_dir(branch)?;
    Ok(dir.join(SENTINEL).exists())
}

/// Cache the game directory and appmanifest for the given branch using hardlinks.
/// Returns the number of game files hardlinked.
pub fn cache_branch(game_dir: &Path, manifest_path: &Path, branch: &Branch) -> Result<usize> {
    let dest = branch_cache_dir(branch)?;
    fs::create_dir_all(&dest)?;

    // Remove sentinel so a partial cache is never treated as valid.
    let sentinel = dest.join(SENTINEL);
    if sentinel.exists() {
        fs::remove_file(&sentinel)?;
    }

    // Copy the appmanifest so Steam sees a consistent state on restore.
    fs::copy(manifest_path, dest.join("appmanifest.acf"))
        .with_context(|| format!("Failed to cache appmanifest for '{}'", branch))?;

    let count = hardlink_recursive(game_dir, &dest)
        .with_context(|| format!("Failed to cache branch '{}'", branch))?;

    // Mark cache as complete.
    fs::write(&sentinel, "")?;

    Ok(count)
}

/// Restore the game directory and appmanifest from the branch cache using hardlinks.
/// Removes files that exist in game_dir but not in the cache.
/// Returns the number of game files restored.
pub fn restore_branch(game_dir: &Path, manifest_path: &Path, branch: &Branch) -> Result<usize> {
    let src = branch_cache_dir(branch)?;

    if !src.join(SENTINEL).exists() {
        return Err(anyhow!(
            "No complete cache found for branch '{}'. Run 'ficswitch cache create' first.",
            branch
        ));
    }

    let count = hardlink_recursive(&src, game_dir)
        .with_context(|| format!("Failed to restore branch '{}'", branch))?;

    remove_extra(game_dir, &src)?;

    // Restore the appmanifest so Steam sees buildid/depots consistent with the game files.
    fs::copy(src.join("appmanifest.acf"), manifest_path)
        .with_context(|| format!("Failed to restore appmanifest for '{}'", branch))?;

    Ok(count)
}

/// Information about a cached branch.
pub struct CacheInfo {
    pub branch: Branch,
    pub file_count: usize,
}

/// Return info for all cached branches.
pub fn list_caches() -> Result<Vec<CacheInfo>> {
    let root = cache_root()?;
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut infos = Vec::new();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(branch) = Branch::from_str(&name) else { continue };
        if !entry.path().join(SENTINEL).exists() { continue }

        let file_count = count_files(&entry.path())?;
        infos.push(CacheInfo { branch, file_count });
    }

    Ok(infos)
}

/// Clear the cache for a branch.
pub fn clear_cache(branch: &Branch) -> Result<()> {
    let dir = branch_cache_dir(branch)?;
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Recursively hardlink all files from src into dest.
/// Existing files at dest are removed before hardlinking.
fn hardlink_recursive(src: &Path, dest: &Path) -> Result<usize> {
    fs::create_dir_all(dest)?;
    let mut count = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let src_path = entry.path();
        let dest_path = dest.join(&name);

        // Skip sentinel files when restoring from cache into game_dir.
        if name.to_string_lossy() == SENTINEL {
            continue;
        }

        if src_path.is_dir() {
            count += hardlink_recursive(&src_path, &dest_path)?;
        } else {
            if dest_path.exists() {
                fs::remove_file(&dest_path)?;
            }
            fs::hard_link(&src_path, &dest_path)
                .with_context(|| format!("hardlink failed: {}", src_path.display()))?;
            count += 1;
        }
    }

    Ok(count)
}

/// Remove files/dirs in `dir` that don't exist in `reference`.
fn remove_extra(dir: &Path, reference: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let ref_path = reference.join(&name);

        if name.to_string_lossy() == SENTINEL {
            continue;
        }

        if entry.path().is_dir() {
            if ref_path.is_dir() {
                remove_extra(&entry.path(), &ref_path)?;
            } else {
                fs::remove_dir_all(entry.path())?;
            }
        } else if !ref_path.exists() {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn count_files(dir: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            count += count_files(&entry.path())?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}
