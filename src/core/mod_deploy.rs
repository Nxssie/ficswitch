use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::profiles;
use super::steam::Branch;

/// SMM download cache directory.
fn smm_download_cache() -> Result<PathBuf> {
    let cache = dirs::cache_dir()
        .ok_or_else(|| anyhow!("Cannot determine cache directory"))?
        .join("ficsit")
        .join("downloadCache");
    Ok(cache)
}

/// Find a mod zip in the SMM download cache by mod name.
/// Returns the path to the zip and the version string extracted from the filename.
fn find_mod_zip(cache_dir: &Path, mod_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(cache_dir).ok()?;

    let prefix = format!("{}_", mod_name);
    let suffix = "_Windows.zip";

    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix) && n.ends_with(suffix))
                .unwrap_or(false)
        })
        .collect();

    // Sort descending so we pick the latest version if multiple exist.
    candidates.sort_by(|a, b| b.cmp(a));
    candidates.into_iter().next()
}

/// Extract a zip archive into a destination directory.
/// Files in the zip are placed directly under dest (zip root → dest).
fn extract_zip(zip_path: &Path, dest: &Path) -> Result<usize> {
    let file = fs::File::open(zip_path)
        .with_context(|| format!("Failed to open zip: {}", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to read zip: {}", zip_path.display()))?;

    let mut count = 0;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = match entry.enclosed_name() {
            Some(p) => p.to_owned(),
            None => continue,
        };

        let out_path = dest.join(&entry_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)
                .with_context(|| format!("Failed to create: {}", out_path.display()))?;
            io::copy(&mut entry, &mut out_file)?;
            count += 1;
        }
    }

    Ok(count)
}

/// Deploy result summary.
pub struct DeployResult {
    pub mods_deployed: Vec<String>,
    pub mods_missing: Vec<String>,
    pub sml_deployed: bool,
}

/// Deploy all enabled mods for the given branch's SMM profile into the game directory.
/// Mods are extracted to FactoryGame/Mods/<ModName>/ and a lock file is written so SML
/// picks them up on the next launch.
pub fn deploy_mods(game_dir: &Path, branch: &Branch) -> Result<DeployResult> {
    let cache_dir = smm_download_cache()?;
    let mods_dir = game_dir.join("FactoryGame").join("Mods");

    // Read which profile is linked to this branch.
    let branch_profiles = profiles::read_branch_profiles()?;
    let profile_name = match branch_profiles.mappings.get(&branch.to_string()) {
        Some(name) => name.clone(),
        None => return Err(anyhow!("No SMM profile linked to '{}' branch", branch)),
    };

    // Read the profile's mod list.
    let smm_profiles = profiles::read_smm_profiles()?;
    let profile = smm_profiles
        .profiles
        .get(&profile_name)
        .ok_or_else(|| anyhow!("SMM profile '{}' not found", profile_name))?;

    fs::create_dir_all(&mods_dir)?;

    let mut result = DeployResult {
        mods_deployed: Vec::new(),
        mods_missing: Vec::new(),
        sml_deployed: false,
    };

    // Deploy SML first.
    if let Some(sml_zip) = find_mod_zip(&cache_dir, "SML") {
        let sml_dest = mods_dir.join("SML");
        fs::create_dir_all(&sml_dest)?;
        extract_zip(&sml_zip, &sml_dest).with_context(|| "Failed to deploy SML")?;
        result.sml_deployed = true;
    }

    // Deploy each enabled mod.
    for (mod_name, entry) in &profile.mods {
        if !entry.enabled {
            continue;
        }

        match find_mod_zip(&cache_dir, mod_name) {
            Some(zip_path) => {
                let mod_dest = mods_dir.join(mod_name);
                fs::create_dir_all(&mod_dest)?;
                extract_zip(&zip_path, &mod_dest)
                    .with_context(|| format!("Failed to deploy mod '{}'", mod_name))?;
                result.mods_deployed.push(mod_name.clone());
            }
            None => {
                result.mods_missing.push(mod_name.clone());
            }
        }
    }

    // Write the lock file for this profile so SML knows which mods are active.
    // Only write if one doesn't already exist (SMM may have written a richer one).
    let lock_path = mods_dir.join(format!("{}-lock.json", profile_name.to_lowercase()));
    if !lock_path.exists() {
        fs::write(&lock_path, r#"{"mods":{},"version":1}"#)?;
    }

    Ok(result)
}
