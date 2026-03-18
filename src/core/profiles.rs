use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::steam::Branch;

/// Represents a mod entry in a SMM profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEntry {
    pub version: String,
    #[serde(default)]
    pub enabled: bool,
}

/// Represents the SMM profiles.json structure.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SmmProfiles {
    #[serde(default)]
    pub profiles: HashMap<String, HashMap<String, ModEntry>>,
}

/// Represents the SMM installations.json structure.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SmmInstallations {
    #[serde(default)]
    pub installations: HashMap<String, InstallationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationConfig {
    pub profile: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Our branch-to-profile mapping.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BranchProfiles {
    #[serde(default)]
    pub mappings: HashMap<String, String>,
}

/// Get the ficsit/SMM config directory.
pub fn smm_config_dir() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let appdata =
            dirs::config_dir().ok_or_else(|| anyhow!("Cannot determine config directory"))?;
        Ok(appdata.join("ficsit"))
    } else {
        let data_dir =
            dirs::data_dir().ok_or_else(|| anyhow!("Cannot determine data directory"))?;
        Ok(data_dir.join("ficsit"))
    }
}

/// Get the satis-switcher config directory.
pub fn switcher_config_dir() -> Result<PathBuf> {
    let dir = if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Cannot determine local app data directory"))?
            .join("satis-switcher")
    } else {
        dirs::data_dir()
            .ok_or_else(|| anyhow!("Cannot determine data directory"))?
            .join("satis-switcher")
    };
    Ok(dir)
}

/// Read SMM profiles.
pub fn read_smm_profiles() -> Result<SmmProfiles> {
    let config_dir = smm_config_dir()?;
    let profiles_path = config_dir.join("profiles.json");
    read_json_file(&profiles_path)
}

/// Read SMM installations.
pub fn read_smm_installations() -> Result<SmmInstallations> {
    let config_dir = smm_config_dir()?;
    let installations_path = config_dir.join("installations.json");
    read_json_file(&installations_path)
}

/// Read our branch-profile mappings.
pub fn read_branch_profiles() -> Result<BranchProfiles> {
    let config_dir = switcher_config_dir()?;
    let path = config_dir.join("branch_profiles.json");

    if !path.exists() {
        return Ok(BranchProfiles::default());
    }

    read_json_file(&path)
}

/// Save our branch-profile mappings.
pub fn write_branch_profiles(profiles: &BranchProfiles) -> Result<()> {
    let config_dir = switcher_config_dir()?;
    fs::create_dir_all(&config_dir)?;
    let path = config_dir.join("branch_profiles.json");
    let json = serde_json::to_string_pretty(profiles)?;
    fs::write(&path, json).with_context(|| format!("Failed to write: {}", path.display()))?;
    Ok(())
}

/// Link a SMM profile to a branch.
pub fn link_profile(profile_name: &str, branch: &Branch) -> Result<()> {
    // Verify profile exists in SMM
    let smm = read_smm_profiles()?;
    if !smm.profiles.contains_key(profile_name) {
        return Err(anyhow!(
            "Profile '{}' not found in SMM. Available profiles: {}",
            profile_name,
            smm.profiles.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
    }

    let mut branch_profiles = read_branch_profiles()?;
    branch_profiles
        .mappings
        .insert(branch.to_string(), profile_name.to_string());
    write_branch_profiles(&branch_profiles)?;

    Ok(())
}

/// Activate the SMM profile for a given branch.
pub fn activate_profile_for_branch(branch: &Branch, install_path: &Path) -> Result<Option<String>> {
    let branch_profiles = read_branch_profiles()?;

    let profile_name = match branch_profiles.mappings.get(&branch.to_string()) {
        Some(name) => name.clone(),
        None => return Ok(None),
    };

    // Update installations.json to point to this profile
    let mut installations = read_smm_installations()?;
    let install_key = install_path.to_string_lossy().to_string();

    if let Some(config) = installations.installations.get_mut(&install_key) {
        config.profile = profile_name.clone();
    } else {
        installations.installations.insert(
            install_key,
            InstallationConfig {
                profile: profile_name.clone(),
                extra: HashMap::new(),
            },
        );
    }

    // Write updated installations
    let config_dir = smm_config_dir()?;
    let installations_path = config_dir.join("installations.json");
    let json = serde_json::to_string_pretty(&installations)?;
    fs::write(&installations_path, json)?;

    Ok(Some(profile_name))
}

/// Read a JSON file, returning default if not found.
fn read_json_file<T: serde::de::DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Ok(T::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))
}
