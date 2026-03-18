use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::System;

const SATISFACTORY_APP_ID: &str = "526870";

#[derive(Debug, Clone, PartialEq)]
pub enum Branch {
    Stable,
    Experimental,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Branch::Stable => write!(f, "stable"),
            Branch::Experimental => write!(f, "experimental"),
        }
    }
}

impl Branch {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "stable" | "public" | "" => Ok(Branch::Stable),
            "experimental" => Ok(Branch::Experimental),
            other => Err(anyhow!("Unknown branch: {}", other)),
        }
    }

    pub fn betakey(&self) -> &str {
        match self {
            Branch::Stable => "",
            Branch::Experimental => "experimental",
        }
    }
}

/// Parse a flat VDF/ACF file into key-value pairs.
/// This is a simplistic parser that works for appmanifest ACF files.
pub fn parse_vdf_flat(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut section_stack: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "{" {
            continue;
        }

        if trimmed == "}" {
            section_stack.pop();
            continue;
        }

        // Try to parse as key-value pair: "key" "value"
        let parts: Vec<&str> = trimmed.split('\t').collect();
        let tokens: Vec<String> = parts
            .iter()
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.len() == 2 {
            let full_key = if section_stack.is_empty() {
                tokens[0].clone()
            } else {
                format!("{}.{}", section_stack.join("."), tokens[0])
            };
            map.insert(full_key, tokens[1].clone());
        } else if tokens.len() == 1 {
            // Section header
            section_stack.push(tokens[0].clone());
        }
    }

    map
}

/// Detect the current branch from the appmanifest file.
pub fn detect_branch(manifest_path: &Path) -> Result<Branch> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;

    let vdf = parse_vdf_flat(&content);

    let betakey = vdf
        .get("UserConfig.betakey")
        .or_else(|| vdf.get("MountedConfig.betakey"))
        .map(|s| s.as_str())
        .unwrap_or("");

    Branch::from_str(betakey)
}

/// Find the Steam installation directory.
pub fn find_steam_dir() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        // Check common Windows paths
        let program_files = std::env::var("ProgramFiles(x86)")
            .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
        let steam_path = PathBuf::from(&program_files).join("Steam");
        if steam_path.exists() {
            return Ok(steam_path);
        }

        let program_files64 =
            std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let steam_path = PathBuf::from(&program_files64).join("Steam");
        if steam_path.exists() {
            return Ok(steam_path);
        }
    } else {
        // Linux paths
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;

        let candidates = [
            home.join(".steam/steam"),
            home.join(".local/share/Steam"),
            home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Ok(candidate.clone());
            }
        }
    }

    Err(anyhow!("Could not find Steam installation directory"))
}

/// Find the appmanifest for Satisfactory.
pub fn find_manifest() -> Result<PathBuf> {
    let steam_dir = find_steam_dir()?;

    // Check main steamapps
    let manifest = steam_dir
        .join("steamapps")
        .join(format!("appmanifest_{}.acf", SATISFACTORY_APP_ID));
    if manifest.exists() {
        return Ok(manifest);
    }

    // Check library folders
    let library_folders = steam_dir.join("steamapps").join("libraryfolders.vdf");
    if library_folders.exists() {
        let content = fs::read_to_string(&library_folders)?;
        let vdf = parse_vdf_flat(&content);

        for (key, value) in &vdf {
            if key.ends_with(".path") {
                let lib_manifest = PathBuf::from(value)
                    .join("steamapps")
                    .join(format!("appmanifest_{}.acf", SATISFACTORY_APP_ID));
                if lib_manifest.exists() {
                    return Ok(lib_manifest);
                }
            }
        }
    }

    Err(anyhow!(
        "Could not find Satisfactory appmanifest (AppID {})",
        SATISFACTORY_APP_ID
    ))
}

/// Check if Steam is currently running.
pub fn is_steam_running() -> bool {
    let sys = System::new_all();
    sys.processes().values().any(|process| {
        let name = process.name().to_lowercase();
        name.contains("steam") && !name.contains("steamvr")
    })
}

