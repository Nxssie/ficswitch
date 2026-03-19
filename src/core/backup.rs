use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::saves;
use super::steam::Branch;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: String,
    pub label: Option<String>,
    pub branch: String,
    pub timestamp: DateTime<Utc>,
    pub save_count: usize,
    pub blueprint_count: usize,
}

/// Get the backup root directory.
pub fn backup_root() -> Result<PathBuf> {
    let data_dir = if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Cannot determine local app data directory"))?
            .join("ficswitch")
    } else {
        dirs::data_dir()
            .ok_or_else(|| anyhow!("Cannot determine data directory"))?
            .join("ficswitch")
    };

    Ok(data_dir.join("backups"))
}

/// Create a backup of saves and blueprints.
pub fn create_backup(
    save_dir: &Path,
    branch: &Branch,
    label: Option<&str>,
) -> Result<BackupManifest> {
    let backup_base = backup_root()?;
    let timestamp = Utc::now();
    let id = timestamp.format("%Y%m%d_%H%M%S").to_string();

    let backup_dir = backup_base.join(&id);
    fs::create_dir_all(&backup_dir)
        .with_context(|| format!("Failed to create backup dir: {}", backup_dir.display()))?;

    // Copy saves
    let save_files = saves::list_saves(save_dir)?;
    let saves_backup_dir = backup_dir.join("saves");
    fs::create_dir_all(&saves_backup_dir)?;

    for save_file in &save_files {
        if let Some(filename) = save_file.file_name() {
            fs::copy(save_file, saves_backup_dir.join(filename)).with_context(|| {
                format!("Failed to copy save file: {}", save_file.display())
            })?;
        }
    }

    // Copy blueprints
    let blueprint_dir = save_dir.join("blueprints");
    let blueprints_backup_dir = backup_dir.join("blueprints");
    let blueprint_count = if blueprint_dir.exists() {
        copy_dir_recursive(&blueprint_dir, &blueprints_backup_dir)?
    } else {
        0
    };

    // Write manifest
    let manifest = BackupManifest {
        id: id.clone(),
        label: label.map(|s| s.to_string()),
        branch: branch.to_string(),
        timestamp,
        save_count: save_files.len(),
        blueprint_count,
    };

    let manifest_path = backup_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;

    Ok(manifest)
}

/// List all backups, sorted by date (newest first).
pub fn list_backups() -> Result<Vec<BackupManifest>> {
    let backup_base = backup_root()?;

    if !backup_base.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(&backup_base)? {
        let entry = entry?;
        let manifest_path = entry.path().join("manifest.json");
        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)?;
            if let Ok(manifest) = serde_json::from_str::<BackupManifest>(&content) {
                backups.push(manifest);
            }
        }
    }

    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(backups)
}

/// Restore a backup by ID.
pub fn restore_backup(backup_id: &str, save_dir: &Path) -> Result<()> {
    let backup_base = backup_root()?;
    let backup_dir = backup_base.join(backup_id);

    if !backup_dir.exists() {
        return Err(anyhow!("Backup not found: {}", backup_id));
    }

    // Restore saves
    let saves_backup_dir = backup_dir.join("saves");
    if saves_backup_dir.exists() {
        for entry in fs::read_dir(&saves_backup_dir)? {
            let entry = entry?;
            let filename = entry.file_name();
            let dest = save_dir.join(&filename);
            fs::copy(entry.path(), &dest)
                .with_context(|| format!("Failed to restore: {}", filename.to_string_lossy()))?;
        }
    }

    // Restore blueprints
    let blueprints_backup_dir = backup_dir.join("blueprints");
    let blueprints_dest = save_dir.join("blueprints");
    if blueprints_backup_dir.exists() {
        copy_dir_recursive(&blueprints_backup_dir, &blueprints_dest)?;
    }

    Ok(())
}

/// Recursively copy a directory, returning the number of files copied.
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<usize> {
    fs::create_dir_all(dest)?;
    let mut count = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            count += copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
            count += 1;
        }
    }

    Ok(count)
}
