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
                tokens[0].to_lowercase()
            } else {
                format!("{}.{}", section_stack.join("."), tokens[0].to_lowercase())
            };
            map.insert(full_key, tokens[1].clone());
        } else if tokens.len() == 1 {
            // Section header
            section_stack.push(tokens[0].to_lowercase());
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
        .get("appstate.userconfig.betakey")
        .or_else(|| vdf.get("appstate.mountedconfig.betakey"))
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

/// Check if the appmanifest indicates a pending download or update.
pub fn is_download_pending(manifest_path: &Path) -> Result<bool> {
    let content = fs::read_to_string(manifest_path)?;
    let vdf = parse_vdf_flat(&content);

    // BytesToDownload > 0 and not equal to BytesDownloaded means download in progress.
    let to_download: u64 = vdf
        .get("appstate.bytestodownload")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let downloaded: u64 = vdf
        .get("appstate.bytesdownloaded")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    // TargetBuildID != buildid means Steam has queued an update to a different build.
    let build_id: u64 = vdf
        .get("appstate.buildid")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let target_build: u64 = vdf
        .get("appstate.targetbuildid")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    Ok((target_build != 0 && target_build != build_id) || (to_download > 0 && downloaded < to_download))
}

/// Find the SteamCMD executable.
pub fn find_steamcmd() -> Result<std::path::PathBuf> {
    // Check PATH first
    if let Ok(path) = which_steamcmd() {
        return Ok(path);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;
        let candidates = [
            home.join(".local/share/steamcmd/steamcmd.sh"),
            home.join(".steam/steamcmd/steamcmd.sh"),
            home.join("Steam/steamcmd/steamcmd.sh"),
            std::path::PathBuf::from("/usr/games/steamcmd"),
            std::path::PathBuf::from("/usr/lib/games/steam/steamcmd"),
        ];
        for c in &candidates {
            if c.exists() {
                return Ok(c.clone());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let steam_dir = find_steam_dir()?;
        let candidate = steam_dir.join("steamcmd.exe");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(anyhow!(
        "steamcmd not found. Install it with your package manager (e.g. apt install steamcmd) or download it from https://developer.valvesoftware.com/wiki/SteamCMD"
    ))
}

fn which_steamcmd() -> Result<std::path::PathBuf> {
    let output = std::process::Command::new("which")
        .arg("steamcmd")
        .output()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Ok(std::path::PathBuf::from(path));
        }
    }
    Err(anyhow!("not in PATH"))
}

/// Download (or update) a branch using SteamCMD.
/// Inherits stdin/stdout so the user can enter credentials interactively.
/// After the first successful login, SteamCMD caches the session token.
pub fn download_with_steamcmd(branch: &Branch, username: &str, install_dir: &Path) -> Result<()> {
    let steamcmd = find_steamcmd()?;

    let mut cmd = std::process::Command::new(&steamcmd);
    cmd.arg("+force_install_dir").arg(install_dir)
        .arg("+login").arg(username)
        .arg("+app_update").arg("526870");

    if let Branch::Experimental = branch {
        cmd.arg("-beta").arg("experimental");
    }

    cmd.arg("validate").arg("+quit");

    let status = cmd.status().context("Failed to run steamcmd")?;

    if !status.success() {
        return Err(anyhow!("steamcmd exited with an error (code: {})", status));
    }

    // SteamCMD sometimes creates a steamapps/ subfolder and an appmanifest.acf
    // inside the install dir when force_install_dir is not applied correctly.
    // Clean them up so they don't end up in the cache.
    let _ = fs::remove_dir_all(install_dir.join("steamapps"));
    let _ = fs::remove_file(install_dir.join("appmanifest.acf"));

    Ok(())
}

/// Launch the Steam client.
pub fn launch_steam() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let steam_dir = find_steam_dir()?;
        std::process::Command::new(steam_dir.join("steam.exe"))
            .spawn()
            .context("Failed to launch Steam")?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("steam")
            .spawn()
            .context("Failed to launch Steam — is it installed and in PATH?")?;
    }
    Ok(())
}

