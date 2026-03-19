use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const SATISFACTORY_APP_ID: &str = "526870";

#[derive(Debug)]
pub struct SaveHeader {
    pub header_version: i32,
    pub save_version: i32,
    pub build_version: i32,
}

/// Parse the first 12 bytes of a .sav file to extract version info.
/// Format: 3x int32 little-endian
pub fn parse_save_header(path: &Path) -> Result<SaveHeader> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open save file: {}", path.display()))?;

    let mut buf = [0u8; 12];
    file.read_exact(&mut buf)
        .with_context(|| format!("Failed to read header from: {}", path.display()))?;

    Ok(SaveHeader {
        header_version: i32::from_le_bytes(buf[0..4].try_into().unwrap()),
        save_version: i32::from_le_bytes(buf[4..8].try_into().unwrap()),
        build_version: i32::from_le_bytes(buf[8..12].try_into().unwrap()),
    })
}

/// Find the save game directory for Satisfactory.
pub fn find_save_dir() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let local_app_data = dirs::data_local_dir()
            .ok_or_else(|| anyhow!("Cannot determine local app data directory"))?;

        let save_base = local_app_data
            .join("FactoryGame")
            .join("Saved")
            .join("SaveGames");

        find_steam_id_dir(&save_base)
    } else {
        // Linux: check Proton path first, then Flatpak Proton path
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;

        let proton_base = home
            .join(".local/share/Steam/steamapps/compatdata")
            .join(SATISFACTORY_APP_ID)
            .join("pfx/drive_c/users/steamuser/AppData/Local/FactoryGame/Saved/SaveGames");

        if proton_base.exists() {
            return find_steam_id_dir(&proton_base);
        }

        // Flatpak path
        let flatpak_base = home
            .join(".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata")
            .join(SATISFACTORY_APP_ID)
            .join("pfx/drive_c/users/steamuser/AppData/Local/FactoryGame/Saved/SaveGames");

        if flatpak_base.exists() {
            return find_steam_id_dir(&flatpak_base);
        }

        Err(anyhow!("Could not find Satisfactory save directory"))
    }
}

/// Find the first Steam ID subdirectory in the save base path.
fn find_steam_id_dir(save_base: &Path) -> Result<PathBuf> {
    if !save_base.exists() {
        return Err(anyhow!(
            "Save directory does not exist: {}",
            save_base.display()
        ));
    }

    let entries: Vec<_> = fs::read_dir(save_base)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    // Look for numeric Steam ID directories
    for entry in &entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.chars().all(|c| c.is_ascii_digit()) && name_str.len() > 5 {
            return Ok(entry.path());
        }
    }

    // Fall back to first directory
    if let Some(entry) = entries.first() {
        return Ok(entry.path());
    }

    Err(anyhow!(
        "No save directories found in: {}",
        save_base.display()
    ))
}

/// List all .sav files in the save directory.
pub fn list_saves(save_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut saves = Vec::new();

    if !save_dir.exists() {
        return Ok(saves);
    }

    for entry in fs::read_dir(save_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "sav") {
            saves.push(path);
        }
    }

    saves.sort();
    Ok(saves)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_save_header() {
        let dir = std::env::temp_dir().join("satis_switcher_test_saves");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.sav");

        let mut file = fs::File::create(&path).unwrap();
        // Write 3 i32 LE values: 13, 46, 264901
        file.write_all(&13i32.to_le_bytes()).unwrap();
        file.write_all(&46i32.to_le_bytes()).unwrap();
        file.write_all(&264901i32.to_le_bytes()).unwrap();

        let header = parse_save_header(&path).unwrap();
        assert_eq!(header.header_version, 13);
        assert_eq!(header.save_version, 46);
        assert_eq!(header.build_version, 264901);

        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }
}