/// Switch the branch in the appmanifest file using atomic write.
pub fn switch_branch(manifest_path: &Path, target: &Branch) -> Result<()> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;

    let new_content = set_betakey_in_acf(&content, target.betakey());

    // Atomic write: write to temp file, then rename
    let temp_path = manifest_path.with_extension("acf.tmp");
    fs::write(&temp_path, &new_content)
        .with_context(|| format!("Failed to write temp manifest: {}", temp_path.display()))?;

    fs::rename(&temp_path, manifest_path).with_context(|| {
        format!(
            "Failed to rename temp manifest to: {}",
            manifest_path.display()
        )
    })?;

    Ok(())
}

/// Set the betakey value in ACF content for both UserConfig and MountedConfig sections.
fn set_betakey_in_acf(content: &str, betakey: &str) -> String {
    let mut result = String::new();
    let mut in_user_config = false;
    let mut in_mounted_config = false;
    let mut found_betakey_user = false;
    let mut found_betakey_mounted = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.contains("\"UserConfig\"") {
            in_user_config = true;
            brace_depth = 0;
        } else if trimmed.contains("\"MountedConfig\"") {
            in_mounted_config = true;
            brace_depth = 0;
        }

        if (in_user_config || in_mounted_config) && trimmed == "{" {
            brace_depth += 1;
        }

        if (in_user_config || in_mounted_config) && trimmed == "}" {
            brace_depth -= 1;
            if brace_depth == 0 {
                // Insert betakey before closing brace if not found
                if in_user_config && !found_betakey_user {
                    result.push_str(&format!("\t\t\"betakey\"\t\t\"{}\"\n", betakey));
                }
                if in_mounted_config && !found_betakey_mounted {
                    result.push_str(&format!("\t\t\"betakey\"\t\t\"{}\"\n", betakey));
                }
                in_user_config = false;
                in_mounted_config = false;
            }
        }

        if (in_user_config || in_mounted_config) && trimmed.contains("\"betakey\"") {
            // Replace the betakey line
            let indent = &line[..line.len() - trimmed.len()];
            result.push_str(&format!("{}\"betakey\"\t\t\"{}\"\n", indent, betakey));
            if in_user_config {
                found_betakey_user = true;
            }
            if in_mounted_config {
                found_betakey_mounted = true;
            }
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

/// Get the Satisfactory install directory from the manifest.
pub fn get_install_dir(manifest_path: &Path) -> Result<PathBuf> {
    let content = fs::read_to_string(manifest_path)?;
    let vdf = parse_vdf_flat(&content);

    let install_dir = vdf
        .get("installdir")
        .ok_or_else(|| anyhow!("installdir not found in manifest"))?;

    let steamapps_dir = manifest_path
        .parent()
        .ok_or_else(|| anyhow!("Manifest has no parent directory"))?;

    Ok(steamapps_dir.join("common").join(install_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_ACF: &str = r#""AppState"
{
	"appid"		"526870"
	"Universe"		"1"
	"name"		"Satisfactory"
	"installdir"		"Satisfactory"
	"UserConfig"
	{
		"betakey"		"experimental"
	}
	"MountedConfig"
	{
		"betakey"		"experimental"
	}
}"#;

    #[test]
    fn test_parse_vdf_flat() {
        let map = parse_vdf_flat(SAMPLE_ACF);
        assert_eq!(map.get("AppState.appid").unwrap(), "526870");
        assert_eq!(map.get("AppState.name").unwrap(), "Satisfactory");
        assert_eq!(
            map.get("AppState.UserConfig.betakey").unwrap(),
            "experimental"
        );
    }

    #[test]
    fn test_branch_from_str() {
        assert_eq!(Branch::from_str("stable").unwrap(), Branch::Stable);
        assert_eq!(Branch::from_str("experimental").unwrap(), Branch::Experimental);
        assert_eq!(Branch::from_str("").unwrap(), Branch::Stable);
        assert_eq!(Branch::from_str("public").unwrap(), Branch::Stable);
        assert!(Branch::from_str("unknown").is_err());
    }

    #[test]
    fn test_branch_betakey() {
        assert_eq!(Branch::Stable.betakey(), "");
        assert_eq!(Branch::Experimental.betakey(), "experimental");
    }

    #[test]
    fn test_set_betakey_in_acf() {
        let result = set_betakey_in_acf(SAMPLE_ACF, "");
        assert!(result.contains("\"betakey\"\t\t\"\""));
        assert!(!result.contains("\"betakey\"\t\t\"experimental\""));
    }
}