/// Download progress read from the appmanifest.
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
}

/// Read current download progress from the appmanifest.
pub fn get_download_progress(manifest_path: &Path) -> Result<DownloadProgress> {
    let content = fs::read_to_string(manifest_path)?;
    let vdf = parse_vdf_flat(&content);

    let bytes_total = vdf
        .get("appstate.bytestodownload")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0u64);
    let bytes_downloaded = vdf
        .get("appstate.bytesdownloaded")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0u64);

    Ok(DownloadProgress { bytes_downloaded, bytes_total })
}

/// Block until Steam has finished downloading the target branch.
/// Prints a live progress line using carriage return.
pub fn wait_for_download(manifest_path: &Path, branch: &Branch) -> Result<()> {
    use std::io::Write;
    use std::time::Duration;

    let mut download_started = false;

    loop {
        let pending = is_download_pending(manifest_path).unwrap_or(true);
        let progress = get_download_progress(manifest_path)?;

        if progress.bytes_total > 0 {
            download_started = true;
        }

        if download_started && !pending {
            break;
        }

        if progress.bytes_total > 0 {
            let pct = (progress.bytes_downloaded * 100) / progress.bytes_total;
            let filled = (pct / 5) as usize;
            let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(20 - filled));
            let dl_gb = progress.bytes_downloaded as f64 / 1_073_741_824.0;
            let total_gb = progress.bytes_total as f64 / 1_073_741_824.0;
            print!(
                "\r  Downloading {}... {:.1} GB / {:.1} GB {} {}%   ",
                branch,
                dl_gb,
                total_gb,
                bar,
                pct
            );
        } else {
            print!("\r  Waiting for Steam to start download...   ");
        }

        std::io::stdout().flush().ok();
        std::thread::sleep(Duration::from_secs(2));
    }

    println!(); // end the progress line
    Ok(())
}

/// Block until Steam is no longer running.
pub fn wait_for_steam_close() {
    use std::time::Duration;
    while is_steam_running() {
        std::thread::sleep(Duration::from_secs(2));
    }
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

        if (in_user_config || in_mounted_config) && trimmed.to_lowercase().contains("\"betakey\"") {
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

    // Preserve trailing newline exactly as in source
    if content.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    } else if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    result
}

/// Get the Satisfactory install directory from the manifest.
pub fn get_install_dir(manifest_path: &Path) -> Result<PathBuf> {
    let content = fs::read_to_string(manifest_path)?;
    let vdf = parse_vdf_flat(&content);

    let install_dir = vdf
        .get("appstate.installdir")
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
        assert_eq!(map.get("appstate.appid").unwrap(), "526870");
        assert_eq!(map.get("appstate.name").unwrap(), "Satisfactory");
        assert_eq!(
            map.get("appstate.userconfig.betakey").unwrap(),
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

    #[test]
    fn test_acf_trailing_newline_preserved() {
        // Regression: if the original ACF ends with '\n', the output must too.
        let acf_with_newline = format!("{}\n", SAMPLE_ACF);
        let result = set_betakey_in_acf(&acf_with_newline, "");
        assert!(
            result.ends_with('\n'),
            "Expected trailing newline to be preserved, but result ends with: {:?}",
            result.chars().last()
        );
    }

    #[test]
    fn test_acf_no_trailing_newline_not_added() {
        // Regression: if the original ACF has no trailing '\n', the output must not add one.
        // SAMPLE_ACF ends with `}"#` — no trailing newline.
        assert!(
            !SAMPLE_ACF.ends_with('\n'),
            "SAMPLE_ACF should not end with newline"
        );
        let result = set_betakey_in_acf(SAMPLE_ACF, "");
        assert!(
            !result.ends_with('\n'),
            "Expected no trailing newline to be added, but result ends with '\\n'. \
             Last 10 chars: {:?}",
            &result[result.len().saturating_sub(10)..]
        );
    }
}
